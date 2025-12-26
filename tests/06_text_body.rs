//! Integration tests for text body extractors
//!
//! Tests Text, Html, Xml, and JavaScript body extraction

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use tower::ServiceExt;

struct TextController;

#[controller(path = "/api")]
impl TextController {
  #[post("/text", extract(content = Text))]
  async fn handle_text(content: String) -> String {
    format!("len:{}", content.len())
  }

  #[post("/html", extract(html = Html))]
  async fn handle_html(html: String) -> String {
    format!("html:{}", html.len())
  }

  #[post("/xml", extract(xml = Xml))]
  async fn handle_xml(xml: String) -> String {
    format!("xml:{}", xml.len())
  }

  #[post("/script", extract(code = JavaScript))]
  async fn handle_js(code: String) -> String {
    format!("js:{}", code.len())
  }
}

#[tokio::test]
async fn test_text_extractor() {
  let app = TextController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
        .header("content-type", "text/plain")
        .body(Body::from("Hello, World!"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"len:13");
}

#[tokio::test]
async fn test_html_extractor() {
  let app = TextController::router();
  let html = "<html><body><h1>Hello</h1></body></html>";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/html")
        .header("content-type", "text/html")
        .body(Body::from(html))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], format!("html:{}", html.len()).as_bytes());
}

#[tokio::test]
async fn test_xml_extractor() {
  let app = TextController::router();
  let xml = r#"<?xml version="1.0"?><root><item>test</item></root>"#;
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/xml")
        .header("content-type", "application/xml")
        .body(Body::from(xml))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_javascript_extractor() {
  let app = TextController::router();
  let js = "console.log('Hello World');";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/script")
        .header("content-type", "application/javascript")
        .body(Body::from(js))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_empty_text() {
  let app = TextController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
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
async fn test_multiline_text() {
  let app = TextController::router();
  let text = "Line 1\nLine 2\nLine 3";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
        .header("content-type", "text/plain")
        .body(Body::from(text))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_unicode_text() {
  let app = TextController::router();
  let text = "Hello 世界 🌍";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
        .header("content-type", "text/plain")
        .body(Body::from(text))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_large_text() {
  let app = TextController::router();
  let text = "A".repeat(100000);
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
        .header("content-type", "text/plain")
        .body(Body::from(text.clone()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], format!("len:{}", text.len()).as_bytes());
}

#[tokio::test]
async fn test_xml_with_text_xml_content_type() {
  let app = TextController::router();
  let xml = "<root>test</root>";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/xml")
        .header("content-type", "text/xml")
        .body(Body::from(xml))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_javascript_with_text_javascript() {
  let app = TextController::router();
  let js = "function test() { return 42; }";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/script")
        .header("content-type", "text/javascript")
        .body(Body::from(js))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_special_characters_in_text() {
  let app = TextController::router();
  let text = "Special: !@#$%^&*()_+-=[]{}|;:',.<>?/~`";
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/text")
        .header("content-type", "text/plain")
        .body(Body::from(text))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}
