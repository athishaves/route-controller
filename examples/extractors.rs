use route_controller::{controller, get, post, put};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
  name: String,
  email: String,
}

#[derive(Deserialize)]
struct SearchFilters {
  query: Option<String>,
  limit: Option<u32>,
}

#[derive(Deserialize)]
struct SearchParams {
  query: Option<String>,
  limit: Option<u32>,
  sort: Option<String>,
}

struct UserController;

#[controller(path = "/users")]
impl UserController {
  // Single Json extractor
  #[post(extract(user = Json))]
  async fn create(user: User) -> String {
    format!("Created user: {} ({})", user.name, user.email)
  }

  // Multiple Path extractors
  // Extract declaration order does not matter. It will be matched by parameter name
  // Even if params are not used, they need to be declared to match the route
  #[get(
		"/{id}/posts/{post_id}",
		extract(post_id = Path, id = Path)
	)]
  async fn get_user_post(id: u32, post_id: u32) -> String {
    format!("User {} - Post {}", id, post_id)
  }

  // Multiple Query extractors are not allowed
  // Axum does not support multiple Query extractors
  // A single struct should be used instead
  #[get("/search", extract(params = Query))]
  async fn search(params: SearchParams) -> String {
    format!(
      "Searching with filters {} {} and sort: {}",
      params.query.unwrap_or_default(),
      params.limit.unwrap_or(10),
      params.sort.unwrap_or_default()
    )
  }

  // Mixed extractors: Path + Json
  #[put("/{id}", extract(id = Path, user = Json))]
  async fn update(id: u32, user: User) -> String {
    format!("Updated user {}: {} ({})", id, user.name, user.email)
  }

  // Mixed extractors: Path + Query
  #[get("/{id}/search", extract(id = Path, filters = Query))]
  async fn search_user(id: u32, filters: SearchFilters) -> String {
    format!(
      "Searching for user: {} with filters {} {}",
      id,
      filters.limit.unwrap_or(0),
      filters.query.unwrap_or("".to_string()),
    )
  }

  // Mixed extractors: Path + Query + Json
  #[put(
    "/{id}/posts/{post_id}/update",
    extract(id = Path, post_id = Path, filters = Query, user = Json)
  )]
  async fn complex_update(
    id: u32,
    post_id: u32,
    filters: SearchFilters,
    user: User,
  ) -> String {
    format!(
      "Complex update for user post {}-{}: {} ({}), filters: {} {}",
      id,
      post_id,
      user.name,
      user.email,
      filters.limit.unwrap_or(0),
      filters.query.unwrap_or("".to_string()),
    )
  }

  // No extractors
  #[get("/list")]
  async fn list() -> &'static str {
    "User list"
  }
}

#[tokio::main]
async fn main() {
  let app = UserController::router();

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3010")
    .await
    .unwrap();

  println!("Server running on http://127.0.0.1:3010");

  axum::serve(listener, app).await.unwrap();
}
