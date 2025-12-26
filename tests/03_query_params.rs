//! Integration tests for Query parameter extraction
//!
//! Tests query strings with optional and required parameters

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize)]
struct SearchQuery {
  q: String,
  limit: Option<u32>,
  offset: Option<u32>,
}

#[derive(Deserialize)]
struct OptionalQuery {
  name: Option<String>,
  age: Option<u32>,
}

#[derive(Deserialize)]
struct BoolQuery {
  active: Option<bool>,
  verified: Option<bool>,
}

struct QueryController;

#[controller(path = "/api")]
impl QueryController {
  #[get("/search", extract(query = Query))]
  async fn search(query: SearchQuery) -> String {
    format!(
      "q:{},limit:{},offset:{}",
      query.q,
      query.limit.unwrap_or(10),
      query.offset.unwrap_or(0)
    )
  }

  #[get("/filter", extract(query = Query))]
  async fn filter(query: OptionalQuery) -> String {
    format!(
      "name:{},age:{}",
      query.name.unwrap_or_else(|| "none".to_string()),
      query.age.unwrap_or(0)
    )
  }

  #[get("/status", extract(query = Query))]
  async fn status(query: BoolQuery) -> String {
    format!(
      "active:{},verified:{}",
      query.active.unwrap_or(false),
      query.verified.unwrap_or(false)
    )
  }
}

#[tokio::test]
async fn test_required_query_param() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?q=rust")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"q:rust,limit:10,offset:0");
}

#[tokio::test]
async fn test_multiple_query_params() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?q=rust&limit=20&offset=5")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"q:rust,limit:20,offset:5");
}

#[tokio::test]
async fn test_optional_query_params() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/filter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"name:none,age:0");
}

#[tokio::test]
async fn test_some_optional_params_provided() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/filter?name=alice")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"name:alice,age:0");
}

#[tokio::test]
async fn test_boolean_query_params() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/status?active=true&verified=false")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"active:true,verified:false");
}

#[tokio::test]
async fn test_missing_required_query_param() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Should return 400 Bad Request for missing required param
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_type_query_param() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?q=rust&limit=not_a_number")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_url_encoded_query_param() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?q=hello%20world")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"q:hello world,limit:10,offset:0");
}

#[tokio::test]
async fn test_special_chars_in_query() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/filter?name=alice%2Bbob")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_duplicate_query_params() {
  // When extracting single values, duplicates cause errors
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/filter?name=alice&name=bob")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_empty_query_value() {
  let app = QueryController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search?q=")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"q:,limit:10,offset:0");
}
