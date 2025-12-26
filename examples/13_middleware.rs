//! Middleware application
//!
//! Run: cargo run --example 13_middleware
//! Test: curl http://localhost:3000/api/hello

use axum::{
  extract::Request,
  middleware::Next,
  response::Response,
};
use route_controller::{controller, get};

// Simple logging middleware
async fn logging_middleware(request: Request, next: Next) -> Response {
  println!("Request: {} {}", request.method(), request.uri());
  next.run(request).await
}

struct ApiController;

#[controller(path = "/api", middleware = logging_middleware)]
impl ApiController {
  #[get("/hello")]
  async fn hello() -> &'static str {
    "Hello with middleware"
  }

  #[get("/world")]
  async fn world() -> &'static str {
    "World with middleware"
  }
}

#[tokio::main]
async fn main() {
  let app = ApiController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry (see middleware logs):");
  println!("  curl http://localhost:3000/api/hello");
  println!("  curl http://localhost:3000/api/world");

  axum::serve(listener, app).await.unwrap();
}
