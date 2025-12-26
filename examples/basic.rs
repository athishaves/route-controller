use route_controller::{
	controller, delete, get, post, put, patch
};
use axum::extract::Path;
use axum::{
  extract::{Request},
  middleware::{Next},
  response::{IntoResponse, Response},
};

pub async fn log(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
  println!("Logging from middleware!");
  Ok(next.run(request).await)
}

struct UserController;

#[controller(
	path = "/api/users",
	middleware = crate::log, // Either use absolute path or just 'log' if in the same crate
)]
impl UserController {
	// Default route '/' is considered when no path is provided
	#[get]
	async fn list() -> String {
		"User list".to_string()
	}

	// Route with path parameter
	#[get("/{id}")]
	async fn get_one(Path(id): Path<u32>) -> String {
		Self::fetch_user(id)
	}

	// This function won't be registered as a route
	fn fetch_user(id: u32) -> String {
		return format!("Getting user with id: {}", id)
	}

	#[post("/")]
	async fn create() -> String {
		"User created".to_string()
	}

	#[delete("/{id}")]
	async fn delete(Path(id): Path<u32>) -> String {
		format!("Deleted user with id: {}", id)
	}

	#[put("/{id}")]
	async fn update(Path(id): Path<u32>) -> String {
		format!("Updated user with id: {}", id)
	}

	#[patch("/{id}")]
	async fn patch(Path(id): Path<u32>) -> String {
		format!("Patched user with id: {}", id)
	}
}

struct HealthController;

#[controller]
impl HealthController {
	#[get("/health")]
	async fn health() -> &'static str {
		"OK"
	}

	#[get("/version")]
	async fn version() -> &'static str {
		"1.0.0"
	}
}

#[tokio::main]
async fn main() {
	let app = axum::Router::new()
		.merge(HealthController::router())
		.merge(UserController::router());

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3003")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3003");
	println!("Try:");
	println!("  GET  /health");
	println!("  GET  /version");
	println!("  GET  /api/users");
	println!("  GET  /api/users/1");
	println!("  POST /api/users");
	println!("  DELETE /api/users/1");
	println!("  PUT /api/users/1");
	println!("  PATCH /api/users/1");
	axum::serve(listener, app).await.unwrap();
}
