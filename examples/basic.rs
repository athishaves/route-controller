use route_controller::{auto_controller, get, post};
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};

// Deserialize: Required for auto_controller to wrap input parameters with Json<T>
// Serialize: Required when returning Json<T> in handler responses
#[derive(Deserialize, Serialize)]
struct User {
	name: String,
	email: String,
}

struct UserController;

#[auto_controller(path = "/users")]
impl UserController {
	#[get]
	async fn list() -> &'static str {
		"User list"
	}

	#[get("/{id}")]
	async fn get_one(Path(id): Path<u32>) -> Json<User> {
		let dummy_user = User {
			name: format!("User{}", id),
			email: format!("user{}@example.com", id),
		};
		Json(dummy_user)
	}

	#[post]
	async fn create(user: User) -> String {
		format!("Created user: {} ({})", user.name, user.email)
	}
}

#[tokio::main]
async fn main() {
	let app = UserController::router();

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3003")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3003");

	axum::serve(listener, app).await.unwrap();
}
