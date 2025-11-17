# Maker Notes Implementation

## Overview

Maker notes are manufacturer-specific EXIF data embedded in images by camera manufacturers. They contain proprietary information beyond standard EXIF tags, including camera settings, lens information, and image processing parameters.

## Current Implementation Status

### ✅ Implemented

**Infrastructure (100%)**
- `src/makernotes/mod.rs` - Core parsing framework
- `src/makernotes/canon.rs` - Canon tag definitions (70+ tags)
- `src/makernotes/nikon.rs` - Nikon tag definitions (40+ tags)
- `src/makernotes/sony.rs` - Sony tag definitions (60+ tags)
- Integration with `ExifData` and JSON output
- Automatic manufacturer detection and parser selection

**What Works Now:**
- Detects maker notes presence (EXIF tag 0x927C)
- Identifies camera manufacturer from Make tag
- Routes to manufacturer-specific parser
- Extracts IFD structure and tag IDs
- Outputs tag names in JSON (e.g., "CanonLensInfo", "NikonVRInfo")

### ⚠️ Partially Implemented

**Tag Value Parsing (30%)**
- Currently returns placeholder values
- Tag structure is parsed but binary data within tags is not decoded
- Complex nested structures not yet interpreted

### ❌ Not Yet Implemented

1. **Full Binary Value Decoding**
   - Most tags show placeholder values instead of actual parsed data
   - Nested arrays and sub-structures need decoding

2. **Human-Readable Interpretations**
   - Numeric values need descriptive strings (like standard EXIF tags)
   - Bit flags and enumerations need interpretation

3. **Additional Manufacturers**
   - Fujifilm
   - Olympus
   - Pentax
   - Panasonic
   - Leica
   - Sigma

---

## Maker Notes Format Details

### General Structure

Maker notes are stored in EXIF tag 0x927C (MakerNote) with data type UNDEFINED. The format varies by manufacturer but generally follows one of these patterns:

#### Type 1: Standard TIFF IFD Format
```
[Entry Count: 2 bytes]
[IFD Entries: 12 bytes each]
  - Tag ID: 2 bytes
  - Data Type: 2 bytes (1=BYTE, 2=ASCII, 3=SHORT, 4=LONG, etc.)
  - Count: 4 bytes
  - Value/Offset: 4 bytes
[Next IFD Offset: 4 bytes]
[Data Area]
```

#### Type 2: Header + TIFF IFD
```
[Manufacturer Header: variable]
[TIFF Header: 8 bytes] (II/MM + 0x002A + IFD offset)
[IFD Entries...]
```

### Manufacturer-Specific Formats

#### Canon
- **Format**: Standard TIFF IFD
- **Endianness**: Same as main EXIF (usually little-endian)
- **Special Features**:
  - Many tags contain arrays of values (camera settings arrays)
  - Sub-IFDs for complex data structures
  - Some tags have model-specific interpretations

**Key Tags to Implement:**
```rust
// Priority 1: Most useful tags
0x0001 CameraSettings - Array of camera parameters
0x0002 FocalLength - Focal length info
0x0004 ShotInfo - Shot-specific data
0x0006 ImageType - Image type string
0x0007 FirmwareVersion - Camera firmware
0x0009 OwnerName - Camera owner
0x000C SerialNumber - Camera serial number
0x0095 LensModel - Lens identification
0x4019 LensInfo - Detailed lens data (min/max focal length, aperture)

// Priority 2: Image quality
0x00A9 ColorBalance - Color balance data
0x4003 ColorInfo - Color information
0x4008 PictureStyleUserDef - Picture style settings
0x4015 VignettingCorr - Vignetting correction
0x4018 LightingOpt - Auto lighting optimizer

// Priority 3: Advanced features
0x0012 AFInfo - Autofocus information
0x0026 AFInfo2 - Extended AF data
0x003C AFInfo3 - Latest AF data
0x4013 AFMicroAdj - AF micro adjustment
```

**Example: Canon LensInfo (0x4019)**
```
Offset Size Description
0      2    Lens ID number
2      2    Min focal length
4      2    Max focal length
6      2    Max aperture at min focal
8      2    Max aperture at max focal
```

#### Nikon
- **Format**: "Nikon\0" header + TIFF header + IFD
- **Endianness**: Usually big-endian (Motorola)
- **Header**: `Nikon\0` (6 bytes) + TIFF header (4 bytes, version 0x0100)
- **Special Features**:
  - Encrypted data in some models (Type 3/4 maker notes)
  - Version-dependent tag availability

**Key Tags to Implement:**
```rust
// Priority 1: Camera identification
0x0001 MakerNoteVersion - Version info
0x001D SerialNumber - Camera serial (varies by model)
0x0098 LensType - Lens type code
0x0099 Lens - Lens specifications (4 rationals: min/max focal, min/max aperture)

// Priority 2: Shooting parameters
0x0005 WhiteBalance - WB mode
0x0006 Sharpness - Sharpness setting
0x0007 FocusMode - Focus mode (AF-S, AF-C, Manual)
0x0022 ActiveDLighting - Active D-Lighting setting
0x0023 PictureControlData - Picture control settings

// Priority 3: Advanced
0x001F VRInfo - VR (Vibration Reduction) information
0x0083 LensData - Detailed lens data
0x0091 ShotInfo - Shot information (varies by model)
0x00A8 FlashInfo - Flash information
```

