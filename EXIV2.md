# Exiv2 Compatibility Roadmap

This document tracks progress toward matching exiv2's most commonly used features.

Exiv2 is a C++ library and command-line tool focused on EXIF, IPTC, and XMP metadata. It has a different philosophy than exiftool - more focused on standards compliance and library integration.

## Reading Features

### Implemented

- [x] **Basic EXIF reading**
  - `fpexif list image.jpg`
  - Implementation: Core parsing already supports EXIF from multiple formats.

### Core Reading Features

- [ ] **Print all metadata (default)**
  - `fpexif exiv2 image.jpg`
  - Exiv2 equivalent: `exiv2 image.jpg`
  - Implementation: Add `exiv2` subcommand with default behavior showing all metadata in exiv2's format: `Exif.Image.Make Ascii 6 Canon`

- [ ] **Print EXIF only (`-pe`)**
  - `fpexif exiv2 -pe image.jpg`
  - Exiv2 equivalent: `exiv2 -pe image.jpg`
  - Implementation: Filter output to only EXIF tags. Already have this data, just filter by group.

- [ ] **Print IPTC only (`-pi`)**
  - `fpexif exiv2 -pi image.jpg`
  - Exiv2 equivalent: `exiv2 -pi image.jpg`
  - Implementation: Requires IPTC parsing support (APP13 segment in JPEG). New parser needed.

- [ ] **Print XMP only (`-px`)**
  - `fpexif exiv2 -px image.jpg`
  - Exiv2 equivalent: `exiv2 -px image.jpg`
  - Implementation: Requires XMP parsing support (XML in APP1 with XMP namespace). Add XML parser dependency.

- [ ] **Print ICC profile (`-pC`)**
  - `fpexif exiv2 -pC image.jpg`
  - Exiv2 equivalent: `exiv2 -pC image.jpg`
  - Implementation: Extract and display ICC color profile info from APP2 segment.

- [ ] **Print summary (`-ps`)**
  - `fpexif exiv2 -ps image.jpg`
  - Exiv2 equivalent: `exiv2 -ps image.jpg`
  - Implementation: Show file structure summary (segments, IFDs, etc.).

- [ ] **Print interpreted values (`-pt`)**
  - `fpexif exiv2 -pt image.jpg`
  - Exiv2 equivalent: `exiv2 -pt image.jpg`
  - Implementation: Show human-readable interpreted values (already partially implemented).

- [ ] **Print raw values (`-pv`)**
  - `fpexif exiv2 -pv image.jpg`
  - Exiv2 equivalent: `exiv2 -pv image.jpg`
  - Implementation: Show raw hex/numeric values without interpretation.

- [ ] **Grep for key (`-g KEY`)**
  - `fpexif exiv2 -g Make image.jpg`
  - Exiv2 equivalent: `exiv2 -g Make image.jpg`
  - Implementation: Filter output to tags matching pattern. Simple string matching on tag names.

