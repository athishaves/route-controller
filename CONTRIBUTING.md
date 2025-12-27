# Contributing to route_controller

First off, thank you for considering contributing!

## Development Process

1. **Fork the repo** and create your branch from `main`.
2. **Install dependencies**: Ensure you have the latest stable Rust toolchain.
3. **Make your changes**:
   - If you add a new macro feature, please add an example in the `examples/` folder
   - Update documentation in README.md if needed
   - Add entry to CHANGELOG.md under `[Unreleased]`
4. **Run Tests**:

   ```bash
   cargo test
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test --all-features
   ```

5. **Submit a PR**: Provide a clear description of the changes.

## Code Style

- We follow the standard `rustfmt` style.
- All public-facing macros must have doc comments and examples.
- Use `proc_macro_error` for user-friendly compile errors.
- Keep functions focused and small.
- Write clear commit messages.

## Adding New Features

When adding a new extractor or feature:

1. Implement the feature in `src/lib.rs`
2. Add comprehensive tests
3. Create an example in `examples/`
4. Document in README.md
5. Update CHANGELOG.md

## Questions?

Feel free to open an issue for discussion before starting work on major features!
