//! Query parameter extraction
//!
//! Run: cargo run --example 03_query_params
//! Test: curl "http://localhost:3000/search?q=rust&limit=10"

use route_controller::{controller, get};
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchQuery {
  q: String,
  limit: Option<u32>,
  sort: Option<String>,
}

struct SearchController;

#[controller]
impl SearchController {
  #[get("/search", extract(query = Query))]
  async fn search(query: SearchQuery) -> String {
    format!(
      "Search: '{}', limit: {}, sort: {}",
      query.q,
      query.limit.unwrap_or(20),
      query.sort.unwrap_or_else(|| "relevance".to_string())
    )
  }
}

#[tokio::main]
async fn main() {
  let app = SearchController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl 'http://localhost:3000/search?q=rust'");
  println!("  curl 'http://localhost:3000/search?q=rust&limit=10'");
  println!("  curl 'http://localhost:3000/search?q=rust&limit=10&sort=date'");

  axum::serve(listener, app).await.unwrap();
}
