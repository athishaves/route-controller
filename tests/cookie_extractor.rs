use axum::http::{Request, StatusCode};
use axum_extra::extract::cookie::Cookie;
use route_controller::{controller, get};
use tower::ServiceExt;

struct CookieController;

#[controller(path = "/cookies")]
impl CookieController {
  #[get("/session", extract(session_id = CookieParam))]
  async fn with_cookie(session_id: String) -> String {
    format!("Session: {}", session_id)
  }

  #[get(
    "/user",
    extract(
      user_id = CookieParam,
      token = CookieParam,
    )
  )]
  async fn with_multiple_cookies(user_id: String, token: String) -> String {
    format!("User: {}, Token: {}", user_id, token)
  }

  #[get(
    "/{id}/profile",
    extract(
      id = Path,
      session = CookieParam,
    )
  )]
  async fn mixed_extractors(id: u32, session: String) -> String {
    format!("ID: {}, Session: {}", id, session)
  }
}

#[tokio::test]
async fn test_single_cookie_extraction() {
  let app = CookieController::router();

  let cookie = Cookie::new("session_id", "abc123xyz");

  let request = Request::builder()
    .uri("/cookies/session")
    .header("cookie", cookie.to_string())
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "Session: abc123xyz");
}

#[tokio::test]
async fn test_missing_cookie() {
  let app = CookieController::router();

  let request = Request::builder()
    .uri("/cookies/session")
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  // Should default to empty string
  assert_eq!(body_str, "Session: ");
}

#[tokio::test]
async fn test_multiple_cookies() {
  let app = CookieController::router();

  let cookie1 = Cookie::new("user_id", "user123");
  let cookie2 = Cookie::new("token", "secret456");
  let cookie_header = format!("{}; {}", cookie1, cookie2);

  let request = Request::builder()
    .uri("/cookies/user")
    .header("cookie", cookie_header)
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "User: user123, Token: secret456");
}

#[tokio::test]
async fn test_mixed_path_and_cookie() {
  let app = CookieController::router();

  let cookie = Cookie::new("session", "session789");

  let request = Request::builder()
    .uri("/cookies/42/profile")
    .header("cookie", cookie.to_string())
    .body(axum::body::Body::empty())
    .unwrap();

  let response = app.oneshot(request).await.unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body_str = String::from_utf8(body.to_vec()).unwrap();

  assert_eq!(body_str, "ID: 42, Session: session789");
}
