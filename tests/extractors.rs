// Integration tests for extractor functionality

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post, put};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize, Debug)]
struct User {
  name: String,
  email: String,
}

#[derive(Deserialize, Debug)]
struct SearchParams {
  query: Option<String>,
  limit: Option<u32>,
}

// Test Json extractor
struct JsonController;

#[controller]
impl JsonController {
  #[post("/users", extract(user = Json))]
  async fn create(user: User) -> String {
    format!("Created: {} ({})", user.name, user.email)
  }
}

#[tokio::test]
async fn test_json_extractor_valid() {
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
        .uri("/users")
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
  assert!(body_str.contains("alice@example.com"));
}

#[tokio::test]
async fn test_json_extractor_invalid() {
  let app = JsonController::router();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap(),
    )
    .await
    .unwrap();

  // Should return 400 Bad Request for invalid JSON
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Test Path extractor (single parameter)
struct PathController;

#[controller]
impl PathController {
  #[get("/{id}", extract(id = Path))]
  async fn get_user(id: u32) -> String {
    format!("User ID: {}", id)
  }

  #[get("/items/{item_id}", extract(item_id = Path))]
  async fn get_item(item_id: String) -> String {
    format!("Item: {}", item_id)
  }
}

#[tokio::test]
async fn test_path_extractor_single_u32() {
  let app = PathController::router();

  let response = app
    .oneshot(Request::builder().uri("/42").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"User ID: 42");
}

#[tokio::test]
async fn test_path_extractor_single_string() {
  let app = PathController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/items/laptop")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Item: laptop");
}

#[tokio::test]
async fn test_path_extractor_type_error() {
  let app = PathController::router();

  // Try to pass non-numeric value to u32 parameter
  let response = app
    .oneshot(
      Request::builder()
        .uri("/not-a-number")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Should return 400 Bad Request for type mismatch
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Test multiple Path extractors
struct MultiplePathController;

#[controller]
impl MultiplePathController {
  #[get("/{user_id}/posts/{post_id}", extract(user_id = Path, post_id = Path))]
  async fn get_user_post(user_id: u32, post_id: u32) -> String {
    format!("User {} - Post {}", user_id, post_id)
  }

  // Test order independence in extract declaration
  #[get("/{category}/items/{item_id}", extract(item_id = Path, category = Path))]
  async fn get_category_item(category: String, item_id: u32) -> String {
    format!("Category: {} - Item: {}", category, item_id)
  }
}

#[tokio::test]
async fn test_multiple_path_extractors() {
  let app = MultiplePathController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/123/posts/456")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"User 123 - Post 456");
}

#[tokio::test]
async fn test_path_extractors_order_independence() {
  let app = MultiplePathController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/electronics/items/999")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Category: electronics - Item: 999");
}

// Test Query extractor
struct QueryController;

#[controller]
impl QueryController {
  #[get("/search", extract(params = Query))]
  async fn search(params: SearchParams) -> String {
    format!(
      "Query: {}, Limit: {}",
      params.query.unwrap_or_default(),
      params.limit.unwrap_or(10)
    )
  }
}

#[tokio::test]
async fn test_query_extractor_with_params() {
  let app = QueryController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/search?query=rust&limit=20")
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
  assert!(body_str.contains("20"));
}

#[tokio::test]
async fn test_query_extractor_with_optional_params() {
  let app = QueryController::router();

  // Test with only one param
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/search?query=rust")
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
  assert!(body_str.contains("10")); // default limit

  // Test with no params
  let response = app
    .oneshot(
      Request::builder()
        .uri("/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

// Test mixed extractors (Path + Json)
struct MixedPathJsonController;

#[controller]
impl MixedPathJsonController {
  #[put("/{id}", extract(id = Path, user = Json))]
  async fn update(id: u32, user: User) -> String {
    format!("Updated user {}: {} ({})", id, user.name, user.email)
  }
}

#[tokio::test]
async fn test_mixed_path_json_extractors() {
  let app = MixedPathJsonController::router();

  let user = User {
    name: "Bob".to_string(),
    email: "bob@example.com".to_string(),
  };

  let body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/99")
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
  assert!(body_str.contains("99"));
  assert!(body_str.contains("Bob"));
  assert!(body_str.contains("bob@example.com"));
}

// Test mixed extractors (Path + Query)
struct MixedPathQueryController;

#[controller]
impl MixedPathQueryController {
  #[get("/{id}/search", extract(id = Path, params = Query))]
  async fn search_user(id: u32, params: SearchParams) -> String {
    format!(
      "User: {}, Query: {}, Limit: {}",
      id,
      params.query.unwrap_or_default(),
      params.limit.unwrap_or(10)
    )
  }
}

#[tokio::test]
async fn test_mixed_path_query_extractors() {
  let app = MixedPathQueryController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/42/search?query=posts&limit=5")
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
  assert!(body_str.contains("posts"));
  assert!(body_str.contains("5"));
}

// Test no extractors
struct NoExtractorController;

#[controller]
impl NoExtractorController {
  #[get("/simple")]
  async fn simple() -> &'static str {
    "simple"
  }
}

#[tokio::test]
async fn test_no_extractors() {
  let app = NoExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/simple")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"simple");
}
