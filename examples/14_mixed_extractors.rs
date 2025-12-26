//! Mixed extractors - combining multiple extractor types
//!
//! Run: cargo run --example 14_mixed_extractors
//! Test: curl "http://localhost:3000/users/42?sort=name" -H "authorization: Bearer token"

use route_controller::{controller, get, put};
use serde::Deserialize;

#[derive(Deserialize)]
struct QueryParams {
  sort: Option<String>,
  limit: Option<u32>,
}

#[derive(Deserialize)]
struct User {
  name: String,
}

struct UserController;

#[controller(path = "/users")]
impl UserController {
  // Path + Query
  #[get("/{id}", extract(id = Path, params = Query))]
  async fn get_user(id: u32, params: QueryParams) -> String {
    format!(
      "User {}, sort: {}, limit: {}",
      id,
      params.sort.unwrap_or_else(|| "id".to_string()),
      params.limit.unwrap_or(10)
    )
  }

  // Path + Json
  #[put("/{id}", extract(id = Path, user = Json))]
  async fn update_user(id: u32, user: User) -> String {
    format!("Updated user {}: {}", id, user.name)
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
  println!("  curl 'http://localhost:3000/users/42?sort=name&limit=5'");
  println!("  curl -X PUT http://localhost:3000/users/42 -H 'Content-Type: application/json' -d '{{\"name\":\"Alice\"}}'");

  axum::serve(listener, app).await.unwrap();
}
