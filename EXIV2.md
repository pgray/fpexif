# Exiv2 Compatibility Roadmap

This document tracks progress toward matching exiv2's most commonly used features.

Exiv2 is a C++ library and command-line tool focused on EXIF, IPTC, and XMP metadata. It has a different philosophy than exiftool - more focused on standards compliance and library integration.

[the book](https://exiv2.org/book/)

## Reading Features

### Implemented

- [x] **V1. Basic EXIF reading**
  - `fpexif list image.jpg`
  - Implementation: Core parsing already supports EXIF from multiple formats.

### Core Reading Features

- [ ] **V2. Print all metadata (default)**
  - `fpexif exiv2 image.jpg`
  - Exiv2 equivalent: `exiv2 image.jpg`
  - Implementation: Add `exiv2` subcommand with default behavior showing all metadata in exiv2's format: `Exif.Image.Make Ascii 6 Canon`

- [ ] **V3. Print EXIF only (`-pe`)**
  - `fpexif exiv2 -pe image.jpg`
  - Exiv2 equivalent: `exiv2 -pe image.jpg`
  - Implementation: Filter output to only EXIF tags. Already have this data, just filter by group.

- [ ] **V4. Print IPTC only (`-pi`)**
  - `fpexif exiv2 -pi image.jpg`
  - Exiv2 equivalent: `exiv2 -pi image.jpg`
  - Implementation: Requires IPTC parsing support (APP13 segment in JPEG). New parser needed.

- [ ] **V5. Print XMP only (`-px`)**
  - `fpexif exiv2 -px image.jpg`
  - Exiv2 equivalent: `exiv2 -px image.jpg`
  - Implementation: Requires XMP parsing support (XML in APP1 with XMP namespace). Add XML parser dependency.

- [ ] **V6. Print ICC profile (`-pC`)**
  - `fpexif exiv2 -pC image.jpg`
  - Exiv2 equivalent: `exiv2 -pC image.jpg`
  - Implementation: Extract and display ICC color profile info from APP2 segment.

- [ ] **V7. Print summary (`-ps`)**
  - `fpexif exiv2 -ps image.jpg`
  - Exiv2 equivalent: `exiv2 -ps image.jpg`
  - Implementation: Show file structure summary (segments, IFDs, etc.).

- [ ] **V8. Print interpreted values (`-pt`)**
  - `fpexif exiv2 -pt image.jpg`
  - Exiv2 equivalent: `exiv2 -pt image.jpg`
  - Implementation: Show human-readable interpreted values (already partially implemented).

- [ ] **V9. Print raw values (`-pv`)**
  - `fpexif exiv2 -pv image.jpg`
  - Exiv2 equivalent: `exiv2 -pv image.jpg`
  - Implementation: Show raw hex/numeric values without interpretation.

- [ ] **V10. Grep for key (`-g KEY`)**
  - `fpexif exiv2 -g Make image.jpg`
  - Exiv2 equivalent: `exiv2 -g Make image.jpg`
  - Implementation: Filter output to tags matching pattern. Simple string matching on tag names.

- [ ] **V11. Exclude key (`-K KEY`)**
  - `fpexif exiv2 -K Exif.Image.Make image.jpg`
  - Exiv2 equivalent: `exiv2 -K Exif.Image.Make image.jpg`
  - Implementation: Exact key match (vs grep's pattern match).

### Output Format Features

- [ ] **V12. Exiv2 key format**
  - Output like: `Exif.Image.Make                              Ascii       6  Canon`
  - Implementation: Format as `{Group}.{Subgroup}.{TagName} {Type} {Count} {Value}`. Map our TagGroup to exiv2's naming convention.

- [ ] **V13. JSON output**
  - `fpexif exiv2 --json image.jpg`
  - Note: Exiv2 doesn't have native JSON, but we can add it for consistency with our exiftool interface.

- [ ] **V14. Verbose output (`-v`)**
  - `fpexif exiv2 -v image.jpg`
  - Exiv2 equivalent: `exiv2 -v image.jpg`
  - Implementation: Show additional details like offsets, byte counts, etc.

### File Information

- [ ] **V15. Print file info only**
  - `fpexif exiv2 -pf image.jpg`
  - Implementation: Show file format, size, MIME type without parsing metadata.

- [ ] **V16. Print structure (`-pS`)**
  - `fpexif exiv2 -pS image.jpg`
  - Exiv2 equivalent: `exiv2 -pS image.jpg`
  - Implementation: Show internal structure (IFDs, segments) in tree format.

- [ ] **V17. Print recursive structure (`-pR`)**
  - `fpexif exiv2 -pR image.jpg`
  - Implementation: Recursively show all nested structures.

## Writing Features

### Basic Writing

- [ ] **V18. Modify tag (`-M "set KEY VALUE"`)**
  - `fpexif exiv2 -M "set Exif.Image.Artist John Doe" image.jpg`
  - Exiv2 equivalent: `exiv2 -M "set Exif.Image.Artist John Doe" image.jpg`
  - Implementation: Parse modify commands, locate tag, update value. Requires EXIF writing support.

- [ ] **V19. Delete tag (`-M "del KEY"`)**
  - `fpexif exiv2 -M "del Exif.Image.Artist" image.jpg`
  - Exiv2 equivalent: `exiv2 -M "del Exif.Image.Artist" image.jpg`
  - Implementation: Remove tag from IFD.

- [ ] **V20. Add tag (`-M "add KEY VALUE"`)**
  - `fpexif exiv2 -M "add Exif.Image.Artist John Doe" image.jpg`
  - Implementation: Add new tag to appropriate IFD.

- [ ] **V21. Batch commands from file (`-m FILE`)**
  - `fpexif exiv2 -m commands.txt image.jpg`
  - Implementation: Read modify commands from file, execute in order.

### Metadata Operations

- [ ] **V22. Delete all EXIF (`-de`)**
  - `fpexif exiv2 -de image.jpg`
  - Implementation: Remove entire EXIF APP1 segment.

- [ ] **V23. Delete all IPTC (`-di`)**
  - `fpexif exiv2 -di image.jpg`
  - Implementation: Remove APP13 segment.

- [ ] **V24. Delete all XMP (`-dx`)**
  - `fpexif exiv2 -dx image.jpg`
  - Implementation: Remove XMP from APP1.

- [ ] **V25. Delete thumbnail (`-dt`)**
  - `fpexif exiv2 -dt image.jpg`
  - Implementation: Remove IFD1 and thumbnail data.

- [ ] **V26. Delete all metadata (`-da`)**
  - `fpexif exiv2 -da image.jpg`
  - Implementation: Strip all metadata segments.

### Copy/Transfer

- [ ] **V27. Extract thumbnail (`-et`)**
  - `fpexif exiv2 -et image.jpg`
  - Implementation: Extract embedded thumbnail to separate file.

- [ ] **V28. Insert thumbnail (`-it FILE`)**
  - `fpexif exiv2 -it thumb.jpg image.jpg`
  - Implementation: Insert/replace thumbnail in EXIF.

- [ ] **V29. Extract XMP (`-eX`)**
  - `fpexif exiv2 -eX image.jpg`
  - Implementation: Extract XMP to .xmp sidecar file.

- [ ] **V30. Insert XMP (`-iX FILE`)**
  - `fpexif exiv2 -iX metadata.xmp image.jpg`
  - Implementation: Insert XMP from sidecar file.

## Advanced Features

### IPTC Support

- [ ] **V31. Parse IPTC-IIM**
  - Implementation: Add parser for APP13 segment containing IPTC-IIM data. Different binary format than EXIF.

- [ ] **V32. IPTC tag definitions**
  - Implementation: Add IPTC tag ID to name mappings (Caption, Keywords, Copyright, etc.).

### XMP Support

- [ ] **V33. Parse XMP**
  - Implementation: Add XML parser (quick-xml or roxmltree) to parse XMP packet in APP1.

- [ ] **V34. XMP namespace handling**
  - Implementation: Support standard XMP namespaces (dc, xmp, exif, tiff, etc.).

- [ ] **V35. XMP-EXIF synchronization**
  - Implementation: Sync values between XMP and EXIF when both exist.

### Maker Notes

- [ ] **V36. Expanded maker note support**
  - Current: Canon, Nikon, Sony (partial)
  - Needed: Fuji, Olympus, Panasonic, Pentax, Sigma, Leica
  - Implementation: Add format-specific parsers for each manufacturer.

### Standards Compliance

- [ ] **V37. EXIF 2.32 compliance**
  - Implementation: Ensure all standard EXIF 2.32 tags are supported and correctly interpreted.

- [ ] **V38. TIFF/EP support**
  - Implementation: Support TIFF/EP specific tags used in DNG and some RAW formats.

- [ ] **V39. DCF compliance**
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

- [ ] **VT1.** Compare output format with exiv2 for common tags
- [ ] **VT2.** Test key naming convention matches exiv2
- [ ] **VT3.** Validate type names match (Ascii, Short, Long, Rational, etc.)
- [ ] **VT4.** Test grep/filter functionality
- [ ] **VT5.** Compare structure output with exiv2 -pS

## Priority Order

1. **High Priority** (core exiv2 compatibility):
   - V2. Default output format
   - V12. Exiv2-style output format (`Exif.Image.Make Ascii 6 Canon`)
   - V3. Print EXIF only (`-pe`)
   - V10. Grep filter (`-g`)
   - V9. Raw values (`-pv`)

2. **Medium Priority** (useful features):
   - V16. Structure printing (`-pS`)
   - V14. Verbose mode (`-v`)
   - V4. IPTC reading (`-pi`)
   - V5. XMP reading (`-px`)

3. **Low Priority** (advanced/writing):
   - V18-V21. Writing support
   - V27-V30. Thumbnail/XMP operations
   - V31-V35. Full IPTC/XMP support

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
