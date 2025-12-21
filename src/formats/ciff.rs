// CIFF (Camera Image File Format) parser for Canon CRW files
//
// CIFF is Canon's proprietary format used in older Canon cameras (before CR2).
// It uses a "heap" structure where data blocks are referenced by offsets.
//
// Reference: ExifTool CanonRaw.pm

use crate::errors::{ExifError, ExifResult};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

pub const CIFF_SIGNATURE: &[u8; 8] = b"HEAPCCDR";

// CIFF Tag IDs (upper 2 bits specify storage location, we mask them off)
// Storage: 0x0000 = in heap, 0x4000 = in directory entry, 0x8000 = in record entry
pub const TAG_NULL_RECORD: u16 = 0x0000;
pub const TAG_FREE_BYTES: u16 = 0x0001;
pub const TAG_COLOR_INFO1: u16 = 0x0032;

// ImageDescription directory tags
pub const TAG_FILE_DESCRIPTION: u16 = 0x0805;
pub const TAG_RAW_MAKE_MODEL: u16 = 0x080a;
pub const TAG_FIRMWARE_VERSION: u16 = 0x080b;
pub const TAG_COMPONENT_VERSION: u16 = 0x080c;
pub const TAG_ROM_OPERATION_MODE: u16 = 0x080d;
pub const TAG_OWNER_NAME: u16 = 0x0810;
pub const TAG_CANON_IMAGE_TYPE: u16 = 0x0815;
pub const TAG_ORIGINAL_FILE_NAME: u16 = 0x0816;
pub const TAG_THUMBNAIL_FILE_NAME: u16 = 0x0817;

// CapturedEvent tags
pub const TAG_TARGET_IMAGE_TYPE: u16 = 0x100a;
pub const TAG_SR_RELEASE_METHOD: u16 = 0x1010;
pub const TAG_SR_RELEASE_TIMING: u16 = 0x1011;
pub const TAG_RELEASE_SETTING: u16 = 0x1016;
pub const TAG_BASE_ISO: u16 = 0x101c;
pub const TAG_FOCAL_LENGTH: u16 = 0x1029;
pub const TAG_SHOT_INFO: u16 = 0x102a;
pub const TAG_COLOR_INFO2: u16 = 0x102c;
pub const TAG_CAMERA_SETTINGS: u16 = 0x102d;
pub const TAG_WHITE_SAMPLE: u16 = 0x1030;
pub const TAG_SENSOR_INFO: u16 = 0x1031;
pub const TAG_CUSTOM_FUNCTIONS: u16 = 0x1033;
pub const TAG_PI_AF_INFO: u16 = 0x1038;
pub const TAG_FLASH_INFO: u16 = 0x1028; // CanonFlashInfo

// ImageProps directory tags
pub const TAG_IMAGE_FORMAT: u16 = 0x1803;
pub const TAG_RECORD_ID: u16 = 0x1804;
pub const TAG_SELF_TIMER_TIME: u16 = 0x1806;
pub const TAG_TARGET_DISTANCE_SETTING: u16 = 0x1807;
pub const TAG_SERIAL_NUMBER: u16 = 0x180b;
pub const TAG_TIME_STAMP: u16 = 0x180e;
pub const TAG_IMAGE_INFO: u16 = 0x1810;
pub const TAG_FLASH_INFO_2: u16 = 0x1813;
pub const TAG_MEASURED_INFO: u16 = 0x1814;
pub const TAG_FILE_NUMBER: u16 = 0x1817;
pub const TAG_EXPOSURE_INFO: u16 = 0x1818;
pub const TAG_DECODER_TABLE: u16 = 0x1835;

// Subdirectory tags (have 0xC000 or 0x8000 in upper bits, but tag value has 0x3xxx)
pub const TAG_RAW_DATA: u16 = 0x2005;
pub const TAG_JPG_FROM_RAW: u16 = 0x2007;
pub const TAG_THUMBNAIL_IMAGE: u16 = 0x2008;
pub const TAG_CAMERA_OBJECT: u16 = 0x2807; // CameraObject subdirectory
pub const TAG_IMAGE_PROPS: u16 = 0x300a; // ImageProps subdirectory
pub const TAG_EXIF_INFORMATION: u16 = 0x300b; // ExifInformation subdirectory

#[derive(Debug)]
pub struct CiffHeader {
    pub is_little_endian: bool,
    pub header_length: u32,
}

#[derive(Debug, Clone)]
pub struct CiffEntry {
    pub tag: u16,
    pub size: u32,
    pub offset: u32,
    pub is_subdirectory: bool,
    pub storage_location: u8, // 0=heap, 1=record entry, 2=directory entry
}

