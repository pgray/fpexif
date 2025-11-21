# Unsupported Tags Analysis

This document analyzes tags that exiftool outputs but fpexif doesn't yet support, based on testing with `test-data/DSCF0062.RAF`.

## Current fpexif Output Summary

fpexif currently outputs ~60 tags for this file, but many show as "Unknown" because we don't have tag definitions for them.

---

## 1. File Metadata Tags (Low Priority)

These are filesystem/container metadata that exiftool calculates, not stored in EXIF.

| Tag | Example Value | Implementation |
|-----|---------------|----------------|
| FileName | DSCF0062.RAF | `Path::file_name()` |
| Directory | test-data | `Path::parent()` |
| FileSize | 33 MB | `fs::metadata().len()` |
| FileModifyDate | 2024:01:15 10:30:00 | `fs::metadata().modified()` |
| FileAccessDate | ... | `fs::metadata().accessed()` |
| FilePermissions | rw-r--r-- | `fs::metadata().permissions()` |
| FileType | RAF | Extension mapping |
| FileTypeExtension | raf | Extension lowercase |
| MIMEType | image/x-fujifilm-raf | MIME type mapping |

**How to implement**: Add a `--file-metadata` flag or include by default. Create a `file_metadata` module that extracts these from `std::fs::Metadata`.

---

## 2. Unknown Standard EXIF Tags (High Priority)

Tags showing as "Unknown" in fpexif output that are standard EXIF tags.

| Tag ID | Name | Current Output |
|--------|------|----------------|
| 0xA401 | CustomRendered | Shows as value only |
| 0xA402 | ExposureMode | Shows as value only |
| 0xA403 | WhiteBalance | Shows as value only |
| 0xA405 | FocalLengthIn35mmFilm | Missing |
| 0xA406 | SceneCaptureType | Shows as value only |
| 0xA407 | GainControl | Missing |
| 0xA408 | Contrast | Missing |
| 0xA409 | Saturation | Missing |
| 0xA40A | Sharpness | Missing |
| 0xA40C | SubjectDistanceRange | Missing |
| 0x9010 | OffsetTime | Missing |
| 0x9011 | OffsetTimeOriginal | Missing |
| 0x9012 | OffsetTimeDigitized | Missing |

**How to implement**: Add these tag definitions to `src/tags.rs` in the `EXIF_TAGS` array:

```rust
(0xA401, "CustomRendered", TagGroup::Exif),
(0xA402, "ExposureMode", TagGroup::Exif),
(0xA403, "WhiteBalance", TagGroup::Exif),
// ... etc
```

---

## 3. Fujifilm Maker Notes (Medium Priority)

The MakerNote field contains 808 bytes of Fujifilm-specific data.

| Tag | Description |
|-----|-------------|
| FujiFilmVersion | Maker note version |
| InternalSerialNumber | Camera serial |
| Quality | Image quality setting |
| Sharpness | Sharpness setting |
| WhiteBalance | WB setting (not EXIF WB) |
| Saturation | Color saturation |
| Contrast | Contrast setting |
| FlashMode | Flash configuration |
| SlowSync | Slow sync setting |
| PictureMode | Scene mode |
| AutoBracketing | Bracketing info |
| BlurWarning | Motion blur detected |
| FocusWarning | Focus issue detected |
| ExposureWarning | Exposure issue |
| DynamicRange | DR mode |
| FilmMode | Film simulation |
| RawImageFullSize | Full sensor size |
| FujiModel | Internal model ID |
| FocusMode | AF mode |
| AFMode | AF point selection |
| FocusPixel | AF point coordinates |

**How to implement**:

1. Create `src/maker_notes/fujifilm.rs`
2. Parse MakerNote structure (typically starts with "FUJIFILM" identifier)
3. Define Fujifilm-specific tag IDs and names
4. Add maker note parsing to the main parser

```rust
// src/maker_notes/fujifilm.rs
pub fn parse_fujifilm_maker_note(data: &[u8]) -> Result<HashMap<String, ExifValue>> {
    // Fujifilm maker notes start with "FUJIFILM" + version (12 bytes header)
    if !data.starts_with(b"FUJIFILM") {
        return Err(ExifError::Format("Not a Fujifilm maker note"));
    }
    // Parse IFD structure starting at offset 12
    // ...
}
```

---

## 4. Derived/Calculated Fields (Low Priority)

Exiftool calculates these from raw EXIF data.

