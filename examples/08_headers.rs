//! Header extraction (requires 'headers' feature)
//!
//! Run: cargo run --example 08_headers --features headers
//! Test: curl http://localhost:3000/api/protected -H "authorization: Bearer token123"

use route_controller::{controller, get};

struct ApiController;

#[controller(path = "/api")]
impl ApiController {
  #[get("/protected", extract(authorization = HeaderParam))]
  async fn protected(authorization: String) -> String {
    format!("Authorization: {}", authorization)
  }

  #[get("/info", extract(user_agent = HeaderParam, authorization = HeaderParam))]
  async fn info(user_agent: String, authorization: String) -> String {
    format!("User-Agent: {}, Auth: {}", user_agent, authorization)
  }
}

#[tokio::main]
async fn main() {
  let app = ApiController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl http://localhost:3000/api/protected -H 'authorization: Bearer token123'");
  println!("  curl http://localhost:3000/api/info -H 'authorization: Bearer token123' -H 'user-agent: MyApp/1.0'");

  axum::serve(listener, app).await.unwrap();
}