/// Extracted CIFF metadata that can be converted to EXIF
#[derive(Debug, Default)]
pub struct CiffMetadata {
    // Basic info
    pub make: Option<String>,
    pub model: Option<String>,
    pub owner_name: Option<String>,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub original_file_name: Option<String>,
    pub canon_image_type: Option<String>,
    pub file_description: Option<String>,
    pub rom_operation_mode: Option<String>,

    // Image dimensions
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub rotation: Option<i32>,
    pub component_bit_depth: Option<u32>,
    pub color_bit_depth: Option<u32>,

    // Timestamp
    pub date_time_original: Option<u32>, // Unix timestamp

    // File info
    pub file_number: Option<u32>,

    // Camera settings (CanonCameraSettings)
    pub macro_mode: Option<u16>,
    pub self_timer: Option<u16>,
    pub quality: Option<u16>,
    pub flash_mode: Option<u16>,
    pub continuous_drive: Option<u16>,
    pub focus_mode: Option<u16>,
    pub record_mode: Option<u16>,
    pub image_size: Option<u16>,
    pub easy_mode: Option<u16>,
    pub digital_zoom: Option<u16>,
    pub contrast: Option<i16>,
    pub saturation: Option<i16>,
    pub sharpness: Option<i16>,
    pub iso_speed: Option<u16>,
    pub metering_mode: Option<u16>,
    pub focus_range: Option<u16>,
    pub af_point: Option<u16>,
    pub exposure_mode: Option<u16>,
    pub lens_type: Option<u16>,
    pub max_focal_length: Option<u16>,
    pub min_focal_length: Option<u16>,
    pub focal_units: Option<u16>,
    pub max_aperture: Option<u16>,
    pub min_aperture: Option<u16>,
    pub flash_activity: Option<u16>,
    pub flash_bits: Option<u16>,
    pub focus_continuous: Option<u16>,
    pub ae_setting: Option<u16>,
    pub image_stabilization: Option<u16>,
    pub zoom_source_width: Option<u16>,
    pub zoom_target_width: Option<u16>,
    pub spot_metering_mode: Option<u16>,

    // Shot info (CanonShotInfo)
    pub auto_iso: Option<u16>,
    pub base_iso_value: Option<u16>,
    pub measured_ev: Option<i16>,
    pub target_aperture: Option<u16>,
    pub target_exposure_time: Option<u16>,
    pub exposure_compensation: Option<i16>,
    pub white_balance: Option<u16>,
    pub slow_shutter: Option<u16>,
    pub sequence_number: Option<u16>,
    pub optical_zoom_code: Option<u16>,
    pub flash_guide_number: Option<u16>,
    pub flash_exposure_comp: Option<i16>,
    pub auto_exposure_bracketing: Option<u16>,
    pub aeb_bracket_value: Option<i16>,
    pub control_mode: Option<u16>,
    pub focus_distance_upper: Option<u16>,
    pub focus_distance_lower: Option<u16>,
    pub f_number: Option<u16>,
    pub exposure_time: Option<u16>,
    pub measured_ev2: Option<i16>,
    pub bulb_duration: Option<u16>,
    pub camera_type: Option<u16>,
    pub auto_rotate: Option<i16>,
    pub nd_filter: Option<i16>,
    pub self_timer2: Option<u16>,

    // Focal length info
    pub focal_type: Option<u16>,
    pub focal_length: Option<u16>,
    pub focal_plane_x_size: Option<u16>,
    pub focal_plane_y_size: Option<u16>,

    // Raw tag data for advanced parsing
    pub raw_tags: HashMap<u16, Vec<u8>>,
}

pub struct CiffParser<R: Read + Seek> {
    reader: R,
    header: CiffHeader,
    file_size: u64,
}

impl<R: Read + Seek> CiffParser<R> {
    pub fn new(mut reader: R) -> ExifResult<Self> {
        // Get file size
        let file_size = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        // Read byte order marker
        let mut byte_order = [0u8; 2];
        reader.read_exact(&mut byte_order)?;

        let is_little_endian = match &byte_order {
            b"II" => true,
            b"MM" => false,
            _ => {
                return Err(ExifError::Format(
                    "Invalid CIFF byte order marker".to_string(),
                ))
            }
        };

        // Read header length
        let header_length = reader.read_u32::<LittleEndian>()?;

        // Verify CIFF signature
        let mut signature = [0u8; 8];
        reader.read_exact(&mut signature)?;
        if &signature != CIFF_SIGNATURE {
            return Err(ExifError::Format("Invalid CIFF signature".to_string()));
        }

        let header = CiffHeader {
            is_little_endian,
            header_length,
        };

        Ok(CiffParser {
            reader,
            header,
            file_size,
        })
    }

