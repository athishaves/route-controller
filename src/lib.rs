//! # Route Controller
//!
//! Generate Axum routers from controller-style implementations.
//!
//! ## Controller Example
//!
//! ```ignore
//! use route_controller::{controller, post, get};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct User {
//!     name: String,
//! }
//!
//! #[controller(path = "/users")]
//! impl UserController {
//!     // Specify extractors explicitly
//!     #[post("/", extract(user = Json))]
//!     async fn create(user: User) -> String {
//!         format!("Created: {}", user.name)
//!     }
//!
//!     // Multiple extractors
//!     #[get("/{id}", extract(id = Path, filters = Query))]
//!     async fn get_user(id: u32, filters: SearchFilters) -> String {
//!         format!("User: {}", id)
//!     }
//! }
//! ```

use proc_macro::TokenStream;

#[macro_use]
mod logger;
mod controller;
mod generator;
mod parser;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
  controller::controller_impl(attr, item)
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
