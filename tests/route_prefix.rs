// Integration tests for route prefix functionality

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize)]
struct User {
  name: String,
}

// Test basic route prefix
struct ApiController;

#[controller(path = "/api/v1")]
impl ApiController {
  #[get("/users")]
  async fn list_users() -> &'static str {
    "users"
  }

  #[post("/users")]
  async fn create_user() -> &'static str {
    "created"
  }

  #[get("/health")]
  async fn health() -> &'static str {
    "healthy"
  }
}

#[tokio::test]
async fn test_route_prefix_basic() {
  let app = ApiController::router();

  // Test GET /api/v1/users
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/api/v1/users")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"users");

  // Test POST /api/v1/users
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/users")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"created");

  // Test GET /api/v1/health
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"healthy");
}

#[tokio::test]
async fn test_routes_without_prefix_not_found() {
  let app = ApiController::router();

  // Try accessing without prefix - should return 404
  let response = app
    .oneshot(
      Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// Test prefix normalization (without leading slash)
struct NormalizedController;

#[controller(path = "api")]
impl NormalizedController {
  #[get("/status")]
  async fn status() -> &'static str {
    "ok"
  }
}

#[tokio::test]
async fn test_route_prefix_normalization() {
  let app = NormalizedController::router();

  // Should work with /api/status (auto-adds leading slash)
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"ok");
}

// Test prefix with path parameters
struct PrefixedParamsController;

#[controller(path = "/v1/users")]
impl PrefixedParamsController {
  #[get("/{id}", extract(id = Path))]
  async fn get_user(id: u32) -> String {
    format!("User {}", id)
  }

  #[get("/{user_id}/posts/{post_id}", extract(user_id = Path, post_id = Path))]
  async fn get_user_post(user_id: u32, post_id: u32) -> String {
    format!("User {} - Post {}", user_id, post_id)
  }
}

#[tokio::test]
async fn test_route_prefix_with_path_params() {
  let app = PrefixedParamsController::router();

  // Test single path param
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/v1/users/42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"User 42");

  // Test multiple path params
  let response = app
    .oneshot(
      Request::builder()
        .uri("/v1/users/10/posts/20")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"User 10 - Post 20");
}

// Test deeply nested prefix
struct DeepPrefixController;

#[controller(path = "/api/v2/admin/dashboard")]
impl DeepPrefixController {
  #[get("/stats")]
  async fn stats() -> &'static str {
    "stats"
  }
}

#[tokio::test]
async fn test_deep_route_prefix() {
  let app = DeepPrefixController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/v2/admin/dashboard/stats")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"stats");
}

// Test prefix with extractors
struct PrefixedExtractorController;

#[controller(path = "/api")]
impl PrefixedExtractorController {
  #[post("/users", extract(user = Json))]
  async fn create_user(user: User) -> String {
    format!("Created: {}", user.name)
  }

  #[get("/search", extract(query = Query))]
  async fn search(query: SearchQuery) -> String {
    format!("Query: {}", query.term.unwrap_or_default())
  }
}

#[derive(Deserialize)]
struct SearchQuery {
  term: Option<String>,
}

#[tokio::test]
async fn test_route_prefix_with_json_extractor() {
  let app = PrefixedExtractorController::router();

  let user = User {
    name: "Alice".to_string(),
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
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert!(body_str.contains("Alice"));
}

#[tokio::test]
async fn test_route_prefix_with_query_extractor() {
  let app = PrefixedExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?term=rust")
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
  assert!(body_str.contains("rust"));
}

// Test controller without prefix
struct NoPrefixController;

#[controller]
impl NoPrefixController {
  #[get("/test")]
  async fn test() -> &'static str {
    "test"
  }
}

#[tokio::test]
async fn test_controller_without_prefix() {
  let app = NoPrefixController::router();

  // Should work at root level
  let response = app
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"test");
}
