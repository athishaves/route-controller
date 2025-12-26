//! Path parameter extraction
//!
//! Run: cargo run --example 02_path_params
//! Test: curl http://localhost:3000/users/42

use route_controller::{controller, get};

struct UserController;

#[controller(path = "/users")]
impl UserController {
  // Single path parameter
  #[get("/{id}", extract(id = Path))]
  async fn get_user(id: u32) -> String {
    format!("User ID: {}", id)
  }

  // Multiple path parameters
  #[get("/{user_id}/posts/{post_id}", extract(user_id = Path, post_id = Path))]
  async fn get_post(user_id: u32, post_id: u32) -> String {
    format!("User {} - Post {}", user_id, post_id)
  }

  // String path parameter
  #[get("/{username}/profile", extract(username = Path))]
  async fn get_profile(username: String) -> String {
    format!("Profile for: {}", username)
  }
}

#[tokio::main]
async fn main() {
  let app = UserController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl http://localhost:3000/users/42");
  println!("  curl http://localhost:3000/users/1/posts/5");
  println!("  curl http://localhost:3000/users/john/profile");

  axum::serve(listener, app).await.unwrap();
}
