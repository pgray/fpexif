// lib.rs - Main API entry point
pub mod data_types;
pub mod errors;
pub mod extract;
pub mod formats;
pub mod io;
pub mod macros;
pub mod makernotes;
pub mod parser;
pub mod tags;

#[cfg(feature = "serde")]
pub mod output;

#[cfg(feature = "cli")]
pub mod mfr_test;

#[cfg(feature = "wasm")]
pub mod wasm;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// The main entry point for parsing EXIF data
pub struct ExifParser {
    // Configuration options
    verbose: bool,
    strict_parsing: bool,
}

impl ExifParser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self {
            verbose: false,
            strict_parsing: true,
        }
    }

    /// Enable or disable verbose logging
    pub fn verbose(mut self, value: bool) -> Self {
        self.verbose = value;
        self
    }

    /// Enable or disable strict parsing mode
    pub fn strict(mut self, value: bool) -> Self {
        self.strict_parsing = value;
        self
    }

    /// Parse EXIF data from a file path
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<ExifData, errors::ExifError> {
        let path = path.as_ref();

        // Check for unsupported file formats
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "cam" {
                return Err(errors::ExifError::UnsupportedFormat(
                    "Casio .cam files use a proprietary format and are not supported. Please convert to JPEG or another standard format.".to_string()
                ));
            }
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.parse_reader(reader)
    }

    /// Parse EXIF data from any reader implementing Read and Seek
    pub fn parse_reader<R: std::io::Read + std::io::Seek>(
        &self,
        reader: R,
    ) -> Result<ExifData, errors::ExifError> {
        // Delegate to the internal parser module
        parser::parse_exif(reader, self.strict_parsing, self.verbose)
    }

    /// Parse EXIF data from a byte slice
    pub fn parse_bytes(&self, data: &[u8]) -> Result<ExifData, errors::ExifError> {
        // Use a cursor as a reader
        let cursor = std::io::Cursor::new(data);
        self.parse_reader(cursor)
    }
}

/// Represents parsed EXIF data
pub struct ExifData {
    // Internal storage for tag data
    tags: std::collections::HashMap<tags::ExifTagId, data_types::ExifValue>,
    // Track the original endianness of the data
    pub endian: data_types::Endianness,
    // Parsed maker notes
    maker_notes: Option<std::collections::HashMap<u16, makernotes::MakerNoteTag>>,
    // RAF-specific metadata (for Fujifilm RAF files)
    pub raf_metadata: Option<formats::RafMetadata>,
    // MRW-specific metadata (for Minolta RAW files)
    pub mrw_metadata: Option<formats::MrwMetadata>,
    // RW2-specific metadata (for Panasonic RAW files)
    pub rw2_metadata: Option<formats::Rw2Metadata>,
    // File size in bytes (populated from WASM or file metadata)
    file_size: Option<u64>,
}

impl ExifData {
    /// Create a new empty EXIF data container
    pub fn new() -> Self {
        Self {
            tags: std::collections::HashMap::new(),
            endian: data_types::Endianness::Little,
            maker_notes: None,
            raf_metadata: None,
            mrw_metadata: None,
            rw2_metadata: None,
            file_size: None,
        }
    }

    /// Set RAF-specific metadata
    pub fn set_raf_metadata(&mut self, metadata: formats::RafMetadata) {
        self.raf_metadata = Some(metadata);
    }

    /// Get RAF-specific metadata
    pub fn get_raf_metadata(&self) -> Option<&formats::RafMetadata> {
        self.raf_metadata.as_ref()
    }

    /// Set MRW-specific metadata (RIF block data)
    pub fn set_mrw_metadata(&mut self, metadata: formats::MrwMetadata) {
        self.mrw_metadata = Some(metadata);
    }

    /// Get MRW-specific metadata
    pub fn get_mrw_metadata(&self) -> Option<&formats::MrwMetadata> {
        self.mrw_metadata.as_ref()
    }

    /// Set RW2-specific metadata (PanasonicRaw IFD0 data)
    pub fn set_rw2_metadata(&mut self, metadata: formats::Rw2Metadata) {
        self.rw2_metadata = Some(metadata);
    }

    /// Get RW2-specific metadata
    pub fn get_rw2_metadata(&self) -> Option<&formats::Rw2Metadata> {
        self.rw2_metadata.as_ref()
    }

    /// Set file size in bytes
    pub fn set_file_size(&mut self, size: u64) {
        self.file_size = Some(size);
    }

    /// Get file size in bytes
    pub fn get_file_size(&self) -> Option<u64> {
        self.file_size
    }

    /// Get a tag value by its numeric ID
    /// Searches all IFD groups (Main, Exif, GPS, Thumbnail, Interop)
    pub fn get_tag_by_id(&self, id: u16) -> Option<&data_types::ExifValue> {
        // Try each IFD group to find the tag
        let groups = [
            tags::TagGroup::Main,
            tags::TagGroup::Exif,
            tags::TagGroup::Gps,
            tags::TagGroup::Thumbnail,
            tags::TagGroup::Interop,
        ];
        for group in groups {
            let tag_id = tags::ExifTagId::new(id, group);
            if let Some(value) = self.tags.get(&tag_id) {
                return Some(value);
            }
        }
        None
    }

    /// Get a tag value by its name
    pub fn get_tag_by_name(&self, name: &str) -> Option<&data_types::ExifValue> {
        // Look up the tag ID from name and then get the value
        tags::get_tag_id_by_name(name).and_then(|id| self.tags.get(&id))
    }

    /// Iterate through all tags
    pub fn iter(&self) -> impl Iterator<Item = (&tags::ExifTagId, &data_types::ExifValue)> {
        self.tags.iter()
    }

    /// Get the count of tags
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Check if there are no tags
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Get the parsed maker notes
    pub fn get_maker_notes(
        &self,
    ) -> Option<&std::collections::HashMap<u16, makernotes::MakerNoteTag>> {
        self.maker_notes.as_ref()
    }
}

// Provide a convenient constructor
impl Default for ExifParser {
    fn default() -> Self {
        Self::new()
    }
}

// Provide a convenient constructor
impl Default for ExifData {
    fn default() -> Self {
        Self::new()
    }
}
