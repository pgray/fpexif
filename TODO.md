# fpexif - Image Format Support TODO

This document tracks planned image format support additions for the fpexif library.

## Currently Supported Formats ✅

- [x] JPEG (.jpg, .jpeg)
- [x] TIFF (.tif, .tiff)
- [x] RAF (.raf) - Fujifilm RAW
- [x] CR2 (.cr2) - Canon RAW 2
- [x] CR3 (.cr3) - Canon RAW 3
- [x] NEF (.nef) - Nikon Electronic Format
- [x] DNG (.dng) - Adobe Digital Negative
- [x] ORF (.orf) - Olympus RAW Format
- [x] SRW (.srw) - Samsung RAW Format
- [x] RW2 (.rw2) - Panasonic RAW Format
- [x] ARW (.arw) - Sony Alpha RAW (via TIFF handler, implicit)
- [x] HEIC/HEIF (.heic, .heif) - Detection only (no extraction)

---

## High Priority - Very Common Formats ⭐⭐⭐

### 1. ARW - Sony Alpha RAW (Explicit Support)
- **Status**: ❌ Not started (currently works via TIFF handler)
- **Priority**: HIGH
- **Complexity**: LOW (already works, just needs explicit detection)
- **Market Share**: Very high (Sony mirrorless cameras dominate professional/enthusiast market)
- **Technical Details**:
  - TIFF-based format with Sony-specific maker notes
  - Magic number: 0x002A (standard TIFF)
  - Can add specific ARW detection to provide better error messages
  - Should detect Sony-specific IFD tags
- **Implementation**: Add explicit format detection in `formats/tiff.rs::detect_tiff_format()`
- **Estimated Effort**: 1-2 hours

### 2. PEF - Pentax Electronic File
- **Status**: ❌ Not started (currently works via TIFF handler)
- **Priority**: HIGH
- **Complexity**: LOW (TIFF-based)
- **Market Share**: Medium (Pentax DSLR users)
- **Technical Details**:
  - TIFF-based RAW format
  - Magic number: 0x002A
  - Pentax-specific maker notes in tag 0x927C
  - Can detect via Pentax camera make in IFD0
- **Implementation**: Add explicit format detection
- **Estimated Effort**: 1-2 hours

### 3. CRW - Canon Raw v1
- **Status**: ❌ Not started
- **Priority**: HIGH
- **Complexity**: HIGH (uses CIFF format, NOT TIFF-based)
- **Market Share**: Low (legacy format, 2000-2004)
- **Technical Details**:
  - Uses CIFF (Camera Image File Format), not TIFF
  - Signature: "HEAPCCDR" at offset 6
  - Different structure from TIFF-based formats
  - Requires new parser implementation
  - Still important for archival/legacy images
- **Implementation**:
  - Create `src/formats/ciff.rs`
  - Implement CIFF heap structure parser
  - Extract EXIF from CIFF records
- **Estimated Effort**: 8-12 hours

---

## Medium Priority - Modern Web Formats ⭐⭐

### 4. AVIF - AV1 Image File Format
- **Status**: ❌ Not started
- **Priority**: MEDIUM-HIGH
- **Complexity**: MEDIUM-HIGH
- **Market Share**: Growing rapidly (modern web format)
- **Technical Details**:
  - Based on AV1 video codec
  - Uses ISO Base Media File Format (like CR3)
  - Signature: ftyp box with "avif" or "avis" brand
  - EXIF stored in metadata boxes
  - Supported by Chrome, Firefox, Safari
- **Implementation**:
  - Can reuse CR3 ISO Base Media parsing logic
  - Create `src/formats/avif.rs`
  - Search for EXIF in meta/iinf/iloc boxes
- **Estimated Effort**: 6-10 hours

### 5. WebP - Google WebP Format
- **Status**: ❌ Not started
- **Priority**: MEDIUM-HIGH
- **Complexity**: MEDIUM
- **Market Share**: High (widespread web use)
- **Technical Details**:
  - Uses RIFF container format
  - Signature: "RIFF" + size + "WEBP"
  - EXIF stored in "EXIF" chunk
  - VP8/VP8L/VP8X variants
