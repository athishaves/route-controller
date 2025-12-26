# route_controller

Generate Axum routers from controller-style implementations.

## Features

- Clean controller-style API similar to Routing Controller (JS) or Rocket
- Route prefixing for organizing endpoints
- Middleware support at the controller level
- HTTP method attributes: `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`

## Installation

```toml
[dependencies]
route_controller = "0.1.2"
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use route_controller::{controller, get, post};
use axum::extract::Path;

struct UserController;

#[controller(path = "/api/users")]
impl UserController {
	#[get("/")]
	async fn list() -> String {
		"User list".to_string()
	}

	#[get("/{id}")]
	async fn get_one(Path(id): Path<u32>) -> String {
		format!("User {}", id)
	}

	#[post("/")]
	async fn create() -> String {
		"User created".to_string()
	}
}

#[tokio::main]
async fn main() {
	let app = UserController::router();
	
	let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
		.await
		.unwrap();
	
	axum::serve(listener, app).await.unwrap();
}
```

## Examples

### Multiple Controllers

```rust
#[controller(path = "/api/users")]
impl UserController {
	#[get]
	async fn list() -> String {
		"Users".to_string()
	}
}

#[controller]
impl HealthController {
	#[get("/health")]
	async fn health() -> &'static str {
		"OK"
	}
}

let app = axum::Router::new()
	.merge(UserController::router())
	.merge(HealthController::router());
```

### With Middleware

```rust
use axum::{middleware::Next, http::Request};

async fn auth_middleware<B>(req: Request<B>, next: Next<B>) -> impl axum::response::IntoResponse {
	// Auth logic here
	next.run(req).await
}

#[controller(path = "/api", middleware = auth_middleware)]
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
route_controller = { version = "0.1.2", features = ["verbose-logging"] }
```

Or build/run using features flag directly:

```bash
cargo build --example basic --features verbose-logging
cargo run --example basic --features verbose-logging
```

This shows detailed information about route registration during compilation.

## License

MIT
