//! Route information extraction from attributes

use quote::ToTokens;
use syn::Attribute;

use super::extractor_types::ExtractorType;

pub struct RouteInfo {
  pub method: String,
  pub path: String,
  pub extractors: std::collections::HashMap<String, ExtractorType>,
  pub response_headers: Vec<(String, String)>, // (header_name, header_value)
  pub content_type: Option<String>,
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
          let mut extractors = std::collections::HashMap::new();
          let mut response_headers = Vec::new();
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
                    let extractor_type = match parts[1] {
                      "Json" => ExtractorType::Json,
                      "Form" => ExtractorType::Form,
                      "Path" => ExtractorType::Path,
                      "Query" => ExtractorType::Query,
                      "HeaderParam" => ExtractorType::HeaderParam,
                      "CookieParam" => ExtractorType::CookieParam,
                      "SessionParam" => ExtractorType::SessionParam,
                      "State" => ExtractorType::State,
                      "Bytes" => ExtractorType::Bytes,
                      "Text" => ExtractorType::Text,
                      "Html" => ExtractorType::Html,
                      "Xml" => ExtractorType::Xml,
                      "JavaScript" => ExtractorType::JavaScript,
                      _ => ExtractorType::None,
                    };
                    extractors.insert(param_name, extractor_type);
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
              }
            }
          }

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
        _ => {}
      }
    }
  }
  None
}