- **Implementation**:
  - Create `src/formats/riff.rs` for RIFF container parsing
  - Create `src/formats/webp.rs`
  - Parse RIFF chunks to find EXIF chunk
- **Estimated Effort**: 6-8 hours

### 6. JPEG XL - Next Generation JPEG
- **Status**: ❌ Not started
- **Priority**: MEDIUM
- **Complexity**: MEDIUM-HIGH
- **Market Share**: Low (still gaining adoption)
- **Technical Details**:
  - Signature: 0xFF 0x0A or "JXL " box
  - Can contain EXIF in metadata boxes
  - Two container types: naked codestream and ISO BMFF
  - Still limited browser/software support
- **Implementation**:
  - Create `src/formats/jxl.rs`
  - Handle both container types
  - Parse metadata boxes for EXIF
- **Estimated Effort**: 8-12 hours

---

## Medium Priority - Additional RAW Formats ⭐⭐

### 7. NRW - Nikon RAW (Coolpix)
- **Status**: ❌ Not started (likely works via TIFF handler)
- **Priority**: MEDIUM
- **Complexity**: LOW (TIFF-based)
- **Market Share**: Low (Nikon compact cameras)
- **Technical Details**:
  - TIFF-based format for Nikon Coolpix cameras
  - Similar to NEF but for compact cameras
  - Magic number: 0x002A
- **Implementation**: Add explicit detection
- **Estimated Effort**: 1-2 hours

### 8. RWL - Leica RAW
- **Status**: ❌ Not started (likely works as DNG)
- **Priority**: MEDIUM
- **Complexity**: LOW (DNG variant)
- **Market Share**: Low (Leica cameras)
- **Technical Details**:
  - TIFF-based DNG variant
  - Used by Leica digital cameras
  - Should already work via TIFF/DNG handler
- **Implementation**: Add explicit detection and validation
- **Estimated Effort**: 1-2 hours

### 9. MRW - Minolta RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (proprietary structure)
- **Market Share**: Very low (legacy Minolta/early Sony cameras)
- **Technical Details**:
  - Signature: 0x00 0x4D 0x52 0x4D (MRM in little-endian)
  - Proprietary structure with TTW blocks
  - EXIF in PRD (parameter data) block
  - Historical importance only
- **Implementation**:
  - Create `src/formats/mrw.rs`
  - Parse MRW header and blocks
  - Extract TIFF data from PRD block
- **Estimated Effort**: 6-8 hours

### 10. X3F - Sigma/Foveon
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM-HIGH (proprietary structure)
- **Market Share**: Very low (Sigma cameras with Foveon sensor)
- **Technical Details**:
  - Signature: "FOVb" at start
  - Proprietary format for unique Foveon sensor
  - Contains property list with EXIF-like data
  - Small but dedicated user base
- **Implementation**:
  - Create `src/formats/x3f.rs`
  - Parse X3F directory structure
  - Extract property list and EXIF data
- **Estimated Effort**: 8-12 hours

---

## Low Priority - Specialized Formats ⭐

### 11. 3FR - Hasselblad RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (TIFF-based with custom structure)
- **Market Share**: Very low (medium format cameras)
- **Technical Details**:
  - Used by Hasselblad digital backs
  - TIFF-based with Hasselblad extensions
  - May already work via TIFF handler
- **Estimated Effort**: 2-4 hours

### 12. FFF - Imacon RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM
- **Market Share**: Very low (legacy medium format)
- **Estimated Effort**: 4-6 hours

### 13. DCR - Kodak RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (TIFF-based)
- **Market Share**: Very low (legacy Kodak cameras)
- **Estimated Effort**: 2-4 hours

### 14. KDC - Kodak Digital Camera
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (TIFF-based)
- **Market Share**: Very low (legacy)
- **Estimated Effort**: 2-4 hours

### 15. MEF - Mamiya RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (TIFF-based)
- **Market Share**: Very low (medium format)
- **Estimated Effort**: 2-4 hours

### 16. MOS - Leaf RAW
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: MEDIUM (TIFF-based)
- **Market Share**: Very low (medium format)
- **Estimated Effort**: 2-4 hours