**Example: Nikon Lens (0x0099)**
```
4 Rational values:
[0] Min focal length (mm)
[1] Max focal length (mm)
[2] Max aperture at min focal (f-number)
[3] Max aperture at max focal (f-number)
```

#### Sony
- **Format**: "SONY DSC " header (12 bytes) OR standard TIFF IFD
- **Endianness**: Little-endian
- **Special Features**:
  - Large tag ID range (0x0010 - 0xB054)
  - Many boolean flags
  - Minolta heritage (some tags inherited from Minolta)

**Key Tags to Implement:**
```rust
// Priority 1: Camera settings
0x0114 CameraSettings - Camera settings array
0xB001 SonyModelID - Specific model ID
0xB027 LensID - Lens identification code
0xB02A LensSpec - Lens specifications

// Priority 2: Image processing
0xB025 DynamicRangeOptimizer - DRO setting
0xB026 ImageStabilization - Stabilization on/off
0x2009 HighISONoiseReduction - NR setting
0x2008 LongExposureNoiseReduction - Long exposure NR
0x200B MultiFrameNoiseReduction - Multi-frame NR

// Priority 3: Creative
0xB020 CreativeStyle - Creative style mode
0x200E PictureEffect - Picture effect applied
0xB023 SceneMode - Scene mode selection
0x2011 VignettingCorrection - Vignetting correction
0x2013 DistortionCorrection - Distortion correction
```

---

## Implementation Roadmap

### Phase 1: Core Value Parsing (High Priority)

**Goal**: Parse actual values from maker note tags instead of placeholders

**Tasks**:
1. Implement value extraction for each data type:
   - ASCII strings (already works for Make tag)
   - SHORT arrays
   - LONG arrays
   - RATIONAL arrays
   - Nested IFDs

2. Update `canon.rs`, `nikon.rs`, `sony.rs`:
   ```rust
   // Instead of:
   let value = ExifValue::Short(vec![0]);

   // Do:
   let value = parse_tag_value(data, value_offset, tag_type, count, endian)?;
   ```

3. Add helper function to `makernotes/mod.rs`:
   ```rust
   pub fn parse_tag_value(
       data: &[u8],
       offset: usize,
       tag_type: u16,
       count: u32,
       endian: Endianness,
   ) -> Result<ExifValue, ExifError> {
       // Decode based on type (1-12)
       match tag_type {
           1 => parse_byte_array(data, offset, count),
           2 => parse_ascii_string(data, offset, count),
           3 => parse_short_array(data, offset, count, endian),
           4 => parse_long_array(data, offset, count, endian),
           5 => parse_rational_array(data, offset, count, endian),
           // ... etc
       }
   }
   ```

**Files to Modify**:
- `src/makernotes/mod.rs` - Add value parsing helpers
- `src/makernotes/canon.rs` - Use helpers in `parse_canon_maker_notes()`
- `src/makernotes/nikon.rs` - Use helpers in `parse_nikon_maker_notes()`
- `src/makernotes/sony.rs` - Use helpers in `parse_sony_maker_notes()`

### Phase 2: Human-Readable Interpretations (Medium Priority)

**Goal**: Add descriptive text for maker note values (like EXIF tag descriptions)

**Tasks**:
1. Add interpretation functions per manufacturer:
   ```rust
   // In canon.rs
   pub fn get_canon_lens_type_description(lens_id: u16) -> &'static str {
       match lens_id {
           1 => "Canon EF 50mm f/1.8",
           2 => "Canon EF 28mm f/2.8",
           // ... hundreds more
           _ => "Unknown lens"
       }
   }

   pub fn get_canon_image_quality_description(value: u16) -> &'static str {
       match value {
           2 => "Normal",
           3 => "Fine",
           4 => "RAW",
           5 => "SuperFine",
           _ => "Unknown"
       }
   }
   ```

2. Update JSON formatter in `cli.rs` to use interpretations

**Resources Needed**:
- Lens databases (Canon, Nikon, Sony lens IDs)
- Enumeration tables for each setting
- Reference: ExifTool's tag databases

### Phase 3: Additional Manufacturers (Lower Priority)

**Manufacturers to Add**:
1. **Fujifilm** - RAF format, Film Simulation modes
2. **Olympus** - ORF format, Art Filters
3. **Pentax** - PEF format, Custom Image modes
4. **Panasonic** - RW2 format, Photo Style
5. **Leica** - DNG/RWL format
6. **Sigma** - Foveon sensor info

**Template for New Manufacturer**:
```rust
// src/makernotes/fujifilm.rs
pub const FUJI_VERSION: u16 = 0x0000;
pub const FUJI_SERIAL_NUMBER: u16 = 0x0010;
pub const FUJI_FILM_MODE: u16 = 0x1001;
pub const FUJI_FILM_SIMULATION: u16 = 0x1401;
// ... more tags

pub fn get_fuji_tag_name(tag_id: u16) -> Option<&'static str> { /*...*/ }
pub fn parse_fujifilm_maker_notes(/*...*/) -> Result</*..*/> { /*...*/ }
```

