//! Integration tests for mixed/combined extractors
//!
//! Tests using multiple extractors together

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
  name: String,
  age: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchQuery {
  q: String,
  limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SimpleQuery {
  q: String,
}

struct MixedExtractorController;

#[controller(path = "/api")]
impl MixedExtractorController {
  #[get("/users/{id}", extract(id = Path, query = Query))]
  async fn path_and_query(id: u32, query: SimpleQuery) -> String {
    format!("id:{},q:{}", id, query.q)
  }

  #[post("/users/{id}", extract(id = Path, user = Json))]
  async fn path_and_json(id: u32, user: UserData) -> String {
    format!("id:{},name:{},age:{}", id, user.name, user.age)
  }

  #[get(
    "/search/{category}",
    extract(category = Path, query = Query)
  )]
  async fn path_and_query_struct(category: String, query: SearchQuery) -> String {
    format!(
      "cat:{},q:{},limit:{}",
      category,
      query.q,
      query.limit.unwrap_or(10)
    )
  }

  #[post(
    "/submit/{id}",
    extract(id = Path, query = Query, data = Json)
  )]
  async fn path_query_json(id: u32, query: SimpleQuery, data: UserData) -> String {
    format!("id:{},q:{},name:{}", id, query.q, data.name)
  }

  #[get("/data/{id}", extract(id = Path, data = Text))]
  async fn path_and_text(id: u32, data: String) -> String {
    format!("id:{},data:{}", id, data)
  }

  #[post("/upload/{id}", extract(id = Path, bytes = Bytes))]
  async fn path_and_bytes(id: u32, bytes: Vec<u8>) -> String {
    format!("id:{},bytes:{}", id, bytes.len())
  }
}

#[tokio::test]
async fn test_path_and_query() {
  let app = MixedExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/123?q=test")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:123,q:test");
}

#[tokio::test]
async fn test_path_and_json() {
  let app = MixedExtractorController::router();

  let user = UserData {
    name: "Alice".to_string(),
    age: 30,
  };
  let json_body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users/42")
        .header("content-type", "application/json")
        .body(Body::from(json_body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:42,name:Alice,age:30");
}

#[tokio::test]
async fn test_path_and_query_struct() {
  let app = MixedExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search/books?q=rust&limit=5")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"cat:books,q:rust,limit:5");
}

#[tokio::test]
async fn test_path_query_json() {
  let app = MixedExtractorController::router();

  let user = UserData {
    name: "Bob".to_string(),
    age: 25,
  };
  let json_body = serde_json::to_string(&user).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/submit/100?q=test")
        .header("content-type", "application/json")
        .body(Body::from(json_body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:100,q:test,name:Bob");
}

#[tokio::test]
async fn test_path_and_text() {
  let app = MixedExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/data/7")
        .header("content-type", "text/plain")
        .body(Body::from("hello world"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:7,data:hello world");
}

#[tokio::test]
async fn test_path_and_bytes() {
  let app = MixedExtractorController::router();

  let bytes = vec![1, 2, 3, 4, 5];

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload/99")
        .header("content-type", "application/octet-stream")
        .body(Body::from(bytes))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:99,bytes:5");
}

// Test with HeaderParam and CookieParam
#[cfg(all(feature = "headers", feature = "cookies"))]
mod header_cookie_tests {
  use super::*;

  struct HeaderCookieController;

  #[controller(path = "/hc")]
  impl HeaderCookieController {
    #[get(
      "/data/{id}",
      extract(id = Path, auth = HeaderParam, session = CookieParam)
    )]
    async fn path_header_cookie(id: u32, auth: String, session: String) -> String {
      format!("id:{},auth:{},session:{}", id, auth, session)
    }

    #[post(
      "/submit",
      extract(user = Json, token = HeaderParam, sid = CookieParam)
    )]
    async fn json_header_cookie(user: UserData, token: String, sid: String) -> String {
      format!("name:{},token:{},sid:{}", user.name, token, sid)
    }
  }

  #[tokio::test]
  async fn test_path_header_cookie() {
    let app = HeaderCookieController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/hc/data/123")
          .header("auth", "token123")
          .header("cookie", "session=abc123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    assert_eq!(&body[..], b"id:123,auth:token123,session:abc123");
  }

  #[tokio::test]
  async fn test_json_header_cookie() {
    let app = HeaderCookieController::router();

    let user = UserData {
      name: "Charlie".to_string(),
      age: 35,
    };
    let json_body = serde_json::to_string(&user).unwrap();

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/hc/submit")
          .header("content-type", "application/json")
          .header("token", "bearer_xyz")
          .header("cookie", "sid=session456")
          .body(Body::from(json_body))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
      .await
      .unwrap();
    assert_eq!(&body[..], b"name:Charlie,token:bearer_xyz,sid:session456");
  }
}

#[tokio::test]
async fn test_missing_query_with_path() {
  let app = MixedExtractorController::router();

  // Missing required query parameter
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Axum returns 400 for missing query params in Path handlers
  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_json_with_path() {
  let app = MixedExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/users/42")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_optional_query_with_path() {
  let app = MixedExtractorController::router();

  // Optional limit parameter not provided
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/search/books?q=rust")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"cat:books,q:rust,limit:10");
}
