//! Multiple controllers with merged routers
//!
//! Run: cargo run --example 15_multiple_controllers
//! Test: curl http://localhost:3000/users and curl http://localhost:3000/posts

use route_controller::{controller, get};

struct UserController;

#[controller(path = "/users")]
impl UserController {
  #[get]
  async fn list() -> &'static str {
    "User list"
  }

  #[get("/{id}", extract(id = Path))]
  async fn get(id: u32) -> String {
    format!("User {}", id)
  }
}

struct PostController;

#[controller(path = "/posts")]
impl PostController {
  #[get]
  async fn list() -> &'static str {
    "Post list"
  }

  #[get("/{id}", extract(id = Path))]
  async fn get(id: u32) -> String {
    format!("Post {}", id)
  }
}

#[tokio::main]
async fn main() {
  // Merge multiple controllers
  let app = axum::Router::new()
    .merge(UserController::router())
    .merge(PostController::router());

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl http://localhost:3000/users");
  println!("  curl http://localhost:3000/users/1");
  println!("  curl http://localhost:3000/posts");
  println!("  curl http://localhost:3000/posts/1");

  axum::serve(listener, app).await.unwrap();
}
