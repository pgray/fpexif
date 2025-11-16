# fpexif: A Pure Rust EXIF Metadata Library

fpexif is a pure Rust library for parsing, manipulating, and writing EXIF metadata in image files.

It aims to be a complete alternative to libraries like libexiv2 and ExifTool, with a focus on safety, performance, and ease of use.

## Features

- Pure Rust implementation with no unsafe code
- Zero dependencies for the core functionality
- Fast and memory-efficient parsing
- Support for all standard EXIF tags
- Support for various file formats (JPEG, TIFF, etc.)
- Simple and intuitive API
- Comprehensive documentation and examples

## Usage

Add fpexif to your `Cargo.toml`:

```toml
[dependencies]
fpexif = "0.1.0"
```

### Basic Example

```rust
use fpexif::ExifParser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a parser with default configuration
    let parser = ExifParser::new();

    // Parse EXIF data from a file
    let exif_data = parser.parse_file(Path::new("path/to/image.jpg"))?;

    // Access specific tags by name
    if let Some(date_time) = exif_data.get_tag_by_name("DateTimeOriginal") {
        println!("Photo taken: {}", date_time);
    }

    // Iterate through all tags
    for (tag_id, value) in exif_data.iter() {
        if let Some(tag_name) = tag_id.name() {
            println!("{}: {}", tag_name, value);
        } else {
            println!("Tag 0x{:04X}: {}", tag_id.id, value);
        }
    }

    Ok(())
}
```

### Command Line Interface

fpexif also includes a command line interface for quickly viewing EXIF metadata:

```bash
# List all EXIF tags in a file
fpexif list path/to/image.jpg

# Extract a specific tag
fpexif extract path/to/image.jpg DateTimeOriginal
```

## Building from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/pgray/fpexif.git
   cd fpexif
   ```

2. Build the library:

   ```bash
   cargo build
   ```

3. Build with CLI support:

   ```bash
   cargo build --release --features cli
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Architecture

fpexif is organized into several modules:

- **parser**: Responsible for binary parsing of EXIF data blocks
- **data_types**: Definitions for EXIF value types like SHORT, LONG, RATIONAL, etc.
- **tags**: Definitions and mappings for standard EXIF tags
- **io**: Utilities for reading and writing EXIF data to files
- **errors**: Error types and handling

This modular design makes the library easy to maintain and extend.

## Comparison with Other Libraries

### vs. libexiv2

- **Advantages**: Pure Rust (no C++ bindings), memory safety guarantees, easier integration in Rust projects
- **Disadvantages**: Currently fewer features, less mature

### vs. ExifTool

- **Advantages**: Better performance, no external dependencies, native Rust API
- **Disadvantages**: Smaller tag database, less file format support (currently)

## Contributing

Contributions are welcome! Here are some ways you can contribute:

- Report bugs and request features by opening an issue
- Improve documentation and examples
- Add support for more file formats
- Implement missing features
- Submit pull requests with bug fixes or improvements

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Roadmap

- [ ] Complete TIFF writer implementation
- [ ] Support for non-standard maker notes (Canon, Nikon, etc.)
- [ ] Support for XMP and IPTC metadata
- [ ] Performance optimizations
- [ ] Support for more image formats (PNG, HEIC, WebP)
- [ ] Integration with popular Rust image processing libraries

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This project draws inspiration from:

- [ExifTool](https://exiftool.org/) by Phil Harvey
- [libexiv2](https://www.exiv2.org/)
- The EXIF specification and documentation

## Authors

- Your Name (@yourusername) - Initial work and maintenance
