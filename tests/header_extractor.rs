use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct ApiController;

#[controller(path = "/api")]
impl ApiController {
  #[get("/header", extract(authorization = HeaderParam))]
  async fn with_header(authorization: String) -> String {
    format!("Auth: {}", authorization)
  }

  #[get(
    "/multiple-headers",
    extract(
      authorization = HeaderParam,
      api_key = HeaderParam,
    )
  )]
  async fn with_multiple_headers(authorization: String, api_key: String) -> String {
    format!("Auth: {}, API Key: {}", authorization, api_key)
  }

  #[get(
    "/{id}/mixed",
    extract(
      id = Path,
      content_type = HeaderParam,
    )
  )]
  async fn mixed_extractors(id: u32, content_type: String) -> String {
    format!("ID: {}, Content-Type: {}", id, content_type)
  }
}

#[tokio::test]
async fn test_single_header_extraction() {
  let app = ApiController::router();

  let request = Request::builder()
    .uri("/api/header")
    .header("authorization", "Bearer token123")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Auth: Bearer token123");
}

#[tokio::test]
async fn test_missing_header() {
  let app = ApiController::router();

  let request = Request::builder()
    .uri("/api/header")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  // Should default to empty string
  assert_eq!(body_str, "Auth: ");
}

#[tokio::test]
async fn test_multiple_headers() {
  let app = ApiController::router();

  let request = Request::builder()
    .uri("/api/multiple-headers")
    .header("authorization", "Bearer abc123")
    .header("api-key", "secret456")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Auth: Bearer abc123, API Key: secret456");
}

#[tokio::test]
async fn test_mixed_path_and_header() {
  let app = ApiController::router();

  let request = Request::builder()
    .uri("/api/42/mixed")
    .header("content-type", "application/json")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "ID: 42, Content-Type: application/json");
}
