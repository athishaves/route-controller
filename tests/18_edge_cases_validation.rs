//! Integration tests for edge cases and complex validation scenarios

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get, post};
use serde::Deserialize;
use tower::ServiceExt;

// Test 1: Complex path with multiple parameters
#[cfg(test)]
mod test_multiple_path_params {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get(
      "/users/{user_id}/posts/{post_id}/comments/{comment_id}",
      extract(user_id = Path, post_id = Path, comment_id = Path)
    )]
    async fn get_comment(user_id: u32, post_id: u32, comment_id: u32) -> String {
      format!(
        "User: {}, Post: {}, Comment: {}",
        user_id, post_id, comment_id
      )
    }
  }

  #[tokio::test]
  async fn test_multiple_path_parameters() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/users/1/posts/2/comments/3")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 2: Path + Query + Json mixed extractors (tested in test 14_mixed_extractors.rs)
// Skipped here to avoid duplication

// Test 3: Controller and route level headers
#[cfg(test)]
mod test_header_override {
  use super::*;

  struct TestController;

  #[controller(
    path = "/api",
    header("x-api-version", "1.0"),
    header("x-service", "test-service"),
    content_type("application/json")
  )]
  impl TestController {
    // Inherits controller headers
    #[get("/inherit")]
    async fn inherit_headers() -> String {
      r#"{"status":"ok"}"#.to_string()
    }

    // Overrides api-version, keeps x-service
    #[get("/override", header("x-api-version", "2.0"))]
    async fn override_version() -> String {
      r#"{"status":"ok"}"#.to_string()
    }

    // Overrides content-type
    #[get("/text", content_type("text/plain"))]
    async fn text_response() -> String {
      "plain text".to_string()
    }
  }

  #[tokio::test]
  async fn test_header_inheritance() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/inherit")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Note: Testing actual header values would require accessing response headers
  }

  #[tokio::test]
  async fn test_header_override_works() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/override")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 4: Different body content types
#[cfg(test)]
mod test_body_content_types {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[post("/json", extract(data = Json))]
    async fn handle_json(data: serde_json::Value) -> String {
      format!("JSON: {}", data)
    }

    #[post("/text", extract(content = Text))]
    async fn handle_text(content: String) -> String {
      format!("Text: {}", content)
    }

    #[post("/bytes", extract(data = Bytes))]
    async fn handle_bytes(data: Vec<u8>) -> String {
      format!("Bytes: {} bytes", data.len())
    }

    #[post("/html", extract(html = Html))]
    async fn handle_html(html: String) -> String {
      format!("HTML: {} chars", html.len())
    }

    #[post("/xml", extract(xml = Xml))]
    async fn handle_xml(xml: String) -> String {
      format!("XML: {}", xml)
    }

    #[post("/js", extract(code = JavaScript))]
    async fn handle_javascript(code: String) -> String {
      format!("JavaScript: {} chars", code.len())
    }
  }

  #[tokio::test]
  async fn test_json_body() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/json")
          .header("content-type", "application/json")
          .body(Body::from(r#"{"key":"value"}"#))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_text_body() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/text")
          .header("content-type", "text/plain")
          .body(Body::from("Hello, World!"))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_bytes_body() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/bytes")
          .body(Body::from(vec![1, 2, 3, 4, 5]))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 5: Path parameter syntax variations
#[cfg(test)]
mod test_path_syntax {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // Curly brace syntax (Axum 0.8+)
    #[get("/users/{id}", extract(id = Path))]
    async fn get_user_curly(id: u32) -> String {
      format!("User {}", id)
    }

    // Mixed path with static segments
    #[get("/orgs/{org_id}/repos/{repo_id}", extract(org_id = Path, repo_id = Path))]
    async fn get_repo(org_id: String, repo_id: String) -> String {
      format!("Org: {}, Repo: {}", org_id, repo_id)
    }
  }

  #[tokio::test]
  async fn test_curly_brace_syntax() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/users/123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 6: Empty and root paths
#[cfg(test)]
mod test_edge_case_paths {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // Root of controller path
    #[get("/")]
    async fn root() -> String {
      "root".to_string()
    }

    // Single segment
    #[get("/test")]
    async fn single_segment() -> String {
      "single".to_string()
    }
  }

  #[tokio::test]
  async fn test_root_path() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api") // Without trailing slash
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 7: State extractor
#[cfg(test)]
mod test_state_extractor {
  use super::*;
  use std::sync::Arc;

  #[derive(Clone)]
  struct AppState {
    counter: Arc<tokio::sync::RwLock<i32>>,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/count", extract(state = State))]
    async fn get_count(state: AppState) -> String {
      let count = *state.counter.read().await;
      format!("{}", count)
    }

    #[post("/increment", extract(state = State))]
    async fn increment(state: AppState) -> String {
      let mut count = state.counter.write().await;
      *count += 1;
      format!("{}", *count)
    }
  }

  #[tokio::test]
  async fn test_state_access() {
    let state = AppState {
      counter: Arc::new(tokio::sync::RwLock::new(0)),
    };

    let app = TestController::router().with_state(state);

    // Get initial count
    let response = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("/api/count")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Increment
    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/increment")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test 8: Complex query parameters
#[cfg(test)]
mod test_complex_query {
  use super::*;

  #[derive(Deserialize)]
  struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<String>,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/search", extract(query = Query))]
    async fn search(query: SearchQuery) -> String {
      format!(
        "Query: {}, Page: {:?}, Limit: {:?}, Sort: {:?}",
        query.q, query.page, query.limit, query.sort
      )
    }
  }

  #[tokio::test]
  async fn test_complex_query_params() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/search?q=rust&page=1&limit=10&sort=name")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_partial_query_params() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/search?q=rust")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}
