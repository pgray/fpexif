// formats/raf.rs - Fujifilm RAF format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

const RAF_SIGNATURE: &[u8] = b"FUJIFILMCCD-RAW";

/// Format a float value with 10 significant figures (for distortion/vignetting params)
/// ExifTool outputs these with 10 significant figures total
fn format_float_10places(val: f64) -> String {
    if val == 0.0 {
        return "0".to_string();
    }

    // ExifTool uses scientific notation for small absolute values
    if val.abs() > 0.0 && val.abs() < 0.0001 {
        // Format with 10 significant figures in scientific notation
        // The mantissa should have 9 decimal places (1 digit before + 9 after = 10 sig figs)
        let s = format!("{:.9e}", val);

        // Find the 'e' position and split
        if let Some(e_pos) = s.find('e') {
            let mantissa_part = &s[..e_pos];
            let exp_part = &s[e_pos..];

            // Trim trailing zeros from mantissa (but keep at least one decimal)
            let trimmed_mantissa = mantissa_part.trim_end_matches('0').trim_end_matches('.');

            // Format exponent with 2 digits
            let formatted_exp = if let Some(neg_pos) = exp_part.find("e-") {
                let exp_digits = &exp_part[neg_pos + 2..];
                if exp_digits.len() == 1 {
                    format!("e-0{}", exp_digits)
                } else {
                    exp_part.to_string()
                }
            } else if let Some(pos_pos) = exp_part.find("e+") {
                let exp_digits = &exp_part[pos_pos + 2..];
                if exp_digits.len() == 1 {
                    format!("e+0{}", exp_digits)
                } else {
                    exp_part.to_string()
                }
            } else {
                exp_part.to_string()
            };

            return format!("{}{}", trimmed_mantissa, formatted_exp);
        }
        return s;
    }

    // For normal values, calculate decimal places based on magnitude
    // to achieve 10 significant figures
    let magnitude = val.abs().log10().floor() as i32;
    let decimal_places = (9 - magnitude).max(0) as usize;

    let s = format!("{:.prec$}", val, prec = decimal_places);

    // Trim trailing zeros after decimal point
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Format a float value with 6 decimal places (for balance values)
fn format_float_6places(val: f64) -> String {
    format_float_with_precision(val, 6)
}

/// Format a float value with specified decimal precision
fn format_float_with_precision(val: f64, precision: u32) -> String {
    let factor = 10f64.powi(precision as i32);
    let rounded = (val * factor).round() / factor;

    // ExifTool uses scientific notation for small values
    if rounded.abs() > 0.0 && rounded.abs() < 0.0001 {
        // Format as scientific notation with 2-digit exponent
        let s = format!("{:.prec$e}", rounded, prec = precision as usize);

        // Find the 'e' position and split
        if let Some(e_pos) = s.find('e') {
            let mantissa_part = &s[..e_pos];
            let exp_part = &s[e_pos..];

            // Trim trailing zeros and decimal point from mantissa
            let trimmed_mantissa = mantissa_part.trim_end_matches('0').trim_end_matches('.');

            // Format exponent with 2 digits
            let formatted_exp = if let Some(neg_pos) = exp_part.find("e-") {
                let exp_digits = &exp_part[neg_pos + 2..];
                if exp_digits.len() == 1 {
                    format!("e-0{}", exp_digits)
                } else {
                    exp_part.to_string()
                }
            } else if let Some(pos_pos) = exp_part.find("e+") {
                let exp_digits = &exp_part[pos_pos + 2..];
                if exp_digits.len() == 1 {
                    format!("e+0{}", exp_digits)
                } else {
                    exp_part.to_string()
                }
            } else {
                exp_part.to_string()
            };

            return format!("{}{}", trimmed_mantissa, formatted_exp);
        }
        s
    } else {
        // Format with specified decimal places
        let s = format!("{:.prec$}", rounded, prec = precision as usize);

        // Trim trailing zeros after decimal point
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            s
        }
    }
}

