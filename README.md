# fpexif: A Pure Rust EXIF Metadata Library

[![CI](https://github.com/pgray/fpexif/actions/workflows/ci.yml/badge.svg)](https://github.com/pgray/fpexif/actions/workflows/ci.yml)

fpexif is a pure Rust library for parsing, manipulating, and writing EXIF metadata in image files.

It aims to be a complete alternative to libraries like libexiv2 and ExifTool, with a focus on safety, performance, and ease of use.

## Features

- Pure Rust implementation with no unsafe code
- Zero dependencies for the core functionality
- Fast and memory-efficient parsing
- Support for all standard EXIF tags
- Support for 23+ image formats including:
  - **Web formats**: JPEG, PNG, WebP, AVIF, HEIC/HEIF, JPEG XL
  - **RAW formats**: CR2, CR3, CRW, NEF, NRW, DNG, ARW, PEF, RWL, ORF, SRW, RW2, RAF, MRW, X3F
  - **Standard formats**: TIFF
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

## Supported Image Formats

### Modern Web Formats
- **JPEG** (.jpg, .jpeg) - Joint Photographic Experts Group
- **PNG** (.png) - Portable Network Graphics (with eXIf chunk)
- **WebP** (.webp) - Google WebP format
- **AVIF** (.avif) - AV1 Image File Format
- **HEIC/HEIF** (.heic, .heif) - High Efficiency Image Format
- **JPEG XL** (.jxl) - Next-generation JPEG format

### RAW Camera Formats
- **CR2** (.cr2) - Canon RAW 2
- **CR3** (.cr3) - Canon RAW 3
- **CRW** (.crw) - Canon RAW v1 (CIFF format)
- **NEF** (.nef) - Nikon Electronic Format (DSLR)
- **NRW** (.nrw) - Nikon RAW (Coolpix)
- **ARW** (.arw) - Sony Alpha RAW
- **DNG** (.dng) - Adobe Digital Negative
- **RAF** (.raf) - Fujifilm RAW
- **MRW** (.mrw) - Minolta RAW
- **X3F** (.x3f) - Sigma/Foveon RAW
- **ORF** (.orf) - Olympus RAW Format
- **PEF** (.pef) - Pentax Electronic File
- **RWL** (.rwl) - Leica RAW
- **SRW** (.srw) - Samsung RAW Format
- **RW2** (.rw2) - Panasonic RAW Format

### Standard Formats
- **TIFF** (.tif, .tiff) - Tagged Image File Format

## Architecture

fpexif is organized into several modules:

- **parser**: Responsible for binary parsing of EXIF data blocks
- **data_types**: Definitions for EXIF value types like SHORT, LONG, RATIONAL, etc.
- **tags**: Definitions and mappings for standard EXIF tags
- **formats**: Format-specific handlers for different image types
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

- [x] Support for modern web formats (PNG, HEIC, WebP, AVIF, JPEG XL) ✅
- [x] Support for major RAW formats (ARW, PEF, NRW, RWL, etc.) ✅
- [x] Support for legacy RAW formats (CRW, MRW, X3F) ✅
- [ ] Complete TIFF writer implementation
- [ ] Support for non-standard maker notes (Canon, Nikon, etc.)
- [ ] Support for XMP and IPTC metadata
- [ ] Performance optimizations
- [ ] Support for medium format RAW (3FR, FFF, DCR, KDC, MEF, MOS)
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
