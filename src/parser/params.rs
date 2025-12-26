//! Parameter analysis for route handlers

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

  for input in &sig.inputs {
    if let FnArg::Typed(pat_type) = input {
      let pat = (*pat_type.pat).clone();
      let ty = (*pat_type.ty).clone();

      // Extract parameter name
      let param_name = if let Pat::Ident(pat_ident) = &pat {
        pat_ident.ident.to_string()
      } else {
        // For complex patterns, try to extract the first identifier
        "unknown".to_string()
      };

      // Get extractor type from the map, default to None
      let extractor_type = extractor_map
        .get(&param_name)
        .cloned()
        .unwrap_or(ExtractorType::None);

      params.push(ParamInfo {
        pat,
        ty,
        extractor_type,
      });
    }
  }

  params
}
