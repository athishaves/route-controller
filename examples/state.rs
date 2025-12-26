use route_controller::{controller, delete, get, post};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simple application state - just a counter
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
  // Get current counter value
  #[get(extract(state = State))]
  async fn get_count(state: AppState) -> axum::Json<CounterResponse> {
    let count = *state.counter.read().await;
    axum::Json(CounterResponse { count })
  }

  // Increment counter
  #[post("/increment", extract(state = State))]
  async fn increment(state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter += 1;
    axum::Json(CounterResponse { count: *counter })
  }

  // Reset counter to zero
  #[delete(extract(state = State))]
  async fn reset(state: AppState) -> axum::Json<CounterResponse> {
    let mut counter = state.counter.write().await;
    *counter = 0;
    axum::Json(CounterResponse { count: 0 })
  }
}

struct HelloController;

#[controller]
impl HelloController {
  #[get("/hello", extract(state = State))]
  async fn hello(state: AppState) -> String {
    state.counter.read().await.to_string()
  }
}

#[tokio::main]
async fn main() {
  // Initialize application state with counter starting at 0
  let app_state = AppState {
    counter: Arc::new(RwLock::new(0)),
  };

  // Build router with state
  let app = axum::Router::new()
    .merge(CounterController::router())
    .merge(HelloController::router())
    // State will be injected for all the routes
    .with_state(app_state);

  // Start server
  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry these endpoints:");
  println!("  GET    /counter");
  println!("  POST   /counter/increment");
  println!("  DELETE /counter");

  axum::serve(listener, app).await.unwrap();
}
