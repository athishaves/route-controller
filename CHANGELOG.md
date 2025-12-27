# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking Changes

- **HTTP Method Attributes**: Changed from `#[route("GET", "/path")]` to dedicated attributes like `#[get("/path")]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`, `#[head]`, `#[options]`, `#[trace]`
- **Router Method Signature**: Changed from `router(state)` to `router()` - application state is now added separately via `.with_state(app_state)`
- **Middleware Syntax**: Changed from string literals `middleware = "auth_middleware"` to function identifiers `middleware = auth_middleware`
- **Handler Parameters**: Removed requirement for `Request` parameter - use declarative extractors instead

### Added

- **Declarative Extractor System**: New `extract()` attribute for specifying parameter extraction
  - Order-independent extractor syntax: `extract(post_id = Path, id = Path)`
  - Supports mixing multiple extractor types in any order
- **Body Extractors**: `Json`, `Form`, `Bytes`, `Text`, `Html`, `Xml`, `JavaScript`
- **URL Extractors**: `Path` and `Query` for route parameters and query strings
- **State Extractor**: `State` for accessing application state
- **Feature-Gated Extractors** (optional):
  - `HeaderParam` - Extract from HTTP headers (requires `headers` feature)
  - `CookieParam` - Extract from cookies (requires `cookies` feature)
  - `SessionParam` - Extract from session storage (requires `sessions` feature)
- **Response Headers**: `header()` and `content_type()` attributes for setting response headers
- **Multiple Middlewares**: Support for applying multiple middleware functions per controller
- **Comprehensive Examples**: 15 working examples covering all features
- **Verbose Logging**: `ROUTE_CONTROLLER_VERBOSE` environment variable for compilation debugging
- **Controller-Level Headers**: Apply `header()` and `content_type()` attributes at the controller level
  - Headers defined on the controller apply to all routes
  - Route-level headers override controller-level headers with the same name
  - Enables consistent API versioning and common headers across all endpoints
  - Example: `#[controller(path = "/api", header("x-api-version", "1.0"))]`

### Changed

- Improved documentation with detailed examples for all extractors
- Enhanced error messages for better debugging experience
- Cleaner, more intuitive API design

### Fixed

- Path parameter extraction now order-independent in `extract()` attribute

## [0.1.0] - 2024-12-27

### Added

- Basic `#[controller]` macro with `path` attribute
- `#[route]` attribute for defining routes with HTTP method and path as strings
- Middleware support with `middleware` attribute (string-based references)
- Application state support with `State` extractor
- Direct `Request` parameter handling
- Router generation with `router(state)` method