    /// Read a directory block
    /// In CIFF, the last 4 bytes of a block contain an offset (relative to block start)
    /// pointing to the entry table location. At that location:
    /// [entry count (2 bytes)][entries (N * 10 bytes)]
    fn read_directory_block(
        &mut self,
        block_offset: u32,
        block_size: u32,
    ) -> ExifResult<Vec<CiffEntry>> {
        let block_end = block_offset as u64 + block_size as u64;
        if block_end > self.file_size {
            return Err(ExifError::Format(format!(
                "Directory block extends beyond file: offset={}, size={}",
                block_offset, block_size
            )));
        }

        // Last 4 bytes of block contain offset to entry table (relative to block start)
        self.reader.seek(SeekFrom::Start(block_end - 4))?;
        let dir_offset_in_block = self.reader.read_u32::<LittleEndian>()?;
        let entry_table_start = block_offset as u64 + dir_offset_in_block as u64;

        // Read entry count (2 bytes at entry table start)
        self.reader.seek(SeekFrom::Start(entry_table_start))?;
        let entry_count = self.reader.read_u16::<LittleEndian>()?;

        if entry_count > 1000 {
            return Err(ExifError::Format(format!(
                "Too many directory entries: {}",
                entry_count
            )));
        }

        // Entries follow immediately after the count

        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let tag_raw = self.reader.read_u16::<LittleEndian>()?;
            let size = self.reader.read_u32::<LittleEndian>()?;
            let offset_in_block = self.reader.read_u32::<LittleEndian>()?;

            // Extract tag ID (lower 14 bits) and storage location (upper 2 bits)
            let tag = tag_raw & 0x3FFF;
            let storage_location = (tag_raw >> 14) as u8;

            // For data in heap (storage_location == 0), convert block-relative
            // offset to file offset by adding block_offset
            let offset = if storage_location == 0 {
                offset_in_block + block_offset
            } else {
                // Value is stored in the entry itself (size and offset fields contain value)
                offset_in_block
            };

            // Check if this is a subdirectory (tag type indicates directory)
            // Directories have tag values like 0x2807, 0x300a, 0x300b
            let is_subdirectory = (tag & 0x3800) == 0x2800 || (tag & 0x3800) == 0x3000;

            entries.push(CiffEntry {
                tag,
                size,
                offset,
                is_subdirectory,
                storage_location,
            });
        }

