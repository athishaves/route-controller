//! # Route Controller
//!
//! Generate Axum routers from controller-style implementations.
//!
//! ## Controllers
//!
//! Two types of controllers are available:
//!
//! - `#[controller]` - Standard controller (requires explicit extractors like `Json<T>`, `Form<T>`)
//! - `#[auto_controller]` - Auto controller (automatically wraps plain types with `Json<T>` or `Form<T>`)
//!
//! ## Standard Controller Example
//!
//! ```ignore
//! use route_controller::{controller, post};
//! use axum::Json;
//!
//! #[derive(Deserialize)]
//! struct User {
//!     name: String,
//! }
//!
//! #[controller(path = "/users")]
//! impl UserController {
//!     #[post("/")]
//!     async fn create(Json(user): Json<User>) -> String {
//!         format!("Created: {}", user.name)
//!     }
//! }
//! ```
//!
//! ## Auto Controller Example
//!
//! ```ignore
//! use route_controller::{auto_controller, post};
//!
//! #[derive(Deserialize)]
//! struct User {
//!     name: String,
//! }
//!
//! #[auto_controller(path = "/users")]
//! impl UserController {
//!     // Plain types automatically wrapped with Json<T> by default
//!     #[post("/")]
//!     async fn create(user: User) -> String {
//!         format!("Created: {}", user.name)
//!     }
//!
//!     // Use content_type = "form" for form data
//!     #[post("/register", content_type = "form")]
//!     async fn register(user: User) -> String {
//!         format!("Registered: {}", user.name)
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
	controller::controller_impl(attr, item, false)
}

#[proc_macro_attribute]
pub fn auto_controller(attr: TokenStream, item: TokenStream) -> TokenStream {
	controller::controller_impl(attr, item, true)
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
