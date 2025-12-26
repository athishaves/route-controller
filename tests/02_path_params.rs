//! Integration tests for Path parameter extraction
//!
//! Tests single and multiple path parameters with various types

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct PathController;

#[controller(path = "/api")]
impl PathController {
  #[get("/users/{id}", extract(id = Path))]
  async fn get_user(id: u32) -> String {
    format!("user:{}", id)
  }

  #[get("/users/{id}/posts/{post_id}", extract(id = Path, post_id = Path))]
  async fn get_post(id: u32, post_id: u32) -> String {
    format!("user:{},post:{}", id, post_id)
  }

  #[get("/profile/{username}", extract(username = Path))]
  async fn get_profile(username: String) -> String {
    format!("username:{}", username)
  }

  #[get("/items/{id}/details/{section}", extract(id = Path, section = Path))]
  async fn get_details(id: u32, section: String) -> String {
    format!("id:{},section:{}", id, section)
  }
}

#[tokio::test]
async fn test_single_u32_path_param() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:42");
}

#[tokio::test]
async fn test_single_string_path_param() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile/john_doe")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"username:john_doe");
}

#[tokio::test]
async fn test_multiple_u32_path_params() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/5/posts/10")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:5,post:10");
}

#[tokio::test]
async fn test_mixed_type_path_params() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/items/99/details/specs")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:99,section:specs");
}

#[tokio::test]
async fn test_invalid_u32_path_param() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/not_a_number")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Should return 400 Bad Request for invalid type
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_zero_value_path_param() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/0")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:0");
}

#[tokio::test]
async fn test_large_number_path_param() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/4294967295")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_string_with_special_chars() {
  let app = PathController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/profile/john-doe_123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"username:john-doe_123");
}

#[tokio::test]
async fn test_empty_path_segment() {
  let app = PathController::router();
  // Double slash results in empty path segment, which fails type parsing
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users//posts/1")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Empty string can't parse to u32, results in BAD_REQUEST
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_path_param_order_independence() {
  // Test that extract order doesn't matter
  struct OrderController;

  #[controller]
  impl OrderController {
    #[get("/{a}/{b}", extract(b = Path, a = Path))]
    async fn test_order(a: String, b: String) -> String {
      format!("a:{},b:{}", a, b)
    }
  }

  let app = OrderController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/first/second")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"a:first,b:second");
}
