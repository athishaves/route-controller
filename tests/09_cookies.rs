//! Integration tests for CookieParam extractor (requires 'cookies' feature)
//!
//! Tests cookie extraction

#![cfg(feature = "cookies")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct CookieController;

#[controller(path = "/api")]
impl CookieController {
  #[get("/profile", extract(session_id = CookieParam))]
  async fn profile(session_id: String) -> String {
    format!("session:{}", session_id)
  }

  #[get("/user", extract(user_id = CookieParam, session_id = CookieParam))]
  async fn user(user_id: String, session_id: String) -> String {
    format!("user:{},session:{}", user_id, session_id)
  }

  #[get("/users/{id}", extract(id = Path, session_id = CookieParam))]
  async fn user_profile(id: u32, session_id: String) -> String {
    format!("id:{},session:{}", id, session_id)
  }
}

#[tokio::test]
async fn test_single_cookie() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .header("cookie", "session_id=abc123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"session:abc123");
}

#[tokio::test]
async fn test_missing_cookie() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"session:");
}

#[tokio::test]
async fn test_multiple_cookies() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/user")
        .header("cookie", "user_id=42; session_id=xyz789")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:42,session:xyz789");
}

#[tokio::test]
async fn test_path_and_cookie() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/5")
        .header("cookie", "session_id=test123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:5,session:test123");
}

#[tokio::test]
async fn test_cookie_with_equals_in_value() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .header("cookie", "session_id=value=with=equals")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_url_encoded_cookie() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .header("cookie", "session_id=abc%20123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_extra_cookies_ignored() {
  let app = CookieController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .header("cookie", "session_id=abc; other=value; another=data")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"session:abc");
}
