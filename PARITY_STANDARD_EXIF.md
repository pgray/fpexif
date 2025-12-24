# Fuji X30 Parity - Missing Standard EXIF Tags

## IFD0/IFD1 Tags

| Tag | Example Value | EXIF Tag ID | Notes |
|-----|---------------|-------------|-------|
| BitsPerSample | 12 | 0x0102 | Bits per component |
| Compression | JPEG (old-style) | 0x0103 | Already have decode, may need output |
| ImageWidth | 2048 | 0x0100 | Already parsed, may need alias |
| ImageHeight | 1536 | 0x0101 | Already parsed, may need alias |
| YCbCrSubSampling | YCbCr4:2:2 (2 1) | 0x0212 | Subsampling ratio |
| StripOffsets | 1085952 | 0x0111 | Offset to image data |
| StripByteCounts | 18505728 | 0x0117 | Size of image data |

## ExifIFD Tags

| Tag | Example Value | EXIF Tag ID | Notes |
|-----|---------------|-------------|-------|
| ExifImageWidth | 2048 | 0xA002 | PixelXDimension - already parsed |
| ExifImageHeight | 1536 | 0xA003 | PixelYDimension - already parsed |
| ColorComponents | 3 | N/A | Derived from JPEG SOF |

## Interoperability IFD Tags

| Tag | Example Value | EXIF Tag ID | Notes |
|-----|---------------|-------------|-------|
| InteropIndex | R98 - DCF basic file (sRGB) | 0x0001 | Interop type |
| InteropVersion | 0100 | 0x0002 | Interop version |

## Thumbnail Tags

| Tag | Example Value | EXIF Tag ID | Notes |
|-----|---------------|-------------|-------|
| ThumbnailOffset | 2016 | 0x0201 | JPEGInterchangeFormat - already output |
| ThumbnailLength | 8964 | 0x0202 | JPEGInterchangeFormatLength - already output |

## Metadata Tags

| Tag | Example Value | Notes |
|-----|---------------|-------|
| ExifByteOrder | Little-endian (Intel, II) | From TIFF header, need to output |
| EncodingProcess | Baseline DCT, Huffman coding | JPEG SOF marker type |
| PrintIMVersion | 0250 | PrintIM tag 0xC4A5 |

## MakerNote Tags

| Tag | Example Value | Notes |
|-----|---------------|-------|
| NumFaceElements | 0 | Fuji MakerNote tag 0x4100 |

## Aliases

| Alias | Source Tag | Notes |
|-------|------------|-------|
| ShutterSpeed | ShutterSpeedValue (0x9201) | APEX to fraction conversion |
| ExposureCompensation | ExposureBiasValue (0x9204) | Same value, different name |

## Implementation Priority

### High Priority
1. InteropIndex/InteropVersion - parse Interop IFD
2. ExifByteOrder - output from parser
3. ExposureCompensation alias
4. ShutterSpeed alias

### Medium Priority
5. BitsPerSample, YCbCrSubSampling
6. StripOffsets, StripByteCounts
7. NumFaceElements - MakerNote tag 0x4100

### Low Priority
8. EncodingProcess - requires JPEG SOF parsing
9. PrintIMVersion - PrintIM tag parsing
10. ColorComponents - JPEG SOF parsing
