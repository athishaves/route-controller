use route_controller::{controller, get, put};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

struct UserController;

#[controller(path = "/users")]
impl UserController {
  // Single SessionParam extractor
  #[get("/profile", extract(user_id = SessionParam))]
  async fn get_profile(user_id: String) -> String {
    format!("User ID: {}", user_id)
  }

  // Multiple SessionParam extractors
  #[get(
    "/details",
    extract(
      user_id = SessionParam,
      username = SessionParam,
      role = SessionParam,
    )
  )]
  async fn get_details(user_id: String, username: String, role: String) -> String {
    format!(
      "User Details - ID: {}, Username: {}, Role: {}",
      user_id, username, role
    )
  }

  // Mixed extractors: Path + SessionParam
  #[get(
    "/{id}/settings",
    extract(
      id = Path,
      user_id = SessionParam,
    )
  )]
  async fn get_settings(id: u32, user_id: String) -> String {
    format!("Settings for user {} (session user: {})", id, user_id)
  }

  // Mixed extractors: Path + SessionParam + HeaderParam
  #[get(
    "/{id}/secure",
    extract(
      id = Path,
      user_id = SessionParam,
      authorization = HeaderParam,
    )
  )]
  async fn secure_endpoint(id: u32, user_id: String, authorization: String) -> String {
    format!(
      "Secure endpoint - ID: {}, Session User: {}, Auth: {}",
      id, user_id, authorization
    )
  }

  // Helper endpoint to set session data (for testing)
  #[put("/session/init")]
  async fn init_session(session: Session) -> &'static str {
    let _ = session.insert("user_id", "12345".to_string()).await;
    let _ = session.insert("username", "john_doe".to_string()).await;
    let _ = session.insert("role", "admin".to_string()).await;
    "Session initialized"
  }

  // Helper endpoint to clear session data
  #[put("/session/clear")]
  async fn clear_session(session: Session) -> &'static str {
    let _ = session.clear().await;
    "Session cleared"
  }

  // Direct session access (without SessionParam)
  #[get("/session/info")]
  async fn get_session_info(session: Session) -> String {
    let user_id = session
      .get::<String>("user_id")
      .await
      .ok()
      .flatten()
      .unwrap_or_default();
    let username = session
      .get::<String>("username")
      .await
      .ok()
      .flatten()
      .unwrap_or_default();

    format!("Session Info - User ID: {}, Username: {}", user_id, username)
  }
}

#[tokio::main]
async fn main() {
  // Create session store
  let session_store = MemoryStore::default();

  // Create session layer
  let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(false) // Set to true in production with HTTPS
    .with_same_site(tower_sessions::cookie::SameSite::Lax);

  // Create router and apply session layer
  let app = UserController::router().layer(session_layer);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3011")
    .await
    .unwrap();

  println!("Server running on http://127.0.0.1:3011");
  println!("\nTest the session endpoints:");
  println!("1. Initialize session:");
  println!("   curl -X PUT http://localhost:3011/users/session/init -c cookies.txt");
  println!("\n2. Test SessionParam extraction:");
  println!("   curl http://localhost:3011/users/profile -b cookies.txt");
  println!("   curl http://localhost:3011/users/details -b cookies.txt");
  println!("\n3. Test mixed extractors:");
  println!("   curl http://localhost:3011/users/42/settings -b cookies.txt");
  println!("   curl http://localhost:3011/users/42/secure -b cookies.txt -H 'authorization: Bearer token123'");
  println!("\n4. Clear session:");
  println!("   curl -X PUT http://localhost:3011/users/session/clear -b cookies.txt");

  axum::serve(listener, app).await.unwrap();
}
