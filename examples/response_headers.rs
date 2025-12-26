use route_controller::{controller, get, post};
use serde::Serialize;

#[derive(Serialize)]
struct ApiResponse {
  message: String,
  timestamp: u64,
}

struct HeaderController;

#[controller(path = "/api")]
impl HeaderController {
  // Single header
  #[get("/json", header("x-api-version", "1.0"))]
  async fn json_with_header() -> axum::Json<ApiResponse> {
    axum::Json(ApiResponse {
      message: "Hello with custom header".to_string(),
      timestamp: 1234567890,
    })
  }

  // Multiple headers
  #[get(
    "/multi",
    header("x-api-version", "1.0"),
    header("x-request-id", "abc-123")
  )]
  async fn multiple_headers() -> &'static str {
    "Response with multiple headers"
  }

  // Content-Type header
  #[get("/xml", content_type("application/xml"))]
  async fn xml_response() -> String {
    r#"<?xml version="1.0"?>
<response>
  <message>Hello XML</message>
</response>"#
      .to_string()
  }

  // Content-Type with custom headers
  #[post(
    "/data",
    content_type("application/json"),
    header("x-api-version", "2.0"),
    header("x-rate-limit", "100")
  )]
  async fn content_type_with_headers() -> axum::Json<ApiResponse> {
    axum::Json(ApiResponse {
      message: "Data with content type and headers".to_string(),
      timestamp: 9876543210,
    })
  }

  // Plain text with content type
  #[get("/text", content_type("text/plain; charset=utf-8"))]
  async fn plain_text() -> String {
    "Plain text response with explicit content type".to_string()
  }

  // HTML response
  #[get("/html", content_type("text/html"))]
  async fn html_response() -> String {
    r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello from route_controller!</h1></body>
</html>"#
      .to_string()
  }
}

#[tokio::main]
async fn main() {
  let app = HeaderController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3006")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3006");
  println!("\nTry these endpoints:");
  println!("  GET /api/json         - JSON with custom header");
  println!("  GET /api/multi        - Multiple custom headers");
  println!("  GET /api/xml          - XML with content-type");
  println!("  POST /api/data        - JSON with content-type and headers");
  println!("  GET /api/text         - Plain text with content-type");
  println!("  GET /api/html         - HTML with content-type");

  axum::serve(listener, app).await.unwrap();
}
