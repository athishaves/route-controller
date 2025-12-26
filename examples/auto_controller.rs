use route_controller::{auto_controller, post, put};
use axum::extract::Path;
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
	name: String,
	email: String,
}

struct ApiController;

// Auto controller - automatically wraps plain types with Json<T>
#[auto_controller(path = "/api/users")]
impl ApiController {
	// Plain type automatically wrapped with Json<User>
	#[post]
	async fn create(user: User) -> String {
		format!("User created: {} ({})", user.name, user.email)
	}

	// Mixed: Path extractor (preserved) + plain type (auto-wrapped with Json<T>)
	#[put("/{id}")]
	async fn update(Path(id): Path<u32>, user: User) -> String {
		format!("Updated user {}: {} ({})", id, user.name, user.email)
	}
}

#[tokio::main]
async fn main() {
	let app = ApiController::router();

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3006")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3006");

	axum::serve(listener, app).await.unwrap();
}