| Tag | Calculation |
|-----|-------------|
| Aperture | f-number formatted as "f/2.8" |
| ShutterSpeed | ExposureTime as "1/15" |
| ISO | ISOSpeedRatings as number |
| LightValue | log2(aperture²/shutter) + log2(ISO/100) |
| FocalLength35efl | FocalLength * crop factor |
| ScaleFactor35efl | Sensor diagonal / 43.27mm |
| CircleOfConfusion | Sensor diagonal / 1500 |
| HyperfocalDistance | f² / (N * c) |
| FOV | 2 * atan(sensor_size / (2 * focal_length)) |
| ImageSize | PixelXDimension x PixelYDimension |
| Megapixels | (X * Y) / 1000000 |

**How to implement**: Add a `derived_values` module:

```rust
pub fn calculate_derived_values(exif: &ExifData) -> HashMap<String, String> {
    let mut derived = HashMap::new();

    if let Some(ExifValue::Rational(exp)) = exif.get_tag_by_name("ExposureTime") {
        let (n, d) = exp[0];
        derived.insert("ShutterSpeed".to_string(), format!("1/{}", d/n));
    }
    // ... etc
}
```

---

## 5. Thumbnail/Preview Data (Medium Priority)

| Tag | Description |
|-----|-------------|
| ThumbnailOffset | Offset to embedded JPEG |
| ThumbnailLength | Size of thumbnail |
| ThumbnailImage | Base64 or binary thumbnail |
| PreviewImage | Larger preview (in RAF files) |

**How to implement**:
- We already parse `JPEGInterchangeFormat` (offset) and `JPEGInterchangeFormatLength`
- Add extraction with `--thumbnail` flag to save embedded JPEG
- For RAF, also parse the RAF-specific preview sections

---

## 6. Interoperability IFD Tags (Low Priority)

| Tag ID | Name |
|--------|------|
| 0x0001 | InteropIndex |
| 0x0002 | InteropVersion |
| 0x1000 | RelatedImageFileFormat |
| 0x1001 | RelatedImageWidth |
| 0x1002 | RelatedImageHeight |

**How to implement**: Already parsing InteroperabilityIFD, just need tag definitions in `src/tags.rs`.

---

## 7. RAF-Specific Metadata (Medium Priority)

RAF files have additional headers outside EXIF.

| Field | Location |
|-------|----------|
| RAFVersion | RAF header bytes 0-4 |
| RawImageFullWidth | RAF directory |
| RawImageFullHeight | RAF directory |
| RawImageCropTop/Left | RAF directory |
| RawImageWidth/Height | RAF directory |
| FujiLayout | Sensor pattern info |

**How to implement**: Parse RAF file structure before EXIF:

```rust
pub fn parse_raf_header(data: &[u8]) -> Result<RafMetadata> {
    // RAF magic: "FUJIFILMCCD-RAW "
    // Version at offset 0x3C (4 bytes)
    // JPEG offset at 0x54
    // JPEG length at 0x58
    // ...
}
```

---

## Priority Implementation Order

1. **High**: Unknown standard EXIF tags (just add to `tags.rs`)
2. **High**: Proper value formatting (enums like CustomRendered, ExposureMode)
3. **Medium**: Fujifilm maker notes (new module)
4. **Medium**: Thumbnail extraction
5. **Low**: Derived fields
6. **Low**: File metadata
7. **Low**: RAF-specific parsing

---

## Quick Wins

These can be fixed immediately by adding to `src/tags.rs`:

```rust
// Add to EXIF_TAGS constant
(0xA401, "CustomRendered", TagGroup::Exif),
(0xA402, "ExposureMode", TagGroup::Exif),
(0xA403, "WhiteBalance", TagGroup::Exif),
(0xA404, "DigitalZoomRatio", TagGroup::Exif),
(0xA405, "FocalLengthIn35mmFilm", TagGroup::Exif),
(0xA406, "SceneCaptureType", TagGroup::Exif),
(0xA407, "GainControl", TagGroup::Exif),
(0xA408, "Contrast", TagGroup::Exif),
(0xA409, "Saturation", TagGroup::Exif),
(0xA40A, "Sharpness", TagGroup::Exif),
(0xA40B, "DeviceSettingDescription", TagGroup::Exif),
(0xA40C, "SubjectDistanceRange", TagGroup::Exif),
(0xA420, "ImageUniqueID", TagGroup::Exif),
(0xA430, "CameraOwnerName", TagGroup::Exif),
(0xA431, "BodySerialNumber", TagGroup::Exif),
(0xA432, "LensSpecification", TagGroup::Exif),
(0xA433, "LensMake", TagGroup::Exif),
(0xA434, "LensModel", TagGroup::Exif),
(0xA435, "LensSerialNumber", TagGroup::Exif),
```
