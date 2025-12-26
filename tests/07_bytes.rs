//! Integration tests for Bytes extractor
//!
//! Tests binary data extraction

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use tower::ServiceExt;

struct BytesController;

#[controller(path = "/api")]
impl BytesController {
  #[post("/upload", extract(data = Bytes))]
  async fn upload(data: Vec<u8>) -> String {
    format!("size:{}", data.len())
  }

  #[post("/echo", extract(data = Bytes))]
  async fn echo(data: Vec<u8>) -> Vec<u8> {
    data
  }
}

#[tokio::test]
async fn test_small_binary() {
  let app = BytesController::router();
  let data = vec![1, 2, 3, 4, 5];
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"size:5");
}

#[tokio::test]
async fn test_large_binary() {
  let app = BytesController::router();
  let data = vec![0u8; 100000];
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"size:100000");
}

#[tokio::test]
async fn test_empty_binary() {
  let app = BytesController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"size:0");
}

#[tokio::test]
async fn test_echo_binary() {
  let app = BytesController::router();
  let data = vec![10, 20, 30, 40, 50];
  let expected = data.clone();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/echo")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], &expected[..]);
}

#[tokio::test]
async fn test_all_byte_values() {
  let app = BytesController::router();
  let data: Vec<u8> = (0..=255).collect();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"size:256");
}

#[tokio::test]
async fn test_binary_with_content_type() {
  let app = BytesController::router();
  let data = vec![1, 2, 3];
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .header("content-type", "application/octet-stream")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_binary_image_data() {
  let app = BytesController::router();
  // Simulate image header bytes
  let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/upload")
        .header("content-type", "image/jpeg")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_random_bytes() {
  let app = BytesController::router();
  let data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
  let expected = data.clone();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/echo")
        .body(Body::from(data))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], &expected[..]);
}
