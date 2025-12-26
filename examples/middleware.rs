use route_controller::{auto_controller, get, post};
use axum::{
	extract::{Request, Path},
	middleware::Next,
	response::{IntoResponse, Response},
	Json,
};
use serde::{Deserialize, Serialize};

// Logging middleware
pub async fn log_middleware(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
	println!("→ {} {}", request.method(), request.uri());
	let response = next.run(request).await;
	println!("← Response sent");
	Ok(response)
}

// Authentication middleware
pub async fn auth_middleware(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
	// Simple auth check - in real app, validate token/session
	if let Some(auth_header) = request.headers().get("authorization") {
		if auth_header == "Bearer secret-token" {
			println!("✓ Authentication successful");
			Ok(next.run(request).await)
		} else {
			println!("✗ Invalid token");
			Err(axum::http::StatusCode::UNAUTHORIZED.into_response())
		}
	} else {
		println!("✗ Missing authorization header");
		Err(axum::http::StatusCode::UNAUTHORIZED.into_response())
	}
}

#[derive(Deserialize, Serialize)]
struct User {
	name: String,
	email: String,
}

// Public controller - no middleware
struct PublicController;

#[auto_controller(path = "/public")]
impl PublicController {
	#[get("/health")]
	async fn health() -> &'static str {
		"Public health check"
	}
}

// Protected controller - with auth middleware
struct ProtectedController;

#[auto_controller(path = "/api", middleware = crate::auth_middleware)]
impl ProtectedController {
	#[get("/users")]
	async fn list_users() -> &'static str {
		"Protected user list"
	}

	#[post("/users")]
	async fn create_user(user: User) -> String {
		format!("Created protected user: {}", user.name)
	}
}

// Admin controller - with both logging and auth middleware
struct AdminController;

#[auto_controller(path = "/admin", middleware = crate::log_middleware)]
impl AdminController {
	#[get("/dashboard")]
	async fn dashboard() -> &'static str {
		"Admin dashboard (logged)"
	}

	#[get("/users/{id}")]
	async fn get_user(Path(id): Path<u32>) -> Json<User> {
		Json(User {
			name: format!("Admin User {}", id),
			email: format!("admin{}@example.com", id),
		})
	}
}

#[tokio::main]
async fn main() {
	let app = axum::Router::new()
		.merge(PublicController::router())
		.merge(ProtectedController::router())
		.merge(AdminController::router());

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3008")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3008");

	axum::serve(listener, app).await.unwrap();
}
