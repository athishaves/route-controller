# route_controller

![CI Status](https://github.com/athishaves/route-controller/actions/workflows/ci.yml/badge.svg)
![Crates.io](https://img.shields.io/crates/v/route_controller)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![Downloads](https://img.shields.io/crates/d/route_controller)

Generate Axum routers from controller-style implementations with declarative extractors

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Path Parameter Syntax](#path-parameter-syntax)
  - [Optional Dependencies](#optional-dependencies)
- [Quick Start](#quick-start)
- [Controller Types](#controller-types)
  - [The `extract()` Attribute](#the-extract-attribute)
  - [Available Extractors](#available-extractors)
    - [Request Body Extractors](#request-body-extractors)
    - [URL Extractors](#url-extractors)
    - [Other Extractors](#other-extractors)
  - [Feature-Gated Extractors](#feature-gated-extractors)
- [Using State](#using-state)
- [Body Extractor Examples](#body-extractor-examples)
  - [Form Data](#form-data)
  - [Binary Data](#binary-data)
  - [Text Content Types](#text-content-types)
- [Response Headers](#response-headers)
  - [Controller-Level Headers](#controller-level-headers)
  - [Route-Level Headers](#route-level-headers)
  - [Multiple Headers](#multiple-headers)
  - [Content-Type Header](#content-type-header)
  - [Combining Controller and Route Headers](#combining-controller-and-route-headers)
- [Examples](#examples)
  - [With Middleware](#with-middleware)
- [Verbose Logging](#verbose-logging)
- [License](#license)

## Features

- Clean controller-style API similar to Routing Controller (JS) or Rocket
- Route prefixing for organizing endpoints
- Declarative extractor syntax with `extract()` attribute
- Built-in extractors:
  - **Body extractors**: `Json`, `Form`, `Bytes`, `Text`, `Html`, `Xml`, `JavaScript`
  - **URL extractors**: `Path`, `Query`
  - **State extractor**: `State`
- Optional extractors (with feature flags):
  - `HeaderParam` - Extract from HTTP headers (requires `headers` feature)
  - `CookieParam` - Extract from cookies (requires `cookies` feature)
  - `SessionParam` - Extract from session storage (requires `sessions` feature)
- **Response header support**: `header()` and `content_type()` attributes
  - **Controller-level headers**: Apply headers to all routes in a controller
  - **Route-level override**: Route headers override controller headers with the same name
- Middleware support at the controller level
- HTTP method attributes: `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`, `#[head]`, `#[options]`, `#[trace]`

## Installation

```toml
[dependencies]
route_controller = "0.2.0"
axum = "0.8"  # Also works with axum 0.7 and earlier versions
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Path Parameter Syntax

The path parameter syntax depends on your Axum version:

- **Axum 0.8+**: Use curly braces `{id}` for path parameters

```rust
#[get("/{id}", extract(id = Path))]
async fn get_user(id: u32) -> String {
    format!("User {}", id)
}
```

- **Axum 0.7 and earlier**: Use colon syntax `:id` for path parameters

```rust
#[get("/:id", extract(id = Path))]
async fn get_user(id: u32) -> String {
    format!("User {}", id)
}
```

### Optional Dependencies

For additional extractors, enable features and add required dependencies:

```toml
[dependencies]
route_controller = { version = "0.2.0", features = ["headers", "cookies", "sessions"] }
axum-extra = { version = "0.12", features = ["cookie"] }  # Required for cookies
tower-sessions = "0.14"  # Required for sessions
```

## Quick Start

```rust
use route_controller::{controller, get, post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
}

struct UserController;

#[controller(path = "/users")]
impl UserController {
    #[get]
    async fn list() -> &'static str {
        "User list"
    }

    #[get("/{id}", extract(id = Path))]
    async fn get_one(id: u32) -> axum::Json<User> {
        let user = User {
            name: format!("User{}", id),
            email: format!("user{}@example.com", id),
        };
        axum::Json(user)
    }

    #[post(extract(user = Json))]
    async fn create(user: User) -> String {
        format!("Created user: {} ({})", user.name, user.email)
    }
}

#[tokio::main]
async fn main() {
    let app = UserController::router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
```

## Controller Types

### The `extract()` Attribute

Use the `extract()` attribute to specify how each parameter should be extracted from the request. The order of extractors in the attribute can differ from the parameter order:

```rust
#[controller(path = "/api/users")]
impl ApiController {
    // Single extractor
    #[post("/", extract(user = Json))]
    async fn create(user: User) -> String {
        format!("Created: {}", user.name)
    }

    // Multiple Path extractors (order independent)
    #[get("/{id}/posts/{post_id}", extract(post_id = Path, id = Path))]
    async fn get_user_post(id: u32, post_id: u32) -> String {
        format!("User {} - Post {}", id, post_id)
    }

    // Mixed extractors: Path + Json
    #[put("/{id}", extract(id = Path, user = Json))]
    async fn update(id: u32, user: User) -> String {
        format!("Updated user {}", id)
    }

    // Path + Query extractors
    #[get("/{id}/search", extract(id = Path, filters = Query))]
    async fn search(id: u32, filters: SearchFilters) -> String {
        format!("Searching for user {}", id)
    }
}
```

### Available Extractors

#### Request Body Extractors

- **`Json`** - Extract JSON request body: `extract(data = Json)`
  - Type: Any deserializable struct (`T where T: serde::Deserialize`)
  - Content-Type: `application/json`

- **`Form`** - Extract form data (form-data or x-www-form-urlencoded): `extract(data = Form)`
  - Type: Any deserializable struct (`T where T: serde::Deserialize`)
  - Content-Type: `application/x-www-form-urlencoded` or `multipart/form-data`

- **`Bytes`** - Extract raw binary data: `extract(data = Bytes)`
  - Type: `Vec<u8>`
  - Useful for file uploads, binary protocols, etc.

- **`Text`** - Extract plain text: `extract(content = Text)`
  - Type: `String`
  - Content-Type: `text/plain`

- **`Html`** - Extract HTML content: `extract(content = Html)`
  - Type: `String`
  - Content-Type: `text/html`

- **`Xml`** - Extract XML content: `extract(content = Xml)`
  - Type: `String`
  - Content-Type: `application/xml` or `text/xml`

- **`JavaScript`** - Extract JavaScript content: `extract(code = JavaScript)`
  - Type: `String`
  - Content-Type: `application/javascript` or `text/javascript`

#### URL Extractors

- **`Path`** - Extract path parameters: `extract(id = Path)`
- **`Query`** - Extract query parameters: `extract(params = Query)`

#### Other Extractors

- **`State`** - Extract application state: `extract(state = State)`

### Feature-Gated Extractors

Enable additional extractors with Cargo features:

```toml
[dependencies]
route_controller = { version = "0.2.0", features = ["headers", "cookies", "sessions"] }
axum-extra = { version = "0.12", features = ["cookie"] }  # Required for cookies
tower-sessions = "0.14"  # Required for sessions
```

- **`HeaderParam`** - Extract from HTTP headers (requires `headers` feature)

  ```rust
  #[get("/api/data", extract(authorization = HeaderParam))]
  async fn get_data(authorization: String) -> String {
      format!("Auth: {}", authorization)
  }
  ```

- **`CookieParam`** - Extract from cookies (requires `cookies` feature + `axum-extra`)

  ```rust
  #[get("/profile", extract(session_id = CookieParam))]
  async fn get_profile(session_id: String) -> String {
      format!("Session: {}", session_id)
  }
  ```

- **`SessionParam`** - Extract from session storage (requires `sessions` feature + `tower-sessions`)

  ```rust
  #[get("/profile", extract(user_id = SessionParam))]
  async fn get_profile(user_id: String) -> String {
      format!("User ID: {}", user_id)
  }
  ```

## Using State

Extract application state in your handlers using the `State` extractor:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    counter: Arc<RwLock<i32>>,
}

struct CounterController;

#[controller(path = "/counter")]
impl CounterController {
    #[get(extract(state = State))]
    async fn get_count(state: AppState) -> axum::Json<i32> {
        let count = *state.counter.read().await;
        axum::Json(count)
    }

    #[post("/increment", extract(state = State))]
    async fn increment(state: AppState) -> axum::Json<i32> {
        let mut counter = state.counter.write().await;
        *counter += 1;
        axum::Json(*counter)
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        counter: Arc::new(RwLock::new(0)),
    };

    let app = axum::Router::new()
        .merge(CounterController::router())
        .with_state(app_state);

    // Start server...
}
```

## Body Extractor Examples

### Form Data

Handle form submissions with the `Form` extractor:

```rust
#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[controller(path = "/auth")]
impl AuthController {
    #[post("/login", extract(form = Form))]
    async fn login(form: LoginForm) -> String {
        format!("Logging in user: {}", form.username)
    }
}
```

Test with:

```bash
curl -X POST http://localhost:3000/auth/login \
  -d 'username=john&password=secret123'
```

### Binary Data

Handle file uploads or binary data with the `Bytes` extractor:

```rust
#[controller(path = "/files")]
impl FileController {
    #[post("/upload", extract(data = Bytes))]
    async fn upload(data: Vec<u8>) -> String {
        format!("Received {} bytes", data.len())
    }
}
```

### Text Content Types

Handle various text-based content types:

```rust
#[controller(path = "/content")]
impl ContentController {
    // Plain text
    #[post("/text", extract(content = Text))]
    async fn handle_text(content: String) -> String {
        format!("Received text: {}", content)
    }

    // HTML content
    #[post("/html", extract(html = Html))]
    async fn handle_html(html: String) -> String {
        format!("Received {} chars of HTML", html.len())
    }

    // XML content
    #[post("/xml", extract(xml = Xml))]
    async fn handle_xml(xml: String) -> String {
        format!("Received XML: {}", xml)
    }

    // JavaScript code
    #[post("/script", extract(code = JavaScript))]
    async fn handle_script(code: String) -> String {
        format!("Received {} chars of JavaScript", code.len())
    }
}
```

## Response Headers

Add custom headers to your responses using the `header()` and `content_type()` attributes at both the controller and route levels.

### Controller-Level Headers

Apply headers to all routes in a controller. Route-level headers with the same name will override controller-level headers:

```rust
#[controller(
    path = "/api",
    header("x-api-version", "1.0"),
    header("x-powered-by", "route-controller")
)]
impl ApiController {
    // Inherits both controller headers
    #[get("/data")]
    async fn get_data() -> String {
        "Data with controller headers".to_string()
    }

    // Overrides x-api-version, keeps x-powered-by
    #[get("/v2", header("x-api-version", "2.0"))]
    async fn get_data_v2() -> String {
        "Data with overridden version".to_string()
    }

    // Adds route-specific header, keeps controller headers
    #[get("/special", header("x-request-id", "abc-123"))]
    async fn special() -> String {
        "Special endpoint".to_string()
    }
}
```

### Route-Level Headers

```rust
#[controller(path = "/api")]
impl ApiController {
    #[get("/data", header("x-api-version", "1.0"))]
    async fn get_data() -> String {
        "Data with custom header".to_string()
    }
}
```

### Multiple Headers

```rust
#[controller(path = "/api")]
impl ApiController {
    #[get(
        "/info",
        header("x-api-version", "2.0"),
        header("x-request-id", "abc-123")
    )]
    async fn get_info() -> String {
        "Info with multiple headers".to_string()
    }
}
```

### Content-Type Header

Set content-type at controller or route level:

```rust
// Controller-level content-type applies to all routes
#[controller(path = "/api", content_type("application/json"))]
impl ApiController {
    // Inherits application/json content-type
    #[get("/data")]
    async fn get_data() -> String {
        r#"{"status":"ok"}"#.to_string()
    }

    // Route overrides to XML
    #[get("/xml", content_type("application/xml"))]
    async fn get_xml() -> String {
        r#"<?xml version="1.0"?>
<response>
    <message>Hello XML</message>
</response>"#.to_string()
    }

    // Route overrides to plain text
    #[get("/text", content_type("text/plain; charset=utf-8"))]
    async fn get_text() -> String {
        "Plain text response".to_string()
    }
}
```

### Combining Controller and Route Headers

Controller headers provide a base set of headers, and routes can override or extend them:

```rust
// Controller provides base headers and content-type
#[controller(
    path = "/api",
    content_type("application/json"),
    header("x-api-version", "1.0"),
    header("x-service", "my-api")
)]
impl ApiController {
    // Inherits all controller headers
    #[get("/info")]
    async fn get_info() -> axum::Json<Response> {
        axum::Json(Response { status: "ok".to_string() })
    }

    // Override version and content-type, keep x-service
    #[post(
        "/data",
        content_type("application/json; charset=utf-8"),
        header("x-api-version", "2.0"),
        header("x-rate-limit", "100")
    )]
    async fn post_data() -> axum::Json<Response> {
        axum::Json(Response { status: "ok".to_string() })
    }
}
```

Test with:

```bash
# Check inherited headers
curl -v http://localhost:3000/api/info
# Output: x-api-version: 1.0, x-service: my-api, content-type: application/json

# Check overridden headers
curl -v http://localhost:3000/api/data
# Output: x-api-version: 2.0, x-service: my-api, x-rate-limit: 100
```

## Examples

The crate includes 15 comprehensive examples demonstrating different features:

```bash
# 1. Basic routing with different HTTP methods (GET, POST, PUT, DELETE)
cargo run --example 01_basic_routing

# 2. Path parameter extraction
cargo run --example 02_path_params

# 3. Query parameter extraction
cargo run --example 03_query_params

# 4. JSON body extraction
cargo run --example 04_json_body

# 5. Form data handling (form-data and x-www-form-urlencoded)
cargo run --example 05_form_data

# 6. Text body extraction
cargo run --example 06_text_body

# 7. Binary data (bytes) handling
cargo run --example 07_bytes

# 8. Header extraction (requires 'headers' feature)
cargo run --example 08_headers --features headers

# 9. Cookie handling (requires 'cookies' feature)
cargo run --example 09_cookies --features cookies

# 10. Session management (requires 'sessions' feature)
cargo run --example 10_sessions --features sessions

# 11. Application state management
cargo run --example 11_state

# 12. Response headers and content types
cargo run --example 12_response_headers

# 13. Middleware application
cargo run --example 13_middleware

# 14. Mixed extractors (Path + Query + Json)
cargo run --example 14_mixed_extractors

# 15. Multiple controllers with merged routers
cargo run --example 15_multiple_controllers
```

Each example includes:

- Clear comments explaining the feature
- Test commands using curl
- Working code that can be run immediately

### With Middleware

Apply middleware at the controller level:

```rust
use axum::{
    middleware::Next,
    extract::Request,
    response::Response,
};

async fn log_middleware(request: Request, next: Next) -> Response {
    println!("Request: {} {}", request.method(), request.uri());
    next.run(request).await
}

#[controller(path = "/api", middleware = log_middleware)]
impl ApiController {
    #[get("/data")]
    async fn get_data() -> String {
        "Protected data".to_string()
    }
}
```

You can also apply multiple middlewares:

```rust
#[controller(
    path = "/api",
    middleware = middleware_a,
    middleware = middleware_b
)]
impl MultiMiddlewareController {
    #[get("/test")]
    async fn test() -> &'static str {
        "ok"
    }
}
```

See [examples/13_middleware.rs](examples/13_middleware.rs) for a complete example.

## Verbose Logging

Enable verbose logging during compilation by setting the `ROUTE_CONTROLLER_VERBOSE` environment variable:

```bash
ROUTE_CONTROLLER_VERBOSE=1 cargo build
ROUTE_CONTROLLER_VERBOSE=1 cargo run --example basic
```

This shows detailed information about route registration during compilation.

## License

MIT
