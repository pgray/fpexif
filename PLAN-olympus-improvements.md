# Olympus EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 41.3%
- Files tested: 36 ORF
- Matching: 2,777 | Mismatched: 1,988 | Missing: 1,966 | Extra: 5,405

**Note:** High mismatch count due to StripOffsets/StripByteCounts array formatting

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| StripOffsets | ~36 | Array vs binary format | ExifTool shows "(Binary data N bytes)" |
| StripByteCounts | ~36 | Array vs binary format | ExifTool shows "(Binary data N bytes)" |
| DigitalZoomRatio | ~10 | "inf" vs "undef" | Handle infinity/undefined |
| MeteringMode | ~8 | "ESP" vs "Multi-segment" | Map Olympus-specific name |
| Compression | ~5 | Wrong value | Check JPEG vs Uncompressed detection |

## Implementation Steps

### Phase 1: Formatting Fixes (High Impact)
1. **StripOffsets/StripByteCounts** - When array is large, format as "(Binary data N bytes)"
2. **DigitalZoomRatio** - Return "undef" instead of "inf" for undefined values
3. **MeteringMode** - Map "ESP" to "Multi-segment" for ExifTool compatibility

### Phase 2: Compression Fix
1. **Compression** - Verify detection logic for JPEG vs Uncompressed

### Phase 3: Missing Tags
Many Olympus-specific tags in nested IFDs:
- Equipment IFD (lens data)
- Camera Settings IFD
- Raw Development IFD
- Image Processing IFD
- Focus Info IFD

## Reference Files
- `src/makernotes/olympus.rs` - Main Olympus MakerNote parser
- `exiftool/lib/Image/ExifTool/Olympus.pm` - ExifTool Olympus module
- `exiv2/src/olympusmn_int.cpp` - exiv2 Olympus implementation

## Olympus IFD Structure
Olympus uses nested IFDs in MakerNotes:
```
MakerNote IFD
├── Equipment IFD (0x2010)
│   ├── LensType
│   ├── LensSerialNumber
│   └── ...
├── CameraSettings IFD (0x2020)
│   ├── ExposureMode
│   ├── MeteringMode
│   └── ...
├── RawDevelopment IFD (0x2030)
├── ImageProcessing IFD (0x2040)
└── FocusInfo IFD (0x2050)
```

## Testing
```bash
./bin/mfr-test olympus --save-baseline
# Make changes
./bin/mfr-test olympus --check
```
