//! Text body extractors (Text, Html, Xml, JavaScript)
//!
//! Run: cargo run --example 06_text_body
//! Test: curl -X POST http://localhost:3000/text -H "Content-Type: text/plain" -d "Hello World"

use route_controller::{controller, post};

struct ContentController;

#[controller]
impl ContentController {
  #[post("/text", extract(content = Text))]
  async fn handle_text(content: String) -> String {
    format!("Received {} characters", content.len())
  }

  #[post("/html", extract(html = Html))]
  async fn handle_html(html: String) -> String {
    format!("Received {} characters of HTML", html.len())
  }

  #[post("/xml", extract(xml = Xml))]
  async fn handle_xml(xml: String) -> String {
    format!("Received {} characters of XML", xml.len())
  }

  #[post("/script", extract(code = JavaScript))]
  async fn handle_javascript(code: String) -> String {
    format!("Received {} characters of JavaScript", code.len())
  }
}

#[tokio::main]
async fn main() {
  let app = ContentController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3000");
  println!("\nTry:");
  println!(
    "  curl -X POST http://localhost:3000/text -H 'Content-Type: text/plain' -d 'Hello World'"
  );
  println!(
    "  curl -X POST http://localhost:3000/html -H 'Content-Type: text/html' -d '<h1>Hello</h1>'"
  );
  println!(
    "  curl -X POST http://localhost:3000/xml -H 'Content-Type: application/xml' -d '<root>data</root>'"
  );
  println!(
    "  curl -X POST http://localhost:3000/script -H 'Content-Type: application/javascript' -d 'console.log(\"hi\");'"
  );

  axum::serve(listener, app).await.unwrap();
}