- [ ] **Exclude key (`-K KEY`)**
  - `fpexif exiv2 -K Exif.Image.Make image.jpg`
  - Exiv2 equivalent: `exiv2 -K Exif.Image.Make image.jpg`
  - Implementation: Exact key match (vs grep's pattern match).

### Output Format Features

- [ ] **Exiv2 key format**
  - Output like: `Exif.Image.Make                              Ascii       6  Canon`
  - Implementation: Format as `{Group}.{Subgroup}.{TagName} {Type} {Count} {Value}`. Map our TagGroup to exiv2's naming convention.

- [ ] **JSON output**
  - `fpexif exiv2 --json image.jpg`
  - Note: Exiv2 doesn't have native JSON, but we can add it for consistency with our exiftool interface.

- [ ] **Verbose output (`-v`)**
  - `fpexif exiv2 -v image.jpg`
  - Exiv2 equivalent: `exiv2 -v image.jpg`
  - Implementation: Show additional details like offsets, byte counts, etc.

### File Information

- [ ] **Print file info only**
  - `fpexif exiv2 -pf image.jpg`
  - Implementation: Show file format, size, MIME type without parsing metadata.

- [ ] **Print structure (`-pS`)**
  - `fpexif exiv2 -pS image.jpg`
  - Exiv2 equivalent: `exiv2 -pS image.jpg`
  - Implementation: Show internal structure (IFDs, segments) in tree format.

- [ ] **Print recursive structure (`-pR`)**
  - `fpexif exiv2 -pR image.jpg`
  - Implementation: Recursively show all nested structures.

## Writing Features

### Basic Writing

- [ ] **Modify tag (`-M "set KEY VALUE"`)**
  - `fpexif exiv2 -M "set Exif.Image.Artist John Doe" image.jpg`
  - Exiv2 equivalent: `exiv2 -M "set Exif.Image.Artist John Doe" image.jpg`
  - Implementation: Parse modify commands, locate tag, update value. Requires EXIF writing support.

- [ ] **Delete tag (`-M "del KEY"`)**
  - `fpexif exiv2 -M "del Exif.Image.Artist" image.jpg`
  - Exiv2 equivalent: `exiv2 -M "del Exif.Image.Artist" image.jpg`
  - Implementation: Remove tag from IFD.

- [ ] **Add tag (`-M "add KEY VALUE"`)**
  - `fpexif exiv2 -M "add Exif.Image.Artist John Doe" image.jpg`
  - Implementation: Add new tag to appropriate IFD.

- [ ] **Batch commands from file (`-m FILE`)**
  - `fpexif exiv2 -m commands.txt image.jpg`
  - Implementation: Read modify commands from file, execute in order.

### Metadata Operations

- [ ] **Delete all EXIF (`-de`)**
  - `fpexif exiv2 -de image.jpg`
  - Implementation: Remove entire EXIF APP1 segment.

- [ ] **Delete all IPTC (`-di`)**
  - `fpexif exiv2 -di image.jpg`
  - Implementation: Remove APP13 segment.

- [ ] **Delete all XMP (`-dx`)**
  - `fpexif exiv2 -dx image.jpg`
  - Implementation: Remove XMP from APP1.

- [ ] **Delete thumbnail (`-dt`)**
  - `fpexif exiv2 -dt image.jpg`
  - Implementation: Remove IFD1 and thumbnail data.

- [ ] **Delete all metadata (`-da`)**
  - `fpexif exiv2 -da image.jpg`
  - Implementation: Strip all metadata segments.

### Copy/Transfer

- [ ] **Extract thumbnail (`-et`)**
  - `fpexif exiv2 -et image.jpg`
  - Implementation: Extract embedded thumbnail to separate file.

- [ ] **Insert thumbnail (`-it FILE`)**
  - `fpexif exiv2 -it thumb.jpg image.jpg`
  - Implementation: Insert/replace thumbnail in EXIF.

- [ ] **Extract XMP (`-eX`)**
  - `fpexif exiv2 -eX image.jpg`
  - Implementation: Extract XMP to .xmp sidecar file.

- [ ] **Insert XMP (`-iX FILE`)**
  - `fpexif exiv2 -iX metadata.xmp image.jpg`
  - Implementation: Insert XMP from sidecar file.

## Advanced Features

### IPTC Support

- [ ] **Parse IPTC-IIM**
  - Implementation: Add parser for APP13 segment containing IPTC-IIM data. Different binary format than EXIF.

- [ ] **IPTC tag definitions**
  - Implementation: Add IPTC tag ID to name mappings (Caption, Keywords, Copyright, etc.).

### XMP Support

- [ ] **Parse XMP**
  - Implementation: Add XML parser (quick-xml or roxmltree) to parse XMP packet in APP1.

- [ ] **XMP namespace handling**
  - Implementation: Support standard XMP namespaces (dc, xmp, exif, tiff, etc.).

- [ ] **XMP-EXIF synchronization**
  - Implementation: Sync values between XMP and EXIF when both exist.

### Maker Notes

- [ ] **Expanded maker note support**
  - Current: Canon, Nikon, Sony (partial)
  - Needed: Fuji, Olympus, Panasonic, Pentax, Sigma, Leica
  - Implementation: Add format-specific parsers for each manufacturer.

### Standards Compliance

- [ ] **EXIF 2.32 compliance**
  - Implementation: Ensure all standard EXIF 2.32 tags are supported and correctly interpreted.

- [ ] **TIFF/EP support**
  - Implementation: Support TIFF/EP specific tags used in DNG and some RAW formats.

- [ ] **DCF compliance**
  - Implementation: Validate DCF (Design rule for Camera File system) compliance.

## Key Differences from Exiftool

| Feature | Exiftool | Exiv2 | fpexif |
|---------|----------|-------|--------|
| Tag naming | `Make` | `Exif.Image.Make` | Both |
| Output format | `Tag : Value` | `Key Type Count Value` | Both |
| IPTC support | Yes | Yes | Planned |
| XMP support | Yes | Yes | Planned |
| Maker notes | Extensive | Good | Partial |
| Writing | Yes | Yes | Planned |
| Scripting | Perl-like | Command-based | TBD |

## Testing Checklist

- [ ] Compare output format with exiv2 for common tags
- [ ] Test key naming convention matches exiv2
- [ ] Validate type names match (Ascii, Short, Long, Rational, etc.)
- [ ] Test grep/filter functionality
- [ ] Compare structure output with exiv2 -pS

## Priority Order

1. **High Priority** (core exiv2 compatibility):
   - Exiv2-style output format (`Exif.Image.Make Ascii 6 Canon`)
   - Print EXIF only (`-pe`)
   - Grep filter (`-g`)
   - Raw values (`-pv`)

2. **Medium Priority** (useful features):
   - Structure printing (`-pS`)
   - Verbose mode (`-v`)
   - IPTC reading (`-pi`)
   - XMP reading (`-px`)

3. **Low Priority** (advanced/writing):
   - Writing support
   - Thumbnail operations
   - XMP sidecar support
   - Full IPTC/XMP writing

## Implementation Notes

### Exiv2 Key Format

Exiv2 uses hierarchical key names:
- `Exif.Image.Make` - IFD0 tags
- `Exif.Photo.DateTimeOriginal` - EXIF SubIFD tags
- `Exif.GPSInfo.GPSLatitude` - GPS SubIFD tags
- `Exif.Thumbnail.Compression` - IFD1 tags
- `Exif.Iop.InteroperabilityIndex` - Interop tags
- `Iptc.Application2.Caption` - IPTC tags
- `Xmp.dc.creator` - XMP tags

Map our TagGroup enum:
```rust
fn to_exiv2_group(group: TagGroup) -> &'static str {
    match group {
        TagGroup::Main => "Exif.Image",
        TagGroup::Exif => "Exif.Photo",
        TagGroup::Gps => "Exif.GPSInfo",
        TagGroup::Thumbnail => "Exif.Thumbnail",
        TagGroup::Interop => "Exif.Iop",
    }
}
```

### Type Names

Map our data types to exiv2 names:
- `Byte` → `Byte`
- `Ascii` → `Ascii`
- `Short` → `Short`
- `Long` → `Long`
- `Rational` → `Rational`
- `SByte` → `SByte`
- `Undefined` → `Undefined`
- `SShort` → `SShort`
- `SLong` → `SLong`
- `SRational` → `SRational`
- `Float` → `Float`
- `Double` → `Double`

## Contributing

When implementing an exiv2 feature:

1. Check this document for implementation notes
2. Add CLI argument in `cli.rs`
3. Implement formatting in new `exiv2_output.rs` module
4. Add tests comparing with actual exiv2 output
5. Update this document
