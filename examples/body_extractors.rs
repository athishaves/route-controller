use route_controller::{controller, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct LoginForm {
  username: String,
  password: String,
  remember_me: Option<bool>,
}

#[derive(Deserialize, Serialize)]
struct FileMetadata {
  filename: String,
  size: usize,
  content_type: String,
}

struct BodyController;

#[controller(path = "/api")]
impl BodyController {
  // Form extractor - handles application/x-www-form-urlencoded and multipart/form-data
  #[post("/login", extract(form = Form))]
  async fn handle_form(form: LoginForm) -> axum::Json<LoginForm> {
    println!(
      "Form data: username={}, remember_me={:?}",
      form.username, form.remember_me
    );
    axum::Json(form)
  }

  // Bytes extractor - handles raw binary data
  #[post("/upload", extract(data = Bytes))]
  async fn handle_bytes(data: Vec<u8>) -> axum::Json<FileMetadata> {
    println!("Received {} bytes of data", data.len());
    axum::Json(FileMetadata {
      filename: "upload.bin".to_string(),
      size: data.len(),
      content_type: "application/octet-stream".to_string(),
    })
  }

  // Text extractor - handles text/plain
  #[post("/text", extract(content = Text))]
  async fn handle_text(content: String) -> String {
    println!("Received text: {} characters", content.len());
    format!("Processed {} characters of text", content.len())
  }

  // Html extractor - handles text/html
  #[post("/html", extract(html = Html))]
  async fn handle_html(html: String) -> String {
    println!("Received HTML: {} characters", html.len());
    format!("Processed {} characters of HTML", html.len())
  }

  // Xml extractor - handles application/xml or text/xml
  #[post("/xml", extract(xml = Xml))]
  async fn handle_xml(xml: String) -> String {
    println!("Received XML: {} characters", xml.len());
    format!("Processed {} characters of XML", xml.len())
  }

  // JavaScript extractor - handles application/javascript or text/javascript
  #[post("/script", extract(code = JavaScript))]
  async fn handle_javascript(code: String) -> String {
    println!("Received JavaScript: {} characters", code.len());
    format!("Processed {} characters of JavaScript", code.len())
  }
}

#[tokio::main]
async fn main() {
  let app = BodyController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3005")
    .await
    .unwrap();

  println!("🚀 Server running on http://127.0.0.1:3005");
  println!("\nTry these endpoints:");
  println!("  POST /api/login       - Form data (application/x-www-form-urlencoded)");
  println!("  POST /api/upload      - Binary data (application/octet-stream)");
  println!("  POST /api/text        - Plain text (text/plain)");
  println!("  POST /api/html        - HTML content (text/html)");
  println!("  POST /api/xml         - XML content (application/xml)");
  println!("  POST /api/script      - JavaScript (application/javascript)");

  axum::serve(listener, app).await.unwrap();
}
