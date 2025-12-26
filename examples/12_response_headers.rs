//! Response headers and content types
//!
//! Run: cargo run --example 12_response_headers
//! Test: curl -i http://localhost:3000/api/json

use route_controller::{controller, get, post};
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
  message: String,
}

struct ApiController;

#[controller(path = "/api")]
impl ApiController {
  // Single custom header
  #[get("/json", header("x-api-version", "1.0"))]
  async fn json_response() -> axum::Json<ApiResponse> {
    axum::Json(ApiResponse {
      message: "Response with custom header".to_string(),
    })
  }

  // Multiple custom headers
  #[get("/multi", header("x-api-version", "1.0"), header("x-request-id", "abc-123"))]
  async fn multi_headers() -> &'static str {
    "Response with multiple headers"
  }

  // Custom content type
  #[get("/xml", content_type("application/xml"))]
  async fn xml_response() -> String {
    r#"<?xml version="1.0"?><response><message>Hello XML</message></response>"#.to_string()
  }

  // Content type with custom headers
  #[post("/data", content_type("application/json"), header("x-api-version", "2.0"))]
  async fn data_with_headers() -> axum::Json<ApiResponse> {
    axum::Json(ApiResponse {
      message: "Data with content type and headers".to_string(),
    })
  }
}

#[tokio::main]
async fn main() {
  let app = ApiController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry (note -i to see headers):");
  println!("  curl -i http://localhost:3000/api/json");
  println!("  curl -i http://localhost:3000/api/multi");
  println!("  curl -i http://localhost:3000/api/xml");
  println!("  curl -i -X POST http://localhost:3000/api/data");

  axum::serve(listener, app).await.unwrap();
}
