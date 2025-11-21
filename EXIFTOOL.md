# Exiftool Compatibility Roadmap

This document tracks progress toward matching exiftool's most commonly used features.

## Reading Features

### Implemented

- [x] **JSON output (`-j`)**
  - `fpexif exiftool -j image.jpg`
  - Implementation: Uses `serde_json` to serialize ExifData to exiftool-compatible JSON format.

- [x] **Multiple file processing**
  - `fpexif exiftool -j *.jpg`
  - Implementation: CLI accepts `Vec<PathBuf>` and iterates through all files.

### Core Reading Features

- [ ] **Print all tags (default behavior)**
  - `fpexif exiftool image.jpg`
  - Implementation: Already works, but output format should match exiftool's `Tag Name : Value` format more closely. Update `print_exif_data()` to use exiftool's exact formatting.

- [ ] **Extract specific tag (`-TAG`)**
  - `fpexif exiftool -Make -Model image.jpg`
  - Implementation: Add `-T` or `--tag` flag that filters output to only specified tags. Parse tag names from args prefixed with `-`.

- [ ] **Group filtering (`-g`, `-G`)**
  - `fpexif exiftool -g image.jpg` (organize by group)
  - `fpexif exiftool -G image.jpg` (show group prefix)
  - Implementation: Already have `TagGroup` enum. Add flags to control output grouping and add group prefixes like `[EXIF]`, `[GPS]`, etc.

- [ ] **Short output (`-s`, `-S`)**
  - `fpexif exiftool -s image.jpg` (tag names without spaces)
  - `fpexif exiftool -S image.jpg` (very short, tag=value)
  - Implementation: Add formatting options enum and modify output based on selected format.

- [ ] **Numeric values (`-n`)**
  - `fpexif exiftool -n image.jpg`
  - Implementation: Skip human-readable descriptions and output raw numeric values. Add a `raw_values: bool` to output config.

- [ ] **Binary output (`-b`)**
  - `fpexif exiftool -b -ThumbnailImage image.jpg > thumb.jpg`
  - Implementation: For Undefined/binary tags, output raw bytes to stdout. Useful for extracting embedded thumbnails.

- [ ] **Coordinates format (`-c`)**
  - `fpexif exiftool -c "%.6f" image.jpg`
  - Implementation: Add GPS coordinate formatting with configurable precision. Parse format string for degrees/decimal conversion.

- [ ] **Date format (`-d`)**
  - `fpexif exiftool -d "%Y-%m-%d" image.jpg`
  - Implementation: Parse EXIF date strings and reformat using strftime-style format strings.

- [ ] **Duplicates (`-a`)**
  - `fpexif exiftool -a image.jpg`
  - Implementation: Show duplicate tags (same tag ID in different IFDs). Currently we overwrite duplicates.

- [ ] **Unknown tags (`-u`, `-U`)**
  - `fpexif exiftool -u image.jpg`
  - Implementation: Include tags we don't have names for, showing as hex IDs.

### Output Format Features

- [ ] **CSV output (`-csv`)**
  - `fpexif exiftool -csv *.jpg`
  - Implementation: Add CSV serialization with headers as tag names, one row per file.

- [ ] **Tab-separated (`-t`)**
  - `fpexif exiftool -t image.jpg`
  - Implementation: Simple tab-delimited output format.

- [ ] **XML output (`-X`)**
  - `fpexif exiftool -X image.jpg`
  - Implementation: Add optional `quick-xml` dependency for XML serialization.

- [ ] **HTML output (`-h`)**
  - `fpexif exiftool -h image.jpg`
  - Implementation: Generate HTML table of tags. Low priority.

- [ ] **PHP output (`-php`)**
  - `fpexif exiftool -php image.jpg`
  - Implementation: Generate PHP array syntax. Low priority.

### File Selection Features

- [ ] **Recursive directory processing (`-r`)**
  - `fpexif exiftool -r -j ./photos/`
  - Implementation: Use `walkdir` crate to recursively find image files in directories.

- [ ] **File extension filter (`-ext`)**
  - `fpexif exiftool -ext jpg -ext png ./photos/`
  - Implementation: Filter files by extension during directory traversal.

