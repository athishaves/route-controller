# route_controller

Generate Axum routers from controller-style implementations with automatic JSON handling.

## Features

- Clean controller-style API similar to Routing Controller (JS) or Rocket
- Route prefixing for organizing endpoints
- **Two controller modes:**
  - `#[controller]` - Explicit extractors (full control)
  - `#[auto_controller]` - Automatic Json/Form wrapping (convenient)
- Middleware support at the controller level
- Form data support with `content_type = "form"`
- HTTP method attributes: `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`

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
use route_controller::{auto_controller, get, post};
use axum::extract::Path;
use serde::{Deserialize, Serialize};

// Deserialize: Required for auto_controller to wrap input parameters
// Serialize: Required when returning Json<T> in responses
#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
}

struct UserController;

#[auto_controller(path = "/users")]
impl UserController {
    #[get]
    async fn list() -> &'static str {
        "User list"
    }

    #[get("/{id}")]
    async fn get_one(Path(id): Path<u32>) -> axum::Json<User> {
        let user = User {
            name: format!("User{}", id),
            email: format!("user{}@example.com", id),
        };
        axum::Json(user)
    }

    // Plain type automatically wrapped with Json<User>
    #[post]
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

### `#[auto_controller]` - Automatic JSON Wrapping

Automatically wraps plain struct parameters with `Json<T>`:

```rust
#[auto_controller(path = "/api/users")]
impl ApiController {
    // Input automatically wrapped with Json<User>
    #[post]
    async fn create(user: User) -> String {
        format!("Created: {}", user.name)
    }

    // Path extractors are preserved as-is
    #[put("/{id}")]
    async fn update(Path(id): Path<u32>, user: User) -> String {
        format!("Updated user {}", id)
    }
}
```

### `#[controller]` - Explicit Extractors

Requires explicit `Json<T>` extractors for full control:

```rust
use axum::Json;

#[controller(path = "/api/users")]
impl ApiController {
    // Must explicitly use Json<User>
    #[post]
    async fn create(Json(user): Json<User>) -> String {
        format!("Created: {}", user.name)
    }
}
```

## Form Data Support

Use `content_type = "form"` for form data instead of JSON:

```rust
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

#[auto_controller(path = "/api")]
impl ApiController {
    // JSON endpoint (default)
    #[post("/users")]
    async fn create_user(user: User) -> String {
        format!("Created: {}", user.name)
    }

    // Form data endpoint
    #[post("/login", content_type = "form")]
    async fn login(credentials: LoginData) -> String {
        format!("Login: {}", credentials.username)
    }
}
```

**Testing:**

```bash
# JSON request
curl -X POST http://localhost:3007/api/users \
  -H 'Content-Type: application/json' \
  -d '{"name":"John","email":"john@example.com"}'

# Form data request
curl -X POST http://localhost:3007/api/login \
  -d 'username=john&password=secret123'
```

## Examples

Run the examples to see different use cases:

```bash
# Basic CRUD operations
cargo run --example basic

# Automatic Json<T> wrapping
cargo run --example auto_controller

# Side-by-side comparison
cargo run --example comparison

# Form data handling
cargo run --example form_support
```

### With Middleware

```rust
use axum::{
    middleware::Next,
    extract::Request,
    response::{IntoResponse, Response},
};

async fn log_middleware(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    println!("Request: {} {}", request.method(), request.uri());
    Ok(next.run(request).await)
}

#[auto_controller(path = "/api", middleware = crate::log_middleware)]
impl SecureController {
    #[get("/data")]
    async fn secure_data() -> String {
        "Protected data".to_string()
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
