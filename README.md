# route_controller

Generate Axum routers from controller-style implementations with declarative extractors.

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
- Middleware support at the controller level
- HTTP method attributes: `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`, `#[head]`, `#[options]`, `#[trace]`

## Installation

```toml
[dependencies]
route_controller = "0.2.0"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
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

    #[post("/", extract(user = Json))]
    async fn create(user: User) -> String {
        format!("Created user: {} ({})", user.name, user.email)
    }
}

#[tokio::main]
async fn main() {
    let app = UserController::router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3003")
        .await
        .unwrap();

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

## Examples

Run the examples to see different use cases:

```bash
# Basic CRUD operations with extractors
cargo run --example basic

# Advanced extractor combinations (Path, Query, Json)
cargo run --example extractors

# Body extractors (Form, Bytes, Text, Html, Xml, JavaScript)
cargo run --example body_extractors

# Application state management
cargo run --example state

# Session handling with tower-sessions
cargo run --example session
```

### With Middleware

Apply middleware at the controller level:

```rust
use axum::{
    middleware::Next,
    extract::Request,
    response::Response,
    body::Body,
};

async fn log_middleware(request: Request<Body>, next: Next) -> Response<Body> {
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
#[controller(middleware = middleware_a, middleware = middleware_b)]
impl MultiMiddlewareController {
    #[get("/test")]
    async fn test() -> &'static str {
        "ok"
    }
}
```

## Verbose Logging

Enable verbose logging during compilation:

```toml
[dependencies]
route_controller = { version = "0.2.0", features = ["verbose-logging"] }
```

Or build/run using features flag directly:

```bash
cargo build --example basic --features verbose-logging
cargo run --example basic --features verbose-logging
```

This shows detailed information about route registration during compilation.

## License

MIT
