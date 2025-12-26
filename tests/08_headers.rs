//! Integration tests for HeaderParam extractor (requires 'headers' feature)
//!
//! Tests HTTP header extraction

#![cfg(feature = "headers")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct HeaderController;

#[controller(path = "/api")]
impl HeaderController {
  #[get("/auth", extract(authorization = HeaderParam))]
  async fn auth(authorization: String) -> String {
    format!("auth:{}", authorization)
  }

  #[get("/multi", extract(authorization = HeaderParam, user_agent = HeaderParam))]
  async fn multi(authorization: String, user_agent: String) -> String {
    format!("auth:{},ua:{}", authorization, user_agent)
  }

  #[get("/users/{id}", extract(id = Path, authorization = HeaderParam))]
  async fn user_auth(id: u32, authorization: String) -> String {
    format!("id:{},auth:{}", id, authorization)
  }

  #[get("/content", extract(content_type = HeaderParam))]
  async fn content(content_type: String) -> String {
    format!("ct:{}", content_type)
  }
}

#[tokio::test]
async fn test_single_header() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/auth")
        .header("authorization", "Bearer token123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"auth:Bearer token123");
}

#[tokio::test]
async fn test_missing_header() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/auth")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"auth:");
}

#[tokio::test]
async fn test_multiple_headers() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/multi")
        .header("authorization", "Bearer abc")
        .header("user-agent", "TestClient/1.0")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"auth:Bearer abc,ua:TestClient/1.0");
}

#[tokio::test]
async fn test_path_and_header() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/42")
        .header("authorization", "Bearer xyz")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:42,auth:Bearer xyz");
}

#[tokio::test]
async fn test_snake_case_to_kebab_case() {
  // content_type parameter should match content-type header
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/content")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"ct:application/json");
}

#[tokio::test]
async fn test_empty_header_value() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/auth")
        .header("authorization", "")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_special_chars_in_header() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/auth")
        .header("authorization", "Bearer token-with-special_chars.123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_case_insensitive_header() {
  let app = HeaderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/auth")
        .header("Authorization", "Bearer case-test")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"auth:Bearer case-test");
}
