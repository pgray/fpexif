// io.rs - File I/O utilities for EXIF data
use crate::errors::{ExifError, ExifResult};
use crate::ExifData;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Check if a file is likely to contain EXIF metadata
///
/// Supported formats:
/// - JPEG (.jpg, .jpeg)
/// - TIFF (.tif, .tiff)
/// - RAF (.raf) - Fujifilm RAW
/// - CR2 (.cr2) - Canon RAW 2
/// - CR3 (.cr3) - Canon RAW 3
/// - NEF (.nef) - Nikon Electronic Format
/// - DNG (.dng) - Adobe Digital Negative
/// - HEIC/HEIF (.heic, .heif)
/// - And other TIFF-based RAW formats (ORF, SRW, RW2, ARW, etc.)
pub fn is_exif_file<P: AsRef<Path>>(path: P) -> ExifResult<bool> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Check for JPEG, TIFF, HEIC, RAF, CR3, etc.
    let mut signature = [0u8; 16];
    let read_bytes = reader.read(&mut signature)?;

    if read_bytes < 4 {
        return Ok(false);
    }

    // Check for RAF signature (FUJIFILMCCD-RAW) - Fujifilm RAW
    if read_bytes >= 15 && &signature[0..15] == b"FUJIFILMCCD-RAW" {
        return Ok(true);
    }

    // Check for JPEG signature (FF D8 FF)
    if signature[0] == 0xFF && signature[1] == 0xD8 && signature[2] == 0xFF {
        return Ok(true);
    }

    // Check for TIFF signature (II or MM) - covers TIFF, CR2, NEF, DNG, and other RAW formats
    // II = little-endian (Intel), MM = big-endian (Motorola)
    if (signature[0] == 0x49 && signature[1] == 0x49)
        || (signature[0] == 0x4D && signature[1] == 0x4D)
    {
        // Verify TIFF magic number at offset 2-3
        let magic = if signature[0] == 0x49 {
            u16::from_le_bytes([signature[2], signature[3]])
        } else {
            u16::from_be_bytes([signature[2], signature[3]])
        };

        // Accept various TIFF magic numbers:
        // 0x002A = standard TIFF (also CR2, NEF, DNG)
        // 0x002B = BigTIFF
        // 0x4F52 = ORF (Olympus)
        // 0x5352 = SRW (Samsung)
        // 0x0055 = RW2 (Panasonic)
        if magic == 0x002A
            || magic == 0x002B
            || magic == 0x4F52
            || magic == 0x5352
            || magic == 0x0055
        {
            return Ok(true);
        }
    }

    // Check for CR3 signature (Canon RAW 3) - ISO Base Media File Format
    // CR3 files start with: [size:4 bytes][ftyp][crx ]
    if read_bytes >= 12 && &signature[4..8] == b"ftyp" && &signature[8..12] == b"crx " {
        return Ok(true);
    }

    // Check for HEIF/HEIC signature (typically starts with 'ftyp' at byte 4)
    if read_bytes >= 12
        && signature[4] == 0x66
        && signature[5] == 0x74
        && signature[6] == 0x79
        && signature[7] == 0x70
    {
        // Further check for HEIC brand
        if (signature[8] == 0x68
            && signature[9] == 0x65
            && signature[10] == 0x69
            && signature[11] == 0x63)
            || (signature[8] == 0x6D
                && signature[9] == 0x69
                && signature[10] == 0x66
                && signature[11] == 0x31)
        {
            return Ok(true);
        }
    }

    // Not recognized as a file type that typically contains EXIF data
    Ok(false)
}

/// Extract raw EXIF data from a JPEG file
pub fn extract_exif_segment<P: AsRef<Path>>(path: P) -> ExifResult<Vec<u8>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Check for JPEG signature
    let mut signature = [0u8; 2];
    reader.read_exact(&mut signature)?;

    if signature[0] != 0xFF || signature[1] != 0xD8 {
        return Err(ExifError::Format("Not a valid JPEG file".to_string()));
    }

    // Find the APP1 marker
    loop {
        let mut marker = [0u8; 2];
        reader.read_exact(&mut marker)?;

        // Check for end of image
        if marker[0] == 0xFF && marker[1] == 0xD9 {
            return Err(ExifError::Format("No EXIF data found".to_string()));
        }

        // Check for APP1 marker
        if marker[0] == 0xFF && marker[1] == 0xE1 {
            // Read segment length
            let mut length_bytes = [0u8; 2];
            reader.read_exact(&mut length_bytes)?;
            let length = u16::from_be_bytes(length_bytes) as usize - 2;

            // Read APP1 data
            let mut data = vec![0u8; length];
            reader.read_exact(&mut data)?;

            // Check for "Exif\0\0" marker
            if data.len() >= 6 && &data[0..6] == b"Exif\0\0" {
                return Ok(data);
            }

            // Not EXIF APP1, continue searching
        } else {
            // Skip this segment
            let mut length_bytes = [0u8; 2];
            reader.read_exact(&mut length_bytes)?;
            let length = u16::from_be_bytes(length_bytes) as usize - 2;
            reader.seek(SeekFrom::Current(length as i64))?;
        }
    }
}

