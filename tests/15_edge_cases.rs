//! Integration tests for edge cases and special scenarios
//!
//! Tests unusual inputs, boundary conditions, and error cases

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, delete, get, post, put};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Debug, Serialize, Deserialize)]
struct ComplexData {
  nested: NestedData,
  items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NestedData {
  value: String,
  count: i32,
}

#[derive(Debug, Deserialize)]
struct TextQuery {
  text: String,
}

#[derive(Debug, Deserialize)]
struct DataQuery {
  data: String,
}

struct EdgeCaseController;

#[controller(path = "/api")]
impl EdgeCaseController {
  // Root path
  #[get("/")]
  async fn root() -> &'static str {
    "root"
  }

  // Very long path
  #[get("/very/long/nested/path/with/many/segments")]
  async fn very_long_path() -> &'static str {
    "long"
  }

  // Special characters in path params
  #[get("/special/{id}", extract(id = Path))]
  async fn special_chars(id: String) -> String {
    format!("id:{}", id)
  }

  // Empty string body
  #[post("/empty", extract(data = Text))]
  async fn empty_body(data: String) -> String {
    format!("len:{}", data.len())
  }

  // Very large path param
  #[get("/large/{num}", extract(num = Path))]
  async fn large_number(num: u64) -> String {
    format!("num:{}", num)
  }

  // Negative numbers
  #[get("/negative/{num}", extract(num = Path))]
  async fn negative_number(num: i32) -> String {
    format!("num:{}", num)
  }

  // Complex nested JSON
  #[post("/complex", extract(data = Json))]
  async fn complex_json(data: ComplexData) -> String {
    format!("nested:{},items:{}", data.nested.value, data.items.len())
  }

  // Multiple slashes
  #[get("/multi/{id}/{sub}", extract(id = Path, sub = Path))]
  async fn multi_segments(id: u32, sub: String) -> String {
    format!("id:{},sub:{}", id, sub)
  }

  // Unicode in query
  #[get("/unicode", extract(query = Query))]
  async fn unicode_query(query: TextQuery) -> String {
    format!("text:{}", query.text)
  }

  // Extremely long query string
  #[get("/long-query", extract(query = Query))]
  async fn long_query(query: DataQuery) -> String {
    format!("len:{}", query.data.len())
  }

  // All HTTP methods on same path
  #[get("/crud")]
  async fn crud_get() -> &'static str {
    "GET"
  }

  #[post("/crud")]
  async fn crud_post() -> &'static str {
    "POST"
  }

  #[put("/crud")]
  async fn crud_put() -> &'static str {
    "PUT"
  }

  #[delete("/crud")]
  async fn crud_delete() -> &'static str {
    "DELETE"
  }
}

#[tokio::test]
async fn test_root_path() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"root");
}

#[tokio::test]
async fn test_very_long_path() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/very/long/nested/path/with/many/segments")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"long");
}

#[tokio::test]
async fn test_special_chars_url_encoded() {
  let app = EdgeCaseController::router();

  // URL encoded space and special chars
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/special/hello%20world")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:hello world");
}

#[tokio::test]
async fn test_empty_string_body() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/empty")
        .header("content-type", "text/plain")
        .body(Body::from(""))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"len:0");
}

#[tokio::test]
async fn test_very_large_number() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/large/18446744073709551615") // u64::MAX
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"num:18446744073709551615");
}

#[tokio::test]
async fn test_negative_number() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/negative/-42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"num:-42");
}

#[tokio::test]
async fn test_number_overflow() {
  let app = EdgeCaseController::router();

  // Number too large for u64
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/large/99999999999999999999999999999")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_complex_nested_json() {
  let app = EdgeCaseController::router();

  let data = ComplexData {
    nested: NestedData {
      value: "test".to_string(),
      count: 42,
    },
    items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
  };
  let json_body = serde_json::to_string(&data).unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/complex")
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
  assert_eq!(&body[..], b"nested:test,items:3");
}

#[tokio::test]
async fn test_multiple_path_segments() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/multi/123/test")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"id:123,sub:test");
}

#[tokio::test]
async fn test_unicode_in_query() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/unicode?text=%E4%BD%A0%E5%A5%BD") // "你好" URL encoded
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], "text:你好".as_bytes());
}

#[tokio::test]
async fn test_extremely_long_query() {
  let app = EdgeCaseController::router();

  let long_string = "a".repeat(1000);
  let uri = format!("/api/long-query?data={}", long_string);

  let response = app
    .oneshot(
      Request::builder()
        .uri(&uri)
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"len:1000");
}

#[tokio::test]
async fn test_all_http_methods() {
  let app = EdgeCaseController::router();

  // GET
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/api/crud")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"GET");

  // POST
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/crud")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"POST");

  // PUT
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/api/crud")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"PUT");

  // DELETE
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/api/crud")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"DELETE");
}

#[tokio::test]
async fn test_invalid_path_param_type() {
  let app = EdgeCaseController::router();

  // String instead of number
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/large/not_a_number")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_zero_values() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/large/0")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"num:0");
}

#[tokio::test]
async fn test_whitespace_in_text_body() {
  let app = EdgeCaseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/empty")
        .header("content-type", "text/plain")
        .body(Body::from("   \n\t   "))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"len:8");
}

// Test empty controller (no routes)
struct EmptyController;

#[controller(path = "/empty")]
impl EmptyController {}

#[tokio::test]
async fn test_empty_controller() {
  let app = EmptyController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/empty/anything")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
