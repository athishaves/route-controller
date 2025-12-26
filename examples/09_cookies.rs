//! Cookie extraction (requires 'cookies' feature)
//!
//! Run: cargo run --example 09_cookies --features cookies
//! Test: curl http://localhost:3000/profile -H "Cookie: session_id=abc123; username=john"

use route_controller::{controller, get};

struct ProfileController;

#[controller]
impl ProfileController {
  #[get("/profile", extract(session_id = CookieParam))]
  async fn profile(session_id: String) -> String {
    format!("Session ID: {}", session_id)
  }

  #[get("/user", extract(username = CookieParam, session_id = CookieParam))]
  async fn user_info(username: String, session_id: String) -> String {
    format!("User: {}, Session: {}", username, session_id)
  }
}

#[tokio::main]
async fn main() {
  let app = ProfileController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl http://localhost:3000/profile -H 'Cookie: session_id=abc123'");
  println!("  curl http://localhost:3000/user -H 'Cookie: session_id=abc123; username=john'");

  axum::serve(listener, app).await.unwrap();
}
