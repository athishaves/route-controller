//! Integration tests for proc_macro_error validation
//!
//! These tests verify that the macro properly validates and reports errors for:
//! - Invalid extractor types
//! - Multiple body extractors
//! - Path parameter mismatches
//! - Invalid HTTP methods
//! - Missing required attributes
//! - Feature-gated extractors

// Note: These are compile-fail tests. They should fail to compile with specific error messages.
// To run these tests, use: cargo test --test 16_error_validation 2>&1 | grep "error:"

// Each test is in a separate module to isolate compilation errors

// Test 1: Invalid extractor type (should fail with helpful error)
#[cfg(feature = "test_invalid_extractor")]
#[allow(dead_code, unused)]
mod test_invalid_extractor {
  use route_controller::{controller, get};

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit an error: Unknown extractor type 'Jsn'
    #[get("/test", extract(data = Jsn))]
    async fn test(data: String) -> String {
      data
    }
  }
}

// Test 2: Multiple body extractors (should fail)
#[cfg(feature = "test_multiple_body_extractors")]
#[allow(dead_code, unused)]
mod test_multiple_body_extractors {
  use route_controller::{controller, post};

  #[derive(serde::Deserialize)]
  struct JsonData {
    value: String,
  }

  #[derive(serde::Deserialize)]
  struct FormData {
    name: String,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit an error: Multiple body extractors
    #[post("/test", extract(json_data = Json, form_data = Form))]
    async fn test(json_data: JsonData, form_data: FormData) -> String {
      format!("{} {}", json_data.value, form_data.name)
    }
  }
}

// Test 3: Path parameter without extractor (should fail)
#[cfg(feature = "test_missing_path_extractor")]
#[allow(dead_code, unused)]
mod test_missing_path_extractor {
  use route_controller::{controller, get};

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit an error: Path parameter 'id' has no extractor
    #[get("/users/{id}")]
    async fn get_user(id: u32) -> String {
      format!("User {}", id)
    }
  }
}

// Test 4: Path extractor without path parameter (should emit warning)
#[cfg(feature = "test_extractor_without_path_param")]
#[allow(dead_code, unused)]
mod test_extractor_without_path_param {
  use route_controller::{controller, get};

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit a warning: Path extractor for 'id' but not in path
    #[get("/users", extract(id = Path))]
    async fn get_user(id: u32) -> String {
      format!("User {}", id)
    }
  }
}

// Test 5: Body extractor on GET method (should emit warning)
#[cfg(feature = "test_body_on_get")]
#[allow(dead_code, unused)]
mod test_body_on_get {
  use route_controller::{controller, get};

  #[derive(serde::Deserialize)]
  struct Data {
    value: String,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit a warning: Body extractor on GET method
    #[get("/data", extract(data = Json))]
    async fn get_data(data: Data) -> String {
      data.value
    }
  }
}

// Test 6: Invalid HTTP method (should fail)
#[cfg(feature = "test_invalid_http_method")]
#[allow(dead_code, unused)]
mod test_invalid_http_method {
  use route_controller::controller;

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This would need a custom attribute, but shows the concept
    // In practice, using an undefined method like #[gett] would fail at compile time
    #[get("/test")]
    async fn test() -> String {
      "test".to_string()
    }
  }
}

// Test 7: Wrong extractor type for path parameter (should fail)
#[cfg(feature = "test_wrong_extractor_for_path")]
#[allow(dead_code, unused)]
mod test_wrong_extractor_for_path {
  use route_controller::{controller, get};

  #[derive(serde::Deserialize)]
  struct QueryData {
    id: u32,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit an error: Path parameter 'id' should use Path extractor
    #[get("/users/{id}", extract(id = Query))]
    async fn get_user(id: QueryData) -> String {
      format!("User {}", id.id)
    }
  }
}

// Test 8: Parameter without extractor (should emit warning)
#[cfg(feature = "test_param_without_extractor")]
#[allow(dead_code, unused)]
mod test_param_without_extractor {
  use route_controller::{controller, post};

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit a warning: Parameter 'value' has no extractor
    #[post("/test")]
    async fn test(value: String) -> String {
      value
    }
  }
}

// Test 9: Extractor without matching parameter (should emit warning)
#[cfg(feature = "test_extractor_without_param")]
#[allow(dead_code, unused)]
mod test_extractor_without_param {
  use route_controller::{controller, get};

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    // This should emit a warning: Extractor 'data' specified but no parameter 'data'
    #[get("/test", extract(data = Query))]
    async fn test() -> String {
      "test".to_string()
    }
  }
}

// Test 10: Invalid middleware path (should emit error)
#[cfg(feature = "test_invalid_middleware")]
#[allow(dead_code, unused)]
mod test_invalid_middleware {
  use route_controller::{controller, get};

  struct TestController;

  // This should emit an error: Invalid middleware path
  #[controller(path = "/api", middleware = 123)]
  impl TestController {
    #[get("/test")]
    async fn test() -> String {
      "test".to_string()
    }
  }
}

// Working test that should compile successfully
#[cfg(test)]
mod valid_usage_test {
  use axum::body::Body;
  use axum::http::{Request, StatusCode};
  use route_controller::{controller, get, post};
  use serde::{Deserialize, Serialize};
  use tower::ServiceExt;

  #[derive(Deserialize, Serialize)]
  struct User {
    name: String,
  }

  struct TestController;

  #[controller(path = "/api")]
  impl TestController {
    #[get("/users/{id}", extract(id = Path))]
    async fn get_user(id: u32) -> String {
      format!("User {}", id)
    }

    #[post("/users", extract(user = Json))]
    async fn create_user(user: User) -> axum::Json<User> {
      axum::Json(user)
    }
  }

  #[tokio::test]
  async fn test_valid_route() {
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

  #[tokio::test]
  async fn test_valid_post() {
    let app = TestController::router();

    let user = User {
      name: "John".to_string(),
    };
    let body = serde_json::to_string(&user).unwrap();

    let response = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/api/users")
          .header("content-type", "application/json")
          .body(Body::from(body))
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }
}

// Documentation tests for error messages
/// # Error Validation Examples
///
/// This test file demonstrates the various compile-time validations:
///
/// ## Invalid Extractor Type
/// ```compile_fail
/// # use route_controller::{controller, get};
/// struct Controller;
/// #[controller(path = "/api")]
/// impl Controller {
///     #[get("/test", extract(data = InvalidType))]
///     async fn test(data: String) -> String { data }
/// }
/// ```
///
/// ## Multiple Body Extractors
/// ```compile_fail
/// # use route_controller::{controller, post};
/// # #[derive(serde::Deserialize)]
/// # struct A { x: String }
/// # #[derive(serde::Deserialize)]
/// # struct B { y: String }
/// struct Controller;
/// #[controller(path = "/api")]
/// impl Controller {
///     #[post("/test", extract(a = Json, b = Form))]
///     async fn test(a: A, b: B) -> String { "ok".to_string() }
/// }
/// ```
///
/// ## Missing Path Extractor
/// ```compile_fail
/// # use route_controller::{controller, get};
/// struct Controller;
/// #[controller(path = "/api")]
/// impl Controller {
///     #[get("/users/{id}")]
///     async fn test(id: u32) -> String { format!("{}", id) }
/// }
/// ```
#[allow(dead_code)]
struct ErrorDocumentation;
