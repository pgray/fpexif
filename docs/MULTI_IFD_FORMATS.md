# Multi-IFD Formats in fpexif

## Overview

Many RAW image formats use the TIFF structure, which supports multiple IFDs (Image File Directories). Each IFD can contain a different image or metadata set. Understanding which formats use multiple IFDs is crucial for properly extracting EXIF data.

## TIFF-Based Formats with Multiple IFDs

### ✅ Confirmed Multi-IFD Formats

These formats are TIFF-based and typically contain multiple IFDs:

#### 1. **CR2 (Canon RAW 2)** 📸
- **Magic**: `0x002A` (standard TIFF) + `CR` marker at offset 8-9
- **IFD Structure**:
  - **IFD0**: Preview JPEG (640x480 or similar)
  - **IFD1**: Full-resolution RAW sensor data
  - **IFD2**: Sometimes thumbnail or additional data
  - **Exif Sub-IFD**: EXIF metadata
- **Issue**: StripByteCounts/StripOffsets differ between preview (IFD0) and RAW (IFD1)
- **Priority**: 🔴 HIGH - Known mismatch with exiftool

#### 2. **NEF (Nikon Electronic Format)** 📸
- **Magic**: `0x002A` (standard TIFF)
- **Make**: "NIKON"
- **IFD Structure**:
  - **IFD0**: Preview JPEG or TIFF
  - **IFD1**: Full-resolution RAW data
  - **Exif Sub-IFD**: EXIF metadata
  - **Nikon MakerNote Sub-IFD**: Nikon-specific data
- **Priority**: 🟡 MEDIUM - Likely similar issues to CR2

#### 3. **DNG (Adobe Digital Negative)** 📸
- **Magic**: `0x002A` (standard TIFF)
- **Marker**: DNGVersion tag (0xC612)
- **IFD Structure**:
  - **IFD0**: Full-resolution or reduced-resolution RAW
  - **IFD1**: Preview JPEG or TIFF
  - **IFD2**: Thumbnail
  - **Sub-IFDs**: Various (EXIF, GPS, etc.)
- **Note**: DNG is highly flexible; structure varies by camera/converter
- **Priority**: 🟡 MEDIUM - Industry standard format

#### 4. **ARW (Sony Alpha RAW)** 📸
- **Magic**: `0x002A` (standard TIFF)
- **Make**: "SONY"
- **IFD Structure**:
  - **IFD0**: Preview or RAW data (varies by model)
  - **IFD1**: Thumbnail or additional image
  - **Exif Sub-IFD**: EXIF metadata
  - **Sony MakerNote Sub-IFD**: Sony-specific data
- **Priority**: 🟡 MEDIUM

#### 5. **PEF (Pentax Electronic File)** 📸
- **Magic**: `0x002A` (standard TIFF)
- **Make**: "PENTAX" or "ASAHI"
- **IFD Structure**:
  - **IFD0**: Preview JPEG
  - **IFD1**: Full-resolution RAW
  - **Exif Sub-IFD**: EXIF metadata
- **Priority**: 🟢 LOW - Less commonly used

#### 6. **ORF (Olympus RAW Format)** 📸
- **Magic**: `0x4F52` (custom - "OR" in ASCII)
- **IFD Structure**:
  - **IFD0**: Thumbnail
  - **IFD1**: Preview or RAW
  - **Exif Sub-IFD**: EXIF metadata
- **Note**: Uses different TIFF magic number
- **Priority**: 🟡 MEDIUM

#### 7. **SRW (Samsung RAW)** 📸
- **Magic**: `0x5352` (custom - "SR" in ASCII)
- **IFD Structure**:
  - **IFD0**: Preview
  - **IFD1**: RAW data
- **Priority**: 🟢 LOW - Samsung stopped making cameras

#### 8. **RW2 (Panasonic RAW)** 📸
- **Magic**: `0x0055` (custom)
- **IFD Structure**:
  - **IFD0**: Preview
  - **IFD1**: RAW data
  - **Exif Sub-IFD**: EXIF metadata
- **Priority**: 🟡 MEDIUM

### 📊 Standard TIFF Files