/// Format WB_GRGBLevels as space-separated int16u values
/// Data is big-endian, format: G R G B (4 x int16u = 8 bytes)
fn format_wb_grgb_levels(data: &[u8]) -> String {
    if data.len() < 8 {
        return String::new();
    }
    let g1 = u16::from_be_bytes([data[0], data[1]]);
    let r = u16::from_be_bytes([data[2], data[3]]);
    let g2 = u16::from_be_bytes([data[4], data[5]]);
    let b = u16::from_be_bytes([data[6], data[7]]);
    format!("{} {} {} {}", g1, r, g2, b)
}

/// RAF-specific metadata extracted from RAF header and directory
#[derive(Debug, Clone, Default)]
pub struct RafMetadata {
    /// Tag name -> string value
    pub tags: HashMap<String, String>,
}

impl RafMetadata {
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: String) {
        self.tags.insert(key.to_string(), value);
    }
}

/// Parse RAF header to extract RAFVersion and RAFCompression
fn parse_raf_header(buffer: &[u8]) -> RafMetadata {
    let mut metadata = RafMetadata::new();

    // RAFVersion at offset 0x3c (4 bytes)
    if buffer.len() >= 0x40 {
        let version = &buffer[0x3c..0x40];
        // Convert to string, trimming null bytes
        let version_str = String::from_utf8_lossy(version)
            .trim_end_matches('\0')
            .to_string();
        if !version_str.is_empty() {
            metadata.insert("RAFVersion", version_str);
        }
    }

    // RAFCompression at offset 0x6c (4 bytes, big-endian)
    // Only valid if starts with 0x00 (not a JPEG header)
    if buffer.len() >= 0x70 && buffer[0x6c] == 0x00 {
        let compression =
            u32::from_be_bytes([buffer[0x6c], buffer[0x6d], buffer[0x6e], buffer[0x6f]]);
        let compression_str = match compression {
            0 => "Uncompressed".to_string(),
            2 => "Lossless".to_string(),
            3 => "Lossy".to_string(),
            _ => format!("{}", compression),
        };
        metadata.insert("RAFCompression", compression_str);
    }

    metadata
}

