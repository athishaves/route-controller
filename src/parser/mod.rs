//! Parsing controller attributes and routes

mod config;
mod extractor_types;
mod params;
mod route;

// Re-export public types and functions
pub use config::parse_controller_attributes;
pub use extractor_types::ExtractorType;
pub use params::analyze_params;
pub use route::extract_route_from_attrs;

// Re-export internal types for use within the crate
#[allow(unused_imports)]
pub(crate) use config::ControllerConfig;
#[allow(unused_imports)]
pub(crate) use params::ParamInfo;
#[allow(unused_imports)]
pub(crate) use route::RouteInfo;
