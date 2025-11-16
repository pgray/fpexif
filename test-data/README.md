# Test Data

This directory contains sample image files used for integration testing.

## Current Files

- `DSCF0062.RAF` - Fujifilm RAF format (33.8 MB)

## Freely Available Test Images

We can add more test files from these sources:

### 1. **RawSamples.ch** (Public Domain RAW Files)
- Website: https://www.rawsamples.ch/
- License: Public domain / CC0
- Formats available:
  - CRW (Canon Raw v1)
  - CR2 (Canon Raw 2)
  - NEF (Nikon)
  - ARW (Sony)
  - DNG (Adobe)
  - ORF (Olympus)
  - PEF (Pentax)
  - And many more

### 2. **Sample Files from Phil Harvey's ExifTool**
- Website: https://exiftool.org/sample_images.html
- Various formats with known EXIF data
- Good for validation testing

### 3. **DPReview Sample Galleries**
- Website: https://www.dpreview.com/
- Sample images from camera reviews
- Check individual licenses

### 4. **Adobe DNG SDK Samples**
- Website: https://helpx.adobe.com/camera-raw/digital-negative.html
- DNG format samples
- Free to use for testing

### 5. **Unsplash / Pixabay** (for modern web formats)
- JPEG, PNG, WebP samples
- Creative Commons / Free license
- AVIF/HEIC samples may be limited

## Recommended Test Files to Add

For comprehensive format coverage, we should add:

### Modern Web Formats
- [ ] PNG with EXIF (small sample, <1 MB)
- [ ] WebP with EXIF (small sample, <1 MB)
- [ ] AVIF with EXIF (if available)
- [ ] HEIC/HEIF with EXIF (iPhone sample)
- [ ] JPEG XL with EXIF (if available)

### RAW Formats
- [ ] CRW (Canon Raw v1) - from RawSamples.ch
- [ ] CR2 (Canon Raw 2) - from RawSamples.ch
- [ ] CR3 (Canon Raw 3) - newer Canon cameras
- [ ] NEF (Nikon) - from RawSamples.ch
- [ ] ARW (Sony) - from RawSamples.ch
- [ ] DNG (Adobe) - from Adobe or RawSamples.ch
- [ ] ORF (Olympus) - from RawSamples.ch
- [ ] PEF (Pentax) - from RawSamples.ch
- [ ] MRW (Minolta) - from RawSamples.ch (if available)
- [ ] X3F (Sigma) - from RawSamples.ch (if available)

### Size Considerations
- Keep total test-data directory under 100 MB if possible
- Prioritize smaller samples (cropped images are fine for EXIF testing)
- EXIF data is in the header, so file size doesn't matter for our tests

## Adding New Test Files

1. Download sample file from a verified free source
2. Verify the license allows redistribution
3. Add to this directory
4. Update tests in `tests/real_file_test.rs`
5. Document source and license here

## License Information

Each test file should be accompanied by license information:
- Source URL
- License type (Public Domain, CC0, etc.)
- Date downloaded
- Original photographer/creator (if known)

## Current File Details

### DSCF0062.RAF
- **Format**: Fujifilm RAF
- **Source**: Unknown (already in repository)
- **Size**: 33.8 MB
- **Camera**: Fujifilm (detected from file)
