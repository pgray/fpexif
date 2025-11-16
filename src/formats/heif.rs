// formats/heif.rs - HEIF/HEIC format EXIF extraction
// HEIF (High Efficiency Image Format) uses ISO Base Media File Format with HEVC compression
use crate::errors::{ExifError, ExifResult};
use crate::formats::isobmff::{find_exif_data, IsobmffParser};
use std::io::{Read, Seek};

/// Extract EXIF data from HEIF/HEIC file
/// HEIF files use ISO Base Media File Format with "heic", "heix", "hevc", "hevx", "mif1" brands
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Parse as ISO Base Media File Format
    let mut parser = IsobmffParser::new(&mut reader);

    // Find ftyp box
    let ftyp_box = parser.find_box(b"ftyp")?;
    match ftyp_box {
        Some(ftyp) => {
            // Read ftyp data to check brand
            let ftyp_data = parser.read_box_data(&ftyp)?;

            if ftyp_data.len() < 4 {
                return Err(ExifError::Format("Invalid ftyp box".to_string()));
            }

            // Check major brand (first 4 bytes of ftyp data)
            let major_brand = &ftyp_data[0..4];

            // HEIF/HEIC brands
            let heif_brands: &[&[u8]] = &[
                b"heic", b"heix", b"hevc", b"hevx", b"heim", b"heis", b"hevm", b"hevs", b"mif1",
                b"msf1",
            ];

            let mut is_heif = heif_brands.contains(&major_brand);

            // If major brand doesn't match, check compatible brands
            if !is_heif {
                for i in (4..ftyp_data.len()).step_by(4) {
                    if i + 4 <= ftyp_data.len() {
                        let brand = &ftyp_data[i..i + 4];
                        if heif_brands.contains(&brand) {
                            is_heif = true;
                            break;
                        }
                    }
                }
            }

            if !is_heif {
                return Err(ExifError::Format(
                    "Not a valid HEIF/HEIC file (wrong brand)".to_string(),
                ));
            }
        }
        None => {
            return Err(ExifError::Format("No ftyp box found".to_string()));
        }
    }

    // For HEIF/HEIC, EXIF data is stored in meta box
    // Similar structure to AVIF
    find_exif_data(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_heif_valid_brand() {
        // Create a minimal ISO BMFF file with heic brand
        let mut data = Vec::new();

        // ftyp box
        data.extend_from_slice(&16u32.to_be_bytes()); // size
        data.extend_from_slice(b"ftyp"); // type
        data.extend_from_slice(b"heic"); // valid brand
        data.extend_from_slice(b"    "); // padding

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        // Will fail because there's no EXIF data, but should not fail on brand check
        if let Err(ExifError::Format(msg)) = result {
            assert!(!msg.contains("wrong brand"));
        }
    }

    #[test]
    fn test_heif_invalid_brand() {
        // Create a minimal ISO BMFF file with wrong brand
        let mut data = Vec::new();

        // ftyp box
        data.extend_from_slice(&16u32.to_be_bytes()); // size
        data.extend_from_slice(b"ftyp"); // type
        data.extend_from_slice(b"test"); // wrong brand
        data.extend_from_slice(b"    "); // padding

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }
}
