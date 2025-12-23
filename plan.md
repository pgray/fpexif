# fpexif Tag Coverage Improvement Plan

## Current State Analysis

### Test Results Summary (2024-12-22)

Ran `exiftool_json` comparison tests against `/fpexif/raws` directory.

#### Files Tested by Format

| Format | Files | Manufacturer | MakerNote Status |
|--------|-------|--------------|------------------|
| CR2 | 54 | Canon | ✅ Implemented |
| NEF | 47 | Nikon | ✅ Implemented |
| ORF | 36 | Olympus | ✅ Implemented |
| ARW | 31 | Sony | ✅ Implemented |
| RAF | 30 | Fuji | ✅ Implemented |
| RW2 | 20 | Panasonic | ✅ Implemented |
| CRW | 18 | Canon (legacy) | ✅ Implemented |
| PEF | 17 | Pentax | ❌ Missing |
| DNG | 13 | Various | N/A (standard TIFF/EXIF) |
| MRW | 8 | Minolta | ❌ Missing |
| KDC | 5 | Kodak | ❌ Missing |
| NRW | 4 | Nikon Coolpix | ✅ Uses Nikon |
| 3FR | 3 | Hasselblad | ❌ Missing |
| MOS | 2 | Leaf | ❌ Missing |
| DCR | 1 | Kodak Pro | ❌ Missing |
| ERF | 1 | Epson | ❌ Missing |
| MEF | 1 | Mamiya | ❌ Missing |

#### Difference Counts by Severity

- **High (>200 differences)**: Kodak DCR (461), many Sony ARW, Canon CR2
- **Medium (100-200)**: Most NEF, ORF, RAF files
- **Low (<100)**: Some DNG, simpler camera files

---

## Issue Categories

### 1. Value Formatting Issues

These affect ALL manufacturers and represent the highest-impact fixes.

| Field | Count | Current Output | Expected Output |
|-------|-------|----------------|-----------------|
| SerialNumber | 56 | `"51001710"` | `51001710` |
| SubSecTime | 47 | `"90"` | `90` |
| SubSecTimeOriginal | 47 | `"90"` | `90` |
| SubSecTimeDigitized | 46 | `"90"` | `90` |
| ReferenceBlackWhite | 53 | `[0.0,255.0,0.0,128.0,0.0,128.0]` | `"0 255 0 128 0 128"` |
| RawImageCenter | 45 | `[2304,1540]` | `"2304 1540"` |
| ActiveArea | varies | `[18,96,3062,4168]` | `"18 96 3062 4168"` |

**Root Cause**: JSON output formatter wrapping values in quotes or using array notation instead of space-separated strings.

### 2. Rational Number Formatting

| Field | Count | Current Output | Expected Output |
|-------|-------|----------------|-----------------|
| FlashExposureComp | 107 | `"00 01 06 00"` | `0` |
| LensFStops | 45 | `"48 01 0c 00"` | `6.0` |
| ProgramShift | 45 | `"00 01 06 00"` | `0` |
| ExposureDifference | 45 | `"00 01 0c 00"` | `0` |
| FlashExposureBracketValue | varies | `"00 01 06 00"` | `0.0` |

**Root Cause**: Raw bytes being output instead of parsed RATIONAL values.

### 3. Enum Decode Mismatches

| Field | Count | Current Output | Expected Output | Notes |
|-------|-------|----------------|-----------------|-------|
| MeteringMode | 83 | `"Multi-segment"` | `"Evaluative"` | Canon-specific name |
| WhiteBalance | 52 | `"Auto"` | `"Auto1"` / `"Auto2"` | Nikon variants |
| AFAreaMode | 63 | `"Unknown"` | `"Single-point AF"` | Missing decode |
| Compression | 41 | `"JPEG (old-style)"` | `"Nikon NEF Compressed"` | Format-specific |
| NEFCompression | varies | `"Unknown"` | `"Packed 12 bits"` | Missing decode |

