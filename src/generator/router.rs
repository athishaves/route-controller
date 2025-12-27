//! Router generation from parsed controller information

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Type};

#[allow(unused_imports)]
use crate::logger::log_verbose;
use crate::parser::ControllerConfig;

pub fn generate_route_registrations(
  impl_block: &ItemImpl,
  controller_config: &ControllerConfig,
) -> Vec<TokenStream> {
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

        let has_response_headers = !route_info.response_headers.is_empty()
          || route_info.content_type.is_some()
          || !controller_config.response_headers.is_empty()
          || controller_config.content_type.is_some();

        if needs_wrapper || has_response_headers {
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

pub fn generate_router_impl(
  impl_block: &ItemImpl,
  name: &Type,
  final_router: TokenStream,
  controller_config: &ControllerConfig,
) -> TokenStream {
  // Generate wrapper functions for handlers that need Json extraction
  let wrapper_functions =
    super::wrappers::generate_wrapper_functions(impl_block, controller_config);

  // Check if any handler uses State extractor and get the state type
  let state_type: Option<Type> = impl_block.items.iter().find_map(|item| {
    if let syn::ImplItem::Fn(method) = item {
      if let Some(route_info) = crate::parser::extract_route_from_attrs(&method.attrs) {
        let params = crate::parser::analyze_params(&method.sig, &route_info.extractors);
        return params
          .iter()
          .find(|p| p.extractor_type == crate::parser::ExtractorType::State)
          .map(|p| p.ty.clone());
      }
    }
    None
  });

  if let Some(state_ty) = state_type {
    quote! {
        #impl_block
        impl #name {
            #(#wrapper_functions)*

            pub fn router() -> axum::Router<#state_ty> {
                #final_router
            }
        }
    }
  } else {
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
}
