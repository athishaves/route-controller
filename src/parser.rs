//! Parsing controller attributes and routes

use proc_macro::TokenStream;
use syn::{Attribute, Path};

pub struct ControllerConfig {
	pub route_prefix: Option<String>,
	pub middlewares: Vec<Path>,
}

pub fn parse_controller_attributes(attr: &TokenStream) -> ControllerConfig {
	let mut route_prefix: Option<String> = None;
	let mut middlewares: Vec<Path> = vec![];

	let binding = attr.to_string();
	let args = binding.split(",");

	for arg in args {
		let arg = arg.trim();

		if arg.starts_with("path") {
			let parts: Vec<&str> = arg.split("=").collect();
			if parts.len() == 2 {
				let mut value = parts[1].trim().replace("\"", "");
				if !value.starts_with('/') {
					value = format!("/{}", value);
				}
				route_prefix = Some(value);
				log_verbose!("Parsed route prefix: [{}]", route_prefix.as_ref().unwrap());
			}
		}
		else if arg.starts_with("middleware") {
			let parts: Vec<&str> = arg.split("=").collect();
			if parts.len() == 2 {
				let value = parts[1].trim();
				if let Ok(middleware_path) = syn::parse_str::<Path>(&value) {
					log_verbose!("Parsed middleware: [{}]", value);
					middlewares.push(middleware_path);
				}
			}
		}
	}

	ControllerConfig {
		route_prefix,
		middlewares,
	}
}

pub struct RouteInfo {
	pub method: String,
	pub path: String,
}

pub fn extract_route_from_attrs(attrs: &[Attribute]) -> Option<RouteInfo> {
	for attr in attrs {
		let path_segments: Vec<String> = attr
			.path()
			.segments
			.iter()
			.map(|seg| seg.ident.to_string())
			.collect();

		if path_segments.len() == 1 {
			let method = path_segments[0].to_lowercase();
			match method.as_str() {
				"get" | "post" | "put" | "delete" | "patch" => {
					if let Ok(route_path) = attr.parse_args::<syn::LitStr>() {
						let mut path = route_path.value();
						if !path.starts_with('/') {
							path = format!("/{}", path);
						}
						log_verbose!("Parsed route: [Method:{}] [Path:{}]", method, path);
						return Some(RouteInfo { method, path });
					} else {
            // If no path is provided, default to "/"
            return Some(RouteInfo { method, path: "/".to_string() });
          }
				}
				_ => {}
			}
		}
	}
	None
}