        Ok(entries)
    }

    /// Read data for an entry from the heap
    fn read_entry_data(&mut self, entry: &CiffEntry) -> ExifResult<Vec<u8>> {
        if entry.storage_location != 0 {
            // Data is stored in the entry itself (for small values)
            // The "size" and "offset" fields contain the actual data
            let mut data = Vec::with_capacity(8);
            data.extend_from_slice(&entry.size.to_le_bytes());
            data.extend_from_slice(&entry.offset.to_le_bytes());
            return Ok(data);
        }

        // Data is in the heap - entry.offset is already a file offset
        if entry.offset as u64 + entry.size as u64 > self.file_size {
            return Err(ExifError::Format(format!(
                "Entry data extends beyond file: offset={}, size={}",
                entry.offset, entry.size
            )));
        }

        self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut data = vec![0u8; entry.size as usize];
        self.reader.read_exact(&mut data)?;
        Ok(data)
    }

    /// Extract all metadata from the CIFF structure
    pub fn extract_metadata(&mut self) -> ExifResult<CiffMetadata> {
        // Read root directory entries using the footer offset
        let root_entries = self.read_root_directory()?;
        let mut metadata = CiffMetadata::default();

        // Process root directory entries
        for entry in root_entries {
            if entry.is_subdirectory && entry.size > 0 {
                // entry.offset is already a file offset
                let _ = self.parse_directory_block(entry.offset, entry.size, &mut metadata);
            } else if let Ok(data) = self.read_entry_data(&entry) {
                self.parse_entry(entry.tag, &data, &mut metadata);
                metadata.raw_tags.insert(entry.tag, data);
            }
        }

        Ok(metadata)
    }

    /// Read the root directory entries using the footer offset
    fn read_root_directory(&mut self) -> ExifResult<Vec<CiffEntry>> {
        // The last 4 bytes of the file contain the heap offset to the root directory entries
        self.reader.seek(SeekFrom::End(-4))?;
        let entry_table_heap_offset = self.reader.read_u32::<LittleEndian>()?;
        let entry_table_file_offset =
            entry_table_heap_offset as u64 + self.header.header_length as u64;

        // Read entry count (first 2 bytes at the entry table position)
        self.reader.seek(SeekFrom::Start(entry_table_file_offset))?;
        let entry_count = self.reader.read_u16::<LittleEndian>()?;

        if entry_count > 1000 {
            return Err(ExifError::Format(format!(
                "Too many root directory entries: {}",
                entry_count
            )));
        }

        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let tag_raw = self.reader.read_u16::<LittleEndian>()?;
            let size = self.reader.read_u32::<LittleEndian>()?;
            let offset_in_heap = self.reader.read_u32::<LittleEndian>()?;

            let tag = tag_raw & 0x3FFF;
            let storage_location = (tag_raw >> 14) as u8;
            let is_subdirectory = (tag & 0x3800) == 0x2800 || (tag & 0x3800) == 0x3000;

            // For root entries, offsets are relative to heap start (header_length)
            // Convert to file offset for consistency
            let offset = if storage_location == 0 {
                offset_in_heap + self.header.header_length
            } else {
                offset_in_heap
            };

            entries.push(CiffEntry {
                tag,
                size,
                offset,
                is_subdirectory,
                storage_location,
            });
        }

        Ok(entries)
    }

    /// Recursively parse a directory block and its subdirectories
    fn parse_directory_block(
        &mut self,
        block_offset: u32,
        block_size: u32,
        metadata: &mut CiffMetadata,
    ) -> ExifResult<()> {
        let entries = self.read_directory_block(block_offset, block_size)?;

        for entry in entries {
            // Handle subdirectories first
            if entry.is_subdirectory && entry.size > 0 {
                // entry.offset is already a file offset (block_offset + offset_in_block)
                let _ = self.parse_directory_block(entry.offset, entry.size, metadata);
                continue;
            }

            // Read and parse entry data
            if let Ok(data) = self.read_entry_data(&entry) {
                self.parse_entry(entry.tag, &data, metadata);
                // Store raw data for later use
                metadata.raw_tags.insert(entry.tag, data);
            }
        }

        Ok(())
    }

    /// Parse a single entry and extract relevant metadata
    fn parse_entry(&mut self, tag: u16, data: &[u8], metadata: &mut CiffMetadata) {
        match tag {
            TAG_RAW_MAKE_MODEL => {
                // Format: Make\0Model\0...
                // The buffer may contain garbage after the null-terminated strings
                // So we parse null-terminated strings until we hit invalid UTF-8
                let mut parts = Vec::new();
                let mut start = 0;
                for (i, &b) in data.iter().enumerate() {
                    if b == 0 {
                        if i > start {
                            if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                                parts.push(s.to_string());
                            }
                        }
                        start = i + 1;
                    }
                }
                if !parts.is_empty() {
                    metadata.make = Some(parts[0].clone());
                }
                if parts.len() > 1 {
                    metadata.model = Some(parts[1].clone());
                }
            }
            TAG_OWNER_NAME => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.owner_name = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_FIRMWARE_VERSION => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.firmware_version = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_SERIAL_NUMBER => {
                if data.len() >= 4 {
                    metadata.serial_number =
                        Some(format!("{:010}", LittleEndian::read_u32(&data[0..4])));
                }
            }
            TAG_ORIGINAL_FILE_NAME => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.original_file_name = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_CANON_IMAGE_TYPE => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.canon_image_type = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_FILE_DESCRIPTION => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.file_description = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_ROM_OPERATION_MODE => {
                if let Ok(s) = std::str::from_utf8(data) {
                    metadata.rom_operation_mode = Some(s.trim_end_matches('\0').to_string());
                }
            }
            TAG_IMAGE_INFO => {
                // ImageInfo: 7 x int32u
                // 0: ImageWidth, 1: ImageHeight, 2: PixelAspectRatio (float),
                // 3: Rotation, 4: ComponentBitDepth, 5: ColorBitDepth, 6: ColorBW
                if data.len() >= 28 {
                    metadata.image_width = Some(LittleEndian::read_u32(&data[0..4]));
                    metadata.image_height = Some(LittleEndian::read_u32(&data[4..8]));
                    metadata.rotation = Some(LittleEndian::read_i32(&data[12..16]));
                    metadata.component_bit_depth = Some(LittleEndian::read_u32(&data[16..20]));
                    metadata.color_bit_depth = Some(LittleEndian::read_u32(&data[20..24]));
                }
            }
            TAG_TIME_STAMP => {
                // TimeStamp: 3 x int32u - DateTimeOriginal, TimeZoneCode, TimeZoneInfo
                if data.len() >= 4 {
                    metadata.date_time_original = Some(LittleEndian::read_u32(&data[0..4]));
                }
            }
            TAG_FILE_NUMBER => {
                if data.len() >= 4 {
                    metadata.file_number = Some(LittleEndian::read_u32(&data[0..4]));
                }
            }
            TAG_FOCAL_LENGTH => {
                // FocalLength: 4 x int16u
                // 0: FocalType, 1: FocalLength, 2: FocalPlaneXSize, 3: FocalPlaneYSize
                if data.len() >= 8 {
                    metadata.focal_type = Some(LittleEndian::read_u16(&data[0..2]));
                    metadata.focal_length = Some(LittleEndian::read_u16(&data[2..4]));
                    metadata.focal_plane_x_size = Some(LittleEndian::read_u16(&data[4..6]));
                    metadata.focal_plane_y_size = Some(LittleEndian::read_u16(&data[6..8]));
                }
            }
            TAG_CAMERA_SETTINGS => {
                self.parse_camera_settings(data, metadata);
            }
            TAG_SHOT_INFO => {
                self.parse_shot_info(data, metadata);
            }
            TAG_BASE_ISO => {
                if data.len() >= 4 {
                    metadata.base_iso_value = Some(LittleEndian::read_u32(&data[0..4]) as u16);
                }
            }
            _ => {
                // Unknown tag, skip
            }
        }
    }

    /// Parse CanonCameraSettings data (tag 0x102d)
    fn parse_camera_settings(&self, data: &[u8], metadata: &mut CiffMetadata) {
        // CanonCameraSettings is an array of int16s
        if data.len() < 2 {
            return;
        }

        let read_u16 = |offset: usize| -> Option<u16> {
            if offset + 2 <= data.len() {
                Some(LittleEndian::read_u16(&data[offset..offset + 2]))
            } else {
                None
            }
        };

        let read_i16 = |offset: usize| -> Option<i16> {
            if offset + 2 <= data.len() {
                Some(LittleEndian::read_i16(&data[offset..offset + 2]))
            } else {
                None
            }
        };

        // Indices based on ExifTool Canon.pm CanonCameraSettings
        metadata.macro_mode = read_u16(2);
        metadata.self_timer = read_u16(4);
        metadata.quality = read_u16(6);
        metadata.flash_mode = read_u16(8);
        metadata.continuous_drive = read_u16(10);
        metadata.focus_mode = read_u16(14);
        metadata.record_mode = read_u16(18);
        metadata.image_size = read_u16(20);
        metadata.easy_mode = read_u16(22);
        metadata.digital_zoom = read_u16(24);
        metadata.contrast = read_i16(26);
        metadata.saturation = read_i16(28);
        metadata.sharpness = read_i16(30);
        metadata.iso_speed = read_u16(32);
        metadata.metering_mode = read_u16(34);
        metadata.focus_range = read_u16(36);
        metadata.af_point = read_u16(38);
        metadata.exposure_mode = read_u16(40);
        metadata.lens_type = read_u16(44);
        metadata.max_focal_length = read_u16(46);
        metadata.min_focal_length = read_u16(48);
        metadata.focal_units = read_u16(50);
        metadata.max_aperture = read_u16(52);
        metadata.min_aperture = read_u16(54);
        metadata.flash_activity = read_u16(56);
        metadata.flash_bits = read_u16(58);
        metadata.zoom_source_width = read_u16(68);
        metadata.zoom_target_width = read_u16(70);
    }

    /// Parse CanonShotInfo data (tag 0x102a)
    fn parse_shot_info(&self, data: &[u8], metadata: &mut CiffMetadata) {
        if data.len() < 2 {
            return;
        }

        let read_u16 = |offset: usize| -> Option<u16> {
            if offset + 2 <= data.len() {
                Some(LittleEndian::read_u16(&data[offset..offset + 2]))
            } else {
                None
            }
        };

        let read_i16 = |offset: usize| -> Option<i16> {
            if offset + 2 <= data.len() {
                Some(LittleEndian::read_i16(&data[offset..offset + 2]))
            } else {
                None
            }
        };

        // Indices based on ExifTool Canon.pm CanonShotInfo
        metadata.auto_iso = read_u16(2);
        metadata.base_iso_value = read_u16(4);
        metadata.measured_ev = read_i16(6);
        metadata.target_aperture = read_u16(8);
        metadata.target_exposure_time = read_u16(10);
        metadata.exposure_compensation = read_i16(12);
        metadata.white_balance = read_u16(14);
        metadata.slow_shutter = read_u16(16);
        metadata.sequence_number = read_u16(18);
        metadata.optical_zoom_code = read_u16(20);
        metadata.flash_guide_number = read_u16(26);
        metadata.flash_exposure_comp = read_i16(28);
        metadata.auto_exposure_bracketing = read_u16(30);
        metadata.aeb_bracket_value = read_i16(32);
        metadata.control_mode = read_u16(34);
        metadata.focus_distance_upper = read_u16(36);
        metadata.focus_distance_lower = read_u16(38);
        // Index 21-24 per ExifTool Canon.pm
        metadata.f_number = read_u16(42); // Index 21
        metadata.exposure_time = read_u16(44); // Index 22
        metadata.measured_ev2 = read_i16(46); // Index 23
        metadata.bulb_duration = read_u16(48); // Index 24
        metadata.camera_type = read_u16(52); // Index 26
        metadata.auto_rotate = read_i16(54); // Index 27
        metadata.nd_filter = read_i16(56); // Index 28
        metadata.self_timer2 = read_u16(58); // Index 29
    }

    /// Build a synthetic TIFF EXIF segment from the extracted metadata
    pub fn build_exif_segment(&mut self) -> ExifResult<Vec<u8>> {
        let metadata = self.extract_metadata()?;

        // Build a minimal TIFF structure with EXIF data
        let mut exif_data = Vec::new();

        // Add Exif header
        exif_data.extend_from_slice(b"Exif\0\0");

        // Build TIFF header and IFD0
        let tiff_data = build_tiff_from_metadata(&metadata)?;
        exif_data.extend_from_slice(&tiff_data);

        Ok(exif_data)
    }
}

