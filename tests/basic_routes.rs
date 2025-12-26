// Integration tests for basic route functionality

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, delete, get, head, options, patch, post, put, trace};
use tower::ServiceExt; // for `oneshot`

// Test controller for basic routes
struct BasicController;

#[controller]
impl BasicController {
  #[get("/test")]
  async fn test() -> &'static str {
    "ok"
  }

  #[get("/hello")]
  async fn hello() -> String {
    "Hello, World!".to_string()
  }

  #[post("/create")]
  async fn create() -> &'static str {
    "created"
  }
}

#[tokio::test]
async fn test_simple_get_route() {
  let app = BasicController::router();

  let response = app
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn test_multiple_routes() {
  let app = BasicController::router();

  // Test first route
  let response = app
    .clone()
    .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  // Test second route
  let response = app
    .oneshot(
      Request::builder()
        .uri("/hello")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Hello, World!");
}

#[tokio::test]
async fn test_post_route() {
  let app = BasicController::router();

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/create")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"created");
}

// Test different HTTP methods
struct HttpMethodsController;

#[controller]
impl HttpMethodsController {
  #[get("/resource")]
  async fn get_resource() -> &'static str {
    "GET"
  }

  #[post("/resource")]
  async fn create_resource() -> &'static str {
    "POST"
  }

  #[put("/resource")]
  async fn update_resource() -> &'static str {
    "PUT"
  }

  #[delete("/resource")]
  async fn delete_resource() -> &'static str {
    "DELETE"
  }

  #[patch("/resource")]
  async fn patch_resource() -> &'static str {
    "PATCH"
  }
}

// Test additional HTTP methods
struct AdditionalMethodsController;

#[controller]
impl AdditionalMethodsController {
  #[head("/check")]
  async fn head_check() -> &'static str {
    "HEAD"
  }

  #[options("/config")]
  async fn options_config() -> &'static str {
    "OPTIONS"
  }

  #[trace("/debug")]
  async fn trace_debug() -> &'static str {
    "TRACE"
  }
}

#[tokio::test]
async fn test_all_http_methods() {
  let app = HttpMethodsController::router();

  // Test GET
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("GET")
        .uri("/resource")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"GET");

  // Test POST
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/resource")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"POST");

  // Test PUT
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/resource")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"PUT");

  // Test DELETE
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/resource")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"DELETE");

  // Test PATCH
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri("/resource")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"PATCH");
}

#[tokio::test]
async fn test_additional_http_methods() {
  let app = AdditionalMethodsController::router();

  // Test HEAD
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("HEAD")
        .uri("/check")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);

  // Test OPTIONS
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("OPTIONS")
        .uri("/config")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"OPTIONS");

  // Test TRACE
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .method("TRACE")
        .uri("/debug")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"TRACE");
}

// Test default slash route
struct DefaultRouteController;

#[controller]
impl DefaultRouteController {
  #[get]
  async fn root() -> &'static str {
    "root"
  }
}

#[tokio::test]
async fn test_default_slash_route() {
  let app = DefaultRouteController::router();

  let response = app
    .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"root");
}

// Test nested paths
struct NestedController;

#[controller]
impl NestedController {
  #[get("/users")]
  async fn users() -> &'static str {
    "users"
  }

  #[get("/users/active")]
  async fn active_users() -> &'static str {
    "active_users"
  }

  #[get("/admin/dashboard")]
  async fn admin_dashboard() -> &'static str {
    "dashboard"
  }
}

#[tokio::test]
async fn test_nested_routes() {
  let app = NestedController::router();

  // Test /users
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"users");

  // Test /users/active
  let response = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/users/active")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"active_users");

  // Test /admin/dashboard
  let response = app
    .oneshot(
      Request::builder()
        .uri("/admin/dashboard")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"dashboard");
}

#[tokio::test]
async fn test_404_not_found() {
  let app = BasicController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/nonexistent")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
