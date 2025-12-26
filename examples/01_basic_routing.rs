//! Basic routing with different HTTP methods
//!
//! Run: cargo run --example 01_basic_routing
//! Test: curl http://localhost:3000/api/hello

use route_controller::{controller, delete, get, post, put};

struct ApiController;

#[controller(path = "/api")]
impl ApiController {
  #[get("/hello")]
  async fn hello() -> &'static str {
    "Hello, World!"
  }

  #[post("/create")]
  async fn create() -> &'static str {
    "Resource created"
  }

  #[put("/update")]
  async fn update() -> &'static str {
    "Resource updated"
  }

  #[delete("/delete")]
  async fn delete() -> &'static str {
    "Resource deleted"
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
  println!("  curl http://localhost:3000/api/hello");
  println!("  curl -X POST http://localhost:3000/api/create");
  println!("  curl -X PUT http://localhost:3000/api/update");
  println!("  curl -X DELETE http://localhost:3000/api/delete");

  axum::serve(listener, app).await.unwrap();
}
