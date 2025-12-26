//! Integration tests for custom response headers
//!
//! Tests response header customization

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct HeaderResponseController;

#[controller(path = "/api")]
impl HeaderResponseController {
  #[get("/custom", header("x-custom", "value"))]
  async fn custom_header() -> &'static str {
    "ok"
  }

  #[get(
    "/multiple",
    header("x-custom-1", "value1"),
    header("x-custom-2", "value2")
  )]
  async fn multiple_headers() -> &'static str {
    "ok"
  }

  #[get(
    "/content",
    content_type("application/custom")
  )]
  async fn custom_content_type() -> &'static str {
    "ok"
  }

  #[get(
    "/cors",
    header("x-cors-origin", "*"),
    header("x-cors-methods", "GET, POST")
  )]
  async fn cors_headers() -> &'static str {
    "ok"
  }

  #[get(
    "/cache",
    header("x-cache-ttl", "3600")
  )]
  async fn cache_control() -> &'static str {
    "ok"
  }

  #[get(
    "/users/{id}",
    extract(id = Path),
    header("x-user-id", "extracted")
  )]
  async fn with_extractor(id: u32) -> String {
    format!("user:{}", id)
  }
}

#[tokio::test]
async fn test_single_custom_header() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/custom")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("x-custom").unwrap(),
    "value"
  );
}

#[tokio::test]
async fn test_multiple_custom_headers() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/multiple")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("x-custom-1").unwrap(),
    "value1"
  );
  assert_eq!(
    response.headers().get("x-custom-2").unwrap(),
    "value2"
  );
}

#[tokio::test]
async fn test_custom_content_type() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/content")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/custom"
  );
}

#[tokio::test]
async fn test_cors_headers() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/cors")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Check that at least one of the custom headers is present
  assert!(
    response.headers().get("x-cors-origin").is_some() ||
    response.headers().get("x-cors-methods").is_some()
  );
}

#[tokio::test]
async fn test_cache_control() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/cache")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("x-cache-ttl").unwrap(),
    "3600"
  );
}

#[tokio::test]
async fn test_headers_with_extractor() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("x-user-id").unwrap(),
    "extracted"
  );
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:42");
}

#[tokio::test]
async fn test_header_values_preserved() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/custom")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let header_value = response.headers().get("x-custom").unwrap();
  assert_eq!(header_value.to_str().unwrap(), "value");
}
