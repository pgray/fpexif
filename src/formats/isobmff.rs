// formats/isobmff.rs - ISO Base Media File Format parser
// Used by CR3, AVIF, HEIF/HEIC, and other formats
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Represents an ISO Base Media File Format box
#[derive(Debug, Clone)]
pub struct Box {
    pub box_type: [u8; 4],
    pub size: u64,
    pub offset: u64,
    pub header_size: u64,
}

impl Box {
    /// Get the data offset (after the box header)
    pub fn data_offset(&self) -> u64 {
        self.offset + self.header_size
    }

    /// Get the data size (size minus header)
    pub fn data_size(&self) -> u64 {
        self.size.saturating_sub(self.header_size)
    }
}

/// ISO Base Media File Format parser
pub struct IsobmffParser<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> IsobmffParser<R> {
    /// Create a new parser
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read a box at the current position
    pub fn read_box(&mut self) -> ExifResult<Option<Box>> {
        let start_pos = self.reader.stream_position()?;

        // Read box header (minimum 8 bytes: size + type)
        let mut header = [0u8; 8];
        match self.reader.read_exact(&mut header) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(None);
            }
            Err(e) => return Err(e.into()),
        }

        let mut size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as u64;
        let mut box_type = [0u8; 4];
        box_type.copy_from_slice(&header[4..8]);

        let mut header_size = 8u64;

        // Handle large boxes (size == 1 means 64-bit size follows)
        if size == 1 {
            let mut largesize = [0u8; 8];
            self.reader.read_exact(&mut largesize)?;
            size = u64::from_be_bytes(largesize);
            header_size = 16;
        }

        // Handle boxes that extend to end of file (size == 0)
        if size == 0 {
            // Seek to end to get file size
            let current_pos = self.reader.stream_position()?;
            let end_pos = self.reader.seek(SeekFrom::End(0))?;
            self.reader.seek(SeekFrom::Start(current_pos))?;
            size = end_pos - start_pos;
        }

        Ok(Some(Box {
            box_type,
            size,
            offset: start_pos,
            header_size,
        }))
    }

    /// Find a box with the specified type at the current level
    pub fn find_box(&mut self, target_type: &[u8; 4]) -> ExifResult<Option<Box>> {
        let start_pos = self.reader.stream_position()?;

        loop {
            let box_info = match self.read_box()? {
                Some(b) => b,
                None => {
                    self.reader.seek(SeekFrom::Start(start_pos))?;
                    return Ok(None);
                }
            };

            if &box_info.box_type == target_type {
                // Seek back to start of this box
                self.reader.seek(SeekFrom::Start(box_info.offset))?;
                return Ok(Some(box_info));
            }

            // Skip to next box
            self.reader
                .seek(SeekFrom::Start(box_info.offset + box_info.size))?;
        }
    }

    /// Read box data
    pub fn read_box_data(&mut self, box_info: &Box) -> ExifResult<Vec<u8>> {
        let data_size = box_info.data_size() as usize;
        if data_size > 100 * 1024 * 1024 {
            // Sanity check: don't read more than 100MB
            return Err(ExifError::Format("Box data too large (>100MB)".to_string()));
        }

        self.reader.seek(SeekFrom::Start(box_info.data_offset()))?;

        let mut data = vec![0u8; data_size];
        self.reader.read_exact(&mut data)?;

        Ok(data)
    }

    /// Enter a container box (seek to its data start)
    pub fn enter_box(&mut self, box_info: &Box) -> ExifResult<()> {
        self.reader.seek(SeekFrom::Start(box_info.data_offset()))?;
        Ok(())
    }

    /// Get the inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Find EXIF data in ISO Base Media File Format file by searching for common patterns
pub fn find_exif_data<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read up to 10MB to search for EXIF data
    let mut buffer = vec![0u8; 10 * 1024 * 1024];
    reader.seek(SeekFrom::Start(0))?;
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Search for "Exif\0\0" marker
    if let Some(pos) = find_subsequence(&buffer, b"Exif\0\0") {
        // Found potential EXIF data
        if pos + 6 + 8 <= buffer.len() {
            let tiff_start = pos + 6;
            if (buffer[tiff_start] == b'I' && buffer[tiff_start + 1] == b'I')
                || (buffer[tiff_start] == b'M' && buffer[tiff_start + 1] == b'M')
            {
                // Found valid TIFF header
                let max_len = std::cmp::min(1024 * 1024, buffer.len() - pos);
                return Ok(buffer[pos..pos + max_len].to_vec());
            }
        }
    }

    // Fallback: Search for standalone TIFF data
    let mut pos = 0;
    while pos + 8 <= buffer.len() {
        if (buffer[pos] == b'I' && buffer[pos + 1] == b'I')
            || (buffer[pos] == b'M' && buffer[pos + 1] == b'M')
        {
            let magic = if buffer[pos] == b'I' {
                u16::from_le_bytes([buffer[pos + 2], buffer[pos + 3]])
            } else {
                u16::from_be_bytes([buffer[pos + 2], buffer[pos + 3]])
            };

            if magic == 0x002A {
                // Found potential TIFF/EXIF data
                let max_len = std::cmp::min(512 * 1024, buffer.len() - pos);
                let tiff_data = &buffer[pos..pos + max_len];

                let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(tiff_data);
                return Ok(app1_data);
            }
        }
        pos += 1;
    }

    Err(ExifError::Format(
        "No EXIF data found in ISO BMFF file".to_string(),
    ))
}

/// Helper function to find a subsequence in a byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_box() {
        // Create a simple box: size(4) + type(4) + data
        let mut data = Vec::new();
        data.extend_from_slice(&16u32.to_be_bytes()); // size = 16
        data.extend_from_slice(b"test"); // type
        data.extend_from_slice(b"12345678"); // 8 bytes of data

        let cursor = Cursor::new(data);
        let mut parser = IsobmffParser::new(cursor);

        let box_info = parser.read_box().unwrap().unwrap();
        assert_eq!(&box_info.box_type, b"test");
        assert_eq!(box_info.size, 16);
        assert_eq!(box_info.data_size(), 8);
    }
}
