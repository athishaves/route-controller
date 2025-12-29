//! Parameter analysis for route handlers

use proc_macro_error::emit_call_site_warning;
use std::collections::{HashMap, HashSet};
use syn::{FnArg, Pat, Type};

use super::extractor_types::ExtractorType;

pub struct ParamInfo<'a> {
  pub pat: &'a Pat,
  pub ty: &'a Type,
  pub extractor_type: ExtractorType,
}

/// Analyzes function parameters using explicit extractor mappings from route attributes
pub fn analyze_params<'a>(
  sig: &'a syn::Signature,
  extractor_map: &HashMap<String, ExtractorType>,
) -> Vec<ParamInfo<'a>> {
  let mut params = Vec::with_capacity(sig.inputs.len()); // Pre-allocate based on signature
  let mut seen_params = HashSet::with_capacity(sig.inputs.len());

  for input in &sig.inputs {
    if let FnArg::Typed(pat_type) = input {
      let pat = &*pat_type.pat;
      let ty = &*pat_type.ty;

      // Extract parameter name
      let param_name = if let Pat::Ident(pat_ident) = &pat {
        pat_ident.ident.to_string()
      } else {
        // For complex patterns, try to extract the first identifier
        emit_call_site_warning!(
          "Complex pattern in function parameter. Extractor mapping may not work correctly"
        );
        "unknown".to_string()
      };

      // Get extractor type from the map, default to None (do this before consuming param_name)
      let extractor_type = *extractor_map
        .get(param_name.as_str())
        .unwrap_or_else(|| {
          // Warn about parameters without extractors
          if param_name != "unknown" {
            emit_call_site_warning!(
              "Parameter '{}' has no extractor specified. It will not receive any data from the request",
              param_name
            );
          }
          &ExtractorType::None
        });

      // Check for duplicate parameter names (insert consumes param_name)
      if !seen_params.insert(param_name) {
        emit_call_site_warning!("Duplicate parameter name found in function signature");
      }

      params.push(ParamInfo {
        pat,
        ty,
        extractor_type,
      });
    }
  }

  // Check for extractors without matching parameters
  for (extractor_name, extractor_type) in extractor_map {
    if !seen_params.contains(extractor_name.as_str()) {
      emit_call_site_warning!(
        "Extractor specified for parameter '{}' ({:?}) but no parameter with that name exists in the function signature",
        extractor_name,
        extractor_type
      );
    }
  }

  params
}
