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
				let handler_ident = syn::Ident::new(
					&route_info.method,
					proc_macro2::Span::call_site(),
				);
				let route_path = &route_info.path;

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

	log_verbose!("Adding middleware: [{}]", middlewares
		.iter()
		.map(|mw| (quote! { #mw }).to_string())
		.collect::<Vec<_>>()
		.join(", ")
	);

	// Reverse the order of middlewares to maintain the wrapping order
	let middlewares_reversed: Vec<_> = middlewares
		.iter()
		.rev()
		.collect();

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
	quote! {
		#impl_block
		impl #name {
			pub fn router() -> axum::Router {
				#final_router
			}
		}
	}
}
