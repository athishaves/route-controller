use route_controller::{
	auto_controller, get, post
};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginData {
	username: String,
	_password: String,
}

#[derive(Deserialize)]
struct UserData {
	name: String,
	email: String,
	age: u32,
}

struct ApiController;

#[auto_controller(path = "/api")]
impl ApiController {
	#[get("/health")]
	async fn health() -> &'static str {
		"OK"
	}

	// JSON endpoint - default behavior
	#[post("/users")]
	async fn create_user_json(user: UserData) -> String {
		format!("Created user (JSON): {} - {} (age: {})", user.name, user.email, user.age)
	}

	// Form data endpoint - using content_type = "form"
	#[post("/login", content_type = "form")]
	async fn login_form(credentials: LoginData) -> String {
		format!("Login attempt (FORM): username={}", credentials.username)
	}

	// Another form endpoint
	#[post("/register", content_type = "form")]
	async fn register_form(user: UserData) -> String {
		format!("Registration (FORM): {} - {}", user.name, user.email)
	}
}

#[tokio::main]
async fn main() {
	let app = ApiController::router();

	let listener = tokio::net::TcpListener::bind("127.0.0.1:3007")
		.await
		.unwrap();

	println!("Server running on http://127.0.0.1:3007");

	axum::serve(listener, app).await.unwrap();
}
