//! Generating Axum router code from controllers

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, Path, Type};

#[allow(unused_imports)]
use crate::logger::log_verbose;

pub fn generate_route_registrations(impl_block: &ItemImpl, config: &crate::parser::ControllerConfig) -> Vec<TokenStream> {
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

				// Analyze parameters to see if we need to generate a wrapper
				let params = crate::parser::analyze_params(&method.sig, &route_info.content_type);
				let needs_wrapper = config.is_auto_controller && params.iter().any(|p| p.needs_wrap);

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
				let params = crate::parser::analyze_params(&method.sig, &route_info.content_type);
				let needs_wrapper = params.iter().any(|p| p.needs_wrap);

				if needs_wrapper {
					let handler_name = &method.sig.ident;
					let wrapper_name = syn::Ident::new(
						&format!("{}_wrapper", handler_name),
						proc_macro2::Span::call_site(),
					);

					let is_async = method.sig.asyncness.is_some();
					let async_token = if is_async { quote! { async } } else { quote! {} };
					let await_token = if is_async { quote! { .await } } else { quote! {} };

					let return_type = &method.sig.output;

					// Build wrapper parameters
					let wrapper_params: Vec<TokenStream> = params.iter().map(|p| {
						let pat = &p.pat;
						let ty = &p.ty;
						if p.needs_wrap {
							// Extract the identifier from the pattern for wrapping
							if let syn::Pat::Ident(pat_ident) = pat {
								let name = &pat_ident.ident;
								match p.extractor_type {
									crate::parser::ExtractorType::Json => {
										quote! { axum::Json(#name): axum::Json<#ty> }
									}
									crate::parser::ExtractorType::Form => {
										quote! { axum::Form(#name): axum::Form<#ty> }
									}
									crate::parser::ExtractorType::None => {
										quote! { #pat: #ty }
									}
								}
							} else {
								// For complex patterns, keep as is
								quote! { #pat: #ty }
							}
						} else {
							quote! { #pat: #ty }
						}
					}).collect();

					// Build call arguments - extract the name/pattern for calling the original function
					let call_args: Vec<TokenStream> = params.iter().map(|p| {
						let pat = &p.pat;
						if p.needs_wrap {
							// For wrapped params, just pass the unwrapped value
							if let syn::Pat::Ident(pat_ident) = pat {
								let name = &pat_ident.ident;
								quote! { #name }
							} else {
								quote! { #pat }
							}
						} else {
							// For extractors, pass the full pattern
							quote! { #pat }
						}
					}).collect();

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
