use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct LoginForm {
  username: String,
  password: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct RegistrationForm {
  username: String,
  email: String,
  age: u32,
}

struct FormController;

#[controller(path = "/forms")]
impl FormController {
  #[post("/login", extract(form = Form))]
  async fn login(form: LoginForm) -> axum::Json<LoginForm> {
    axum::Json(form)
  }

  #[post("/register", extract(form = Form))]
  async fn register(form: RegistrationForm) -> axum::Json<RegistrationForm> {
    axum::Json(form)
  }
}

#[tokio::test]
async fn test_form_extractor_simple() {
  let app = FormController::router();

  let request = Request::builder()
    .uri("/forms/login")
    .method("POST")
    .header("content-type", "application/x-www-form-urlencoded")
    .body(Body::from("username=john&password=secret123"))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: LoginForm = serde_json::from_slice(&body).unwrap();

  assert_eq!(
    result,
    LoginForm {
      username: "john".to_string(),
      password: "secret123".to_string(),
    }
  );
}

#[tokio::test]
async fn test_form_extractor_with_multiple_fields() {
  let app = FormController::router();

  let request = Request::builder()
    .uri("/forms/register")
    .method("POST")
    .header("content-type", "application/x-www-form-urlencoded")
    .body(Body::from("username=alice&email=alice@example.com&age=25"))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: RegistrationForm = serde_json::from_slice(&body).unwrap();

  assert_eq!(
    result,
    RegistrationForm {
      username: "alice".to_string(),
      email: "alice@example.com".to_string(),
      age: 25,
    }
  );
}

#[tokio::test]
async fn test_form_extractor_url_encoded() {
  let app = FormController::router();

  // Test with URL-encoded special characters
  let request = Request::builder()
    .uri("/forms/login")
    .method("POST")
    .header("content-type", "application/x-www-form-urlencoded")
    .body(Body::from("username=user%40example&password=pass%20word"))
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let result: LoginForm = serde_json::from_slice(&body).unwrap();

  assert_eq!(
    result,
    LoginForm {
      username: "user@example".to_string(),
      password: "pass word".to_string(),
    }
  );
}
