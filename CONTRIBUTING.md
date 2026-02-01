# Contributing to fpexif

Thank you for your interest in contributing to fpexif! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/fpexif.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Submit a pull request

## Development Setup

```bash
# Build the library
cargo build

# Build with CLI support
cargo build --features cli

# Run tests
cargo test

# Run specific test
cargo test test_name
```

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Run `cargo clippy` and address any warnings
- Write tests for new functionality
- Keep commits focused and atomic

## Adding MakerNote Support

See [MAKERNOTES.md](MAKERNOTES.md) for detailed instructions on adding manufacturer-specific parsing.

### Quick Start

1. Create a new file in `src/makernotes/` (e.g., `manufacturer.rs`)
2. Add tag constants and `get_*_tag_name()` function
3. Use the `define_tag_decoder!` macro for value decoding:

```rust
use crate::define_tag_decoder;

define_tag_decoder! {
    focus_mode,
    both: {
        0 => "Auto",
        1 => "Manual",
    }
}
```

4. Add the module to `src/makernotes/mod.rs`
5. Test against real camera files

### Reference Implementations

The codebase includes reference implementations as submodules:
- `exiftool/` - ExifTool (Perl)
- `exiv2/` - Exiv2 (C++)

Use these to find tag definitions and value mappings.

## Testing

### Running Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific module
cargo test makernotes::canon
```

### Regression Testing

Use `./bin/mfr-test` to prevent regressions when modifying makernote parsers:

```bash
# Save baseline before changes
./bin/mfr-test canon --save-baseline

# Check for regressions after changes
./bin/mfr-test canon --check
```

## Pull Request Guidelines

1. **Title**: Use a clear, descriptive title
2. **Description**: Explain what the PR does and why
3. **Tests**: Include tests for new functionality
4. **Documentation**: Update relevant docs
5. **Size**: Keep PRs focused - smaller is better

## Reporting Issues

When reporting issues, please include:
- Rust version (`rustc --version`)
- Operating system
- Minimal reproduction case
- Sample file (if applicable and shareable)

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
