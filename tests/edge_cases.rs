// Integration tests for edge cases and special scenarios

use axum::Json;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

// Test mixing async and sync handlers (though sync isn't typical in Axum)
struct MixedAsyncController;

#[controller]
impl MixedAsyncController {
  #[get("/async")]
  async fn async_handler() -> &'static str {
    "async"
  }

  #[get("/sync")]
  async fn sync_handler() -> &'static str {
    "sync"
  }
}

#[tokio::test]
async fn test_async_handlers() {
  let app = MixedAsyncController::router();

  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/async")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let response = app
    .oneshot(Request::builder().uri("/sync").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
}

// Test different return types
struct DifferentReturnTypesController;

#[derive(Serialize)]
struct ApiResponse {
  message: String,
  code: u32,
}

#[controller]
impl DifferentReturnTypesController {
  #[get("/str")]
  async fn return_str() -> &'static str {
    "static str"
  }

  #[get("/string")]
  async fn return_string() -> String {
    "owned string".to_string()
  }

  #[get("/json")]
  async fn return_json() -> Json<ApiResponse> {
    Json(ApiResponse {
      message: "success".to_string(),
      code: 200,
    })
  }

  #[get("/status")]
  async fn return_status() -> StatusCode {
    StatusCode::CREATED
  }

  #[get("/tuple")]
  async fn return_tuple() -> (StatusCode, &'static str) {
    (StatusCode::ACCEPTED, "accepted")
  }
}

#[tokio::test]
async fn test_different_return_types() {
  let app = DifferentReturnTypesController::router();

  // Test &str
  let response = app
    .clone()
    .oneshot(Request::builder().uri("/str").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"static str");

  // Test String
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/string")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"owned string");

  // Test Json
  let response = app
    .clone()
    .oneshot(Request::builder().uri("/json").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );

  // Test StatusCode
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/status")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::CREATED);

  // Test tuple
  let response = app
    .oneshot(
      Request::builder()
        .uri("/tuple")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::ACCEPTED);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"accepted");
}

// Test empty impl block (controller with no routes)
struct EmptyController;

#[controller]
impl EmptyController {
  // No route handlers
}

#[tokio::test]
async fn test_empty_controller() {
  let app = EmptyController::router();

  // Should return 404 for any path
  let response = app
    .oneshot(
      Request::builder()
        .uri("/any-path")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// Test impl block with non-route methods
struct MixedMethodsController;

#[controller]
impl MixedMethodsController {
  // Regular method without route attribute - should be ignored
  fn helper_function() -> String {
    "helper".to_string()
  }

  #[get("/route")]
  async fn route_handler() -> String {
    Self::helper_function()
  }
}

#[tokio::test]
async fn test_non_route_methods_ignored() {
  let app = MixedMethodsController::router();

  // Only /route should work
  let response = app
    .oneshot(
      Request::builder()
        .uri("/route")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"helper");
}

// Test complex nested types
#[derive(Deserialize, Serialize)]
struct NestedData {
  user: UserData,
  metadata: Metadata,
}

#[derive(Deserialize, Serialize)]
struct UserData {
  name: String,
  age: u32,
}

#[derive(Deserialize, Serialize)]
struct Metadata {
  tags: Vec<String>,
  active: bool,
}

struct ComplexTypesController;

#[controller]
impl ComplexTypesController {
  #[post("/nested", extract(data = Json))]
  async fn handle_nested(data: NestedData) -> Json<NestedData> {
    Json(data)
  }
}

#[tokio::test]
async fn test_complex_nested_types() {
  let app = ComplexTypesController::router();

  let nested = r#"{
        "user": {
            "name": "Alice",
            "age": 30
        },
        "metadata": {
            "tags": ["rust", "programming"],
            "active": true
        }
    }"#;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/nested")
        .header("content-type", "application/json")
        .body(Body::from(nested))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );
}

// Test optional types in extractors
#[derive(Deserialize)]
struct OptionalQuery {
  required: String,
  optional: Option<String>,
  count: Option<u32>,
}

struct OptionalController;

#[controller]
impl OptionalController {
  #[get("/search", extract(query = Query))]
  async fn search(query: OptionalQuery) -> String {
    format!(
      "Required: {}, Optional: {}, Count: {}",
      query.required,
      query.optional.unwrap_or_else(|| "none".to_string()),
      query.count.unwrap_or(0)
    )
  }
}

#[tokio::test]
async fn test_optional_query_params() {
  let app = OptionalController::router();

  // With all params
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/search?required=test&optional=value&count=5")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert!(body_str.contains("test"));
  assert!(body_str.contains("value"));
  assert!(body_str.contains("5"));

  // With only required param
  let response = app
    .oneshot(
      Request::builder()
        .uri("/search?required=test")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert!(body_str.contains("test"));
  assert!(body_str.contains("none"));
  assert!(body_str.contains("0"));
}

// Test path parameters not actively used in function body
struct UnusedParamsController;

#[controller]
impl UnusedParamsController {
  // Declare path param and extract it but just return static response
  #[get("/{id}", extract(id = Path))]
  async fn unused_param(id: u32) -> String {
    // Parameter is extracted but we can choose not to use it
    format!("Received ID: {}", id)
  }
}

#[tokio::test]
async fn test_unused_path_parameters() {
  let app = UnusedParamsController::router();

  let response = app
    .oneshot(Request::builder().uri("/123").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert!(body_str.contains("123"));
}

// Test very long path
struct LongPathController;

#[controller(path = "/api/v1/admin")]
impl LongPathController {
  #[get("/dashboard/users/settings/profile")]
  async fn long_path() -> &'static str {
    "long path"
  }
}

#[tokio::test]
async fn test_very_long_path() {
  let app = LongPathController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/v1/admin/dashboard/users/settings/profile")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"long path");
}

// Test multiple path params with different types
struct MixedPathTypesController;

#[controller]
impl MixedPathTypesController {
  #[get("/{id}/{name}/{active}", extract(id = Path, name = Path, active = Path))]
  async fn mixed_types(id: u32, name: String, active: bool) -> String {
    format!("ID: {}, Name: {}, Active: {}", id, name, active)
  }
}

#[tokio::test]
async fn test_mixed_path_param_types() {
  let app = MixedPathTypesController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/42/alice/true")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert!(body_str.contains("42"));
  assert!(body_str.contains("alice"));
  assert!(body_str.contains("true"));
}

// Test route with special characters in path
struct SpecialCharsController;

#[controller]
impl SpecialCharsController {
  #[get("/items/{id}")]
  async fn get_item() -> &'static str {
    "item"
  }
}

#[tokio::test]
async fn test_special_chars_in_url() {
  let app = SpecialCharsController::router();

  // Standard path should work
  let response = app
    .oneshot(
      Request::builder()
        .uri("/items/test-item-123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

// Test Result return type
struct ResultController;

#[controller]
impl ResultController {
  #[get("/success")]
  async fn success() -> Result<String, StatusCode> {
    Ok("success".to_string())
  }

  #[get("/error")]
  async fn error() -> Result<String, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
  }
}

#[tokio::test]
async fn test_result_return_type() {
  let app = ResultController::router();

  // Test success case
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/success")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  // Test error case
  let response = app
    .oneshot(
      Request::builder()
        .uri("/error")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
