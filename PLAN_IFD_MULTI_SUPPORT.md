# Plan: Multi-IFD Support for CR2 Files

## Problem Statement

CR2 (Canon Raw 2) files contain multiple Image File Directories (IFDs):
- **IFD0**: Preview/thumbnail JPEG image
- **IFD1**: Full-resolution RAW sensor data
- **IFD2**: Sometimes additional data

Currently, fpexif reads EXIF data but doesn't distinguish between these IFDs, leading to incorrect values for tags like:
- `StripByteCounts`: Expected "9845180" (RAW data), Actual "842753" (preview data)
- `StripOffsets`: Expected "1834108" (RAW data), Actual "46568" (preview data)

ExifTool reports values from the RAW data IFD, while we're reading from the preview IFD.

## Current Architecture

### File: `src/formats/tiff.rs`
- `extract_exif_segment()`: Reads entire TIFF file into memory
- Wraps data in APP1 format ("Exif\0\0" + TIFF data)
- Passes to main parser which only processes first IFD (IFD0)

### File: `src/parser.rs`
- `parse_ifd()`: Parses a single IFD
- Follows NextIFD offset but doesn't expose multiple IFDs separately
- All tags from all IFDs are merged into a single `ExifData` structure

## Design Goals

1. **Preserve existing behavior**: Don't break current functionality
2. **Expose IFD context**: Let users know which IFD each tag came from
3. **Handle CR2 specifics**: Properly identify which IFD contains RAW vs preview data
4. **Maintain exiftool compatibility**: Match exiftool's behavior for tag selection

## Proposed Solution

### Phase 1: Add IFD Tracking to Tags

#### 1.1 Extend `TagId` structure
```rust
// src/tags.rs
pub struct TagId {
    pub id: u16,
    pub ifd: TagGroup,
    pub ifd_index: Option<usize>,  // NEW: Track which IFD (0, 1, 2, etc.)
}
```

#### 1.2 Update parser to track IFD index
```rust
// src/parser.rs
fn parse_ifd(&mut self, ifd_offset: usize, ifd_index: usize) -> ExifResult<()> {
    // Pass ifd_index when creating TagId
    // Store in ExifData with IFD context
}
```

### Phase 2: Multi-IFD Parsing

#### 2.1 Parse all IFDs in chain
```rust
// src/parser.rs
pub fn parse_tiff_with_all_ifds(&mut self) -> ExifResult<Vec<HashMap<TagId, ExifValue>>> {
    let mut ifds = Vec::new();
    let mut current_offset = self.read_u32(4)?;
    let mut ifd_index = 0;

    while current_offset != 0 {
        let ifd_data = self.parse_single_ifd(current_offset, ifd_index)?;
        ifds.push(ifd_data);
        current_offset = self.get_next_ifd_offset(current_offset)?;
        ifd_index += 1;
    }

    Ok(ifds)
}
```

#### 2.2 Identify IFD types for CR2
```rust
// src/formats/tiff.rs or cr2.rs
enum CR2IfdType {
    Preview,      // IFD0 - JPEG preview
    RawData,      // IFD1 - RAW sensor data
    ExifData,     // Embedded Exif IFD
}

fn identify_cr2_ifd_type(ifd_data: &HashMap<TagId, ExifValue>, index: usize) -> CR2IfdType {
    // Check for CR2-specific markers:
    // - IFD0: Contains Compression=6 (JPEG), smaller StripByteCounts
    // - IFD1: Contains Compression=1 (uncompressed), larger StripByteCounts
    // - Look for NewSubFileType tag (0x00FE) to distinguish
}
```

### Phase 3: Smart Tag Selection

#### 3.1 Implement tag priority rules
```rust
// When merging IFDs into single ExifData, use these rules:

// For most tags: Use IFD0 (EXIF standard)
// For image data tags in CR2: Use IFD1 (RAW data)
const RAW_DATA_TAGS: &[u16] = &[
    0x0111, // StripOffsets
    0x0117, // StripByteCounts
    0x0100, // ImageWidth (for RAW)
    0x0101, // ImageLength (for RAW)
];

fn merge_ifds_with_priority(ifds: Vec<HashMap<TagId, ExifValue>>) -> ExifData {
    // For RAW_DATA_TAGS: prefer IFD1 if it exists
    // For all others: prefer IFD0 (standard EXIF behavior)
}
```

### Phase 4: Testing Strategy

#### 4.1 Unit tests
```rust
#[test]
fn test_cr2_multi_ifd_parsing() {
    // Test with real CR2 file
    // Verify IFD0 and IFD1 are both parsed
    // Verify StripByteCounts from IFD1 is selected
}

#[test]
fn test_ifd_type_identification() {
    // Test CR2IfdType detection logic
}
```

#### 4.2 Integration tests
- Add CR2 file to test-data if not already present
- Compare with exiftool output for StripByteCounts/StripOffsets
- Verify other tags remain correct

### Phase 5: Backwards Compatibility

#### 5.1 API considerations
```rust
// Keep existing API working:
impl ExifData {
    // Existing method - works as before
    pub fn get_tag_by_name(&self, name: &str) -> Option<&ExifValue>

    // NEW: Get tag from specific IFD
    pub fn get_tag_from_ifd(&self, name: &str, ifd_index: usize) -> Option<&ExifValue>

    // NEW: Get all IFDs separately
    pub fn get_all_ifds(&self) -> Vec<&HashMap<TagId, ExifValue>>
}
```

## Implementation Order

1. ✅ **Phase 1**: Add IFD tracking (2-3 hours)
   - Update TagId structure
   - Update parser to track IFD index
   - Ensure tests still pass

2. ✅ **Phase 2**: Multi-IFD parsing (3-4 hours)
   - Implement IFD chain traversal
   - Parse all IFDs separately
   - Add CR2 IFD type identification

3. ✅ **Phase 3**: Smart tag selection (2-3 hours)
   - Implement tag priority rules
   - Update merge logic
   - Handle RAW_DATA_TAGS specially

4. ✅ **Phase 4**: Testing (2-3 hours)
   - Add unit tests
   - Add CR2 integration tests
   - Verify exiftool compatibility

5. ✅ **Phase 5**: Documentation (1 hour)
   - Update README with CR2 support details
   - Document multi-IFD API
   - Add examples

**Total Estimated Time**: 10-14 hours

## Risks and Mitigations

### Risk 1: Breaking existing behavior
**Mitigation**: Extensive testing, feature flag for new behavior

### Risk 2: Complex CR2 structure variations
**Mitigation**: Test with multiple CR2 files from different cameras

### Risk 3: Performance impact
**Mitigation**: Only parse multiple IFDs when needed (detect CR2 format)

## Success Criteria

- ✅ StripByteCounts matches exiftool for CR2 files
- ✅ StripOffsets matches exiftool for CR2 files
- ✅ All existing tests continue to pass
- ✅ No performance regression for non-CR2 files
- ✅ Exiftool text comparison tests pass for CR2

## References

- TIFF 6.0 Specification: https://www.adobe.io/open/standards/TIFF.html
- CR2 Format Documentation: https://libopenraw.freedesktop.org/formats/cr2/
- ExifTool CR2 Support: https://exiftool.org/TagNames/Canon.html

## Notes

- CR2 files use TIFF as base format but with Canon-specific extensions
- Some Canon cameras have 3+ IFDs with various purposes
- ExifTool's behavior is: prefer RAW data IFD for image dimension tags
- Preview JPEG is typically much smaller than RAW data
