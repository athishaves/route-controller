//! Integration tests for JSON body extraction
//!
//! Tests JSON parsing with valid and invalid payloads

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post, put};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct User {
  name: String,
  email: String,
}

#[derive(Deserialize, Serialize)]
struct NestedData {
  user: User,
  metadata: Metadata,
}

#[derive(Deserialize, Serialize)]
struct Metadata {
  created_at: String,
  version: u32,
}

struct JsonController;

#[controller(path = "/api")]
impl JsonController {
  #[post("/users", extract(user = Json))]
  async fn create_user(user: User) -> axum::Json<User> {
    axum::Json(user)
  }

  #[put("/users/{id}", extract(id = Path, user = Json))]
  async fn update_user(id: u32, user: User) -> String {
    format!("Updated user {}: {}", id, user.name)
  }

  #[post("/nested", extract(data = Json))]
  async fn nested_data(data: NestedData) -> String {
    format!("User: {}, Version: {}", data.user.name, data.metadata.version)
  }
}

#[tokio::test]
async fn test_valid_json() {
  let app = JsonController::router();
  let user = User {
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
  };
  let body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: User = serde_json::from_slice(&body).unwrap();
  assert_eq!(result, user);
}

#[tokio::test]
async fn test_invalid_json() {
  let app = JsonController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from("{invalid json"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_required_fields() {
  let app = JsonController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Alice"}"#))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_extra_fields_ignored() {
  let app = JsonController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(
          r#"{"name":"Alice","email":"alice@example.com","extra":"field"}"#,
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_empty_json() {
  let app = JsonController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_nested_json() {
  let app = JsonController::router();
  let json = r#"{"user":{"name":"Bob","email":"bob@example.com"},"metadata":{"created_at":"2024-01-01","version":1}}"#;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/nested")
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"User: Bob, Version: 1");
}

#[tokio::test]
async fn test_path_and_json_combined() {
  let app = JsonController::router();
  let user = User {
    name: "Charlie".to_string(),
    email: "charlie@example.com".to_string(),
  };
  let body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/api/users/42")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Updated user 42: Charlie");
}

#[tokio::test]
async fn test_missing_content_type_header() {
  let app = JsonController::router();
  let user = User {
    name: "Dave".to_string(),
    email: "dave@example.com".to_string(),
  };
  let body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();

  // Axum's Json extractor requires content-type header
  assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_wrong_content_type() {
  let app = JsonController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "text/plain")
        .body(Body::from(r#"{"name":"Eve","email":"eve@example.com"}"#))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn test_large_json_payload() {
  let app = JsonController::router();
  let name = "A".repeat(10000);
  let json = format!(r#"{{"name":"{}","email":"test@example.com"}}"#, name);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_unicode_in_json() {
  let app = JsonController::router();
  let json = r#"{"name":"José García","email":"jose@example.com"}"#;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}
