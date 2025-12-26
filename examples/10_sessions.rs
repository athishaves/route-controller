//! Session extraction (requires 'sessions' feature)
//!
//! Run: cargo run --example 10_sessions --features sessions
//! Test: See curl commands in output

use route_controller::{controller, get, put};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

struct UserController;

#[controller(path = "/users")]
impl UserController {
  // Initialize session data
  #[put("/session/init")]
  async fn init_session(session: Session) -> &'static str {
    let _ = session.insert("user_id", "12345".to_string()).await;
    let _ = session.insert("username", "john_doe".to_string()).await;
    "Session initialized"
  }

  // Extract single session value
  #[get("/profile", extract(user_id = SessionParam))]
  async fn profile(user_id: String) -> String {
    format!("User ID: {}", user_id)
  }

  // Extract multiple session values
  #[get("/info", extract(user_id = SessionParam, username = SessionParam))]
  async fn info(user_id: String, username: String) -> String {
    format!("ID: {}, Username: {}", user_id, username)
  }
}

#[tokio::main]
async fn main() {
  let session_store = MemoryStore::default();
  let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

  let app = UserController::router().layer(session_layer);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl -X PUT http://localhost:3000/users/session/init -c cookies.txt");
  println!("  curl http://localhost:3000/users/profile -b cookies.txt");
  println!("  curl http://localhost:3000/users/info -b cookies.txt");

  axum::serve(listener, app).await.unwrap();
}