### Phase 4: Advanced Features

1. **Nested Structure Parsing**
   - Canon's camera settings arrays (tag 0x0001)
   - Sub-IFDs within maker notes

2. **Encrypted Maker Notes**
   - Some Nikon models encrypt portions of maker notes
   - Would need decryption algorithms (may have legal concerns)

3. **Version-Specific Parsing**
   - Different camera models use different tag interpretations
   - Need model detection and version tables

4. **Maker Note Modification**
   - Write support for maker notes (advanced feature)
   - Maintain manufacturer signatures and checksums

---

## Testing Strategy

### Unit Tests Needed

```rust
#[test]
fn test_parse_canon_lens_info() {
    let sample_data = [...]; // Binary lens info data
    let result = parse_canon_maker_notes(&sample_data, Endianness::Little);

    assert!(result.is_ok());
    let notes = result.unwrap();

    // Check lens info tag exists
    assert!(notes.contains_key(&0x4019));

    // Verify parsed values
    if let Some(tag) = notes.get(&0x4019) {
        // Should have lens focal length, aperture, etc.
    }
}
```

### Integration Tests

Use real camera files from test suite:
- Canon CR2/CR3 files
- Nikon NEF files
- Sony ARW files

Compare output with exiftool:
```bash
exiftool -json -G1 sample.cr2 > exiftool_output.json
fpexif list sample.cr2 --json > fpexif_output.json
# Compare maker note tags
```

---

## References and Resources

### ExifTool Tag Databases
- Canon: https://exiftool.org/TagNames/Canon.html
- Nikon: https://exiftool.org/TagNames/Nikon.html
- Sony: https://exiftool.org/TagNames/Sony.html

### EXIF Specifications
- EXIF 2.3 Standard: https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf
- TIFF 6.0 Spec: https://www.adobe.io/open/standards/TIFF.html

### Open Source Implementations
- ExifTool (Perl): https://github.com/exiftool/exiftool
- libexif (C): https://github.com/libexif/libexif
- exiv2 (C++): https://github.com/Exiv2/exiv2

### Lens Databases
- Canon: https://github.com/exiftool/exiftool/blob/master/lib/Image/ExifTool/Canon.pm
- Nikon: https://photographylife.com/nikon-lens-codes
- Sony: https://github.com/lclevy/sony_maker_notes

---

## Code Examples

### Example 1: Parsing Canon LensInfo (Full Implementation)

```rust
// In canon.rs
pub fn parse_canon_lens_info(data: &[u8], endian: Endianness) -> Option<LensInfo> {
    if data.len() < 10 {
        return None;
    }

    let lens_id = read_u16(&data[0..2], endian);
    let min_focal = read_u16(&data[2..4], endian);
    let max_focal = read_u16(&data[4..6], endian);
    let max_aperture_min = read_u16(&data[6..8], endian);
    let max_aperture_max = read_u16(&data[8..10], endian);

    Some(LensInfo {
        id: lens_id,
        focal_range: (min_focal, max_focal),
        aperture_range: (max_aperture_min, max_aperture_max),
    })
}

struct LensInfo {
    id: u16,
    focal_range: (u16, u16),
    aperture_range: (u16, u16),
}
```

### Example 2: Adding Human-Readable Output

```rust
// In cli.rs format_exif_value_for_json()
match tag_id {
    // Canon maker notes
    0x0006 if maker == "Canon" => {
        // ImageType - format as string
        if let ExifValue::Ascii(s) = value {
            Value::String(s.clone())
        } else {
            default_value
        }
    }
    0x0095 if maker == "Canon" => {
        // LensModel
        if let ExifValue::Ascii(s) = value {
            Value::String(s.clone())
        } else {
            default_value
        }
    }
    // ... more maker-specific formatting
}
```

---

## Performance Considerations

1. **Lazy Parsing**: Only parse maker notes when explicitly requested
2. **Caching**: Cache parsed values to avoid re-parsing
3. **Memory**: Maker notes can be large (100KB+), consider streaming
4. **Validation**: Some maker notes may be corrupted, need error handling

---

## Known Issues and Limitations

1. **Encrypted Data**: Some Nikon Type 3/4 maker notes are encrypted
2. **Model-Specific**: Tag meanings vary by camera model
3. **Undocumented Tags**: Many tags are reverse-engineered, not official
4. **Checksums**: Some manufacturers include checksums that we don't validate
5. **Write Support**: Currently read-only, writing may invalidate signatures

---

## Contributing

When adding support for new tags:

1. **Research**: Find tag ID and format from ExifTool or other sources
2. **Define**: Add constant to appropriate manufacturer file
3. **Parse**: Implement value parsing if complex
4. **Interpret**: Add description function if enumerated
5. **Test**: Add test with sample data
6. **Document**: Update this file with tag details

**Pull requests welcome!**