**Root Cause**: Decode functions missing mappings or using wrong reference (exiv2 vs exiftool).

### 4. CFAPattern Issues

| Field | Count | Issue |
|-------|-------|-------|
| CFAPattern | 78 | `[Blue,Red][Blue,Red]` vs `[Red,Green][Green,Blue]` |

**Root Cause**: CFA pattern bytes being misinterpreted or 2x2 pattern not being read correctly.

### 5. Missing MakerNote Fields (Top 20)

| Field | Missing Count | Category |
|-------|---------------|----------|
| BlueBalance | 138 | Color Balance |
| AFAreaMode | 90 | AF Info |
| AFPoint | 81 | AF Info |
| CircleOfConfusion | 71 | Calculated |
| AFPointsInFocus | 70 | AF Info |
| BitsPerSample | 67 | Image Info |
| AELock | 66 | Exposure |
| AFImageWidth | 65 | AF Info |
| AFImageHeight | 65 | AF Info |
| ExifByteOrder | 58 | Meta |
| BlackLevel | 48 | Sensor |
| CFAPattern2 | 47 | Image Info |
| Compression | 41 | Image Info |
| AFFineTuneAdj | 40 | AF Info |
| AFFineTune | 40 | AF Info |
| AFAperture | 40 | AF Info |
| AFAssistBeam | 36 | AF Info |
| AFPointSelected | 34 | AF Info |
| ColorComponents | 33 | Image Info |
| ActiveD-Lighting | 31 | Processing |

---

## Implementation Plan

### Phase 1: Value Formatting Fixes (High Impact)

**Goal**: Fix output formatting to match ExifTool conventions.

#### 1.1 String Value Quote Handling
- [ ] Remove extra quotes from SerialNumber output
- [ ] Remove extra quotes from SubSecTime fields
- [ ] Review all string field outputs for unnecessary quoting

#### 1.2 Array to Space-Separated String
- [ ] Fix ReferenceBlackWhite formatting
- [ ] Fix RawImageCenter formatting
- [ ] Fix ActiveArea formatting
- [ ] Fix RetouchHistory formatting (`[0,0,0,...]` → `"None"`)

#### 1.3 Rational Number Parsing
- [ ] Fix FlashExposureComp to parse as rational and output decimal
- [ ] Fix LensFStops parsing
- [ ] Fix ProgramShift parsing
- [ ] Fix ExposureDifference parsing
- [ ] Audit all RATIONAL type fields for proper parsing

**Files to modify**:
- `src/types.rs` - ExifValue formatting
- `src/parser.rs` - Rational parsing
- `src/output/json.rs` - JSON output formatting

---

### Phase 2: Enum Decode Fixes (Medium Impact)

**Goal**: Align decoded values with ExifTool output.

#### 2.1 MeteringMode
- [ ] Add Canon-specific "Evaluative" mapping (vs generic "Multi-segment")
- [ ] Review all manufacturer-specific metering mode names

#### 2.2 WhiteBalance
- [ ] Add Nikon "Auto1", "Auto2" variants
- [ ] Add Sony white balance variants
- [ ] Review all WB decode functions

#### 2.3 AFAreaMode
- [ ] Add missing Canon AFAreaMode values
- [ ] Add missing Nikon AFAreaMode values
- [ ] Add missing Sony AFAreaMode values

#### 2.4 Compression
- [ ] Add Nikon NEF compression types
- [ ] Add manufacturer-specific compression names

**Files to modify**:
- `src/makernotes/canon.rs`
- `src/makernotes/nikon.rs`
- `src/makernotes/sony.rs`
- `src/tags.rs` - Standard EXIF tag decodes

---

### Phase 3: CFAPattern Fix

**Goal**: Correctly parse and display CFA patterns.

- [ ] Review CFAPattern parsing logic
- [ ] Ensure 2x2 pattern is read in correct order
- [ ] Match ExifTool output format: `[Red,Green][Green,Blue]`

