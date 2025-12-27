# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-27

### Added

- Response header support with `header()` and `content_type()` attributes
- Multiple body extractors: `Bytes`, `Text`, `Html`, `Xml`, `JavaScript`
- Feature-gated extractors: `HeaderParam`, `CookieParam`, `SessionParam`
- Comprehensive examples
- Middleware support at controller level

### Changed

- Improved documentation with more examples
- Enhanced error messages

### Fixed

- Path parameter extraction order independence

## [0.1.0] - Initial Release

### Added

- Basic controller macro
- HTTP method attributes
- Path and Query extractors
- JSON and Form body extractors
- State management
