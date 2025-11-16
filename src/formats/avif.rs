// formats/avif.rs - AVIF format EXIF extraction
// AVIF uses ISO Base Media File Format with AV1 image codec
use crate::errors::{ExifError, ExifResult};
use crate::formats::isobmff::{find_exif_data, IsobmffParser};
use std::io::{Read, Seek};

/// Extract EXIF data from AVIF file
/// AVIF files use ISO Base Media File Format with "avif" or "avis" brand
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

            // AVIF brands: "avif", "avis", "avio"
            if major_brand != b"avif" && major_brand != b"avis" && major_brand != b"avio" {
                // Check compatible brands (rest of ftyp data)
                let mut is_avif = false;
                for i in (4..ftyp_data.len()).step_by(4) {
                    if i + 4 <= ftyp_data.len() {
                        let brand = &ftyp_data[i..i + 4];
                        if brand == b"avif" || brand == b"avis" || brand == b"avio" {
                            is_avif = true;
                            break;
                        }
                    }
                }

                if !is_avif {
                    return Err(ExifError::Format(
                        "Not a valid AVIF file (wrong brand)".to_string(),
                    ));
                }
            }
        }
        None => {
            return Err(ExifError::Format("No ftyp box found".to_string()));
        }
    }

    // For AVIF, EXIF data is typically stored in:
    // - meta box -> iinf box -> item references
    // - Exif item with mime type "application/exif"
    // For simplicity, we'll use the generic search approach
    find_exif_data(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_avif_invalid_ftyp() {
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
