//! Controller configuration parsing

use proc_macro::TokenStream;
use syn::Path;

pub struct ControllerConfig {
  pub route_prefix: Option<String>,
  pub middlewares: Vec<Path>,
  pub response_headers: Vec<(String, String)>,
  pub content_type: Option<String>,
}

pub fn parse_controller_attributes(attr: &TokenStream) -> ControllerConfig {
  let mut route_prefix: Option<String> = None;
  let mut middlewares: Vec<Path> = vec![];
  let mut response_headers: Vec<(String, String)> = vec![];
  let mut content_type: Option<String> = None;

  let attr_str = attr.to_string();

  // Parse path and middleware using split
  let args = attr_str.split(",");
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
        if let Ok(middleware_path) = syn::parse_str::<Path>(value) {
          log_verbose!("Parsed middleware: [{}]", value);
          middlewares.push(middleware_path);
        }
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
          response_headers.push((header_name, header_value));
        } else {
          // Try parsing as tuple: ("name", "value")
          let parts: Vec<&str> = header_content.split(',').collect();
          if parts.len() == 2 {
            let header_name = parts[0].trim().replace('"', "");
            let header_value = parts[1].trim().replace('"', "");
            response_headers.push((header_name, header_value));
          }
        }
        log_verbose!(
          "Parsed controller header: [{}: {}]",
          response_headers.last().unwrap().0,
          response_headers.last().unwrap().1
        );
        search_pos = paren_start + paren_end + 1;
      } else {
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
        content_type = Some(ct_content.trim().replace('"', ""));
        log_verbose!(
          "Parsed controller content_type: [{}]",
          content_type.as_ref().unwrap()
        );
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
