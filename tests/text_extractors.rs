use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use tower::ServiceExt;

struct TextController;

#[controller(path = "/content")]
impl TextController {
  #[post("/text", extract(content = Text))]
  async fn handle_text(content: String) -> String {
    format!("Received: {}", content)
  }

  #[post("/html", extract(html = Html))]
  async fn handle_html(html: String) -> String {
    format!("HTML length: {}", html.len())
  }

  #[post("/xml", extract(xml = Xml))]
  async fn handle_xml(xml: String) -> String {
    format!("XML length: {}", xml.len())
  }

  #[post("/js", extract(code = JavaScript))]
  async fn handle_javascript(code: String) -> String {
    format!("JS length: {}", code.len())
  }
}

#[tokio::test]
async fn test_text_extractor() {
  let app = TextController::router();

  let request = Request::builder()
    .uri("/content/text")
    .method("POST")
    .header("content-type", "text/plain")
    .body(Body::from("Hello, World!"))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Received: Hello, World!");
}

#[tokio::test]
async fn test_text_extractor_multiline() {
  let app = TextController::router();

  let text = "Line 1\nLine 2\nLine 3";

  let request = Request::builder()
    .uri("/content/text")
    .method("POST")
    .header("content-type", "text/plain")
    .body(Body::from(text))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("Received: {}", text));
}

#[tokio::test]
async fn test_html_extractor() {
  let app = TextController::router();

  let html = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body><h1>Hello</h1></body>
</html>"#;

  let request = Request::builder()
    .uri("/content/html")
    .method("POST")
    .header("content-type", "text/html")
    .body(Body::from(html))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("HTML length: {}", html.len()));
}

#[tokio::test]
async fn test_xml_extractor() {
  let app = TextController::router();

  let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <item id="1">Test</item>
  <item id="2">Data</item>
</root>"#;

  let request = Request::builder()
    .uri("/content/xml")
    .method("POST")
    .header("content-type", "application/xml")
    .body(Body::from(xml))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("XML length: {}", xml.len()));
}

#[tokio::test]
async fn test_xml_extractor_with_text_xml_content_type() {
  let app = TextController::router();

  let xml = "<note><to>User</to><from>System</from></note>";

  let request = Request::builder()
    .uri("/content/xml")
    .method("POST")
    .header("content-type", "text/xml")
    .body(Body::from(xml))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("XML length: {}", xml.len()));
}

#[tokio::test]
async fn test_javascript_extractor() {
  let app = TextController::router();

  let js = r#"function hello() {
  console.log("Hello, World!");
}
hello();"#;

  let request = Request::builder()
    .uri("/content/js")
    .method("POST")
    .header("content-type", "application/javascript")
    .body(Body::from(js))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("JS length: {}", js.len()));
}

#[tokio::test]
async fn test_javascript_extractor_with_text_javascript_content_type() {
  let app = TextController::router();

  let js = "const x = 42;";

  let request = Request::builder()
    .uri("/content/js")
    .method("POST")
    .header("content-type", "text/javascript")
    .body(Body::from(js))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("JS length: {}", js.len()));
}

#[tokio::test]
async fn test_text_extractor_empty_string() {
  let app = TextController::router();

  let request = Request::builder()
    .uri("/content/text")
    .method("POST")
    .header("content-type", "text/plain")
    .body(Body::from(""))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Received: ");
}

#[tokio::test]
async fn test_text_extractor_unicode() {
  let app = TextController::router();

  let text = "Hello 世界 🌍";

  let request = Request::builder()
    .uri("/content/text")
    .method("POST")
    .header("content-type", "text/plain; charset=utf-8")
    .body(Body::from(text))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, format!("Received: {}", text));
}
