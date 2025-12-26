//! Form data extraction
//!
//! Run: cargo run --example 05_form_data
//! Test: curl -X POST http://localhost:3000/login -d "username=john&password=secret123"

use route_controller::{controller, post};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginForm {
  username: String,
  password: String,
}

struct AuthController;

#[controller]
impl AuthController {
  #[post("/login", extract(form = Form))]
  async fn login(form: LoginForm) -> String {
    format!("Login attempt: username={}, password={}", form.username, form.password)
  }
}

#[tokio::main]
async fn main() {
  let app = AuthController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl -X POST http://localhost:3000/login -d 'username=john&password=secret123'");

  axum::serve(listener, app).await.unwrap();
}
