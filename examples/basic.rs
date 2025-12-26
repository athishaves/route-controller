use route_controller::{controller, get, post};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
	name: String,
	email: String,
}

#[derive(Deserialize)]
struct UserQuery {
	id: u32,
	_name: Option<String>,
}

struct UserController;

#[controller(path = "/users")]
impl UserController {
	#[get]
	async fn list() -> &'static str {
		"User list"
	}

	#[get("/{id}", extract(id = Path))]
	async fn get_one(id: u32) -> Json<User> {
		let dummy_user = User {
			name: format!("User{}", id),
			email: format!("user{}@example.com", id),
		};
		Json(dummy_user)
	}

	#[post("/", extract(user = Json))]
	async fn create(user: User) -> String {
		format!("Created user: {} ({})", user.name, user.email)
	}

	#[get("/info", extract(params = Query))]
	async fn get_user_info(params: UserQuery) -> Json<User> {
		let dummy_user = User {
			name: format!("User{}", params.id),
			email: format!("user{}@example.com", params.id),
		};
		Json(dummy_user)
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
