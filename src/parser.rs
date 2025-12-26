//! Parsing controller attributes and routes

use proc_macro::TokenStream;
use syn::{Attribute, Path, FnArg, Pat, Type};
use quote::ToTokens;

pub struct ControllerConfig {
	pub route_prefix: Option<String>,
	pub middlewares: Vec<Path>,
	pub is_auto_controller: bool,
}

#[derive(Clone)]
pub struct ParamInfo {
	pub pat: Pat,
	pub ty: Type,
	pub needs_wrap: bool,
	pub extractor_type: ExtractorType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtractorType {
	Json,
	Form,
	None,
}

pub fn parse_controller_attributes(attr: &TokenStream, is_auto_controller: bool) -> ControllerConfig {
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
		is_auto_controller,
	}
}

pub struct RouteInfo {
	pub method: String,
	pub path: String,
	pub content_type: ContentType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentType {
	Json,
	Form,
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
					// Try to parse the attribute tokens to extract path and content_type
					let mut route_path = "/".to_string();
					let mut content_type = ContentType::Json; // default

					if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
						route_path = lit_str.value();
						if !route_path.starts_with('/') {
							route_path = format!("/{}", route_path);
						}
					} else {
						// Try parsing as meta list with multiple arguments
						let attr_str = attr.meta.to_token_stream().to_string();

						// Parse content_type if present
						if attr_str.contains("content_type") {
							if attr_str.contains("\"form\"") || attr_str.contains("form") {
								content_type = ContentType::Form;
							}
						}

						// Extract path from the attribute string
						if let Some(start) = attr_str.find('"') {
							if let Some(end) = attr_str[start + 1..].find('"') {
								let mut path = attr_str[start + 1..start + 1 + end].to_string();
								if !path.is_empty() && !path.starts_with('/') {
									path = format!("/{}", path);
								}
								if !path.is_empty() {
									route_path = path;
								}
							}
						}
					}

					log_verbose!("Parsed route: [Method:{}] [Path:{}] [ContentType:{:?}]", method, route_path, content_type);
					return Some(RouteInfo { method, path: route_path, content_type });
				}
				_ => {}
			}
		}
	}
	None
}

/// Checks if a type is an Axum extractor that shouldn't be wrapped
fn is_axum_extractor(ty: &Type) -> bool {
	if let Type::Path(type_path) = ty {
		let segments: Vec<String> = type_path
			.path
			.segments
			.iter()
			.map(|seg| seg.ident.to_string())
			.collect();

		let last_segment = segments.last().map(|s| s.as_str());

		// Known Axum extractors
		matches!(
			last_segment,
			Some("Path") | Some("Query") | Some("Json") |
			Some("State") | Some("Request") | Some("Extension") |
			Some("Form") | Some("Multipart") | Some("WebSocketUpgrade") |
			Some("ConnectInfo") | Some("OriginalUri") | Some("MatchedPath")
		)
	} else {
		false
	}
}

/// Analyzes function parameters to determine which need wrapping and with what extractor
pub fn analyze_params(sig: &syn::Signature, content_type: &ContentType) -> Vec<ParamInfo> {
	let mut params = Vec::new();

	for input in &sig.inputs {
		if let FnArg::Typed(pat_type) = input {
			let pat = (*pat_type.pat).clone();
			let ty = (*pat_type.ty).clone();
			let is_extractor = is_axum_extractor(&ty);

			let (needs_wrap, extractor_type) = if is_extractor {
				(false, ExtractorType::None)
			} else {
				// Determine which extractor to use based on content_type
				match content_type {
					ContentType::Json => (true, ExtractorType::Json),
					ContentType::Form => (true, ExtractorType::Form),
				}
			};

			params.push(ParamInfo {
				pat,
				ty,
				needs_wrap,
				extractor_type,
			});
		}
	}

	params
}
