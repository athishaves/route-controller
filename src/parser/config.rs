//! Controller configuration parsing

use proc_macro::TokenStream;
use syn::Path;

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
    } else if arg.starts_with("middleware") {
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
