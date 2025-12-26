//! # Route Controller
//!
//! Generate Axum routers from controller-style implementations with declarative extractors.
//!
//! ## Controller Example
//!
//! ```ignore
//! use route_controller::{controller, post, get};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize, Serialize)]
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! struct UserController;
//!
//! #[controller(path = "/users")]
//! impl UserController {
//!     // List all users
//!     #[get]
//!     async fn list() -> &'static str {
//!         "User list"
//!     }
//!
//!     // Get a single user by ID
//!     #[get("/{id}", extract(id = Path))]
//!     async fn get_one(id: u32) -> axum::Json<User> {
//!         let user = User {
//!             name: format!("User{}", id),
//!             email: format!("user{}@example.com", id),
//!         };
//!         axum::Json(user)
//!     }
//!
//!     // Create a new user
//!     #[post("/", extract(user = Json))]
//!     async fn create(user: User) -> String {
//!         format!("Created user: {} ({})", user.name, user.email)
//!     }
//!
//!     // Multiple extractors: Path + Query
//!     #[get("/{id}/search", extract(id = Path, filters = Query))]
//!     async fn search(id: u32, filters: SearchFilters) -> String {
//!         format!("Searching for user {}", id)
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
//!
//! // Use the controller
//! #[tokio::main]
//! async fn main() {
//!     let app = UserController::router();
//!
//!     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//!         .await
//!         .unwrap();
//!
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! - **Built-in extractors**: `Path`, `Query`, `Json`, `Form`, `Bytes`, `Text`, `Html`, `Xml`, `JavaScript`, `State`
//! - **Response headers**: `header()` and `content_type()` attributes for custom response headers
//! - **Middleware support**: Apply middleware at the controller level
//! - **Feature-gated extractors**:
//!   - `headers` - Enable `HeaderParam` extractor (extracts from request headers)
//!   - `cookies` - Enable `CookieParam` extractor (requires axum-extra with cookie feature)
//!   - `sessions` - Enable `SessionParam` extractor (requires tower-sessions)
//!   - `verbose-logging` - Enable verbose logging during macro expansion
//!
//! ## Extractor Types
//!
//! ### Request Body Extractors
//! - **`Json`** - Extract JSON request body: `extract(data = Json)`
//! - **`Form`** - Extract form data: `extract(data = Form)`
//! - **`Bytes`** - Extract raw binary data: `extract(data = Bytes)` → `Vec<u8>`
//! - **`Text`** - Extract plain text: `extract(content = Text)` → `String`
//! - **`Html`** - Extract HTML content: `extract(content = Html)` → `String`
//! - **`Xml`** - Extract XML content: `extract(content = Xml)` → `String`
//! - **`JavaScript`** - Extract JavaScript content: `extract(code = JavaScript)` → `String`
//!
//! ### URL Extractors
//! - **`Path`** - Extract path parameters: `extract(id = Path)`
//! - **`Query`** - Extract query parameters: `extract(params = Query)`
//!
//! ### Other Extractors
//! - **`State`** - Extract application state: `extract(state = State)`
//!
//! ### Feature-Gated Extractors
//!
//! #### HeaderParam (requires `headers` feature)
//! Extracts values from HTTP headers. Header names with underscores are automatically
//! converted to kebab-case (e.g., `user_agent` becomes `user-agent`).
//! No additional dependencies required.
//!
//! #### CookieParam (requires `cookies` feature)
//! Extracts values from cookies. Requires adding `axum-extra` with the `cookie` feature:
//! ```toml
//! axum-extra = { version = "0.12", features = ["cookie"] }
//! ```
//!
//! #### SessionParam (requires `sessions` feature)
//! Extracts values from session storage. Requires adding `tower-sessions` and configuring
//! a session layer:
//! ```toml
//! tower-sessions = "0.14"
//! ```
//!
//! Note: SessionParam requires the session middleware layer to be applied to your router.
//! Refer to tower-sessions documentation for proper setup.
//!
//! ## Response Headers
//!
//! Add custom headers to your responses:
//!
//! ```ignore
//! #[get("/data", header("x-api-version", "1.0"))]
//! async fn get_data() -> String {
//!     "Data with custom header".to_string()
//! }
//!
//! // Multiple headers
//! #[get(
//!     "/info",
//!     header("x-api-version", "2.0"),
//!     header("x-request-id", "abc-123")
//! )]
//! async fn get_info() -> String {
//!     "Info with multiple headers".to_string()
//! }
//!
//! // Custom content type
//! #[get("/xml", content_type("application/xml"))]
//! async fn get_xml() -> String {
//!     r#"<?xml version="1.0"?><response>Hello</response>"#.to_string()
//! }
//! ```
//!
//! ## Middleware
//!
//! Apply middleware at the controller level:
//!
//! ```ignore
//! async fn log_middleware(request: Request, next: Next) -> Response {
//!     println!("Request: {} {}", request.method(), request.uri());
//!     next.run(request).await
//! }
//!
//! #[controller(path = "/api", middleware = log_middleware)]
//! impl ApiController {
//!     #[get("/data")]
//!     async fn get_data() -> String {
//!         "Protected data".to_string()
//!     }
//! }
//! ```
//!
//! ## Examples
//!
//! The crate includes comprehensive examples demonstrating different features:
//!
//! ```bash
//! # Basic routing with different HTTP methods
//! cargo run --example 01_basic_routing
//!
//! # Path parameters
//! cargo run --example 02_path_params
//!
//! # Query parameters
//! cargo run --example 03_query_params
//!
//! # JSON body extraction
//! cargo run --example 04_json_body
//!
//! # Form data handling
//! cargo run --example 05_form_data
//!
//! # Text body extraction
//! cargo run --example 06_text_body
//!
//! # Binary data (bytes)
//! cargo run --example 07_bytes
//!
//! # Header extraction (requires 'headers' feature)
//! cargo run --example 08_headers --features headers
//!
//! # Cookie handling (requires 'cookies' feature)
//! cargo run --example 09_cookies --features cookies
//!
//! # Session management (requires 'sessions' feature)
//! cargo run --example 10_sessions --features sessions
//!
//! # Application state
//! cargo run --example 11_state
//!
//! # Response headers
//! cargo run --example 12_response_headers
//!
//! # Middleware
//! cargo run --example 13_middleware
//!
//! # Mixed extractors (Path + Query + Json)
//! cargo run --example 14_mixed_extractors
//!
//! # Multiple controllers
//! cargo run --example 15_multiple_controllers
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
pub fn head(_: TokenStream, item: TokenStream) -> TokenStream {
  item
}

#[proc_macro_attribute]
pub fn delete(_: TokenStream, item: TokenStream) -> TokenStream {
  item
}

#[proc_macro_attribute]
pub fn options(_: TokenStream, item: TokenStream) -> TokenStream {
  item
}

#[proc_macro_attribute]
pub fn patch(_: TokenStream, item: TokenStream) -> TokenStream {
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
pub fn trace(_: TokenStream, item: TokenStream) -> TokenStream {
  item
}
