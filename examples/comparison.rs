use route_controller::{controller, auto_controller, post};
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
	name: String,
	email: String,
}

// Standard controller - requires explicit Json<T> extractors
struct StandardController;

#[controller(path = "/standard")]
impl StandardController {
	// Must explicitly use Json<User> extractor
	#[post("/users")]
	async fn create_user(Json(user): Json<User>) -> String {
		format!("Standard: {} ({})", user.name, user.email)
	}
}

// Auto controller - automatically wraps plain types with Json<T>
struct AutoController;

#[auto_controller(path = "/auto")]
impl AutoController {
	// Plain type automatically wrapped with Json<User>
	#[post("/users")]
	async fn create_user(user: User) -> String {
		format!("Auto: {} ({})", user.name, user.email)
	}
}

#[tokio::main]
async fn main() {
	let app = axum::Router::new()
		.merge(StandardController::router())
		.merge(AutoController::router());

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3004")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3004");

	axum::serve(listener, app).await.unwrap();
}
