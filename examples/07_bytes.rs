//! Binary data (Bytes) extraction
//!
//! Run: cargo run --example 07_bytes
//! Test: curl -X POST http://localhost:3000/upload --data-binary @somefile.bin

use route_controller::{controller, post};

struct UploadController;

#[controller]
impl UploadController {
  #[post("/upload", extract(data = Bytes))]
  async fn upload(data: Vec<u8>) -> String {
    format!("Received {} bytes", data.len())
  }
}

#[tokio::main]
async fn main() {
  let app = UploadController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!("  echo 'binary data' > test.bin");
  println!("  curl -X POST http://localhost:3000/upload --data-binary @test.bin");

  axum::serve(listener, app).await.unwrap();
}
