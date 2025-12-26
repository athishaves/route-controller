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
//!
//!     // Header, Cookie, and Session extractors (require features: headers, cookies, sessions)
//!     #[get(
//!         "/profile",
//!         extract(
//!             authorization = HeaderParam,
//!             session_id = CookieParam,
//!             user_id = SessionParam
//!         )
//!     )]
//!     async fn get_profile(
//!         authorization: String,
//!         session_id: String,
//!         user_id: String
//!     ) -> String {
//!         format!("Profile for user: {}", user_id)
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `headers` - Enable HeaderParam extractor (extracts from request headers)
//! - `cookies` - Enable CookieParam extractor (requires axum-extra with cookie feature)
//! - `sessions` - Enable SessionParam extractor (requires tower-sessions)
//! - `verbose-logging` - Enable verbose logging during macro expansion
//!
//! ## Extractor Requirements
//!
//! ### HeaderParam
//! Extracts values from HTTP headers. Header names with underscores are automatically
//! converted to kebab-case (e.g., `content_type` becomes `content-type`).
//! No additional dependencies required.
//!
//! ### CookieParam
//! Extracts values from cookies. Requires adding `axum-extra` with the `cookie` feature:
//! ```toml
//! axum-extra = { version = "0.12", features = ["cookie"] }
//! ```
//!
//! ### SessionParam
//! Extracts values from session storage. Requires adding `tower-sessions` and configuring
//! a session layer. The SessionParam extractor requires async operations and may need
//! additional setup:
//! ```toml
//! tower-sessions = "0.13"
//! ```
//!
//! Note: SessionParam requires the session middleware layer to be applied to your router
//! and may have limitations in some scenarios. Refer to tower-sessions documentation
//! for proper setup.

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
