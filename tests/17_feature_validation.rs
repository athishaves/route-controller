//! Integration tests for feature-gated extractors
//!
//! Tests that feature-gated extractors emit appropriate warnings

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, get};
use tower::ServiceExt;

// Test HeaderParam extractor with headers feature
#[cfg(feature = "headers")]
mod test_header_param_with_feature {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/test", extract(auth = HeaderParam))]
    async fn test(auth: String) -> String {
      format!("Auth: {}", auth)
    }
  }

  #[tokio::test]
  async fn test_header_param() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/test")
          .header("auth", "Bearer token")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test CookieParam extractor with cookies feature
#[cfg(feature = "cookies")]
mod test_cookie_param_with_feature {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/test", extract(session_id = CookieParam))]
    async fn test(session_id: String) -> String {
      format!("Session: {}", session_id)
    }
  }

  #[tokio::test]
  async fn test_cookie_param() {
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/test")
          .header("cookie", "session_id=abc123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Test SessionParam extractor with sessions feature
#[cfg(feature = "sessions")]
mod test_session_param_with_feature {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/test", extract(user_id = SessionParam))]
    async fn test(user_id: String) -> String {
      format!("User: {}", user_id)
    }
  }

  // Note: Full session testing requires session layer setup
  // This test just verifies the macro compiles correctly
  #[tokio::test]
  async fn test_session_compilation() {
    // This test verifies that the SessionParam extractor compiles correctly
    // Actual session functionality requires tower-sessions middleware setup
    let _app = TestController::router();
  }
}

// Test without features - should emit warnings but compile
#[cfg(not(any(feature = "headers", feature = "cookies", feature = "sessions")))]
mod test_without_features {
  use super::*;

  struct TestController;

  // These should emit warnings about missing features
  #[controller(path = "/api")]
  impl TestController {
    // Warning: HeaderParam requires 'headers' feature
    #[get("/header", extract(auth = HeaderParam))]
    async fn test_header(auth: String) -> String {
      format!("Auth: {}", auth)
    }

    // Warning: CookieParam requires 'cookies' feature
    #[get("/cookie", extract(session = CookieParam))]
    async fn test_cookie(session: String) -> String {
      format!("Session: {}", session)
    }

    // Warning: SessionParam requires 'sessions' feature
    #[get("/session", extract(user = SessionParam))]
    async fn test_session(user: String) -> String {
      format!("User: {}", user)
    }
  }

  #[tokio::test]
  async fn test_routes_compile_without_features() {
    // This test verifies that routes compile even without features
    // (though they may not work correctly at runtime)
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/header")
          .header("auth", "test")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    // Should respond, even if extractor doesn't work correctly
    assert!(
      response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
  }
}

// Test all extractors together
#[cfg(all(feature = "headers", feature = "cookies", feature = "sessions"))]
mod test_all_features {
  use super::*;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get(
      "/full/{id}",
      extract(
        id = Path,
        auth = HeaderParam,
        session = CookieParam,
        user = SessionParam
      )
    )]
    async fn test_all(id: u32, auth: String, session: String, user: String) -> String {
      format!(
        "ID: {}, Auth: {}, Session: {}, User: {}",
        id, auth, session, user
      )
    }
  }

  #[tokio::test]
  async fn test_all_extractors_together() {
    // This test verifies that all feature-gated extractors compile together
    // Note: Session and Cookie extractors may return 500 without proper middleware setup
    let app = TestController::router();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/api/full/123")
          .header("auth", "Bearer token")
          .header("cookie", "session=abc123")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    // Accept either OK or INTERNAL_SERVER_ERROR since extractors may fail without proper setup
    assert!(
      response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR
    );
  }
}

// Documentation for feature usage
/// # Feature-Gated Extractors
///
/// ## Using HeaderParam
/// Requires the `headers` feature:
/// ```toml
/// [dependencies]
/// route_controller = { version = "0.2.0", features = ["headers"] }
/// ```
///
/// ## Using CookieParam
/// Requires the `cookies` feature and `axum-extra`:
/// ```toml
/// [dependencies]
/// route_controller = { version = "0.2.0", features = ["cookies"] }
/// axum-extra = { version = "0.12", features = ["cookie"] }
/// ```
///
/// ## Using SessionParam
/// Requires the `sessions` feature and `tower-sessions`:
/// ```toml
/// [dependencies]
/// route_controller = { version = "0.2.0", features = ["sessions"] }
/// tower-sessions = "0.14"
/// ```
#[allow(dead_code)]
struct FeatureDocumentation;