/// Parse RAF directory to extract image size and layout information
fn parse_raf_directory(buffer: &[u8]) -> RafMetadata {
    let mut metadata = RafMetadata::new();

    // Get RAF directory offset and length from header
    if buffer.len() < 0x64 {
        return metadata;
    }

    let dir_offset =
        u32::from_be_bytes([buffer[0x5c], buffer[0x5d], buffer[0x5e], buffer[0x5f]]) as usize;

    let dir_length =
        u32::from_be_bytes([buffer[0x60], buffer[0x61], buffer[0x62], buffer[0x63]]) as usize;

    if dir_offset == 0 || dir_offset >= buffer.len() || dir_offset + dir_length > buffer.len() {
        return metadata;
    }

    let dir_data = &buffer[dir_offset..];

    // Read number of entries (4 bytes, big-endian)
    if dir_data.len() < 4 {
        return metadata;
    }
    let num_entries =
        u32::from_be_bytes([dir_data[0], dir_data[1], dir_data[2], dir_data[3]]) as usize;

    if num_entries > 256 {
        return metadata; // Sanity check
    }

    // Parse each directory entry: tag (2 bytes) + length (2 bytes) + data
    let mut pos = 4;
    for _ in 0..num_entries {
        if pos + 4 > dir_data.len() {
            break;
        }

        let tag = u16::from_be_bytes([dir_data[pos], dir_data[pos + 1]]);
        let len = u16::from_be_bytes([dir_data[pos + 2], dir_data[pos + 3]]) as usize;
        pos += 4;

        if pos + len > dir_data.len() {
            break;
        }

        let value_data = &dir_data[pos..pos + len];
        pos += len;

        match tag {
            0x100 => {
                // RawImageFullSize (2 x int16u, height then width)
                if len >= 4 {
                    let height = u16::from_be_bytes([value_data[0], value_data[1]]);
                    let width = u16::from_be_bytes([value_data[2], value_data[3]]);
                    metadata.insert("RawImageFullSize", format!("{}x{}", width, height));
                    metadata.insert("RawImageFullWidth", format!("{}", width));
                    metadata.insert("RawImageFullHeight", format!("{}", height));
                }
            }
            0x110 => {
                // RawImageCropTopLeft (2 x int16u, top margin then left margin)
                if len >= 4 {
                    let top = u16::from_be_bytes([value_data[0], value_data[1]]);
                    let left = u16::from_be_bytes([value_data[2], value_data[3]]);
                    metadata.insert("RawImageCropTopLeft", format!("{} {}", top, left));
                }
            }
            0x111 => {
                // RawImageCroppedSize (2 x int16u, height then width)
                if len >= 4 {
                    let height = u16::from_be_bytes([value_data[0], value_data[1]]);
                    let width = u16::from_be_bytes([value_data[2], value_data[3]]);
                    metadata.insert("RawImageCroppedSize", format!("{}x{}", width, height));
                    metadata.insert("RawImageCroppedWidth", format!("{}", width));
                    metadata.insert("RawImageCroppedHeight", format!("{}", height));
                }
            }
            0x121 => {
                // RawImageSize (2 x int16u, height then width)
                if len >= 4 {
                    let height = u16::from_be_bytes([value_data[0], value_data[1]]);
                    let width = u16::from_be_bytes([value_data[2], value_data[3]]);
                    metadata.insert("RawImageWidth", format!("{}", width));
                    metadata.insert("RawImageHeight", format!("{}", height));
                }
            }
            0x130 => {
                // FujiLayout (variable length int8u)
                if !value_data.is_empty() {
                    let layout: Vec<String> = value_data.iter().map(|b| format!("{}", b)).collect();
                    metadata.insert("FujiLayout", layout.join(" "));
                }
            }
            0x131 => {
                // XTransLayout (36 int8u -> 6x6 grid with RGB mapping)
                // ExifTool uses: 0=R, 1=G, 2=B (from PrintConv => '$val =~ tr/012 /RGB/d')
                if len >= 36 {
                    let rgb_chars = ['R', 'G', 'B'];
                    let mut layout_str = String::new();
                    for row in 0..6 {
                        if row > 0 {
                            layout_str.push(' ');
                        }
                        for col in 0..6 {
                            let idx = row * 6 + col;
                            let ch = rgb_chars.get(value_data[idx] as usize).unwrap_or(&'?');
                            layout_str.push(*ch);
                        }
                    }
                    metadata.insert("XTransLayout", layout_str);
                }
            }
            0x9650 => {
                // RawExposureBias (rational32s - 2 x int16s in first 4 bytes)
                if len >= 4 {
                    let num = i16::from_be_bytes([value_data[0], value_data[1]]) as f64;
                    let den = i16::from_be_bytes([value_data[2], value_data[3]]) as f64;
                    if den != 0.0 {
                        let val = num / den;
                        // Format like ExifTool: +/-X.X or 0
                        if val == 0.0 {
                            metadata.insert("RawExposureBias", "0".to_string());
                        } else {
                            metadata.insert("RawExposureBias", format!("{:+.1}", val));
                        }
                    }
                }
            }
            0xc000 => {
                // RAFData - binary data containing RawImageWidth and RawImageHeight
                // This is little-endian data (unlike the rest which is big-endian)
                // ExifTool logic: try multiple offsets for width/height
                if len >= 16 {
                    let v0 = u32::from_le_bytes([
                        value_data[0],
                        value_data[1],
                        value_data[2],
                        value_data[3],
                    ]);
                    let v4 = u32::from_le_bytes([
                        value_data[4],
                        value_data[5],
                        value_data[6],
                        value_data[7],
                    ]);
                    let v8 = u32::from_le_bytes([
                        value_data[8],
                        value_data[9],
                        value_data[10],
                        value_data[11],
                    ]);

                    // Try offset 0 for width, fallback to offset 4, then offset 8
                    let (width, height_offset) = if v0 > 0 && v0 < 10000 {
                        (Some(v0), 4) // width at 0, height at 4
                    } else if v4 > 0 && v4 < 10000 {
                        (Some(v4), 8) // width at 4, height at 8
                    } else if v8 > 0 && v8 < 10000 {
                        (Some(v8), 12) // width at 8, height at 12
                    } else {
                        (None, 0)
                    };

                    if let Some(w) = width {
                        metadata.insert("RawImageWidth", format!("{}", w));

                        // Get height from next offset
                        if height_offset + 4 <= len {
                            let height = u32::from_le_bytes([
                                value_data[height_offset],
                                value_data[height_offset + 1],
                                value_data[height_offset + 2],
                                value_data[height_offset + 3],
                            ]);
                            if height > 0 && height < 10000 {
                                metadata.insert("RawImageHeight", format!("{}", height));
                            }
                        }
                    }
                }
            }
            // WB_GRGBLevels tags (0x2xxx) - int16u[4] in GRGB order
            0x2000 => {
                // WB_GRGBLevelsAuto
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsAuto", formatted);
                }
            }
            0x2100 => {
                // WB_GRGBLevelsDaylight
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsDaylight", formatted);
                }
            }
            0x2200 => {
                // WB_GRGBLevelsCloudy
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsCloudy", formatted);
                }
            }
            0x2300 => {
                // WB_GRGBLevelsDaylightFluor
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsDaylightFluor", formatted);
                }
            }
            0x2301 => {
                // WB_GRGBLevelsDayWhiteFluor
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsDayWhiteFluor", formatted);
                }
            }
            0x2302 => {
                // WB_GRGBLevelsWhiteFluorescent
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsWhiteFluorescent", formatted);
                }
            }
            0x2310 => {
                // WB_GRGBLevelsWarmWhiteFluor
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsWarmWhiteFluor", formatted);
                }
            }
            0x2311 => {
                // WB_GRGBLevelsLivingRoomWarmWhiteFluor
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsLivingRoomWarmWhiteFluor", formatted);
                }
            }
            0x2400 => {
                // WB_GRGBLevelsTungsten
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevelsTungsten", formatted);
                }
            }
            0x2ff0 => {
                // WB_GRGBLevels (base) - also compute RedBalance and BlueBalance
                if len >= 8 {
                    let formatted = format_wb_grgb_levels(value_data);
                    metadata.insert("WB_GRGBLevels", formatted);

                    // Compute RedBalance and BlueBalance from GRGB values
                    let g1 = u16::from_be_bytes([value_data[0], value_data[1]]) as f64;
                    let r = u16::from_be_bytes([value_data[2], value_data[3]]) as f64;
                    let b = u16::from_be_bytes([value_data[6], value_data[7]]) as f64;

                    if g1 > 0.0 {
                        let red_balance = r / g1;
                        let blue_balance = b / g1;
                        metadata.insert("RedBalance", format_float_6places(red_balance));
                        metadata.insert("BlueBalance", format_float_6places(blue_balance));
                    }
                }
            }
            _ => {}
        }
    }

    metadata
}

