//! Route information extraction from attributes

use proc_macro_error::{emit_call_site_error, emit_call_site_warning};
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::Attribute;

use super::extractor_types::{ExtractorType, validate_extractors};

pub struct RouteInfo {
  pub method: String,
  pub path: String,
  pub extractors: HashMap<String, ExtractorType>,
  pub response_headers: Vec<(String, String)>, // (header_name, header_value)
  pub content_type: Option<String>,
}

/// Validates path parameters and emits errors/warnings
fn validate_path_parameters(path: &str, extractors: &HashMap<String, ExtractorType>) {
  // Extract path parameters from the path string
  let mut path_params = HashSet::with_capacity(4); // Most paths have 0-4 params

  // Support both {param} and :param syntax
  for capture in path.split('/') {
    if let Some(param) = capture.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
      path_params.insert(param.to_string());
    } else if let Some(param) = capture.strip_prefix(':') {
      path_params.insert(param.to_string());
    }
  }

  // Check if all path parameters have Path extractors
  for param in &path_params {
    match extractors.get(param) {
      Some(ExtractorType::Path) => {
        // Correct usage
      }
      Some(other_type) => {
        emit_call_site_error!(
          "Path parameter '{}' in route path '{}' should use Path extractor, but {:?} was specified",
          param,
          path,
          other_type
        );
      }
      None => {
        emit_call_site_error!(
          "Path parameter '{}' found in route path '{}' but no extractor specified. \
           Add 'extract({} = Path)' to the route attribute",
          param,
          path,
          param
        );
      }
    }
  }

  // Warn if Path extractors are specified but not in the path
  let path_extractors: Vec<_> = extractors
    .iter()
    .filter(|(_, ext)| matches!(ext, ExtractorType::Path))
    .map(|(name, _)| name)
    .collect();

  for param_name in path_extractors {
    if !path_params.contains(param_name) {
      emit_call_site_warning!(
        "Path extractor specified for parameter '{}' but it's not found in route path '{}'. \
         Make sure the path contains '{{{}}}' or ':{}'",
        param_name,
        path,
        param_name,
        param_name
      );
    }
  }
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
        "get" | "head" | "delete" | "options" | "patch" | "post" | "put" | "trace" | "connect" => {
          let mut route_path = "/".to_string();
          let mut extractors = HashMap::with_capacity(4); // Most routes have 0-4 extractors
          let mut response_headers = Vec::with_capacity(2); // Most routes have 0-2 headers
          let mut content_type = None;

          // Parse attribute content
          let attr_str = attr.meta.to_token_stream().to_string();

          // Extract path (first quoted string)
          if let Some(start) = attr_str.find('"') {
            if let Some(end) = attr_str[start + 1..].find('"') {
              let mut path = attr_str[start + 1..start + 1 + end].to_string();
              if !path.is_empty() {
                if !path.starts_with('/') {
                  path = format!("/{}", path);
                }
                route_path = path;
              }
            }
          }

          // Parse extract(...) if present
          if let Some(extract_start) = attr_str.find("extract") {
            if let Some(paren_start) = attr_str[extract_start..].find('(') {
              let paren_start = extract_start + paren_start + 1;
              if let Some(paren_end) = attr_str[paren_start..].find(')') {
                let extract_content = &attr_str[paren_start..paren_start + paren_end];

                // Parse param = Type pairs
                for pair in extract_content.split(',') {
                  let parts: Vec<&str> = pair.trim().split('=').map(|s| s.trim()).collect();
                  if parts.len() == 2 {
                    let param_name = parts[0].to_string();
                    let extractor_str = parts[1];

                    // Use from_str for validation
                    match ExtractorType::from_str(extractor_str) {
                      Ok(extractor_type) => {
                        extractors.insert(param_name, extractor_type);
                      }
                      Err(err_msg) => {
                        emit_call_site_error!(
                          "{}. Valid extractors are: Json, Form, Path, Query, State, Bytes, Text, Html, Xml, JavaScript, HeaderParam, CookieParam, SessionParam",
                          err_msg
                        );
                        // Insert None to continue parsing
                        extractors.insert(param_name, ExtractorType::None);
                      }
                    }
                  } else if parts.len() == 1 && !parts[0].is_empty() {
                    emit_call_site_error!(
                      "Invalid extractor syntax '{}'. Expected format: 'param_name = ExtractorType'",
                      pair.trim()
                    );
                  }
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
                  if header_name.is_empty() || header_value.is_empty() {
                    emit_call_site_warning!("Empty header name or value in header attribute");
                  }
                  response_headers.push((header_name, header_value));
                } else {
                  // Try parsing as tuple: ("name", "value")
                  let parts: Vec<&str> = header_content.split(',').collect();
                  if parts.len() == 2 {
                    let header_name = parts[0].trim().replace('"', "");
                    let header_value = parts[1].trim().replace('"', "");
                    if header_name.is_empty() || header_value.is_empty() {
                      emit_call_site_warning!("Empty header name or value in header attribute");
                    }
                    response_headers.push((header_name, header_value));
                  } else {
                    emit_call_site_warning!(
                      "Invalid header attribute format. Expected: header(\"name\", \"value\") or header(name = \"value\")"
                    );
                  }
                }
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
                let ct_value = ct_content.trim().replace('"', "");
                if ct_value.is_empty() {
                  emit_call_site_warning!("Empty content_type value");
                }
                content_type = Some(ct_value);
              }
            }
          }

          // Validate extractors
          validate_extractors(&extractors, &method);

          // Validate path parameters
          validate_path_parameters(&route_path, &extractors);

          log_verbose!(
            "Parsed route: [Method:{}] [Path:{}] [Extractors:{:?}] [Headers:{:?}] [ContentType:{:?}]",
            method,
            route_path,
            extractors,
            response_headers,
            content_type
          );

          return Some(RouteInfo {
            method,
            path: route_path,
            extractors,
            response_headers,
            content_type,
          });
        }
        _ => {
          // Unknown HTTP method
          emit_call_site_error!(
            "Unknown HTTP method '{}'. Valid methods are: get, post, put, patch, delete, head, options, trace",
            method
          );
        }
      }
    }
  }
  None
}
