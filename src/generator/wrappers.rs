//! Wrapper function generation for route handlers

use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemImpl;

pub fn generate_wrapper_functions(impl_block: &ItemImpl) -> Vec<TokenStream> {
  let mut wrappers = vec![];

  for item in &impl_block.items {
    if let syn::ImplItem::Fn(method) = item {
      if let Some(route_info) = crate::parser::extract_route_from_attrs(&method.attrs) {
        let params = crate::parser::analyze_params(&method.sig, &route_info.extractors);

        let mut needs_wrapper = false;
        // Group Path parameters for tuple extraction
        let mut path_types: Vec<_> = vec![];
        let mut path_names: Vec<_> = vec![];

        for p in params.iter() {
          match p.extractor_type {
            crate::parser::ExtractorType::None => {}
            _ => {
              needs_wrapper = true;
              // Path extractors need special handling
              // Collect all path types and names to combine into a tuple
              if p.extractor_type == crate::parser::ExtractorType::Path {
                if let syn::Pat::Ident(pat_ident) = &p.pat {
                  path_types.push(&p.ty);
                  path_names.push(&pat_ident.ident);
                }
              }
            }
          }
        }

        let has_response_headers =
          !route_info.response_headers.is_empty() || route_info.content_type.is_some();

        if needs_wrapper || has_response_headers {
          let handler_name = &method.sig.ident;
          let wrapper_name = syn::Ident::new(
            &format!("{}_wrapper", handler_name),
            proc_macro2::Span::call_site(),
          );

          let is_async = method.sig.asyncness.is_some();
          let async_token = if is_async {
            quote! { async }
          } else {
            quote! {}
          };
          let await_token = if is_async {
            quote! { .await }
          } else {
            quote! {}
          };

          let return_type = &method.sig.output;

          // Determine if we need to wrap the return type with headers
          let needs_header_wrapping =
            !route_info.response_headers.is_empty() || route_info.content_type.is_some();

          let wrapper_return_type = if needs_header_wrapping {
            quote! { -> impl axum::response::IntoResponse }
          } else {
            quote! { #return_type }
          };

          // Build wrapper parameters
          let mut wrapper_param_set = std::collections::HashSet::new();
          let mut wrapper_params = Vec::new();
          let mut call_args = Vec::new();

          // Handle Path extractors (must be first and combined into tuple if multiple)
          if !path_types.is_empty() {
            if path_types.len() > 1 { // Multiple paths
              wrapper_params.push(quote! {
                axum::extract::Path((#(#path_names),*)): axum::extract::Path<(#(#path_types),*)>
              });

              // Add individual path args to call_args
              for name in &path_names {
                call_args.push(quote! { #name });
              }
            } else {
              // Single path: extract normally
              let name = path_names[0];
              let ty = &path_types[0];
              wrapper_params
                .push(quote! { axum::extract::Path(#name): axum::extract::Path<#ty> });
              call_args.push(quote! { #name });
            }
          }

          // Handle non-Path extractors
          for p in params.iter() {
            let pat = &p.pat;
            let ty = &p.ty;

            match &p.extractor_type {
              crate::parser::ExtractorType::Path => {
                // Already handled above
              }
              crate::parser::ExtractorType::Json => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params.push(quote! { axum::Json(#name): axum::Json<#ty> });
                  call_args.push(quote! { #name });
                }
              }
              crate::parser::ExtractorType::Form => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params.push(quote! { axum::Form(#name): axum::Form<#ty> });
                  call_args.push(quote! { #name });
                }
              }
              crate::parser::ExtractorType::Query => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params
                    .push(quote! { axum::extract::Query(#name): axum::extract::Query<#ty> });
                  call_args.push(quote! { #name });
                }
              }
              crate::parser::ExtractorType::Bytes => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params.push(quote! { #name: axum::body::Bytes });
                  call_args.push(quote! { #name.to_vec() });
                }
              }
              crate::parser::ExtractorType::Text
              | crate::parser::ExtractorType::Html
              | crate::parser::ExtractorType::Xml
              | crate::parser::ExtractorType::JavaScript => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params.push(quote! { #name: String });
                  call_args.push(quote! { #name });
                }
              }
              crate::parser::ExtractorType::HeaderParam => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  let name_str = name.to_string();
                  // Convert snake_case to kebab-case for header names (e.g., content_type -> content-type)
                  let header_name = name_str.replace('_', "-");
                  call_args.push(quote! {
                    headers.get(#header_name)
                      .and_then(|v| v.to_str().ok())
                      .unwrap_or("")
                      .to_string()
                  });
                  if wrapper_param_set.insert("HeaderParam") {
                    wrapper_params.push(quote! { headers: axum::http::HeaderMap });
                  }
                }
              }
              crate::parser::ExtractorType::CookieParam => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  let name_str = name.to_string();
                  call_args.push(quote! {
                    cookies.get(#name_str)
                      .map(|c| c.value().to_string())
                      .unwrap_or_default()
                  });
                  if wrapper_param_set.insert("CookieParam") {
                    wrapper_params.push(quote! { cookies: axum_extra::extract::CookieJar });
                  }
                }
              }
              crate::parser::ExtractorType::SessionParam => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  let name_str = name.to_string();
                  // Session.get() returns a Future, so we need to await it
                  call_args.push(quote! {
                    session.get::<#ty>(#name_str)
                      .await
                      .ok()
                      .flatten()
                      .unwrap_or_default()
                  });
                  if wrapper_param_set.insert("SessionParam") {
                    wrapper_params.push(quote! { session: tower_sessions::Session });
                  }
                }
              }
              crate::parser::ExtractorType::State => {
                // Extract state and pass it through
                call_args.push(quote! { state.0.clone() });
                // Only one state is allowed, no check for duplicates needed
                let state_ty = &p.ty;
                wrapper_params.push(quote! { state: axum::extract::State<#state_ty> });
              }
              crate::parser::ExtractorType::None => {
                wrapper_params.push(quote! { #pat: #ty });
                call_args.push(quote! { #pat });
              }
            }
          }

          let wrapper_signature =
            quote! {
              #async_token fn #wrapper_name(#(#wrapper_params),*) #wrapper_return_type
            };

          // Build header additions
          let header_additions: Vec<_> = route_info
            .response_headers
            .iter()
            .map(|(name, value)| {
              quote! { (axum::http::header::HeaderName::from_static(#name), #value) }
            })
            .collect();

          let wrapper_body = if let Some(ref ct) = route_info.content_type {
            if !header_additions.is_empty() {
              quote! {
                let response = Self::#handler_name(#(#call_args),*)#await_token;
                (
                  [
                    (axum::http::header::CONTENT_TYPE, #ct),
                    #(#header_additions),*
                  ],
                  response
                )
              }
            } else {
              quote! {
                let response = Self::#handler_name(#(#call_args),*)#await_token;
                ([(axum::http::header::CONTENT_TYPE, #ct)], response)
              }
            }
          } else if !header_additions.is_empty() {
            quote! {
              let response = Self::#handler_name(#(#call_args),*)#await_token;
              ([#(#header_additions),*], response)
            }
          } else {
            quote! {
              Self::#handler_name(#(#call_args),*)#await_token
            }
          };

          wrappers.push(quote! {
              #wrapper_signature {
                  #wrapper_body
              }
          });

          log_verbose!(
            "Generated wrapper function: [{}] for [{}]",
            quote! { #wrapper_name },
            quote! { #handler_name }
          );
        }
      }
    }
  }

  wrappers
}