- **Magic**: `0x002A`
- **IFD Structure**:
  - **IFD0**: Main image
  - **IFD1+**: Additional images (multi-page TIFF, thumbnails)
- **Note**: Standard TIFF can have unlimited IFDs
- **Priority**: 🟡 MEDIUM - Common format

## Non-Multi-IFD Formats

These formats do NOT use multiple IFDs:

### ❌ Single-IFD or Non-TIFF Formats

1. **RAF (Fujifilm RAW)** - Custom format, embeds JPEG with EXIF
2. **CRW (Canon RAW)** - Uses CIFF format, not TIFF
3. **CR3 (Canon RAW 3)** - Uses ISO Base Media (MP4-like), not TIFF
4. **MRW (Minolta RAW)** - Custom format with MRM signature
5. **X3F (Sigma RAW)** - Custom format with FOVb signature
6. **JPEG** - Uses APP1 segments, not IFDs
7. **PNG** - Uses eXIf chunk, not IFDs
8. **WebP** - Uses EXIF chunk, not IFDs
9. **HEIC/AVIF** - Uses ISO Base Media format, not TIFF

## Impact of Multi-IFD Structure

### Current Behavior
fpexif currently:
1. Parses only the first IFD (IFD0)
2. Follows Sub-IFD pointers (Exif, GPS, etc.)
3. Merges all tags into single `ExifData` structure

### Problems
1. **Incorrect values** for image dimension tags (using preview instead of RAW)
2. **Missing data** from secondary IFDs
3. **Incompatibility** with exiftool for certain tags

### Affected Tags

Tags that differ between IFDs:

| Tag | Description | IFD0 (Preview) | IFD1 (RAW) |
|-----|-------------|----------------|------------|
| `StripOffsets` (0x0111) | Image data offset | Preview offset | RAW offset |
| `StripByteCounts` (0x0117) | Image data size | Small (~800KB) | Large (~10MB) |
| `ImageWidth` (0x0100) | Image width | Preview width | RAW width |
| `ImageLength` (0x0101) | Image height | Preview height | RAW height |
| `RowsPerStrip` (0x0116) | Rows per strip | Preview rows | RAW rows |
| `BitsPerSample` (0x0102) | Bits per sample | 8 bits | 12-14 bits |
| `Compression` (0x0103) | Compression type | JPEG (6) | Uncompressed (1) |

## Solution Approach

See [`PLAN_IFD_MULTI_SUPPORT.md`](../PLAN_IFD_MULTI_SUPPORT.md) for implementation plan.

### Priority Order

1. 🔴 **CR2** - Known issues, widely used
2. 🟡 **NEF** - Widely used, likely similar issues
3. 🟡 **DNG** - Industry standard
4. 🟡 **ARW** - Sony cameras popular
5. 🟡 **ORF** - Custom magic, different structure
6. 🟡 **RW2** - Panasonic cameras
7. 🟡 **Standard TIFF** - Multi-page support
8. 🟢 **PEF** - Less common
9. 🟢 **SRW** - Discontinued

## References

- [TIFF 6.0 Specification](https://www.adobe.io/open/standards/TIFF.html)
- [DNG Specification](https://helpx.adobe.com/camera-raw/digital-negative.html)
- [LibRaw Format Documentation](https://www.libraw.org/docs/)
- [ExifTool Tag Names](https://exiftool.org/TagNames/)

## Testing Matrix

| Format | Sample File | IFD Count | Test Status |
|--------|-------------|-----------|-------------|
| CR2 | ✅ Yes | 2-3 | ⚠️ Mismatches |
| NEF | ❓ Need sample | 2+ | ❓ Untested |
| DNG | ❓ Need sample | 2-3 | ❓ Untested |
| ARW | ❓ Need sample | 2+ | ❓ Untested |
| ORF | ❓ Need sample | 2+ | ❓ Untested |
| RW2 | ❓ Need sample | 2+ | ❓ Untested |
| PEF | ❓ Need sample | 2+ | ❓ Untested |
| SRW | ❓ Need sample | 2+ | ❓ Untested |
| TIFF | ✅ Yes | 1+ | ✅ Working |

---

**Last Updated**: 2025-12-13
**Status**: Documentation - Implementation Pending
