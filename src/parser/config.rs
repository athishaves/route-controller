//! Controller configuration parsing

use proc_macro::TokenStream;
use proc_macro_error::{emit_call_site_error, emit_call_site_warning};
use syn::Path;

pub struct ControllerConfig {
  pub route_prefix: Option<String>,
  pub middlewares: Vec<Path>,
  pub response_headers: Vec<(String, String)>,
  pub content_type: Option<String>,
}

pub fn parse_controller_attributes(attr: &TokenStream) -> ControllerConfig {
  let mut route_prefix: Option<String> = None;
  let mut middlewares: Vec<Path> = Vec::with_capacity(2); // Most controllers have 0-2 middlewares
  let mut response_headers: Vec<(String, String)> = Vec::with_capacity(4); // Typical controllers have 0-4 headers
  let mut content_type: Option<String> = None;

  let attr_str = attr.to_string();

  // Parse path and middleware using split
  for arg in attr_str.split(',').map(str::trim) {
    if arg.starts_with("path") {
      let parts: Vec<&str> = arg.split("=").collect();
      if parts.len() == 2 {
        let mut value = parts[1].trim().replace("\"", "");
        if value.is_empty() {
          emit_call_site_warning!("Empty path value in controller attribute");
        }
        if !value.starts_with('/') {
          value = format!("/{}", value);
        }
        log_verbose!("Parsed route prefix: [{}]", value);
        route_prefix = Some(value);
      } else {
        emit_call_site_error!("Invalid path attribute format. Expected: path = \"/route\"");
      }
    } else if arg.starts_with("middleware") {
      let parts: Vec<&str> = arg.split("=").collect();
      if parts.len() == 2 {
        let value = parts[1].trim();
        if value.is_empty() {
          emit_call_site_error!("Empty middleware value in controller attribute");
          continue;
        }
        match syn::parse_str::<Path>(value) {
          Ok(middleware_path) => {
            log_verbose!("Parsed middleware: [{}]", value);
            middlewares.push(middleware_path);
          }
          Err(_) => {
            emit_call_site_error!(
              "Invalid middleware path '{}'. Expected a valid Rust path (e.g., my_middleware or module::middleware)",
              value
            );
          }
        }
      } else {
        emit_call_site_error!(
          "Invalid middleware attribute format. Expected: middleware = my_middleware"
        );
      }
    }
  }

  // Parse header(...) attributes
  let mut search_pos = 0;
  while let Some(header_start) = attr_str[search_pos..].find("header") {
    let header_start = search_pos + header_start;
    if let Some(paren_start) = attr_str[header_start..].find('(') {
      let paren_start = header_start + paren_start + 1;
      if let Some(paren_end) = attr_str[paren_start..].find(')') {
        let header_content = &attr_str[paren_start..paren_start + paren_end];
        // Parse header_name = "header_value" or (header_name, header_value)
        if let Some(eq_pos) = header_content.find('=') {
          let header_name = header_content[..eq_pos].trim().replace('"', "");
          let header_value = header_content[eq_pos + 1..].trim().replace('"', "");
          if header_name.is_empty() || header_value.is_empty() {
            emit_call_site_warning!("Empty header name or value in controller header attribute");
          }
          log_verbose!(
            "Parsed controller header: [{}: {}]",
            header_name,
            header_value
          );
          response_headers.push((header_name, header_value));
        } else {
          // Try parsing as tuple: ("name", "value")
          let parts: Vec<&str> = header_content.split(',').collect();
          if parts.len() == 2 {
            let header_name = parts[0].trim().replace('"', "");
            let header_value = parts[1].trim().replace('"', "");
            if header_name.is_empty() || header_value.is_empty() {
              emit_call_site_warning!("Empty header name or value in controller header attribute");
            }
            log_verbose!(
              "Parsed controller header: [{}: {}]",
              header_name,
              header_value
            );
            response_headers.push((header_name, header_value));
          } else {
            emit_call_site_warning!(
              "Invalid header attribute format in controller. Expected: header(\"name\", \"value\") or header(name = \"value\")"
            );
          }
        }
        search_pos = paren_start + paren_end + 1;
      } else {
        emit_call_site_warning!("Unclosed parenthesis in header attribute");
        break;
      }
    } else {
      break;
    }
  }

  // Parse content_type(...) attribute
  if let Some(ct_start) = attr_str.find("content_type") {
    if let Some(paren_start) = attr_str[ct_start..].find('(') {
      let paren_start = ct_start + paren_start + 1;
      if let Some(paren_end) = attr_str[paren_start..].find(')') {
        let ct_content = &attr_str[paren_start..paren_start + paren_end];
        let ct_value = ct_content.trim().replace('"', "");
        if ct_value.is_empty() {
          emit_call_site_warning!("Empty content_type value in controller attribute");
        }
        log_verbose!("Parsed controller content_type: [{}]", ct_value);
        content_type = Some(ct_value);
      } else {
        emit_call_site_warning!("Unclosed parenthesis in content_type attribute");
      }
    }
  }

  ControllerConfig {
    route_prefix,
    middlewares,
    response_headers,
    content_type,
  }
}
