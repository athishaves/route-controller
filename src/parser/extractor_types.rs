//! Extractor type definitions for different parameter extraction strategies

use proc_macro_error::{abort_call_site, emit_call_site_warning};

#[derive(Clone, Debug, PartialEq)]
pub enum ExtractorType {
  Path,
  Query,
  HeaderParam,
  CookieParam,
  SessionParam,
  State,
  // Body extractors
  Json,
  Form,
  Bytes,
  Text,
  Html,
  Xml,
  JavaScript,

  None,
}

impl ExtractorType {
  pub fn from_str(s: &str) -> Result<Self, String> {
    match s {
      "Json" => Ok(ExtractorType::Json),
      "Form" => Ok(ExtractorType::Form),
      "Path" => Ok(ExtractorType::Path),
      "Query" => Ok(ExtractorType::Query),
      "HeaderParam" => Ok(ExtractorType::HeaderParam),
      "CookieParam" => Ok(ExtractorType::CookieParam),
      "SessionParam" => Ok(ExtractorType::SessionParam),
      "State" => Ok(ExtractorType::State),
      "Bytes" => Ok(ExtractorType::Bytes),
      "Text" => Ok(ExtractorType::Text),
      "Html" => Ok(ExtractorType::Html),
      "Xml" => Ok(ExtractorType::Xml),
      "JavaScript" => Ok(ExtractorType::JavaScript),
      _ => Err(format!("Unknown extractor type: '{}'", s)),
    }
  }

  pub fn is_body_extractor(&self) -> bool {
    matches!(
      self,
      ExtractorType::Json
        | ExtractorType::Form
        | ExtractorType::Bytes
        | ExtractorType::Text
        | ExtractorType::Html
        | ExtractorType::Xml
        | ExtractorType::JavaScript
    )
  }

  pub fn requires_feature(&self) -> Option<&'static str> {
    match self {
      ExtractorType::HeaderParam => Some("headers"),
      ExtractorType::CookieParam => Some("cookies"),
      ExtractorType::SessionParam => Some("sessions"),
      _ => None,
    }
  }
}

/// Validates extractors and emits appropriate errors/warnings
pub fn validate_extractors(
  extractors: &std::collections::HashMap<String, ExtractorType>,
  route_method: &str,
) {
  let body_extractors: Vec<_> = extractors
    .iter()
    .filter(|(_, ext)| ext.is_body_extractor())
    .collect();

  // Error: Multiple body extractors
  if body_extractors.len() > 1 {
    let extractor_names: Vec<String> = body_extractors
      .iter()
      .map(|(name, ext)| format!("{} ({:?})", name, ext))
      .collect();
    abort_call_site!(
      "Multiple body extractors found: {}. Only one body extractor is allowed per route.",
      extractor_names.join(", ")
    );
  }

  // Warning: Body extractors on GET/HEAD/DELETE methods
  if matches!(route_method, "get" | "head" | "delete") && !body_extractors.is_empty() {
    let (name, ext) = body_extractors[0];
    emit_call_site_warning!(
      "Body extractor '{}' ({:?}) on {} method. HTTP {} requests typically don't have request bodies.",
      name,
      ext,
      route_method.to_uppercase(),
      route_method.to_uppercase()
    );
  }

  // Check for feature-gated extractors
  for (param_name, extractor) in extractors {
    if let Some(feature) = extractor.requires_feature() {
      emit_call_site_warning!(
        "Extractor '{:?}' for parameter '{}' requires the '{}' feature to be enabled. \
         Add it to your Cargo.toml: route_controller = {{ version = \"*\", features = [\"{}\"]}}",
        extractor,
        param_name,
        feature,
        feature
      );
    }
  }
}
