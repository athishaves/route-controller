use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Deserialize, Serialize)]
struct UploadResponse {
  size: usize,
  hash: String,
}

struct BytesController;

#[controller(path = "/data")]
impl BytesController {
  #[post("/upload", extract(data = Bytes))]
  async fn upload(data: Vec<u8>) -> axum::Json<UploadResponse> {
    // Simple hash calculation (sum of bytes mod 256)
    let hash = format!("{:x}", data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32)));

    axum::Json(UploadResponse {
      size: data.len(),
      hash,
    })
  }

  #[post("/echo", extract(data = Bytes))]
  async fn echo(data: Vec<u8>) -> Vec<u8> {
    data
  }
}

#[tokio::test]
async fn test_bytes_extractor_empty() {
  let app = BytesController::router();

  let request = Request::builder()
    .uri("/data/upload")
    .method("POST")
    .body(Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: UploadResponse = serde_json::from_slice(&body).unwrap();

  assert_eq!(result.size, 0);
}

#[tokio::test]
async fn test_bytes_extractor_small_binary() {
  let app = BytesController::router();

  let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in bytes

  let request = Request::builder()
    .uri("/data/upload")
    .method("POST")
    .body(Body::from(data.clone()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: UploadResponse = serde_json::from_slice(&body).unwrap();

  assert_eq!(result.size, 5);
}

#[tokio::test]
async fn test_bytes_extractor_large_binary() {
  let app = BytesController::router();

  let data = vec![0xFF; 1024]; // 1KB of 0xFF

  let request = Request::builder()
    .uri("/data/upload")
    .method("POST")
    .body(Body::from(data.clone()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: UploadResponse = serde_json::from_slice(&body).unwrap();

  assert_eq!(result.size, 1024);
}

#[tokio::test]
async fn test_bytes_extractor_echo() {
  let app = BytesController::router();

  let data = vec![1, 2, 3, 4, 5, 255, 128, 64];

  let request = Request::builder()
    .uri("/data/echo")
    .method("POST")
    .body(Body::from(data.clone()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();

  assert_eq!(body.to_vec(), data);
}

#[tokio::test]
async fn test_bytes_extractor_with_content_type() {
  let app = BytesController::router();

  let data = b"Binary file content";

  let request = Request::builder()
    .uri("/data/upload")
    .method("POST")
    .header("content-type", "application/octet-stream")
    .body(Body::from(data.as_ref()))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: UploadResponse = serde_json::from_slice(&body).unwrap();

  assert_eq!(result.size, 19);
}
