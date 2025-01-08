# Route Controller

A Rust procedural macro crate that provides a structured approach for defining routes and attaching middleware for Axum-based web servers. This crate helps generate Axum routers by combining annotated methods into routes with specified HTTP methods and paths, while also allowing middleware functions to be applied to these routes.

## Features

- Attribute Macros: #[controller]: Defines a controller with a base path and middleware. #[route]: Associates a method with an HTTP method and path.
- Middleware Support: Apply middleware functions to the entire controller or specific routes.
- Structured Routing: Easily map methods to routes and combine them into an Axum Router.

## Installation

```sh
cargo add route_controller
```

## Usage

```rust
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json, Router,
};
use route_controller::{controller, route};

#[derive(Clone)]
pub struct AppState {}

// Auth Middleware using AppState
async fn auth_middleware(
    State(\_app_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    Ok(next.run(req).await)
}

// Context Middleware without AppState
async fn context_middleware(
    mut req: Request,
    next: Next
) -> Result<Response<Body>, StatusCode> {
    let trace_id = "123"; // Replace with a uuid
    req.headers_mut().append("trace_id", trace_id.parse().unwrap());
    Ok(next.run(req).await)
}

pub struct ApiController;

// Controller with base_path and middlewares
#[controller(
    path = "/api",
    middleware = "auth_middleware",
    middleware = "context_middleware"
)]
impl ApiController {
    // route with http methods and endpoint
    #[route("GET", "/users")]
    pub async fn get_users(\_request: Request) -> impl IntoResponse {
        Json(vec!["user1", "user2"])
    }

    // route using AppState
    #[route("POST", "/users")]
    pub async fn create_user(
        State(_app_state): State<AppState>,
        _request: Request
    ) -> impl IntoResponse {
        Json("User Created")
    }

}

#[tokio::main]
async fn main() {
    let app_state = AppState {};
    let router = Router::new()
        .merge(ApiController::router(app_state.clone()))
        .with_state(app_state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3030").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
```

## Explanation

- Controller Definition: Use #[controller] to define the base path and middleware for the controller.
- Route Definition: Use #[route] to define the HTTP method and path for each handler method.
- Middleware: Middleware functions are defined separately and applied using the middleware argument in #[controller].

## Generated Code

The #[controller] macro generates a router function for the controller, which constructs an Axum::Router by combining the specified routes and applying the middleware.
