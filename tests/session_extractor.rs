use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct UserProfile {
  id: u32,
  name: String,
  email: String,
  role: String,
}

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

  // Save a struct to session
  #[post("/profile/save", extract(profile = Json))]
  async fn save_profile(session: Session, profile: UserProfile) -> String {
    let _ = session.insert("profile", profile.clone()).await;
    format!("Profile saved: {:?}", profile)
  }

  // Extract struct from session using SessionParam
  #[get("/profile/get", extract(profile = SessionParam))]
  async fn get_saved_profile(profile: Option<UserProfile>) -> String {
    match profile {
      Some(p) => format!(
        "Profile from session - ID: {}, Name: {}, Email: {}, Role: {}",
        p.id, p.name, p.email, p.role
      ),
      None => "No profile found in session".to_string(),
    }
  }

  // Direct struct access from session (without SessionParam)
  #[get("/profile/direct")]
  async fn get_profile_direct(session: Session) -> String {
    match session.get::<UserProfile>("profile").await {
      Ok(Some(profile)) => format!(
        "Direct Profile - ID: {}, Name: {}, Email: {}, Role: {}",
        profile.id, profile.name, profile.email, profile.role
      ),
      _ => "No profile found in session".to_string(),
    }
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

#[tokio::test]
async fn test_save_and_retrieve_struct_with_session_param() {
  let app = setup_app().await;

  let profile = UserProfile {
    id: 1,
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    role: "admin".to_string(),
  };

  // Save profile to session
  let request = Request::builder()
    .uri("/sessions/profile/save")
    .method("POST")
    .header("content-type", "application/json")
    .body(axum::body::Body::from(serde_json::to_string(&profile).unwrap()))
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Retrieve profile using SessionParam
  let request = Request::builder()
    .uri("/sessions/profile/get")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(
    body_str,
    "Profile from session - ID: 1, Name: John Doe, Email: john@example.com, Role: admin"
  );
}

#[tokio::test]
async fn test_save_and_retrieve_struct_direct() {
  let app = setup_app().await;

  let profile = UserProfile {
    id: 2,
    name: "Jane Smith".to_string(),
    email: "jane@example.com".to_string(),
    role: "user".to_string(),
  };

  // Save profile to session
  let request = Request::builder()
    .uri("/sessions/profile/save")
    .method("POST")
    .header("content-type", "application/json")
    .body(axum::body::Body::from(serde_json::to_string(&profile).unwrap()))
    .unwrap();

  let response = app.clone().oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let cookie_header = response
    .headers()
    .get("set-cookie")
    .and_then(|v| v.to_str().ok())
    .unwrap();

  // Retrieve profile directly from session
  let request = Request::builder()
    .uri("/sessions/profile/direct")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(
    body_str,
    "Direct Profile - ID: 2, Name: Jane Smith, Email: jane@example.com, Role: user"
  );
}

#[tokio::test]
async fn test_retrieve_missing_struct_with_session_param() {
  let app = setup_app().await;

  // Request without saving profile first
  let request = Request::builder()
    .uri("/sessions/profile/get")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "No profile found in session");
}

#[tokio::test]
async fn test_retrieve_missing_struct_direct() {
  let app = setup_app().await;

  // Request without saving profile first
  let request = Request::builder()
    .uri("/sessions/profile/direct")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "No profile found in session");
}
