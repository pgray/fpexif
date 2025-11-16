// formats/riff.rs - RIFF (Resource Interchange File Format) container parser
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

// RIFF signature: "RIFF"
const RIFF_SIGNATURE: [u8; 4] = *b"RIFF";

/// Represents a RIFF chunk
#[derive(Debug)]
pub struct RiffChunk {
    pub fourcc: [u8; 4],
    pub size: u32,
    pub data_offset: u64,
}

/// RIFF container parser
pub struct RiffParser<R: Read + Seek> {
    reader: R,
    #[allow(dead_code)]
    file_size: u32,
    form_type: [u8; 4],
}

impl<R: Read + Seek> RiffParser<R> {
    /// Create a new RIFF parser and validate the RIFF header
    pub fn new(mut reader: R) -> ExifResult<Self> {
        // Read RIFF header: "RIFF" + file_size + form_type
        let mut header = [0u8; 12];
        reader.read_exact(&mut header)?;

        // Verify RIFF signature
        if header[0..4] != RIFF_SIGNATURE {
            return Err(ExifError::Format("Not a valid RIFF file".to_string()));
        }

        // Read file size (little-endian)
        let file_size = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);

        // Read form type (e.g., "WEBP", "AVI ", "WAVE")
        let mut form_type = [0u8; 4];
        form_type.copy_from_slice(&header[8..12]);

        Ok(Self {
            reader,
            file_size,
            form_type,
        })
    }

    /// Get the form type (e.g., "WEBP")
    pub fn form_type(&self) -> &[u8; 4] {
        &self.form_type
    }

    /// Find a chunk with the specified FourCC
    pub fn find_chunk(&mut self, target_fourcc: &[u8; 4]) -> ExifResult<Option<RiffChunk>> {
        // Reset to start of chunks (after RIFF header)
        self.reader.seek(SeekFrom::Start(12))?;

        loop {
            // Read chunk header: fourcc + size
            let mut chunk_header = [0u8; 8];
            match self.reader.read_exact(&mut chunk_header) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of file reached
                    return Ok(None);
                }
                Err(e) => return Err(e.into()),
            }

            let mut fourcc = [0u8; 4];
            fourcc.copy_from_slice(&chunk_header[0..4]);

            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]);

            // Get current position (start of chunk data)
            let data_offset = self.reader.stream_position()?;

            // Check if this is the chunk we're looking for
            if &fourcc == target_fourcc {
                return Ok(Some(RiffChunk {
                    fourcc,
                    size: chunk_size,
                    data_offset,
                }));
            }

            // Skip to next chunk (chunks are padded to even byte boundaries)
            let skip_size = if chunk_size % 2 == 0 {
                chunk_size as i64
            } else {
                chunk_size as i64 + 1
            };
            self.reader.seek(SeekFrom::Current(skip_size))?;
        }
    }

    /// Read the data from a chunk
    pub fn read_chunk_data(&mut self, chunk: &RiffChunk) -> ExifResult<Vec<u8>> {
        // Seek to chunk data
        self.reader.seek(SeekFrom::Start(chunk.data_offset))?;

        // Read chunk data
        let mut data = vec![0u8; chunk.size as usize];
        self.reader.read_exact(&mut data)?;

        Ok(data)
    }

    /// Get the inner reader back
    pub fn into_inner(self) -> R {
        self.reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_riff_header() {
        // Create a minimal RIFF file
        let mut riff_data = Vec::new();
        riff_data.extend_from_slice(b"RIFF"); // Signature
        riff_data.extend_from_slice(&20u32.to_le_bytes()); // File size
        riff_data.extend_from_slice(b"TEST"); // Form type

        let cursor = Cursor::new(riff_data);
        let parser = RiffParser::new(cursor).unwrap();

        assert_eq!(parser.form_type(), b"TEST");
    }

    #[test]
    fn test_invalid_riff() {
        let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
        let cursor = Cursor::new(invalid_data);
        let result = RiffParser::new(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }
}
