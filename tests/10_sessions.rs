//! Integration tests for SessionParam extractor (requires 'sessions' feature)
//!
//! Tests session data extraction

#![cfg(feature = "sessions")]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, put};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, SessionManagerLayer};

struct SessionController;

#[controller(path = "/api")]
impl SessionController {
  #[put("/init")]
  async fn init(session: tower_sessions::Session) -> &'static str {
    let _ = session.insert("user_id", "123".to_string()).await;
    let _ = session.insert("username", "alice".to_string()).await;
    "ok"
  }

  #[get("/profile", extract(user_id = SessionParam))]
  async fn profile(user_id: String) -> String {
    format!("user:{}", user_id)
  }

  #[get("/info", extract(user_id = SessionParam, username = SessionParam))]
  async fn info(user_id: String, username: String) -> String {
    format!("user:{},name:{}", user_id, username)
  }

  #[get("/users/{id}", extract(id = Path, user_id = SessionParam))]
  async fn user_session(id: u32, user_id: String) -> String {
    format!("id:{},session:{}", id, user_id)
  }
}

#[tokio::test]
async fn test_session_extraction() {
  let session_store = MemoryStore::default();
  let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
  let app = SessionController::router().layer(session_layer);

  // Initialize session
  let init_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/api/init")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(init_response.status(), StatusCode::OK);

  // Extract cookie from response
  let cookie = init_response
    .headers()
    .get("set-cookie")
    .and_then(|h| h.to_str().ok())
    .unwrap();

  // Use session cookie
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_session_data() {
  let session_store = MemoryStore::default();
  let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
  let app = SessionController::router().layer(session_layer);

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
}

#[tokio::test]
async fn test_multiple_session_params() {
  let session_store = MemoryStore::default();
  let session_layer = SessionManagerLayer::new(session_store).with_secure(false);
  let app = SessionController::router().layer(session_layer);

  // Initialize
  let init_response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/api/init")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let cookie = init_response
    .headers()
    .get("set-cookie")
    .and_then(|h| h.to_str().ok())
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/info")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}
