//! Integration tests for basic HTTP routing
//!
//! Tests all HTTP methods and basic route patterns

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, delete, get, head, options, patch, post, put, trace};
use tower::ServiceExt;

struct BasicController;

#[controller(path = "/api")]
impl BasicController {
  #[get("/hello")]
  async fn hello() -> &'static str {
    "Hello, World!"
  }

  #[post("/create")]
  async fn create() -> &'static str {
    "created"
  }

  #[put("/update")]
  async fn update() -> &'static str {
    "updated"
  }

  #[delete("/delete")]
  async fn delete_resource() -> &'static str {
    "deleted"
  }

  #[patch("/patch")]
  async fn patch_resource() -> &'static str {
    "patched"
  }

  #[head("/head")]
  async fn head_request() -> &'static str {
    "head"
  }

  #[options("/options")]
  async fn options_request() -> &'static str {
    "options"
  }

  #[trace("/trace")]
  async fn trace_request() -> &'static str {
    "trace"
  }
}

#[tokio::test]
async fn test_get_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/hello")
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
async fn test_post_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/create")
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

#[tokio::test]
async fn test_put_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("PUT")
        .uri("/api/update")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"updated");
}

#[tokio::test]
async fn test_delete_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("DELETE")
        .uri("/api/delete")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"deleted");
}

#[tokio::test]
async fn test_patch_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("PATCH")
        .uri("/api/patch")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_head_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("HEAD")
        .uri("/api/head")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_options_method() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("OPTIONS")
        .uri("/api/options")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_404_not_found() {
  let app = BasicController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_wrong_method() {
  let app = BasicController::router();
  // POST to a GET endpoint should return 405 Method Not Allowed
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/hello")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_root_path() {
  struct RootController;

  #[controller]
  impl RootController {
    #[get("/")]
    async fn root() -> &'static str {
      "root"
    }
  }

  let app = RootController::router();
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

#[tokio::test]
async fn test_nested_paths() {
  struct NestedController;

  #[controller(path = "/api/v1")]
  impl NestedController {
    #[get("/users/profile")]
    async fn profile() -> &'static str {
      "profile"
    }
  }

  let app = NestedController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/v1/users/profile")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_empty_controller() {
  struct EmptyController;

  #[controller]
  impl EmptyController {}

  let app = EmptyController::router();
  let response = app
    .oneshot(
      Request::builder()
        .uri("/anything")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_multiple_routes_same_controller() {
  struct MultiController;

  #[controller]
  impl MultiController {
    #[get("/route1")]
    async fn route1() -> &'static str {
      "route1"
    }

    #[get("/route2")]
    async fn route2() -> &'static str {
      "route2"
    }

    #[get("/route3")]
    async fn route3() -> &'static str {
      "route3"
    }
  }

  let app = MultiController::router();

  let response1 = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/route1")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response1.status(), StatusCode::OK);

  let response2 = app
    .clone()
    .oneshot(
      Request::builder()
        .uri("/route2")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response2.status(), StatusCode::OK);

  let response3 = app
    .oneshot(
      Request::builder()
        .uri("/route3")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();
  assert_eq!(response3.status(), StatusCode::OK);
}