/// Write EXIF data back to a JPEG file
pub fn write_exif<P: AsRef<Path>>(
    exif_data: &ExifData,
    source_path: P,
    dest_path: P,
) -> ExifResult<()> {
    // First, read the source file to memory
    let source_file = File::open(source_path)?;
    let mut reader = BufReader::new(source_file);
    let mut jpeg_data = Vec::new();
    reader.read_to_end(&mut jpeg_data)?;

    // Create destination file
    let mut dest_file = File::create(dest_path)?;

    // Write JPEG header (SOI marker)
    if jpeg_data.len() < 2 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        return Err(ExifError::Format("Not a valid JPEG file".to_string()));
    }
    dest_file.write_all(&[0xFF, 0xD8])?;

    // TODO: Serialize the EXIF data to a binary format
    // This is a complex task that would need to be implemented
    let exif_segment = serialize_exif(exif_data)?;

    // Write the APP1 marker
    dest_file.write_all(&[0xFF, 0xE1])?;

    // Write the segment length (including the length bytes)
    let segment_length = exif_segment.len() + 2;
    dest_file.write_all(&(segment_length as u16).to_be_bytes())?;

    // Write the EXIF data
    dest_file.write_all(&exif_segment)?;

    // Find the start of the original image data (after all metadata segments)
    let mut pos = 2; // Skip SOI marker
    while pos < jpeg_data.len() - 1 {
        if jpeg_data[pos] != 0xFF {
            return Err(ExifError::Format("Invalid JPEG format".to_string()));
        }

        let marker = jpeg_data[pos + 1];
        pos += 2;

        // Check for SOS marker (Start of Scan) which indicates the beginning of image data
        if marker == 0xDA {
            break;
        }

        // Check for EOI marker (End of Image) - shouldn't happen before SOS
        if marker == 0xD9 {
            return Err(ExifError::Format(
                "Unexpected end of image marker".to_string(),
            ));
        }

        // Skip this segment
        if pos + 2 > jpeg_data.len() {
            return Err(ExifError::Format("Truncated JPEG file".to_string()));
        }
        let length = ((jpeg_data[pos] as usize) << 8) | (jpeg_data[pos + 1] as usize);
        pos += length;
    }

    // Write the rest of the JPEG data (all segments after metadata and image data)
    dest_file.write_all(&jpeg_data[pos..])?;

    Ok(())
}

/// Serialize EXIF data to binary format
/// This is a stub function that would need to be implemented
fn serialize_exif(exif_data: &ExifData) -> ExifResult<Vec<u8>> {
    // This is a complex task that would require implementing the TIFF format serialization
    // For now, return a placeholder implementation

    // Start with the "Exif\0\0" marker
    let mut data = b"Exif\0\0".to_vec();

    // Add TIFF header (II for little endian or MM for big endian)
    match exif_data.endian {
        crate::data_types::Endianness::Little => {
            data.extend_from_slice(b"II");
        }
        crate::data_types::Endianness::Big => {
            data.extend_from_slice(b"MM");
        }
    }

    // Add TIFF version (0x002A)
    let version: u16 = 0x002A;
    match exif_data.endian {
        crate::data_types::Endianness::Little => {
            data.extend_from_slice(&version.to_le_bytes());
        }
        crate::data_types::Endianness::Big => {
            data.extend_from_slice(&version.to_be_bytes());
        }
    }

    // TODO: Implement proper TIFF serialization
    // This would involve:
    // 1. Building IFD entries for each tag
    // 2. Arranging the data values after the IFD entries
    // 3. Setting up offsets correctly

    // For now, return this placeholder
    Err(ExifError::Unsupported(
        "Writing EXIF data is not yet implemented".to_string(),
    ))
}
