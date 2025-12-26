//! Application state management
//!
//! Run: cargo run --example 11_state
//! Test: curl http://localhost:3000/counter

use route_controller::{controller, get, post};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
  counter: Arc<RwLock<i32>>,
}

#[derive(Serialize)]
struct CounterResponse {
  count: i32,
}

struct CounterController;

#[controller(path = "/counter")]
impl CounterController {
  #[get(extract(state = State))]
  async fn get(state: AppState) -> axum::Json<CounterResponse> {
    let count = *state.counter.read().await;
    axum::Json(CounterResponse { count })
  }

  #[post("/increment", extract(state = State))]
  async fn increment(state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter += 1;
    axum::Json(CounterResponse { count: *counter })
  }
}

#[tokio::main]
async fn main() {
  let app_state = AppState {
    counter: Arc::new(RwLock::new(0)),
  };

  let app = CounterController::router().with_state(app_state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  curl http://localhost:3000/counter");
  println!("  curl -X POST http://localhost:3000/counter/increment");
  println!("  curl http://localhost:3000/counter");

  axum::serve(listener, app).await.unwrap();
}
