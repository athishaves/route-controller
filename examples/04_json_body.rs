//! JSON body extraction
//!
//! Run: cargo run --example 04_json_body
//! Test: curl -X POST http://localhost:3000/users -H "Content-Type: application/json" -d '{"name":"Alice","email":"alice@example.com"}'

use route_controller::{controller, post, put};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
  name: String,
  email: String,
}

struct UserController;

#[controller(path = "/users")]
impl UserController {
  #[post(extract(user = Json))]
  async fn create(user: User) -> axum::Json<User> {
    println!("Created: {} - {}", user.name, user.email);
    axum::Json(user)
  }

  #[put("/{id}", extract(id = Path, user = Json))]
  async fn update(id: u32, user: User) -> String {
    format!("Updated user {}: {} - {}", id, user.name, user.email)
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
  println!(
    "  curl -X POST http://localhost:3000/users -H 'Content-Type: application/json' -d '{{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}'"
  );
  println!(
    "  curl -X PUT http://localhost:3000/users/1 -H 'Content-Type: application/json' -d '{{\"name\":\"Bob\",\"email\":\"bob@example.com\"}}'"
  );

  axum::serve(listener, app).await.unwrap();
}