/// Build a TIFF structure from CIFF metadata
fn build_tiff_from_metadata(metadata: &CiffMetadata) -> ExifResult<Vec<u8>> {
    let mut data = Vec::new();

    // TIFF header (little-endian)
    data.extend_from_slice(b"II"); // Little endian
    data.extend_from_slice(&42u16.to_le_bytes()); // TIFF magic
    data.extend_from_slice(&8u32.to_le_bytes()); // Offset to IFD0

    // We'll build IFD0 with basic tags
    let mut ifd_entries: Vec<(u16, u16, u32, Vec<u8>)> = Vec::new();

    // Add Make (tag 0x010F)
    if let Some(ref make) = metadata.make {
        let mut make_bytes = make.as_bytes().to_vec();
        make_bytes.push(0); // Null terminator
        ifd_entries.push((0x010F, 2, make_bytes.len() as u32, make_bytes));
    }

    // Add Model (tag 0x0110)
    if let Some(ref model) = metadata.model {
        let mut model_bytes = model.as_bytes().to_vec();
        model_bytes.push(0);
        ifd_entries.push((0x0110, 2, model_bytes.len() as u32, model_bytes));
    }

    // Add Orientation (tag 0x0112)
    if let Some(rotation) = metadata.rotation {
        let orientation: u16 = match rotation {
            0 => 1,   // Normal
            90 => 6,  // Rotate 90 CW
            180 => 3, // Rotate 180
            270 => 8, // Rotate 270 CW
            _ => 1,
        };
        ifd_entries.push((0x0112, 3, 1, orientation.to_le_bytes().to_vec()));
    }

    // Add ImageWidth (tag 0x0100) and ImageHeight (tag 0x0101)
    if let Some(width) = metadata.image_width {
        ifd_entries.push((0x0100, 4, 1, width.to_le_bytes().to_vec()));
    }
    if let Some(height) = metadata.image_height {
        ifd_entries.push((0x0101, 4, 1, height.to_le_bytes().to_vec()));
    }

    // Add DateTimeOriginal (tag 0x9003) in EXIF IFD
    // For now, we'll add it in IFD0 as DateTime (0x0132)
    if let Some(timestamp) = metadata.date_time_original {
        // Convert Unix timestamp to EXIF date format "YYYY:MM:DD HH:MM:SS"
        let datetime = format_unix_timestamp(timestamp);
        let mut dt_bytes = datetime.as_bytes().to_vec();
        dt_bytes.push(0);
        ifd_entries.push((0x0132, 2, dt_bytes.len() as u32, dt_bytes));
    }

    // Add Software/Firmware (tag 0x0131)
    if let Some(ref firmware) = metadata.firmware_version {
        let mut fw_bytes = firmware.as_bytes().to_vec();
        fw_bytes.push(0);
        ifd_entries.push((0x0131, 2, fw_bytes.len() as u32, fw_bytes));
    }

    // Add Artist/Owner (tag 0x013B)
    if let Some(ref owner) = metadata.owner_name {
        if !owner.is_empty() {
            let mut owner_bytes = owner.as_bytes().to_vec();
            owner_bytes.push(0);
            ifd_entries.push((0x013B, 2, owner_bytes.len() as u32, owner_bytes));
        }
    }

    // Add ISO (tag 0x8827 - ISOSpeedRatings)
    // Use auto_iso which is stored as: ISO = 100 * 2^(value/32)
    // A value of 0 means ISO 100
    if let Some(auto_iso_raw) = metadata.auto_iso {
        let iso = if auto_iso_raw == 0 {
            100u16
        } else {
            (100.0 * 2.0_f64.powf(auto_iso_raw as f64 / 32.0)).round() as u16
        };
        ifd_entries.push((0x8827, 3, 1, iso.to_le_bytes().to_vec()));
    }

    // Add ExposureTime (tag 0x829A) - RATIONAL
    // Canon stores as APEX value, convert: time = 2^(-value/32)
    if let Some(apex_time) = metadata.exposure_time {
        if apex_time > 0 {
            let time_secs = 2.0_f64.powf(-(apex_time as f64) / 32.0);
            // Express as rational: 1/x for short exposures, x/1 for long
            let (num, denom) = if time_secs >= 1.0 {
                ((time_secs * 10.0) as u32, 10u32)
            } else {
                (1u32, (1.0 / time_secs).round() as u32)
            };
            let mut rational = Vec::new();
            rational.extend_from_slice(&num.to_le_bytes());
            rational.extend_from_slice(&denom.to_le_bytes());
            ifd_entries.push((0x829A, 5, 1, rational));
        }
    }

    // Add FNumber (tag 0x829D) - RATIONAL
    // Canon stores as APEX value, convert: fnumber = 2^(value/64)
    if let Some(apex_fnum) = metadata.f_number {
        if apex_fnum > 0 {
            let fnum = 2.0_f64.powf((apex_fnum as f64) / 64.0);
            // Express as rational with denominator 10 for one decimal place
            let num = (fnum * 10.0).round() as u32;
            let denom = 10u32;
            let mut rational = Vec::new();
            rational.extend_from_slice(&num.to_le_bytes());
            rational.extend_from_slice(&denom.to_le_bytes());
            ifd_entries.push((0x829D, 5, 1, rational));
        }
    }

    // Add FocalLength (tag 0x920A) - RATIONAL
    if let Some(fl) = metadata.focal_length {
        let units = metadata.focal_units.unwrap_or(1) as u32;
        if units > 0 && fl > 0 {
            let fl_mm = fl as u32;
            let mut rational = Vec::new();
            rational.extend_from_slice(&fl_mm.to_le_bytes());
            rational.extend_from_slice(&units.to_le_bytes());
            ifd_entries.push((0x920A, 5, 1, rational));
        }
    }

    // Add ExposureBiasValue (tag 0x9204) - SRATIONAL
    if let Some(ev_comp) = metadata.exposure_compensation {
        // Canon stores as value * 32 (e.g., -32 = -1 EV)
        // EXIF wants it as a rational in EV
        let num = ev_comp as i32;
        let denom = 32i32;
        let mut rational = Vec::new();
        rational.extend_from_slice(&num.to_le_bytes());
        rational.extend_from_slice(&denom.to_le_bytes());
        ifd_entries.push((0x9204, 10, 1, rational)); // type 10 = SRATIONAL
    }

    // Add MeteringMode (tag 0x9207)
    if let Some(metering) = metadata.metering_mode {
        // Canon: 0=Default, 1=Spot, 2=Average, 3=Evaluative, 4=Partial, 5=CenterWeighted
        // EXIF: 1=Average, 2=CenterWeighted, 3=Spot, 4=MultiSpot, 5=Pattern
        let exif_metering: u16 = match metering {
            1 => 3, // Spot
            2 => 1, // Average
            3 => 5, // Evaluative -> Pattern
            4 => 6, // Partial
            5 => 2, // CenterWeighted
            _ => 0, // Unknown
        };
        ifd_entries.push((0x9207, 3, 1, exif_metering.to_le_bytes().to_vec()));
    }

    // Add Flash (tag 0x9209)
    if let Some(flash) = metadata.flash_mode {
        // Simple mapping: 0 = no flash, otherwise flash fired
        let exif_flash: u16 = if flash == 0 { 0 } else { 1 };
        ifd_entries.push((0x9209, 3, 1, exif_flash.to_le_bytes().to_vec()));
    }

    // Add FocalPlaneXResolution (tag 0xA20E) and YResolution (tag 0xA20F)
    if let Some(xsize) = metadata.focal_plane_x_size {
        if let Some(width) = metadata.image_width {
            if xsize > 0 {
                // xsize is in 1/1000 mm, calculate pixels per mm then convert to inches
                // Resolution = width / (xsize/1000) pixels per mm
                // For EXIF we want pixels per inch = resolution * 25.4
                let res_x = ((width as f64) / (xsize as f64 / 1000.0) * 25.4) as u32;
                let mut rational = Vec::new();
                rational.extend_from_slice(&res_x.to_le_bytes());
                rational.extend_from_slice(&1u32.to_le_bytes());
                ifd_entries.push((0xA20E, 5, 1, rational));
            }
        }
    }
    if let Some(ysize) = metadata.focal_plane_y_size {
        if let Some(height) = metadata.image_height {
            if ysize > 0 {
                let res_y = ((height as f64) / (ysize as f64 / 1000.0) * 25.4) as u32;
                let mut rational = Vec::new();
                rational.extend_from_slice(&res_y.to_le_bytes());
                rational.extend_from_slice(&1u32.to_le_bytes());
                ifd_entries.push((0xA20F, 5, 1, rational));
            }
        }
    }

    // Add FocalPlaneResolutionUnit (tag 0xA210) - 2 = inches
    if metadata.focal_plane_x_size.is_some() || metadata.focal_plane_y_size.is_some() {
        ifd_entries.push((0xA210, 3, 1, 2u16.to_le_bytes().to_vec()));
    }

    // Sort entries by tag number (required by TIFF spec)
    ifd_entries.sort_by_key(|e| e.0);

    // Calculate offsets
    let ifd_start = 8u32; // After TIFF header
    let entry_count = ifd_entries.len() as u16;
    let ifd_size = 2 + (entry_count as u32 * 12) + 4; // count + entries + next IFD offset
    let mut data_offset = ifd_start + ifd_size;

    // Write IFD0
    data.extend_from_slice(&entry_count.to_le_bytes());

    for (tag, type_id, count, value_data) in &ifd_entries {
        data.extend_from_slice(&tag.to_le_bytes());
        data.extend_from_slice(&type_id.to_le_bytes());
        data.extend_from_slice(&count.to_le_bytes());

        let value_size = match *type_id {
            1 => *count,      // BYTE
            2 => *count,      // ASCII
            3 => *count * 2,  // SHORT
            4 => *count * 4,  // LONG
            5 => *count * 8,  // RATIONAL
            6 => *count,      // SBYTE
            7 => *count,      // UNDEFINED
            8 => *count * 2,  // SSHORT
            9 => *count * 4,  // SLONG
            10 => *count * 8, // SRATIONAL
            _ => *count,
        };

        if value_size <= 4 {
            // Value fits in offset field
            let mut value_field = [0u8; 4];
            let copy_len = value_data.len().min(4);
            value_field[..copy_len].copy_from_slice(&value_data[..copy_len]);
            data.extend_from_slice(&value_field);
        } else {
            // Value stored in data area
            data.extend_from_slice(&data_offset.to_le_bytes());
            data_offset += value_size;
        }
    }

    // Next IFD offset (0 = no more IFDs)
    data.extend_from_slice(&0u32.to_le_bytes());

    // Write data area for values that didn't fit
    for (_, type_id, count, value_data) in &ifd_entries {
        let value_size = match *type_id {
            1 | 6 | 7 => *count,  // BYTE, SBYTE, UNDEFINED
            2 => *count,          // ASCII
            3 | 8 => *count * 2,  // SHORT, SSHORT
            4 | 9 => *count * 4,  // LONG, SLONG
            5 | 10 => *count * 8, // RATIONAL, SRATIONAL
            _ => *count,
        };

        if value_size > 4 {
            data.extend_from_slice(value_data);
        }
    }

    Ok(data)
}

/// Format a Unix timestamp as EXIF date string
fn format_unix_timestamp(timestamp: u32) -> String {
    // Simple conversion - not handling timezones
    let secs = timestamp as i64;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year, month, day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!(
        "{:04}:{:02}:{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to year, month, day
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    // Simplified algorithm
    let mut remaining_days = days;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days as u32 + 1;

    (year, month, day)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_invalid_signature() {
        let mut data = Vec::new();
        data.extend_from_slice(b"II"); // Byte order
        data.extend_from_slice(&26u32.to_le_bytes()); // Header length
        data.extend_from_slice(b"INVALID!"); // Bad signature

        let cursor = Cursor::new(data);
        let result = CiffParser::new(cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_format_timestamp() {
        // 2004:04:02 09:53:00 = 1080899580
        let formatted = format_unix_timestamp(1080899580);
        assert_eq!(formatted, "2004:04:02 09:53:00");
    }

    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01 = day 0
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        // 2000-01-01 = day 10957
        assert_eq!(days_to_ymd(10957), (2000, 1, 1));
        // 2004-04-02 = day 12510
        assert_eq!(days_to_ymd(12510), (2004, 4, 2));
    }
}
