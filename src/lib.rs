//! # Route Controller
//!
//! Generate Axum routers from controller-style implementations.
//!
//! ## Example
//!
//! ```ignore
//! use route_controller::{controller, get, post};
//!
//! struct UserController;
//!
//! #[controller(path = "/users")]
//! impl UserController {
//!     #[get("/")]
//!     async fn list() -> String {
//!         "User list".to_string()
//!     }
//!
//!     #[post("/")]
//!     async fn create() -> String {
//!         "User created".to_string()
//!     }
//! }
//!
//! let app = UserController::router();
//! ```

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl};

#[macro_use]
mod logger;
mod generator;
mod parser;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
	let impl_block = parse_macro_input!(item as ItemImpl);
	let name = &impl_block.self_ty;

	log_verbose!("Generating router for: [{}]", quote::quote! { #name }.to_string());

	let config = parser::parse_controller_attributes(&attr);

	let route_registrations = generator::generate_route_registrations(&impl_block);
	let base_router = generator::generate_base_router(&route_registrations);

	if route_registrations.is_empty() {
		log_info!("Warning: No routes found in controller");
		return TokenStream::from(generator::generate_router_impl(
			&impl_block,
			name,
			base_router,
		));
	}

	let router_with_middleware = generator::apply_middlewares(
		base_router, &config.middlewares
	);
	let final_router = generator::apply_route_prefix(
		router_with_middleware, config.route_prefix.as_ref()
	);

	TokenStream::from(generator::generate_router_impl(
		&impl_block, name, final_router
	))
}

#[proc_macro_attribute]
pub fn get(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn post(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn put(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn delete(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}

#[proc_macro_attribute]
pub fn patch(_: TokenStream, item: TokenStream) -> TokenStream {
	item
}
