//! Generating Axum router code from controllers

mod middleware;
mod router;
mod wrappers;

// Re-export public functions
pub use middleware::{apply_middlewares, apply_route_prefix};
pub use router::{generate_base_router, generate_route_registrations, generate_router_impl};