- [ ] **Condition filtering (`-if`)**
  - `fpexif exiftool -if '$Make eq "Canon"' *.jpg`
  - Implementation: Add simple expression parser for tag-based filtering. Complex feature.

## Writing Features

### Basic Writing

- [ ] **Set tag value (`-TAG=VALUE`)**
  - `fpexif exiftool -Artist="John Doe" image.jpg`
  - Implementation: Requires EXIF writing support in `io.rs`. Parse assignment syntax, locate tag in IFD, update value, rewrite file.

- [ ] **Remove tag (`-TAG=`)**
  - `fpexif exiftool -Artist= image.jpg`
  - Implementation: Same as above but remove the tag entry from IFD.

- [ ] **Copy tags from another file (`-TagsFromFile`)**
  - `fpexif exiftool -TagsFromFile src.jpg dst.jpg`
  - Implementation: Parse source file, extract specified tags, write to destination.

- [ ] **Remove all metadata (`-all=`)**
  - `fpexif exiftool -all= image.jpg`
  - Implementation: Strip entire APP1 segment from JPEG or equivalent for other formats.

### Writing Safety

- [ ] **Backup originals (`-overwrite_original_in_place`)**
  - `fpexif exiftool -overwrite_original_in_place image.jpg`
  - Implementation: Create backup before modifying, or modify in-place without backup.

- [ ] **Preserve modification time (`-P`)**
  - `fpexif exiftool -P -Artist="John" image.jpg`
  - Implementation: Save file mtime before write, restore after.

## Advanced Features

### Maker Notes

- [ ] **Full maker note parsing**
  - Currently partial support for Canon, Nikon, Sony
  - Implementation: Expand maker note parsers for more camera brands (Fuji, Olympus, Panasonic, etc.)

- [ ] **Maker note writing**
  - Implementation: Very complex due to proprietary formats. Low priority.

### Geolocation

- [ ] **Geotagging (`-geotag`)**
  - `fpexif exiftool -geotag track.gpx *.jpg`
  - Implementation: Parse GPX files, match timestamps to photos, write GPS coordinates. Requires GPX parser.

- [ ] **Reverse geocoding**
  - Implementation: Call external API to convert GPS coordinates to place names. Would require network access.

### Special Operations

- [ ] **Rename files (`-filename`)**
  - `fpexif exiftool '-filename<DateTimeOriginal' -d %Y%m%d_%H%M%S.%%e *.jpg`
  - Implementation: Template-based renaming using tag values. Parse template syntax, extract values, rename files.

- [ ] **JSON to tags (`-json=`)**
  - `fpexif exiftool -json=metadata.json image.jpg`
  - Implementation: Read JSON file, map keys to tag names, write values.

- [ ] **Execute command (`-execute`)**
  - Implementation: Process multiple command sets in one invocation. Batch operation support.

## Testing Checklist

For each implemented feature, we should have:

- [ ] Unit tests for the parsing/formatting logic
- [ ] Integration test comparing output with exiftool
- [ ] Test with multiple file formats (JPEG, TIFF, RAW variants)
- [ ] Edge case handling (missing tags, corrupt data, etc.)

### CI Test Improvements Needed

- [ ] Add direct JSON output comparison test for `fpexif exiftool -j`
- [ ] Test multiple file output matches exiftool array format
- [ ] Validate tag name compatibility (e.g., "FNumber" vs "F-Number")
- [ ] Test numeric value precision matches exiftool

## Priority Order

1. **High Priority** (commonly used, relatively easy):
   - Extract specific tags (`-TAG`)
   - Numeric values (`-n`)
   - Group display (`-g`, `-G`)
   - Recursive processing (`-r`)
   - Short output formats (`-s`, `-S`)

2. **Medium Priority** (useful, moderate complexity):
   - CSV output (`-csv`)
   - Date formatting (`-d`)
   - GPS coordinate formatting (`-c`)
   - Tag filtering (`-if` simple cases)

3. **Low Priority** (complex or rarely needed):
   - Writing features (requires significant new code)
   - Geotagging
   - File renaming
   - XML/HTML/PHP output

## Contributing

When implementing a feature:

1. Check this box when starting work
2. Add the CLI argument parsing in `cli.rs`
3. Implement the logic (often in `output.rs` for formatting)
4. Add tests comparing with exiftool output
5. Update this document with completion status
