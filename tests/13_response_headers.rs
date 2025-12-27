//! Integration tests for custom response headers
//!
//! Tests response header customization

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

struct HeaderResponseController;

#[controller(path = "/api")]
impl HeaderResponseController {
  #[get("/custom", header("x-custom", "value"))]
  async fn custom_header() -> &'static str {
    "ok"
  }

  #[get(
    "/multiple",
    header("x-custom-1", "value1"),
    header("x-custom-2", "value2")
  )]
  async fn multiple_headers() -> &'static str {
    "ok"
  }

  #[get("/content", content_type("application/custom"))]
  async fn custom_content_type() -> &'static str {
    "ok"
  }

  #[get(
    "/cors",
    header("x-cors-origin", "*"),
    header("x-cors-methods", "GET, POST")
  )]
  async fn cors_headers() -> &'static str {
    "ok"
  }

  #[get("/cache", header("x-cache-ttl", "3600"))]
  async fn cache_control() -> &'static str {
    "ok"
  }

  #[get(
    "/users/{id}",
    extract(id = Path),
    header("x-user-id", "extracted")
  )]
  async fn with_extractor(id: u32) -> String {
    format!("user:{}", id)
  }
}

#[tokio::test]
async fn test_single_custom_header() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/custom")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-custom").unwrap(), "value");
}

#[tokio::test]
async fn test_multiple_custom_headers() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/multiple")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-custom-1").unwrap(), "value1");
  assert_eq!(response.headers().get("x-custom-2").unwrap(), "value2");
}

#[tokio::test]
async fn test_custom_content_type() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/content")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/custom"
  );
}

#[tokio::test]
async fn test_cors_headers() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/cors")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Check that at least one of the custom headers is present
  assert!(
    response.headers().get("x-cors-origin").is_some()
      || response.headers().get("x-cors-methods").is_some()
  );
}

#[tokio::test]
async fn test_cache_control() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/cache")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-cache-ttl").unwrap(), "3600");
}

#[tokio::test]
async fn test_headers_with_extractor() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/users/42")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-user-id").unwrap(), "extracted");
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"user:42");
}

#[tokio::test]
async fn test_header_values_preserved() {
  let app = HeaderResponseController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/api/custom")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let header_value = response.headers().get("x-custom").unwrap();
  assert_eq!(header_value.to_str().unwrap(), "value");
}

// ============================================================================
// Controller-level headers tests
// ============================================================================

struct ControllerHeaderController;

#[controller(
  path = "/ctrl",
  header("x-api-version", "1.0"),
  header("x-powered-by", "route-controller")
)]
impl ControllerHeaderController {
  // Route inherits controller headers
  #[get("/inherit")]
  async fn inherit_headers() -> &'static str {
    "inherited"
  }

  // Route overrides one controller header
  #[get("/override", header("x-api-version", "2.0"))]
  async fn override_header() -> &'static str {
    "overridden"
  }

  // Route adds additional headers
  #[get("/add", header("x-request-id", "abc-123"))]
  async fn add_header() -> &'static str {
    "added"
  }

  // Route overrides and adds
  #[get("/mixed", header("x-api-version", "3.0"), header("x-custom", "test"))]
  async fn mixed_headers() -> &'static str {
    "mixed"
  }
}

#[tokio::test]
async fn test_controller_headers_inherited() {
  let app = ControllerHeaderController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ctrl/inherit")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-api-version").unwrap(), "1.0");
  assert_eq!(
    response.headers().get("x-powered-by").unwrap(),
    "route-controller"
  );
}

#[tokio::test]
async fn test_route_header_overrides_controller() {
  let app = ControllerHeaderController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ctrl/override")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Route overrides x-api-version to 2.0
  assert_eq!(response.headers().get("x-api-version").unwrap(), "2.0");
  // Controller header x-powered-by is still present
  assert_eq!(
    response.headers().get("x-powered-by").unwrap(),
    "route-controller"
  );
}

#[tokio::test]
async fn test_route_adds_to_controller_headers() {
  let app = ControllerHeaderController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ctrl/add")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Controller headers are present
  assert_eq!(response.headers().get("x-api-version").unwrap(), "1.0");
  assert_eq!(
    response.headers().get("x-powered-by").unwrap(),
    "route-controller"
  );
  // Route-specific header is also present
  assert_eq!(response.headers().get("x-request-id").unwrap(), "abc-123");
}

#[tokio::test]
async fn test_mixed_controller_and_route_headers() {
  let app = ControllerHeaderController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ctrl/mixed")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Route overrides x-api-version
  assert_eq!(response.headers().get("x-api-version").unwrap(), "3.0");
  // Controller header x-powered-by is still present
  assert_eq!(
    response.headers().get("x-powered-by").unwrap(),
    "route-controller"
  );
  // Route-specific header is present
  assert_eq!(response.headers().get("x-custom").unwrap(), "test");
}

// Test controller-level content_type
struct ControllerContentTypeController;

#[controller(path = "/ct", content_type("application/json"))]
impl ControllerContentTypeController {
  #[get("/default")]
  async fn default_content_type() -> &'static str {
    r#"{"status":"ok"}"#
  }

  #[get("/override", content_type("text/plain"))]
  async fn override_content_type() -> &'static str {
    "plain text"
  }
}

#[tokio::test]
async fn test_controller_content_type_inherited() {
  let app = ControllerContentTypeController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ct/default")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );
}

#[tokio::test]
async fn test_route_content_type_overrides_controller() {
  let app = ControllerContentTypeController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/ct/override")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain"
  );
}

// Test controller with both headers and content_type
struct ControllerFullFeaturesController;

#[controller(
  path = "/full",
  header("x-api-version", "1.0"),
  header("x-service", "api"),
  content_type("application/json")
)]
impl ControllerFullFeaturesController {
  #[get("/all")]
  async fn all_features() -> &'static str {
    r#"{"message":"all"}"#
  }

  #[get("/partial", header("x-api-version", "2.0"), content_type("text/plain"))]
  async fn partial_override() -> &'static str {
    "partial"
  }
}

#[tokio::test]
async fn test_controller_headers_and_content_type() {
  let app = ControllerFullFeaturesController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/full/all")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  assert_eq!(response.headers().get("x-api-version").unwrap(), "1.0");
  assert_eq!(response.headers().get("x-service").unwrap(), "api");
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "application/json"
  );
}

#[tokio::test]
async fn test_route_overrides_controller_headers_and_content_type() {
  let app = ControllerFullFeaturesController::router();

  let response = app
    .oneshot(
      Request::builder()
        .uri("/full/partial")
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  // Route overrides x-api-version
  assert_eq!(response.headers().get("x-api-version").unwrap(), "2.0");
  // Controller header x-service is still present
  assert_eq!(response.headers().get("x-service").unwrap(), "api");
  // Route overrides content-type
  assert_eq!(
    response.headers().get("content-type").unwrap(),
    "text/plain"
  );
}
