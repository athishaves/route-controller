use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize)]
struct TestResponse {
  message: String,
}

struct ResponseHeaderController;

#[controller(path = "/test")]
impl ResponseHeaderController {
  #[get("/single-header", header("x-custom-header", "test-value"))]
  async fn single_header() -> &'static str {
    "response with single header"
  }

  #[get(
    "/multiple-headers",
    header("x-header-1", "value1"),
    header("x-header-2", "value2")
  )]
  async fn multiple_headers() -> &'static str {
    "response with multiple headers"
  }

  #[get("/content-type", content_type("application/xml"))]
  async fn custom_content_type() -> String {
    "<response>test</response>".to_string()
  }

  #[post(
    "/combined",
    content_type("application/json"),
    header("x-api-version", "1.0")
  )]
  async fn combined() -> axum::Json<TestResponse> {
    axum::Json(TestResponse {
      message: "combined".to_string(),
    })
  }
}

#[tokio::test]
async fn test_single_custom_header() {
  let app = ResponseHeaderController::router();

  let request = Request::builder()
    .uri("/test/single-header")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("x-custom-header").unwrap(),
    "test-value"
  );

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(body_str, "response with single header");
}

#[tokio::test]
async fn test_multiple_custom_headers() {
  let app = ResponseHeaderController::router();

  let request = Request::builder()
    .uri("/test/multiple-headers")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-header-1").unwrap(), "value1");
  assert_eq!(response.headers().get("x-header-2").unwrap(), "value2");

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(body_str, "response with multiple headers");
}

#[tokio::test]
async fn test_custom_content_type() {
  let app = ResponseHeaderController::router();

  let request = Request::builder()
    .uri("/test/content-type")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/xml"
  );

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(body_str, "<response>test</response>");
}

#[tokio::test]
async fn test_combined_content_type_and_headers() {
  let app = ResponseHeaderController::router();

  let request = Request::builder()
    .uri("/test/combined")
    .method("POST")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );
  assert_eq!(response.headers().get("x-api-version").unwrap(), "1.0");

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: TestResponse = serde_json::from_slice(&body).unwrap();
  assert_eq!(result.message, "combined");
}

#[tokio::test]
async fn test_header_values_are_correct() {
  let app = ResponseHeaderController::router();

  let request = Request::builder()
    .uri("/test/single-header")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  let header_value = response.headers().get("x-custom-header").unwrap();
  assert_eq!(header_value.to_str().unwrap(), "test-value");
}