**Files to modify**:
- `src/parser.rs` - CFA pattern parsing
- `src/tags.rs` - CFA pattern display

---

### Phase 4: New Manufacturer MakerNote Support

#### 4.1 Pentax MakerNotes (17 PEF files)

**Reference files**:
- ExifTool: `exiftool/lib/Image/ExifTool/Pentax.pm`
- exiv2: `exiv2/src/pentaxmn_int.cpp`

**Priority tags**:
- [ ] FocusMode
- [ ] WhiteBalance
- [ ] MeteringMode
- [ ] Quality
- [ ] LensType
- [ ] AFPoint
- [ ] DriveMode
- [ ] PictureMode

**Files to create/modify**:
- `src/makernotes/pentax.rs` (new)
- `src/makernotes/mod.rs`
- `src/parser.rs`

#### 4.2 Minolta MakerNotes (8 MRW files)

**Reference files**:
- ExifTool: `exiftool/lib/Image/ExifTool/Minolta.pm`
- exiv2: `exiv2/src/minoltamn_int.cpp`

**Priority tags**:
- [ ] FocusMode
- [ ] WhiteBalance
- [ ] ExposureMode
- [ ] Quality
- [ ] LensID
- [ ] ColorMode

**Files to create/modify**:
- `src/makernotes/minolta.rs` (new)
- `src/makernotes/mod.rs`
- `src/parser.rs`

#### 4.3 Kodak MakerNotes (6 KDC/DCR files)

**Reference files**:
- ExifTool: `exiftool/lib/Image/ExifTool/Kodak.pm`

**Priority tags**:
- [ ] WhiteBalance
- [ ] ImageWidth/Height
- [ ] Quality
- [ ] FlashMode
- [ ] FocusMode

**Files to create/modify**:
- `src/makernotes/kodak.rs` (new)
- `src/makernotes/mod.rs`
- `src/parser.rs`

---

### Phase 5: Missing Field Coverage

#### 5.1 AF Information Fields
- [ ] AFPoint - All manufacturers
- [ ] AFPointsInFocus - All manufacturers
- [ ] AFAreaMode - All manufacturers
- [ ] AFFineTune/AFFineTuneAdj - Nikon, Canon
- [ ] AFImageWidth/AFImageHeight - Canon

#### 5.2 Color Balance Fields
- [ ] BlueBalance - Olympus, Nikon
- [ ] BlackLevel - All manufacturers
- [ ] BlackLevelRed/Green/Blue - Canon, Nikon

#### 5.3 Processing Fields
- [ ] ActiveD-Lighting - Nikon
- [ ] DynamicRangeOptimizer - Sony
- [ ] HighlightTonePriority - Canon

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Average differences per CR2 | ~180 | <50 |
| Average differences per NEF | ~160 | <50 |
| Average differences per ARW | ~200 | <50 |
| Average differences per PEF | ~149 | <80 |
| Files with 0 critical differences | 0% | >50% |

---

## Testing Commands

```bash
# Run full comparison
FPEXIF_TEST_FILES=/fpexif/raws cargo test exiftool_json -- --nocapture

# Test specific format
FPEXIF_TEST_FILES=/fpexif/raws cargo test exiftool_json_compatibility_cr2 -- --nocapture

# Count total differences
FPEXIF_TEST_FILES=/fpexif/raws cargo test exiftool_json -- --nocapture 2>&1 | grep -aE "Found.*differences:" | wc -l

# Top missing fields
FPEXIF_TEST_FILES=/fpexif/raws cargo test exiftool_json -- --nocapture 2>&1 | grep -aE "Missing field" | sed 's/.*: //' | sort | uniq -c | sort -rn | head -20
```

---

## Notes

- ExifTool is the primary reference for output formatting
- exiv2 provides additional validation for makernote structures
- Some "missing" fields are calculated by ExifTool (CircleOfConfusion, DOF, FOV) and may not need implementation
- Binary data fields (ContrastCurve, StripOffsets, StripByteCounts) may intentionally differ