/// Extract RAF-specific metadata from RAF header and directory
pub fn extract_raf_metadata<R: Read + Seek>(mut reader: R) -> ExifResult<RafMetadata> {
    // Read enough of the file to cover header and directory
    reader.seek(SeekFrom::Start(0))?;

    // First read the header to find directory offset
    let mut header = [0u8; 0x100];
    reader.read_exact(&mut header)?;

    // Check signature
    if &header[0..15] != RAF_SIGNATURE {
        return Ok(RafMetadata::new());
    }

    // Parse header
    let mut metadata = parse_raf_header(&header);

    // Get directory offset
    let dir_offset =
        u32::from_be_bytes([header[0x5c], header[0x5d], header[0x5e], header[0x5f]]) as usize;

    let dir_length =
        u32::from_be_bytes([header[0x60], header[0x61], header[0x62], header[0x63]]) as usize;

    if dir_offset > 0 && dir_length > 0 {
        // Read the directory
        reader.seek(SeekFrom::Start(dir_offset as u64))?;
        let mut dir_data = vec![0u8; dir_length.min(64 * 1024)]; // Limit to 64KB
        let bytes_read = reader.read(&mut dir_data)?;
        dir_data.truncate(bytes_read);

        // Build a buffer with header + directory for parse_raf_directory
        let mut full_buffer = vec![0u8; dir_offset + bytes_read];
        full_buffer[..0x100].copy_from_slice(&header);
        if dir_offset + bytes_read <= full_buffer.len() {
            full_buffer[dir_offset..dir_offset + bytes_read].copy_from_slice(&dir_data);
        }

        let dir_metadata = parse_raf_directory(&full_buffer);
        for (k, v) in dir_metadata.tags {
            metadata.insert(&k, v);
        }
    }

    // Parse FujiIFD for WB levels and other parameters
    let fuji_ifd_offset =
        u32::from_be_bytes([header[0x64], header[0x65], header[0x66], header[0x67]]) as u64;
    let fuji_ifd_len =
        u32::from_be_bytes([header[0x68], header[0x69], header[0x6a], header[0x6b]]) as usize;

    if fuji_ifd_offset > 0 && fuji_ifd_len > 0 {
        reader.seek(SeekFrom::Start(fuji_ifd_offset))?;
        let mut ifd_data = vec![0u8; fuji_ifd_len.min(64 * 1024)];
        let bytes_read = reader.read(&mut ifd_data)?;
        ifd_data.truncate(bytes_read);

        if let Some(fuji_tags) = parse_fuji_ifd(&ifd_data) {
            for (k, v) in fuji_tags.tags {
                metadata.insert(&k, v);
            }
        }
    }

    Ok(metadata)
}

