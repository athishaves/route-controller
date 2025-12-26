//! Extractor type definitions for different parameter extraction strategies

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