### 17. RAW - Panasonic/Leica (extension clash)
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: LOW-MEDIUM
- **Market Share**: Very low
- **Note**: Generic .raw extension used by some Panasonic/Leica cameras
- **Estimated Effort**: 2-4 hours

---

## Other Potential Enhancements

### HEIC/HEIF Full Support
- **Status**: ❌ Currently detection only
- **Priority**: MEDIUM-HIGH
- **Complexity**: HIGH
- **Technical Details**:
  - Uses ISO Base Media File Format
  - HEVC (H.265) image compression
  - EXIF in metadata boxes
  - Complex box structure
- **Implementation**:
  - Extend CR3 ISO Base Media parsing
  - Handle HEIC-specific box types
  - Extract EXIF from meta/iinf/iloc boxes
- **Estimated Effort**: 10-15 hours

### PNG EXIF Support
- **Status**: ❌ Not started
- **Priority**: MEDIUM
- **Complexity**: LOW-MEDIUM
- **Technical Details**:
  - PNG can contain EXIF in eXIf chunk (PNG 1.5+)
  - Signature: 0x89 0x50 0x4E 0x47
  - Parse PNG chunks to find eXIf
  - Less common than JPEG but widely supported
- **Implementation**:
  - Create `src/formats/png.rs`
  - Parse PNG chunk structure
  - Extract EXIF from eXIf chunk
- **Estimated Effort**: 4-6 hours

### GIF EXIF Support
- **Status**: ❌ Not started
- **Priority**: LOW
- **Complexity**: LOW
- **Technical Details**:
  - GIF89a can contain EXIF in application extension
  - Very rare in practice
  - Limited usefulness
- **Estimated Effort**: 2-4 hours

---

## Non-EXIF Metadata Support (Future Consideration)

### XMP Sidecar Files
- Extract and parse .xmp files alongside images
- XML-based metadata format
- Very common in professional workflows

### IPTC Metadata
- Parse IPTC/IIM metadata from JPEG/TIFF
- Photojournalism standard
- Often coexists with EXIF

### ICC Color Profiles
- Extract embedded color profiles
- Important for color-managed workflows

---

## Implementation Strategy

### Phase 1: Quick Wins (1-2 weeks)
1. ARW explicit support
2. PEF explicit support
3. NRW support
4. RWL support
5. PNG EXIF support

### Phase 2: Modern Web Formats (2-3 weeks)
1. WebP support (high priority)
2. AVIF support (high priority)
3. HEIC/HEIF full extraction
4. JPEG XL support

### Phase 3: Legacy RAW Formats (2-3 weeks)
1. CRW support (important for archives)
2. MRW support
3. X3F support

### Phase 4: Specialized/Medium Format (ongoing)
1. 3FR, FFF, DCR, KDC, MEF, MOS, etc.
2. As requested by users

---

## Testing Requirements

For each new format, we need:
- [ ] Sample test files (at least 2-3 per format)
- [ ] Unit tests for format detection
- [ ] Unit tests for EXIF extraction
- [ ] Integration tests with real-world images
- [ ] Documentation updates
- [ ] README updates with supported formats list

---

## Documentation Tasks

- [ ] Update README.md with complete format support list
- [ ] Add format support matrix (format vs. feature)
- [ ] Create CONTRIBUTING.md with instructions for adding new formats
- [ ] Document format detection logic
- [ ] Add examples for each format type

---

## Performance Optimization

- [ ] Benchmark large RAW file parsing
- [ ] Optimize memory usage for streaming large files
- [ ] Add optional lazy loading for large files
- [ ] Profile hot paths in TIFF parsing
- [ ] Consider parallel processing for batch operations

---

## Compatibility & Standards

- [ ] Ensure EXIF 2.32 spec compliance
- [ ] Test with libexiv2 output for validation
- [ ] Test with ExifTool output for validation
- [ ] Cross-platform testing (Windows, macOS, Linux)
- [ ] Big-endian system testing

---

**Last Updated**: 2025-11-16
**Maintained By**: fpexif contributors
