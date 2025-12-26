//! Generating Axum router code from controllers

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Path, Type};

#[allow(unused_imports)]
use crate::logger::log_verbose;

pub fn generate_route_registrations(impl_block: &ItemImpl) -> Vec<TokenStream> {
  let mut route_registrations = vec![];

  for item in &impl_block.items {
    if let ImplItem::Fn(method) = item {
      if let Some(route_info) = crate::parser::extract_route_from_attrs(&method.attrs) {
        let handler_name = &method.sig.ident;
        let handler_ident = syn::Ident::new(&route_info.method, proc_macro2::Span::call_site());
        let route_path = &route_info.path;

        // Analyze parameters with explicit extractors
        let params = crate::parser::analyze_params(&method.sig, &route_info.extractors);
        let needs_wrapper = params
          .iter()
          .any(|p| p.extractor_type != crate::parser::ExtractorType::None);

        if needs_wrapper {
          // Generate a wrapper function that handles extraction
          let wrapper_name = syn::Ident::new(
            &format!("{}_wrapper", handler_name),
            proc_macro2::Span::call_site(),
          );

          route_registrations.push(quote! {
              .route(#route_path, axum::routing::#handler_ident(Self::#wrapper_name))
          });

          log_verbose!(
            "Registering route with wrapper: [Method:{}] [Endpoint:{}] [Handler:{}]",
            route_info.method,
            route_path,
            quote! { #handler_name }
          );
        } else {
          route_registrations.push(quote! {
              .route(#route_path, axum::routing::#handler_ident(Self::#handler_name))
          });

          log_verbose!(
            "Registering route: [Method:{}] [Endpoint:{}] [Handler:{}]",
            route_info.method,
            route_path,
            quote! { #handler_name }
          );
        }
      }
    }
  }

  route_registrations
}

pub fn generate_base_router(route_registrations: &[TokenStream]) -> TokenStream {
  quote! {
      axum::Router::new()
          #(#route_registrations)*
  }
}

pub fn apply_middlewares(base_router: TokenStream, middlewares: &[Path]) -> TokenStream {
  if middlewares.is_empty() {
    log_verbose!("No middlewares to apply");
    return base_router;
  }

  log_verbose!(
    "Adding middleware: [{}]",
    middlewares
      .iter()
      .map(|mw| (quote! { #mw }).to_string())
      .collect::<Vec<_>>()
      .join(", ")
  );

  // Reverse the order of middlewares to maintain the wrapping order
  let middlewares_reversed: Vec<_> = middlewares.iter().rev().collect();

  quote! {
      {
          let router = #base_router;
          #(
          let router = router.layer(axum::middleware::from_fn(#middlewares_reversed));
          )*
          router
      }
  }
}

pub fn apply_route_prefix(router: TokenStream, prefix: Option<&String>) -> TokenStream {
  if let Some(prefix) = prefix {
    log_verbose!("Adding route prefix: [{}]", prefix);
    quote! {
        axum::Router::new().nest(#prefix, #router)
    }
  } else {
    log_verbose!("No route prefix to apply");
    router
  }
}

pub fn generate_router_impl(
  impl_block: &ItemImpl,
  name: &Box<Type>,
  final_router: TokenStream,
) -> TokenStream {
  // Generate wrapper functions for handlers that need Json extraction
  let wrapper_functions = generate_wrapper_functions(impl_block);

  quote! {
      #impl_block
      impl #name {
          #(#wrapper_functions)*

          pub fn router() -> axum::Router {
              #final_router
          }
      }
  }
}

fn generate_wrapper_functions(impl_block: &ItemImpl) -> Vec<TokenStream> {
  let mut wrappers = vec![];

  for item in &impl_block.items {
    if let syn::ImplItem::Fn(method) = item {
      if let Some(route_info) = crate::parser::extract_route_from_attrs(&method.attrs) {
        let params = crate::parser::analyze_params(&method.sig, &route_info.extractors);
        let needs_wrapper = params
          .iter()
          .any(|p| p.extractor_type != crate::parser::ExtractorType::None);

        if needs_wrapper {
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

          // Group Path parameters for tuple extraction
          let path_params: Vec<_> = params
            .iter()
            .filter(|p| p.extractor_type == crate::parser::ExtractorType::Path)
            .collect();

          let has_multiple_paths = path_params.len() > 1;

          // Build wrapper parameters
          let mut wrapper_params = Vec::new();
          let mut call_args = Vec::new();
          
          // Track if we need shared extractors
          let has_headers = params.iter().any(|p| p.extractor_type == crate::parser::ExtractorType::HeaderParam);
          let has_cookies = params.iter().any(|p| p.extractor_type == crate::parser::ExtractorType::CookieParam);
          let has_session = params.iter().any(|p| p.extractor_type == crate::parser::ExtractorType::SessionParam);

          // Handle Path extractors (must be first and combined into tuple if multiple)
          if !path_params.is_empty() {
            if has_multiple_paths {
              // Multiple paths: extract as tuple
              let path_types: Vec<_> = path_params.iter().map(|p| &p.ty).collect();
              let path_names: Vec<_> = path_params
                .iter()
                .filter_map(|p| {
                  if let syn::Pat::Ident(pat_ident) = &p.pat {
                    Some(&pat_ident.ident)
                  } else {
                    None
                  }
                })
                .collect();

              wrapper_params.push(quote! {
                axum::extract::Path((#(#path_names),*)): axum::extract::Path<(#(#path_types),*)>
              });

              // Add individual path args to call_args
              for name in &path_names {
                call_args.push(quote! { #name });
              }
            } else {
              // Single path: extract normally
              let p = path_params[0];
              if let syn::Pat::Ident(pat_ident) = &p.pat {
                let name = &pat_ident.ident;
                let ty = &p.ty;
                wrapper_params
                  .push(quote! { axum::extract::Path(#name): axum::extract::Path<#ty> });
                call_args.push(quote! { #name });
              }
            }
          }
          
          // Add shared extractors once if needed
          if has_headers {
            wrapper_params.push(quote! { headers: axum::http::HeaderMap });
          }
          if has_cookies {
            wrapper_params.push(quote! { cookies: axum_extra::extract::CookieJar });
          }
          if has_session {
            wrapper_params.push(quote! { session: tower_sessions::Session });
          }

          // Handle non-Path extractors
          for p in params.iter() {
            let pat = &p.pat;
            let ty = &p.ty;

            match &p.extractor_type {
              crate::parser::ExtractorType::Path => {
                // Already handled above
                continue;
              }
              crate::parser::ExtractorType::Json => {
                if let syn::Pat::Ident(pat_ident) = pat {
                  let name = &pat_ident.ident;
                  wrapper_params.push(quote! { axum::Json(#name): axum::Json<#ty> });
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
                }
              }
              crate::parser::ExtractorType::None => {
                wrapper_params.push(quote! { #pat: #ty });
                call_args.push(quote! { #pat });
              }
            }
          }

          wrappers.push(quote! {
              #async_token fn #wrapper_name(#(#wrapper_params),*) #return_type {
                  Self::#handler_name(#(#call_args),*)#await_token
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
