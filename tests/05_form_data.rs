//! Integration tests for Form data extraction
//!
//! Tests application/x-www-form-urlencoded data

use axum::body::Body;
use axum::http::{Request, StatusCode};
use route_controller::{controller, post};
use serde::Deserialize;
use tower::ServiceExt;

#[derive(Deserialize, Debug)]
struct LoginForm {
  username: String,
  password: String,
}

#[derive(Deserialize, Debug)]
struct OptionalForm {
  name: String,
  age: Option<u32>,
  active: Option<bool>,
}

struct FormController;

#[controller(path = "/api")]
impl FormController {
  #[post("/login", extract(form = Form))]
  async fn login(form: LoginForm) -> String {
    format!("Login: {} {}", form.username, form.password)
  }

  #[post("/register", extract(form = Form))]
  async fn register(form: OptionalForm) -> String {
    format!(
      "Register: {}, age: {}, active: {}",
      form.name,
      form.age.unwrap_or(0),
      form.active.unwrap_or(false)
    )
  }
}

#[tokio::test]
async fn test_simple_form() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=alice&password=secret123"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Login: alice secret123");
}

#[tokio::test]
async fn test_form_with_optional_fields() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/register")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("name=bob"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Register: bob, age: 0, active: false");
}

#[tokio::test]
async fn test_form_with_all_fields() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/register")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("name=charlie&age=25&active=true"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  assert_eq!(&body[..], b"Register: charlie, age: 25, active: true");
}

#[tokio::test]
async fn test_url_encoded_values() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=alice%40example&password=my%20secret"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_required_form_field() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=alice"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_empty_form() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(""))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_invalid_form_type() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/register")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("name=dave&age=not_a_number"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_special_characters() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=alice%2Bbob&password=pass%26word"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_duplicate_form_fields() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("username=alice&username=bob&password=secret"))
        .unwrap(),
    )
    .await
    .unwrap();

  // Duplicate fields cause errors
  assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_missing_content_type() {
  let app = FormController::router();
  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/login")
        .body(Body::from("username=alice&password=secret"))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
