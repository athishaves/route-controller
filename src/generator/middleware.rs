//! Middleware application for routers

use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

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
