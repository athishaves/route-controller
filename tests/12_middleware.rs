//! Integration tests for middleware functionality
//!
//! Tests middleware application

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use route_controller::{controller, get};
use serde::Deserialize;
use tower::ServiceExt;

struct MiddlewareController;

#[controller(path = "/api", middleware = auth_middleware)]
impl MiddlewareController {
  #[get("/public")]
  async fn public() -> &'static str {
    "public"
  }

  #[get("/private")]
  async fn private() -> &'static str {
    "private"
  }
}

async fn auth_middleware(request: Request<Body>, next: Next) -> Response {
  if let Some(auth) = request.headers().get("authorization") {
    if auth == "Bearer valid_token" {
      return next.run(request).await;
    }
  }
  Response::builder()
    .status(StatusCode::UNAUTHORIZED)
    .body(Body::empty())
    .unwrap()
}

#[tokio::test]
async fn test_middleware_authenticated() {
  let app = MiddlewareController::router();

  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/api/public")
        .header("authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_middleware_unauthorized() {
  let app = MiddlewareController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/public")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_middleware_invalid_token() {
  let app = MiddlewareController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/private")
        .header("authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// Test multiple middlewares
async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
  let response = next.run(request).await;
  response
}

struct MultiMiddlewareController;

#[controller(
  path = "/multi",
  middleware = auth_middleware,
  middleware = logging_middleware,
)]
impl MultiMiddlewareController {
  #[get("/test")]
  async fn test() -> &'static str {
    "ok"
  }
}

#[tokio::test]
async fn test_multiple_middlewares() {
  let app = MultiMiddlewareController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/multi/test")
        .header("authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

// Test middleware with extractors
#[derive(Deserialize)]
struct SearchQuery {
  q: String,
}

struct MiddlewareWithExtractorController;

#[controller(path = "/mw", middleware = auth_middleware)]
impl MiddlewareWithExtractorController {
  #[get("/users/{id}", extract(id = Path))]
  async fn get_user(id: u32) -> String {
    format!("user:{}", id)
  }

  #[get("/search", extract(query = Query))]
  async fn search(query: SearchQuery) -> String {
    format!("search:{}", query.q)
  }
}

#[tokio::test]
async fn test_middleware_with_path_param() {
  let app = MiddlewareWithExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/mw/users/123")
        .header("authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:123");
}

#[tokio::test]
async fn test_middleware_with_query_param() {
  let app = MiddlewareWithExtractorController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/mw/search?q=rust")
        .header("authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"search:rust");
}
