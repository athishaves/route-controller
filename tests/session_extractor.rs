use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

struct SessionController;

#[controller(path = "/sessions")]
impl SessionController {
  #[get("/user", extract(user_id = SessionParam))]
  async fn with_session(user_id: String) -> String {
    format!("User ID: {}", user_id)
  }

  #[get(
    "/data",
    extract(
      username = SessionParam,
      role = SessionParam,
    )
  )]
  async fn with_multiple_session_params(username: String, role: String) -> String {
    format!("Username: {}, Role: {}", username, role)
  }

  #[get(
    "/{id}/profile",
    extract(
      id = Path,
      user_name = SessionParam,
    )
  )]
  async fn mixed_extractors(id: u32, user_name: String) -> String {
    format!("ID: {}, Username: {}", id, user_name)
  }

  #[get(
    "/{id}/secure",
    extract(
      id = Path,
      user_id = SessionParam,
      api_key = HeaderParam,
    )
  )]
  async fn mixed_with_header(id: u32, user_id: String, api_key: String) -> String {
    format!("ID: {}, User: {}, API Key: {}", id, user_id, api_key)
  }

  // Helper endpoint to set session data for testing
  #[get("/set")]
  async fn set_session(session: Session) -> String {
    let _ = session.insert("user_id", "12345".to_string()).await;
    let _ = session.insert("username", "testuser".to_string()).await;
    let _ = session.insert("role", "admin".to_string()).await;
    let _ = session.insert("user_name", "john_doe".to_string()).await;
    "Session set".to_string()
  }
}

async fn setup_app() -> axum::Router {
  let session_store = MemoryStore::default();
  let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

  SessionController::router().layer(session_layer)
}

#[tokio::test]
async fn test_single_session_extraction() {
  let app = setup_app().await;

  // First, set the session
  let request = Request::builder()
    .uri("/sessions/set")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  // Extract session cookie
  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Now test the session extraction
  let request = Request::builder()
    .uri("/sessions/user")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "User ID: 12345");
}

#[tokio::test]
async fn test_missing_session_data() {
  let app = setup_app().await;

  let request = Request::builder()
    .uri("/sessions/user")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  // Should default to empty string
  assert_eq!(body_str, "User ID: ");
}

#[tokio::test]
async fn test_multiple_session_params() {
  let app = setup_app().await;

  // Set session
  let request = Request::builder()
    .uri("/sessions/set")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Test multiple session params
  let request = Request::builder()
    .uri("/sessions/data")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Username: testuser, Role: admin");
}

#[tokio::test]
async fn test_mixed_path_and_session() {
  let app = setup_app().await;

  // Set session
  let request = Request::builder()
    .uri("/sessions/set")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Test mixed extractors
  let request = Request::builder()
    .uri("/sessions/42/profile")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "ID: 42, Username: john_doe");
}

#[tokio::test]
async fn test_mixed_path_session_and_header() {
  let app = setup_app().await;

  // Set session
  let request = Request::builder()
    .uri("/sessions/set")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Test mixed extractors with header
  let request = Request::builder()
    .uri("/sessions/99/secure")
    .header("cookie", cookie_header)
    .header("api-key", "secret123")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "ID: 99, User: 12345, API Key: secret123");
}

#[tokio::test]
async fn test_session_persistence_across_requests() {
  let app = setup_app().await;

  // Initialize session
  let request = Request::builder()
    .uri("/sessions/set")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap()
    .to_string();

  // First request
  let request = Request::builder()
    .uri("/sessions/user")
    .header("cookie", &cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(body_str, "User ID: 12345");

  // Second request with same cookie should still work
  let request = Request::builder()
    .uri("/sessions/data")
    .header("cookie", &cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(body_str, "Username: testuser, Role: admin");
}