/// Parse FujiIFD TIFF structure to extract WB levels and other parameters
fn parse_fuji_ifd(data: &[u8]) -> Option<RafMetadata> {
    if data.len() < 8 {
        return None;
    }

    // Check TIFF header (II or MM)
    let little_endian = match (&data[0], &data[1]) {
        (b'I', b'I') => true,
        (b'M', b'M') => false,
        _ => return None,
    };

    let read_u16 = |d: &[u8]| -> u16 {
        if little_endian {
            u16::from_le_bytes([d[0], d[1]])
        } else {
            u16::from_be_bytes([d[0], d[1]])
        }
    };

    let read_u32 = |d: &[u8]| -> u32 {
        if little_endian {
            u32::from_le_bytes([d[0], d[1], d[2], d[3]])
        } else {
            u32::from_be_bytes([d[0], d[1], d[2], d[3]])
        }
    };

    let read_i32 = |d: &[u8]| -> i32 {
        if little_endian {
            i32::from_le_bytes([d[0], d[1], d[2], d[3]])
        } else {
            i32::from_be_bytes([d[0], d[1], d[2], d[3]])
        }
    };

    // Get IFD0 offset
    let ifd0_offset = read_u32(&data[4..8]) as usize;
    if ifd0_offset + 2 > data.len() {
        return None;
    }

    let mut metadata = RafMetadata::new();

    // Parse IFD0
    let num_entries = read_u16(&data[ifd0_offset..]) as usize;
    if num_entries == 0 || ifd0_offset + 2 + num_entries * 12 > data.len() {
        return None;
    }

    for i in 0..num_entries {
        let entry_offset = ifd0_offset + 2 + i * 12;
        let tag = read_u16(&data[entry_offset..]);
        let dtype = read_u16(&data[entry_offset + 2..]);
        let _count = read_u32(&data[entry_offset + 4..]) as usize;
        let value_offset = read_u32(&data[entry_offset + 8..]) as usize;

        // Look for SubIFD pointer (0xf000)
        if tag == 0xf000 && dtype == 13 {
            // Parse SubIFD
            if value_offset + 2 <= data.len() {
                let sub_entries = read_u16(&data[value_offset..]) as usize;
                if value_offset + 2 + sub_entries * 12 <= data.len() {
                    for j in 0..sub_entries {
                        let se_offset = value_offset + 2 + j * 12;
                        let stag = read_u16(&data[se_offset..]);
                        let sdtype = read_u16(&data[se_offset + 2..]);
                        let scount = read_u32(&data[se_offset + 4..]) as usize;
                        let svalue_offset = read_u32(&data[se_offset + 8..]) as usize;

                        match stag {
                            0xf00a => {
                                // BlackLevel (LONG array)
                                if sdtype == 4 && svalue_offset + scount * 4 <= data.len() {
                                    let vals: Vec<String> = (0..scount)
                                        .map(|k| {
                                            read_u32(&data[svalue_offset + k * 4..]).to_string()
                                        })
                                        .collect();
                                    metadata.insert("BlackLevel", vals.join(" "));
                                }
                            }
                            0xf00c => {
                                // WB_GRBLevelsStandard (LONG array)
                                if sdtype == 4 && svalue_offset + scount * 4 <= data.len() {
                                    let vals: Vec<String> = (0..scount)
                                        .map(|k| {
                                            read_u32(&data[svalue_offset + k * 4..]).to_string()
                                        })
                                        .collect();
                                    metadata.insert("WB_GRBLevelsStandard", vals.join(" "));
                                }
                            }
                            0xf00d => {
                                // WB_GRBLevelsAuto (LONG array)
                                if sdtype == 4 {
                                    let vals: Vec<String> = if scount * 4 <= 4 {
                                        // Value in offset field
                                        let offset_bytes = if little_endian {
                                            (svalue_offset as u32).to_le_bytes()
                                        } else {
                                            (svalue_offset as u32).to_be_bytes()
                                        };
                                        (0..scount.min(1))
                                            .map(|_| read_u32(&offset_bytes).to_string())
                                            .collect()
                                    } else if svalue_offset + scount * 4 <= data.len() {
                                        (0..scount)
                                            .map(|k| {
                                                read_u32(&data[svalue_offset + k * 4..]).to_string()
                                            })
                                            .collect()
                                    } else {
                                        continue;
                                    };
                                    metadata.insert("WB_GRBLevelsAuto", vals.join(" "));
                                }
                            }
                            0xf00e => {
                                // WB_GRBLevels (LONG array)
                                if sdtype == 4 {
                                    let vals: Vec<String> = if scount * 4 <= 4 {
                                        let offset_bytes = if little_endian {
                                            (svalue_offset as u32).to_le_bytes()
                                        } else {
                                            (svalue_offset as u32).to_be_bytes()
                                        };
                                        (0..scount.min(1))
                                            .map(|_| read_u32(&offset_bytes).to_string())
                                            .collect()
                                    } else if svalue_offset + scount * 4 <= data.len() {
                                        (0..scount)
                                            .map(|k| {
                                                read_u32(&data[svalue_offset + k * 4..]).to_string()
                                            })
                                            .collect()
                                    } else {
                                        continue;
                                    };
                                    metadata.insert("WB_GRBLevels", vals.join(" "));
                                }
                            }
                            0xf00b => {
                                // GeometricDistortionParams (SRATIONAL array)
                                if sdtype == 10 && svalue_offset + scount * 8 <= data.len() {
                                    let vals: Vec<String> = (0..scount)
                                        .map(|k| {
                                            let offset = svalue_offset + k * 8;
                                            let num = read_i32(&data[offset..]);
                                            let den = read_i32(&data[offset + 4..]);
                                            if den != 0 {
                                                let val = num as f64 / den as f64;
                                                format_float_10places(val)
                                            } else {
                                                "0".to_string()
                                            }
                                        })
                                        .collect();
                                    metadata.insert("GeometricDistortionParams", vals.join(" "));
                                }
                            }
                            0xf00f => {
                                // ChromaticAberrationParams (SRATIONAL array)
                                if sdtype == 10 && svalue_offset + scount * 8 <= data.len() {
                                    let vals: Vec<String> = (0..scount)
                                        .map(|k| {
                                            let offset = svalue_offset + k * 8;
                                            let num = read_i32(&data[offset..]);
                                            let den = read_i32(&data[offset + 4..]);
                                            if den != 0 {
                                                let val = num as f64 / den as f64;
                                                format_float_10places(val)
                                            } else {
                                                "0".to_string()
                                            }
                                        })
                                        .collect();
                                    metadata.insert("ChromaticAberrationParams", vals.join(" "));
                                }
                            }
                            0xf010 => {
                                // VignettingParams (SRATIONAL array)
                                if sdtype == 10 && svalue_offset + scount * 8 <= data.len() {
                                    let vals: Vec<String> = (0..scount)
                                        .map(|k| {
                                            let offset = svalue_offset + k * 8;
                                            let num = read_i32(&data[offset..]);
                                            let den = read_i32(&data[offset + 4..]);
                                            if den != 0 {
                                                let val = num as f64 / den as f64;
                                                format_float_10places(val)
                                            } else {
                                                "0".to_string()
                                            }
                                        })
                                        .collect();
                                    metadata.insert("VignettingParams", vals.join(" "));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Some(metadata)
}

/// Extract EXIF APP1 segment from a Fujifilm RAF file
/// RAF files contain embedded JPEG data with EXIF information
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Verify RAF signature
    let mut signature = [0u8; 15];
    reader.read_exact(&mut signature)?;

    if signature != RAF_SIGNATURE {
        return Err(ExifError::Format("Not a valid RAF file".to_string()));
    }

    // Reset to beginning to search for embedded JPEG
    reader.seek(SeekFrom::Start(0))?;

    // Read the file into memory (RAF files typically have EXIF in the first few KB)
    // We'll read up to 1MB to be safe
    let mut buffer = vec![0u8; 1024 * 1024];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Search for JPEG SOI marker followed by APP1 marker (FF D8 FF E1)
    let mut pos = 0;
    while pos + 4 <= buffer.len() {
        if buffer[pos] == 0xFF
            && buffer[pos + 1] == 0xD8
            && buffer[pos + 2] == 0xFF
            && buffer[pos + 3] == 0xE1
        {
            // Found JPEG with APP1 marker, now read the APP1 segment
            pos += 4; // Skip the markers

            if pos + 2 > buffer.len() {
                return Err(ExifError::Format("Truncated APP1 segment".to_string()));
            }

            // Read APP1 length (2 bytes, big-endian)
            let app1_length = u16::from_be_bytes([buffer[pos], buffer[pos + 1]]) as usize - 2;
            pos += 2;

            if pos + app1_length > buffer.len() {
                return Err(ExifError::Format(
                    "APP1 segment extends beyond buffer".to_string(),
                ));
            }

            // Extract APP1 data
            let app1_data = buffer[pos..pos + app1_length].to_vec();

            // Verify "Exif\0\0" marker
            if app1_data.len() < 6 || &app1_data[0..6] != b"Exif\0\0" {
                return Err(ExifError::Format(
                    "Not a valid EXIF APP1 segment in RAF".to_string(),
                ));
            }

            return Ok(app1_data);
        }
        pos += 1;
    }

    Err(ExifError::Format(
        "No embedded EXIF data found in RAF file".to_string(),
    ))
}
