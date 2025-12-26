use axum::{
  body::Body,
  http::{Request, StatusCode},
  Router,
};
use route_controller::{controller, delete, get, post, put};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

// Simple application state for testing
#[derive(Clone)]
struct AppState {
  counter: Arc<RwLock<i32>>,
  message: Arc<String>,
}

impl AppState {
  fn new(initial_count: i32, message: &str) -> Self {
    Self {
      counter: Arc::new(RwLock::new(initial_count)),
      message: Arc::new(message.to_string()),
    }
  }
}

#[derive(Serialize, Deserialize)]
struct CounterResponse {
  count: i32,
  message: String,
}

#[derive(Deserialize)]
struct IncrementRequest {
  amount: i32,
}

// Test controller using state
struct StateController;

#[controller(path = "/state")]
impl StateController {
  #[get("/count", extract(state = State))]
  async fn get_count(state: AppState) -> axum::Json<CounterResponse> {
    let count = *state.counter.read().await;
    axum::Json(CounterResponse {
      count,
      message: (*state.message).clone(),
    })
  }

  #[post("/increment", extract(state = State))]
  async fn increment(state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter += 1;
    axum::Json(CounterResponse {
      count: *counter,
      message: (*state.message).clone(),
    })
  }

  #[post("/increment-by", extract(req = Json, state = State))]
  async fn increment_by(req: IncrementRequest, state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter += req.amount;
    axum::Json(CounterResponse {
      count: *counter,
      message: (*state.message).clone(),
    })
  }

  #[delete("/reset", extract(state = State))]
  async fn reset_counter(state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter = 0;
    axum::Json(CounterResponse {
      count: 0,
      message: "Counter reset".to_string(),
    })
  }

  #[get("/message", extract(state = State))]
  async fn get_message(state: AppState) -> String {
    (*state.message).clone()
  }
}

// Test controller with state and path parameters
struct ItemController;

#[controller(path = "/items")]
impl ItemController {
  #[get("/{id}", extract(id = Path, state = State))]
  async fn get_item(id: u32, state: AppState) -> axum::Json<serde_json::Value> {
    let count = *state.counter.read().await;
    axum::Json(serde_json::json!({
      "id": id,
      "counter": count,
      "message": (*state.message).clone()
    }))
  }

  #[put("/{id}/count/{value}", extract(id = Path, value = Path, state = State))]
  async fn update_item_count(
    id: u32,
    value: i32,
    state: AppState,
  ) -> axum::Json<serde_json::Value> {
    let mut counter = state.counter.write().await;
    *counter = value;
    axum::Json(serde_json::json!({
      "id": id,
      "new_count": value,
      "message": (*state.message).clone()
    }))
  }
}

#[tokio::test]
async fn test_state_basic_access() {
  let state = AppState::new(42, "Hello from state");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .uri("/state/count")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: CounterResponse = serde_json::from_slice(&body).unwrap();
  assert_eq!(json.count, 42);
  assert_eq!(json.message, "Hello from state");
}

#[tokio::test]
async fn test_state_mutation() {
  let state = AppState::new(0, "Counting");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state.clone());

  // Increment
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/state/increment")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: CounterResponse = serde_json::from_slice(&body).unwrap();
  assert_eq!(json.count, 1);

  // Verify the state was actually mutated
  let count = *state.counter.read().await;
  assert_eq!(count, 1);
}

#[tokio::test]
async fn test_state_with_json_body() {
  let state = AppState::new(10, "Testing");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state.clone());

  let body = serde_json::json!({ "amount": 5 }).to_string();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/state/increment-by")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: CounterResponse = serde_json::from_slice(&body).unwrap();
  assert_eq!(json.count, 15);
}

#[tokio::test]
async fn test_state_reset() {
  let state = AppState::new(100, "Reset test");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state.clone());

  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/state/reset")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: CounterResponse = serde_json::from_slice(&body).unwrap();
  assert_eq!(json.count, 0);

  // Verify state was reset
  let count = *state.counter.read().await;
  assert_eq!(count, 0);
}

#[tokio::test]
async fn test_state_message_only() {
  let state = AppState::new(0, "Just a message");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .uri("/state/message")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let text = String::from_utf8(body.to_vec()).unwrap();
  assert_eq!(text, "Just a message");
}

#[tokio::test]
async fn test_state_with_path_param() {
  let state = AppState::new(77, "Path test");
  let app = Router::new()
    .merge(ItemController::router())
    .with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .uri("/items/123")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["id"], 123);
  assert_eq!(json["counter"], 77);
  assert_eq!(json["message"], "Path test");
}

#[tokio::test]
async fn test_state_with_multiple_path_params() {
  let state = AppState::new(50, "Multi-path test");
  let app = Router::new()
    .merge(ItemController::router())
    .with_state(state.clone());

  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/items/456/count/999")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["id"], 456);
  assert_eq!(json["new_count"], 999);

  // Verify state was updated
  let count = *state.counter.read().await;
  assert_eq!(count, 999);
}

#[tokio::test]
async fn test_multiple_controllers_share_state() {
  let state = AppState::new(5, "Shared state");
  let app = Router::new()
    .merge(StateController::router())
    .merge(ItemController::router())
    .with_state(state.clone());

  // Modify state via StateController
  let _ = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/state/increment")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  // Verify state change is visible in ItemController
  let response = app
    .oneshot(
      Request::builder()
        .uri("/items/1")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
  assert_eq!(json["counter"], 6); // Was 5, incremented to 6
}

#[tokio::test]
async fn test_concurrent_state_access() {
  let state = AppState::new(0, "Concurrency test");
  let app = Router::new()
    .merge(StateController::router())
    .with_state(state.clone());

  // Spawn multiple concurrent requests
  let mut handles = vec![];
  for _ in 0..10 {
    let app_clone = app.clone();
    let handle = tokio::spawn(async move {
      app_clone
        .oneshot(
          Request::builder()
            .method("POST")
            .uri("/state/increment")
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap()
    });
    handles.push(handle);
  }

  // Wait for all requests to complete
  for handle in handles {
    handle.await.unwrap();
  }

  // Verify final count
  let final_count = *state.counter.read().await;
  assert_eq!(final_count, 10);
}
