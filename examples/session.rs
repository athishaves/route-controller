use route_controller::{controller, get, post, put};
use serde::{Deserialize, Serialize};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserProfile {
  id: u32,
  name: String,
  email: String,
  role: String,
}

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

  // Save a struct to session
  #[post("/profile/save", extract(profile = Json))]
  async fn save_profile(session: Session, profile: UserProfile) -> String {
    let _ = session.insert("profile", profile.clone()).await;
    format!("Profile saved: {:?}", profile)
  }

  // Extract struct from session using SessionParam
  #[get("/profile/get", extract(profile = SessionParam))]
  async fn get_saved_profile(profile: Option<UserProfile>) -> String {
    match profile {
      Some(p) => format!(
        "Profile from session - ID: {}, Name: {}, Email: {}, Role: {}",
        p.id, p.name, p.email, p.role
      ),
      None => "No profile found in session".to_string(),
    }
  }

  // Direct struct access from session (without SessionParam)
  #[get("/profile/direct")]
  async fn get_profile_direct(session: Session) -> String {
    match session.get::<UserProfile>("profile").await {
      Ok(Some(profile)) => format!(
        "Direct Profile - ID: {}, Name: {}, Email: {}, Role: {}",
        profile.id, profile.name, profile.email, profile.role
      ),
      _ => "No profile found in session".to_string(),
    }
  }

  // Helper endpoint to clear session data
  #[put("/session/clear")]
  async fn clear_session(session: Session) -> &'static str {
    let _ = session.clear().await;
    "Session cleared"
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
  println!("\n4. Save and retrieve struct from session:");
  println!("   curl -X POST http://localhost:3011/users/profile/save -c cookies.txt -H 'Content-Type: application/json' -d '{{\"id\":1,\"name\":\"John Doe\",\"email\":\"john@example.com\",\"role\":\"admin\"}}'");
  println!("   curl http://localhost:3011/users/profile/get -b cookies.txt");
  println!("   curl http://localhost:3011/users/profile/direct -b cookies.txt");
  println!("\n5. Clear session:");
  println!("   curl -X PUT http://localhost:3011/users/session/clear -b cookies.txt");

  axum::serve(listener, app).await.unwrap();
}
