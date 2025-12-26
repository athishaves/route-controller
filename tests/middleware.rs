// Integration tests for middleware functionality

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::middleware::Next;
use route_controller::{controller, get, post};
use tower::ServiceExt;

// Middleware that tracks if it was called
async fn tracking_middleware(request: Request<Body>, next: Next) -> Response<Body> {
  let mut response = next.run(request).await;
  response
    .headers_mut()
    .insert("x-middleware-called", "true".parse().unwrap());
  response
}

// Middleware that can be used for testing order
async fn middleware_a(request: Request<Body>, next: Next) -> Response<Body> {
  let mut response = next.run(request).await;
  response
    .headers_mut()
    .insert("x-middleware-a", "called".parse().unwrap());
  response
}

async fn middleware_b(request: Request<Body>, next: Next) -> Response<Body> {
  let mut response = next.run(request).await;
  response
    .headers_mut()
    .insert("x-middleware-b", "called".parse().unwrap());
  response
}

// Test single middleware
struct SingleMiddlewareController;

#[controller(middleware = tracking_middleware)]
impl SingleMiddlewareController {
  #[get("/test")]
  async fn test() -> &'static str {
    "ok"
  }
}

#[tokio::test]
async fn test_single_middleware() {
  let app = SingleMiddlewareController::router();

  let response = app
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Verify middleware added the header
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );
}

// Test multiple middlewares
struct MultipleMiddlewareController;

#[controller(middleware = middleware_a, middleware = middleware_b)]
impl MultipleMiddlewareController {
  #[get("/test")]
  async fn test() -> &'static str {
    "ok"
  }
}

#[tokio::test]
async fn test_multiple_middlewares() {
  let app = MultipleMiddlewareController::router();

  let response = app
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Both middlewares should have been called
  assert_eq!(response.headers().get("x-middleware-a").unwrap(), "called");
  assert_eq!(response.headers().get("x-middleware-b").unwrap(), "called");
}

// Test middleware with route prefix
struct MiddlewareWithPrefixController;

#[controller(path = "/api", middleware = tracking_middleware)]
impl MiddlewareWithPrefixController {
  #[get("/users")]
  async fn users() -> &'static str {
    "users"
  }
}

#[tokio::test]
async fn test_middleware_with_route_prefix() {
  let app = MiddlewareWithPrefixController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Middleware should be applied
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );
}

// Test middleware with extractors
struct MiddlewareWithExtractorsController;

#[controller(middleware = tracking_middleware)]
impl MiddlewareWithExtractorsController {
  #[get("/{id}", extract(id = Path))]
  async fn get_item(id: u32) -> String {
    format!("Item {}", id)
  }

  #[post("/", extract(data = Json))]
  async fn create(data: TestData) -> String {
    format!("Created: {}", data.value)
  }
}

#[derive(serde::Deserialize)]
struct TestData {
  value: String,
}

#[tokio::test]
async fn test_middleware_with_path_extractor() {
  let app = MiddlewareWithExtractorsController::router();

  let response = app
    .oneshot(Request::builder().uri("/42").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Middleware should be applied
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Item 42");
}

#[tokio::test]
async fn test_middleware_with_json_extractor() {
  let app = MiddlewareWithExtractorsController::router();

  let body = r#"{"value":"test"}"#;

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Middleware should be applied
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );
}

// Test that middleware runs for all routes
struct MultiRouteMiddlewareController;

#[controller(middleware = tracking_middleware)]
impl MultiRouteMiddlewareController {
  #[get("/route1")]
  async fn route1() -> &'static str {
    "route1"
  }

  #[get("/route2")]
  async fn route2() -> &'static str {
    "route2"
  }

  #[post("/route3")]
  async fn route3() -> &'static str {
    "route3"
  }
}

#[tokio::test]
async fn test_middleware_applies_to_all_routes() {
  let app = MultiRouteMiddlewareController::router();

  // Test route1
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/route1")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );

  // Test route2
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/route2")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );

  // Test route3
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/route3")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(
    response.headers().get("x-middleware-called").unwrap(),
    "true"
  );
}

// Test controller without middleware
struct NoMiddlewareController;

#[controller]
impl NoMiddlewareController {
  #[get("/test")]
  async fn test() -> &'static str {
    "ok"
  }
}

#[tokio::test]
async fn test_no_middleware() {
  let app = NoMiddlewareController::router();

  let response = app
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // No middleware header should be present
  assert!(response.headers().get("x-middleware-called").is_none());
}

// Test combined: prefix + multiple middlewares + extractors
struct CombinedController;

#[controller(path = "/api/v1", middleware = middleware_a, middleware = middleware_b)]
impl CombinedController {
  #[get("/{id}", extract(id = Path))]
  async fn get_item(id: u32) -> String {
    format!("Item {}", id)
  }
}

#[tokio::test]
async fn test_combined_prefix_middlewares_extractors() {
  let app = CombinedController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/v1/999")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  // Both middlewares should be applied
  assert_eq!(response.headers().get("x-middleware-a").unwrap(), "called");
  assert_eq!(response.headers().get("x-middleware-b").unwrap(), "called");

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Item 999");
}
