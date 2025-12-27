//! Parameter analysis for route handlers

use proc_macro_error::emit_call_site_warning;
use syn::{FnArg, Pat, Type};

use super::extractor_types::ExtractorType;

#[derive(Clone)]
pub struct ParamInfo {
  pub pat: Pat,
  pub ty: Type,
  pub extractor_type: ExtractorType,
}

/// Analyzes function parameters using explicit extractor mappings from route attributes
pub fn analyze_params(
  sig: &syn::Signature,
  extractor_map: &std::collections::HashMap<String, ExtractorType>,
) -> Vec<ParamInfo> {
  let mut params = Vec::new();
  let mut seen_params = std::collections::HashSet::new();

  for input in &sig.inputs {
    if let FnArg::Typed(pat_type) = input {
      let pat = (*pat_type.pat).clone();
      let ty = (*pat_type.ty).clone();

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

      // Check for duplicate parameter names
      if !seen_params.insert(param_name.clone()) {
        emit_call_site_warning!(
          "Duplicate parameter name '{}' found in function signature",
          param_name
        );
      }

      // Get extractor type from the map, default to None
      let extractor_type = extractor_map
        .get(&param_name)
        .cloned()
        .unwrap_or_else(|| {
          // Warn about parameters without extractors
          if param_name != "unknown" {
            emit_call_site_warning!(
              "Parameter '{}' has no extractor specified. It will not receive any data from the request",
              param_name
            );
          }
          ExtractorType::None
        });

      params.push(ParamInfo {
        pat,
        ty,
        extractor_type,
      });
    }
  }

  // Check for extractors without matching parameters
  for (extractor_name, extractor_type) in extractor_map {
    if !seen_params.contains(extractor_name) {
      emit_call_site_warning!(
        "Extractor specified for parameter '{}' ({:?}) but no parameter with that name exists in the function signature",
        extractor_name,
        extractor_type
      );
    }
  }

  params
}
