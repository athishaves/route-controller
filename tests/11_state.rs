//! Integration tests for State extractor
//!
//! Tests application state management

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, delete, get, post};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

#[derive(Clone)]
struct AppState {
  counter: Arc<RwLock<i32>>,
  message: Arc<RwLock<String>>,
}

struct StateController;

#[controller(path = "/api")]
impl StateController {
  #[get("/counter", extract(state = State))]
  async fn get_counter(state: AppState) -> String {
    let count = *state.counter.read().await;
    format!("count:{}", count)
  }

  #[post("/counter", extract(state = State))]
  async fn increment(state: AppState) -> String {
    let mut counter = state.counter.write().await;
    *counter += 1;
    format!("count:{}", *counter)
  }

  #[delete("/counter", extract(state = State))]
  async fn reset(state: AppState) -> String {
    let mut counter = state.counter.write().await;
    *counter = 0;
    "count:0".to_string()
  }

  #[get("/message", extract(state = State))]
  async fn get_message(state: AppState) -> String {
    let msg = state.message.read().await;
    format!("msg:{}", msg)
  }

  #[post("/message/{id}", extract(id = Path, state = State))]
  async fn set_message(id: u32, state: AppState) -> String {
    let mut msg = state.message.write().await;
    *msg = format!("Message {}", id);
    format!("msg:{}", msg)
  }
}

#[tokio::test]
async fn test_read_state() {
  let state = AppState {
    counter: Arc::new(RwLock::new(0)),
    message: Arc::new(RwLock::new("hello".to_string())),
  };

  let app = StateController::router().with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/counter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"count:0");
}

#[tokio::test]
async fn test_modify_state() {
  let state = AppState {
    counter: Arc::new(RwLock::new(0)),
    message: Arc::new(RwLock::new("hello".to_string())),
  };

  let app = StateController::router().with_state(state);

  // Increment
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/counter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"count:1");

  // Read modified state
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/counter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"count:1");
}

#[tokio::test]
async fn test_reset_state() {
  let state = AppState {
    counter: Arc::new(RwLock::new(10)),
    message: Arc::new(RwLock::new("test".to_string())),
  };

  let app = StateController::router().with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/api/counter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"count:0");
}

#[tokio::test]
async fn test_path_and_state() {
  let state = AppState {
    counter: Arc::new(RwLock::new(0)),
    message: Arc::new(RwLock::new("".to_string())),
  };

  let app = StateController::router().with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/message/42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"msg:Message 42");
}

#[tokio::test]
async fn test_concurrent_state_access() {
  let state = AppState {
    counter: Arc::new(RwLock::new(0)),
    message: Arc::new(RwLock::new("test".to_string())),
  };

  let app = StateController::router().with_state(state);

  // Multiple increments
  let mut handles = vec![];
  for _ in 0..5 {
    let app_clone = app.clone();
    let handle = tokio::spawn(async move {
      app_clone
        .oneshot(
          Request::builder()
            .method("POST")
            .uri("/api/counter")
            .body(Body::empty())
            .unwrap(),
        )
        .await
    });
    handles.push(handle);
  }

  for handle in handles {
    handle.await.unwrap().unwrap();
  }

  // Check final count
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/counter")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"count:5");
}
