// formats/cr3.rs - Canon CR3 format EXIF extraction
// CR3 uses ISO Base Media File Format (similar to MP4)
// EXIF data is stored in CMT (Canon Metadata) boxes:
//   CMT1 - IFD0 (basic image info)
//   CMT2 - EXIF IFD (detailed EXIF tags)
//   CMT3 - MakerNote IFD (Canon-specific tags)
//   CMT4 - GPS IFD (if present)
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Extract EXIF APP1 segment from a Canon CR3 file
/// CR3 files use ISO Base Media File Format and store EXIF in CMT metadata boxes
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read file type box (first 12 bytes minimum)
    let mut header = [0u8; 20];
    reader.read_exact(&mut header)?;

    // Verify ftyp box
    if &header[4..8] != b"ftyp" {
        return Err(ExifError::Format("Not a valid CR3 file".to_string()));
    }

    // Verify CR3 brand (crx )
    if &header[8..12] != b"crx " {
        return Err(ExifError::Format(
            "Not a Canon CR3 file (wrong brand)".to_string(),
        ));
    }

    // Reset to beginning to search for metadata
    reader.seek(SeekFrom::Start(0))?;

    // Read up to 10MB to find EXIF data (CR3 files can be very large)
    // EXIF metadata is typically in the first few MB
    let mut buffer = vec![0u8; 10 * 1024 * 1024];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Find all CMT boxes and extract their TIFF data
    // CMT1 = IFD0, CMT2 = EXIF IFD, CMT3 = MakerNotes, CMT4 = GPS
    let cmt_boxes = [b"CMT1", b"CMT2", b"CMT3", b"CMT4"];
    let mut cmt_data: Vec<Option<(usize, usize)>> = vec![None; 4];

    for (i, cmt_tag) in cmt_boxes.iter().enumerate() {
        if let Some(pos) = find_subsequence(&buffer, *cmt_tag) {
            // CMT box structure: size(4) + "CMT#"(4) + TIFF data
            // The TIFF header starts immediately after "CMT#"
            let tiff_start = pos + 4; // Skip "CMT#" marker

            if tiff_start + 8 <= buffer.len() {
                // Verify TIFF header
                if (buffer[tiff_start] == b'I' && buffer[tiff_start + 1] == b'I')
                    || (buffer[tiff_start] == b'M' && buffer[tiff_start + 1] == b'M')
                {
                    // Find the end by looking for next CMT box or end of metadata area
                    let mut end = buffer.len();
                    for next_tag in cmt_boxes.iter() {
                        if let Some(next_pos) = find_subsequence(&buffer[tiff_start..], *next_tag) {
                            let next_abs = tiff_start + next_pos;
                            if next_abs > tiff_start && next_abs < end {
                                // Back up to find the size field before the CMT tag
                                if next_abs >= 4 {
                                    end = next_abs - 4;
                                } else {
                                    end = next_abs;
                                }
                            }
                        }
                    }
                    // Also look for other markers that indicate end of CMT area
                    if let Some(free_pos) = find_subsequence(&buffer[tiff_start..], b"free") {
                        let free_abs = tiff_start + free_pos;
                        if free_abs > tiff_start && free_abs < end {
                            end = free_abs;
                        }
                    }
                    cmt_data[i] = Some((tiff_start, end));
                }
            }
        }
    }

    // CR3 stores EXIF in separate CMT boxes that need to be merged:
    // CMT1 = IFD0 (Make, Model, etc.), CMT2 = EXIF IFD, CMT3 = MakerNotes, CMT4 = GPS
    // We need to merge CMT1 (IFD0), CMT2 (EXIF), and CMT3 (MakerNotes) into a single structure

    // Prefer CMT1 first since it has IFD0 with basic tags
    // Then we'll need to append CMT2 data for full EXIF coverage
    // CMT3 contains Canon MakerNotes which need to be included
    if let Some((cmt1_start, cmt1_end)) = cmt_data[0] {
        if let Some((cmt2_start, cmt2_end)) = cmt_data[1] {
            // We have both CMT1 and CMT2 - merge them
            // Build a synthetic TIFF structure:
            // - Copy CMT1 as IFD0
            // - Patch the ExifOffset pointer to point to CMT2 data appended after
            // - Include CMT3 (MakerNotes) if available
            let cmt3_bounds = cmt_data[2]; // CMT3 = MakerNotes
            return merge_cmt_boxes(
                &buffer,
                cmt1_start,
                cmt1_end,
                cmt2_start,
                cmt2_end,
                cmt3_bounds,
            );
        }

        // Only CMT1 available
        let tiff_data = &buffer[cmt1_start..cmt1_end.min(cmt1_start + 512 * 1024)];
        let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
        app1_data.extend_from_slice(b"Exif\0\0");
        app1_data.extend_from_slice(tiff_data);
        return Ok(app1_data);
    }

    // Only CMT2 available (unlikely but handle it)
    if let Some((start, end)) = cmt_data[1] {
        let tiff_data = &buffer[start..end.min(start + 512 * 1024)];
        let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
        app1_data.extend_from_slice(b"Exif\0\0");
        app1_data.extend_from_slice(tiff_data);
        return Ok(app1_data);
    }

    // Fallback: Search for "Exif\0\0" marker
    if let Some(pos) = find_subsequence(&buffer, b"Exif\0\0") {
        let tiff_start = pos + 6;
        if tiff_start + 8 <= buffer.len()
            && ((buffer[tiff_start] == b'I' && buffer[tiff_start + 1] == b'I')
                || (buffer[tiff_start] == b'M' && buffer[tiff_start + 1] == b'M'))
        {
            let max_len = std::cmp::min(1024 * 1024, buffer.len() - pos);
            return Ok(buffer[pos..pos + max_len].to_vec());
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
        "No EXIF data found in CR3 file".to_string(),
    ))
}

/// Helper function to find a subsequence in a byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Merge CMT1 (IFD0), CMT2 (EXIF IFD), and CMT3 (MakerNotes) into a single TIFF structure
/// CR3 stores these as separate TIFF blocks, but we need to combine them
/// into a standard TIFF structure where IFD0 has an ExifOffset pointer to the EXIF IFD
/// and the EXIF IFD has a MakerNote pointer to the Canon MakerNotes
fn merge_cmt_boxes(
    buffer: &[u8],
    cmt1_start: usize,
    cmt1_end: usize,
    cmt2_start: usize,
    cmt2_end: usize,
    cmt3_bounds: Option<(usize, usize)>,
) -> ExifResult<Vec<u8>> {
    // Get the TIFF data from CMT boxes
    let cmt1_data = &buffer[cmt1_start..cmt1_end.min(cmt1_start + 512 * 1024)];
    let cmt2_data = &buffer[cmt2_start..cmt2_end.min(cmt2_start + 512 * 1024)];
    let cmt3_data: Option<&[u8]> =
        cmt3_bounds.map(|(start, end)| &buffer[start..end.min(start + 512 * 1024)]);

    // Both CMT boxes have their own TIFF headers - verify they match in endianness
    if cmt1_data.len() < 8 || cmt2_data.len() < 8 {
        // Not enough data, just return CMT1
        let mut app1_data = Vec::with_capacity(6 + cmt1_data.len());
        app1_data.extend_from_slice(b"Exif\0\0");
        app1_data.extend_from_slice(cmt1_data);
        return Ok(app1_data);
    }

    let is_little_endian = cmt1_data[0] == b'I';

    // Parse CMT1 IFD0 to find the structure
    let ifd0_offset = if is_little_endian {
        u32::from_le_bytes([cmt1_data[4], cmt1_data[5], cmt1_data[6], cmt1_data[7]]) as usize
    } else {
        u32::from_be_bytes([cmt1_data[4], cmt1_data[5], cmt1_data[6], cmt1_data[7]]) as usize
    };

    if ifd0_offset + 2 > cmt1_data.len() {
        let mut app1_data = Vec::with_capacity(6 + cmt1_data.len());
        app1_data.extend_from_slice(b"Exif\0\0");
        app1_data.extend_from_slice(cmt1_data);
        return Ok(app1_data);
    }

    let num_entries = if is_little_endian {
        u16::from_le_bytes([cmt1_data[ifd0_offset], cmt1_data[ifd0_offset + 1]]) as usize
    } else {
        u16::from_be_bytes([cmt1_data[ifd0_offset], cmt1_data[ifd0_offset + 1]]) as usize
    };

    // Find where IFD0 entries end (for locating next IFD pointer)
    let entries_start = ifd0_offset + 2;
    let ifd0_entries_end = entries_start + num_entries * 12;

    // Check if ExifOffset (0x8769) already exists
    let mut has_exif_offset = false;
    let mut exif_offset_pos: Option<usize> = None;
    for i in 0..num_entries {
        let entry_offset = entries_start + i * 12;
        if entry_offset + 12 > cmt1_data.len() {
            break;
        }
        let tag_id = if is_little_endian {
            u16::from_le_bytes([cmt1_data[entry_offset], cmt1_data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([cmt1_data[entry_offset], cmt1_data[entry_offset + 1]])
        };
        if tag_id == 0x8769 {
            has_exif_offset = true;
            exif_offset_pos = Some(entry_offset + 8);
            break;
        }
    }

    // CMT2 structure info
    let cmt2_ifd_offset = if is_little_endian {
        u32::from_le_bytes([cmt2_data[4], cmt2_data[5], cmt2_data[6], cmt2_data[7]]) as usize
    } else {
        u32::from_be_bytes([cmt2_data[4], cmt2_data[5], cmt2_data[6], cmt2_data[7]]) as usize
    };

    if has_exif_offset {
        // CMT1 already has ExifOffset - just patch it and append CMT2
        let cmt1_len = cmt1_data.len();
        let new_exif_offset = cmt1_len as u32;

        let mut merged = Vec::with_capacity(cmt1_len + cmt2_data.len());
        merged.extend_from_slice(cmt1_data);

        // Patch ExifOffset to point to appended CMT2 IFD
        if let Some(offset_pos) = exif_offset_pos {
            if is_little_endian {
                let bytes = new_exif_offset.to_le_bytes();
                merged[offset_pos..offset_pos + 4].copy_from_slice(&bytes);
            } else {
                let bytes = new_exif_offset.to_be_bytes();
                merged[offset_pos..offset_pos + 4].copy_from_slice(&bytes);
            }
        }

        // Append CMT2 IFD data starting from its IFD
        if cmt2_ifd_offset < cmt2_data.len() {
            merged.extend_from_slice(&cmt2_data[cmt2_ifd_offset..]);
        }

        let mut app1_data = Vec::with_capacity(6 + merged.len());
        app1_data.extend_from_slice(b"Exif\0\0");
        app1_data.extend_from_slice(&merged);
        return Ok(app1_data);
    }

    // CMT1 doesn't have ExifOffset - need to rebuild IFD0 with added entry
    // Strategy: Build new TIFF with:
    // 1. TIFF header (8 bytes)
    // 2. IFD0 entry count (2 bytes) - original + 1
    // 3. Original IFD0 entries (num_entries * 12 bytes)
    // 4. New ExifOffset entry (12 bytes)
    // 5. Next IFD pointer (4 bytes)
    // 6. IFD0 value data (from original)
    // 7. CMT2 EXIF IFD and value data

    // Calculate where EXIF IFD will be in new structure
    // New IFD0 will be at offset 8 (right after header)
    // New IFD0 size = 2 + (num_entries + 1) * 12 + 4 = 2 + num_entries * 12 + 12 + 4
    let new_ifd0_size = 2 + (num_entries + 1) * 12 + 4;

    // Original IFD0 value data starts after entries + next IFD ptr
    // In new structure, we need to copy value data and adjust pointers

    // For simplicity, append CMT1's data area (everything after IFD0 entries)
    // Then append CMT2's IFD and data

    let cmt1_data_area_start = ifd0_entries_end + 4; // Skip next IFD pointer
    let cmt1_data_area = if cmt1_data_area_start < cmt1_data.len() {
        &cmt1_data[cmt1_data_area_start..]
    } else {
        &[]
    };

    // New structure layout:
    // [0..8]   TIFF header
    // [8..N]   IFD0 (modified with ExifOffset added)
    // [N..]    Original value data from CMT1
    // [M..]    CMT2 IFD and data

    let tiff_header_size = 8;
    let new_ifd0_offset = tiff_header_size;
    let value_data_offset = tiff_header_size + new_ifd0_size;
    let exif_ifd_offset = value_data_offset + cmt1_data_area.len();

    let mut merged = Vec::with_capacity(exif_ifd_offset + cmt2_data.len());

    // 1. TIFF header
    if is_little_endian {
        merged.extend_from_slice(b"II");
        merged.extend_from_slice(&42u16.to_le_bytes());
        merged.extend_from_slice(&(new_ifd0_offset as u32).to_le_bytes());
    } else {
        merged.extend_from_slice(b"MM");
        merged.extend_from_slice(&42u16.to_be_bytes());
        merged.extend_from_slice(&(new_ifd0_offset as u32).to_be_bytes());
    }

    // 2. IFD0 entry count (original + 1)
    let new_num_entries = (num_entries + 1) as u16;
    if is_little_endian {
        merged.extend_from_slice(&new_num_entries.to_le_bytes());
    } else {
        merged.extend_from_slice(&new_num_entries.to_be_bytes());
    }

    // Calculate offset adjustment for value pointers
    // Original values were relative to CMT1 TIFF start
    // New values will be at value_data_offset
    // Original data started at cmt1_data_area_start (relative to CMT1 start)
    // So adjustment = value_data_offset - cmt1_data_area_start
    let offset_adjustment = value_data_offset as i64 - cmt1_data_area_start as i64;

    // 3. Copy original IFD0 entries, adjusting offsets for values > 4 bytes
    for i in 0..num_entries {
        let entry_offset = entries_start + i * 12;
        if entry_offset + 12 > cmt1_data.len() {
            break;
        }

        let tag_id = if is_little_endian {
            u16::from_le_bytes([cmt1_data[entry_offset], cmt1_data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([cmt1_data[entry_offset], cmt1_data[entry_offset + 1]])
        };
        let tag_type = if is_little_endian {
            u16::from_le_bytes([cmt1_data[entry_offset + 2], cmt1_data[entry_offset + 3]])
        } else {
            u16::from_be_bytes([cmt1_data[entry_offset + 2], cmt1_data[entry_offset + 3]])
        };
        let count = if is_little_endian {
            u32::from_le_bytes([
                cmt1_data[entry_offset + 4],
                cmt1_data[entry_offset + 5],
                cmt1_data[entry_offset + 6],
                cmt1_data[entry_offset + 7],
            ])
        } else {
            u32::from_be_bytes([
                cmt1_data[entry_offset + 4],
                cmt1_data[entry_offset + 5],
                cmt1_data[entry_offset + 6],
                cmt1_data[entry_offset + 7],
            ])
        };
        let value_or_offset = if is_little_endian {
            u32::from_le_bytes([
                cmt1_data[entry_offset + 8],
                cmt1_data[entry_offset + 9],
                cmt1_data[entry_offset + 10],
                cmt1_data[entry_offset + 11],
            ])
        } else {
            u32::from_be_bytes([
                cmt1_data[entry_offset + 8],
                cmt1_data[entry_offset + 9],
                cmt1_data[entry_offset + 10],
                cmt1_data[entry_offset + 11],
            ])
        };

        // Calculate value size to determine if it's inline or an offset
        let type_size = match tag_type {
            1 | 2 | 6 | 7 => 1, // BYTE, ASCII, SBYTE, UNDEFINED
            3 | 8 => 2,         // SHORT, SSHORT
            4 | 9 | 11 => 4,    // LONG, SLONG, FLOAT
            5 | 10 | 12 => 8,   // RATIONAL, SRATIONAL, DOUBLE
            _ => 1,
        };
        let total_size = type_size * count as usize;

        // Write entry
        if is_little_endian {
            merged.extend_from_slice(&tag_id.to_le_bytes());
            merged.extend_from_slice(&tag_type.to_le_bytes());
            merged.extend_from_slice(&count.to_le_bytes());
        } else {
            merged.extend_from_slice(&tag_id.to_be_bytes());
            merged.extend_from_slice(&tag_type.to_be_bytes());
            merged.extend_from_slice(&count.to_be_bytes());
        }

        // Adjust offset if value is stored externally
        if total_size > 4 {
            let new_offset = (value_or_offset as i64 + offset_adjustment) as u32;
            if is_little_endian {
                merged.extend_from_slice(&new_offset.to_le_bytes());
            } else {
                merged.extend_from_slice(&new_offset.to_be_bytes());
            }
        } else {
            // Value stored inline - copy as-is
            merged.extend_from_slice(&cmt1_data[entry_offset + 8..entry_offset + 12]);
        }
    }

    // 4. Add ExifOffset entry (0x8769)
    // ExifOffset tag: type=LONG(4), count=1, value=offset to EXIF IFD
    if is_little_endian {
        merged.extend_from_slice(&0x8769u16.to_le_bytes()); // Tag
        merged.extend_from_slice(&4u16.to_le_bytes()); // Type = LONG
        merged.extend_from_slice(&1u32.to_le_bytes()); // Count
        merged.extend_from_slice(&(exif_ifd_offset as u32).to_le_bytes()); // Value = offset
    } else {
        merged.extend_from_slice(&0x8769u16.to_be_bytes());
        merged.extend_from_slice(&4u16.to_be_bytes());
        merged.extend_from_slice(&1u32.to_be_bytes());
        merged.extend_from_slice(&(exif_ifd_offset as u32).to_be_bytes());
    }

    // 5. Next IFD pointer (0 = no more IFDs)
    merged.extend_from_slice(&[0u8; 4]);

    // 6. Copy CMT1 value data
    merged.extend_from_slice(cmt1_data_area);

    // 7. Append CMT2 EXIF IFD and data
    // CMT2's IFD starts at cmt2_ifd_offset, need to rebase its offsets
    if cmt2_ifd_offset < cmt2_data.len() {
        let cmt2_ifd_data = &cmt2_data[cmt2_ifd_offset..];
        // For offset rebasing in CMT2, the adjustment is:
        // Values in CMT2 are relative to CMT2 TIFF start (0)
        // In merged structure, CMT2 data starts at exif_ifd_offset
        // So new_offset = old_offset - cmt2_ifd_offset + exif_ifd_offset
        let cmt2_offset_adjustment = exif_ifd_offset as i64 - cmt2_ifd_offset as i64;

        // Check if we have CMT3 (MakerNotes) to add
        if let Some(cmt3) = cmt3_data {
            if cmt3.len() >= 8 {
                // CMT3 has its own TIFF structure with IFD entries that use
                // offsets relative to CMT3's TIFF header (at offset 0 within CMT3).
                // We skip the TIFF header and rebase the IFD offsets so they work
                // relative to the start of the MakerNote blob we provide.
                let cmt3_ifd_offset = if is_little_endian {
                    u32::from_le_bytes([cmt3[4], cmt3[5], cmt3[6], cmt3[7]]) as usize
                } else {
                    u32::from_be_bytes([cmt3[4], cmt3[5], cmt3[6], cmt3[7]]) as usize
                };

                if cmt3_ifd_offset < cmt3.len() {
                    let cmt3_ifd_data = &cmt3[cmt3_ifd_offset..];
                    let makernote_size = cmt3_ifd_data.len();

                    // Calculate where MakerNote will be in the merged structure:
                    // - After current merged data
                    // - After CMT2 rebased data (original length + 12 bytes for MakerNote entry)
                    let cmt2_rebased_len = cmt2_ifd_data.len() + 12;
                    let makernote_offset = merged.len() + cmt2_rebased_len;

                    // Rebase CMT3 IFD offsets to be TIFF-relative.
                    // Original offsets are relative to CMT3's TIFF header.
                    // We're copying from cmt3_ifd_offset, so data that was at position X
                    // in CMT3 is now at position (X - cmt3_ifd_offset) in our blob.
                    // But the Canon parser uses TIFF-relative offsets, so the final offset
                    // should be: makernote_offset + (X - cmt3_ifd_offset) = X + (makernote_offset - cmt3_ifd_offset)
                    let cmt3_offset_adjustment = makernote_offset as i64 - cmt3_ifd_offset as i64;
                    let cmt3_rebased =
                        rebase_ifd(cmt3_ifd_data, is_little_endian, cmt3_offset_adjustment);

                    // Rebase CMT2 IFD and add MakerNote entry
                    let cmt2_rebased = rebase_ifd_with_makernote(
                        cmt2_ifd_data,
                        is_little_endian,
                        cmt2_offset_adjustment,
                        makernote_offset,
                        makernote_size,
                    );
                    merged.extend_from_slice(&cmt2_rebased);

                    // Append rebased CMT3 MakerNote data (IFD with adjusted offsets)
                    merged.extend_from_slice(&cmt3_rebased);
                } else {
                    // Invalid cmt3_ifd_offset, just rebase CMT2 normally
                    let cmt2_rebased =
                        rebase_ifd(cmt2_ifd_data, is_little_endian, cmt2_offset_adjustment);
                    merged.extend_from_slice(&cmt2_rebased);
                }
            } else {
                // CMT3 too small, just rebase CMT2 normally
                let cmt2_rebased =
                    rebase_ifd(cmt2_ifd_data, is_little_endian, cmt2_offset_adjustment);
                merged.extend_from_slice(&cmt2_rebased);
            }
        } else {
            // No CMT3, just rebase CMT2 normally
            let cmt2_rebased = rebase_ifd(cmt2_ifd_data, is_little_endian, cmt2_offset_adjustment);
            merged.extend_from_slice(&cmt2_rebased);
        }
    }

    let mut app1_data = Vec::with_capacity(6 + merged.len());
    app1_data.extend_from_slice(b"Exif\0\0");
    app1_data.extend_from_slice(&merged);
    Ok(app1_data)
}

/// Rebase offsets in an IFD and add a MakerNote entry pointing to appended MakerNote data
/// This is used to inject CMT3 (Canon MakerNotes) into the EXIF IFD from CMT2
fn rebase_ifd_with_makernote(
    ifd_data: &[u8],
    is_little_endian: bool,
    offset_adjustment: i64,
    makernote_offset: usize,
    makernote_size: usize,
) -> Vec<u8> {
    if ifd_data.len() < 2 {
        return ifd_data.to_vec();
    }

    let num_entries = if is_little_endian {
        u16::from_le_bytes([ifd_data[0], ifd_data[1]]) as usize
    } else {
        u16::from_be_bytes([ifd_data[0], ifd_data[1]]) as usize
    };

    let entries_end = 2 + num_entries * 12;
    let next_ifd_ptr_end = entries_end + 4;

    if ifd_data.len() < entries_end {
        return ifd_data.to_vec();
    }

    // Build new IFD with one more entry (MakerNote 0x927c)
    // Find the right position to insert MakerNote entry (entries are sorted by tag ID)
    // MakerNote tag 0x927c (37500) should be inserted in sorted order

    let mut new_entries: Vec<[u8; 12]> = Vec::with_capacity(num_entries + 1);
    let mut makernote_inserted = false;

    for i in 0..num_entries {
        let entry_offset = 2 + i * 12;
        if entry_offset + 12 > ifd_data.len() {
            break;
        }

        let tag_id = if is_little_endian {
            u16::from_le_bytes([ifd_data[entry_offset], ifd_data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([ifd_data[entry_offset], ifd_data[entry_offset + 1]])
        };

        // Insert MakerNote entry before any tag > 0x927c
        if !makernote_inserted && tag_id > 0x927c {
            new_entries.push(make_makernote_entry(
                is_little_endian,
                makernote_offset,
                makernote_size,
            ));
            makernote_inserted = true;
        }

        // Skip if this is an existing MakerNote entry (shouldn't happen in CMT2)
        if tag_id == 0x927c {
            continue;
        }

        // Copy entry, adjusting offset if needed
        let mut entry = [0u8; 12];
        entry.copy_from_slice(&ifd_data[entry_offset..entry_offset + 12]);

        let tag_type = if is_little_endian {
            u16::from_le_bytes([entry[2], entry[3]])
        } else {
            u16::from_be_bytes([entry[2], entry[3]])
        };
        let count = if is_little_endian {
            u32::from_le_bytes([entry[4], entry[5], entry[6], entry[7]])
        } else {
            u32::from_be_bytes([entry[4], entry[5], entry[6], entry[7]])
        };

        let type_size = match tag_type {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 => 4,
            5 | 10 | 12 => 8,
            _ => 1,
        };
        let total_size = type_size * count as usize;

        if total_size > 4 {
            let old_offset = if is_little_endian {
                u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]])
            } else {
                u32::from_be_bytes([entry[8], entry[9], entry[10], entry[11]])
            };
            // Add 12 to account for the additional MakerNote entry we're inserting
            let new_offset = (old_offset as i64 + offset_adjustment + 12) as u32;
            if is_little_endian {
                entry[8..12].copy_from_slice(&new_offset.to_le_bytes());
            } else {
                entry[8..12].copy_from_slice(&new_offset.to_be_bytes());
            }
        }

        new_entries.push(entry);
    }

    // Add MakerNote at end if not yet inserted
    if !makernote_inserted {
        new_entries.push(make_makernote_entry(
            is_little_endian,
            makernote_offset,
            makernote_size,
        ));
    }

    // Build result: new entry count + entries + next IFD ptr + remaining data
    let new_num_entries = new_entries.len() as u16;
    let mut result = Vec::with_capacity(
        2 + new_entries.len() * 12 + 4 + ifd_data.len().saturating_sub(next_ifd_ptr_end),
    );

    if is_little_endian {
        result.extend_from_slice(&new_num_entries.to_le_bytes());
    } else {
        result.extend_from_slice(&new_num_entries.to_be_bytes());
    }

    for entry in new_entries {
        result.extend_from_slice(&entry);
    }

    // Next IFD pointer (copy original or zero)
    if next_ifd_ptr_end <= ifd_data.len() {
        result.extend_from_slice(&ifd_data[entries_end..next_ifd_ptr_end]);
    } else {
        result.extend_from_slice(&[0u8; 4]);
    }

    // Copy remaining data (value area)
    if next_ifd_ptr_end < ifd_data.len() {
        result.extend_from_slice(&ifd_data[next_ifd_ptr_end..]);
    }

    result
}

/// Create a MakerNote IFD entry (tag 0x927c, type UNDEFINED, pointing to offset)
fn make_makernote_entry(is_little_endian: bool, offset: usize, size: usize) -> [u8; 12] {
    let mut entry = [0u8; 12];

    // Tag 0x927c = MakerNote
    if is_little_endian {
        entry[0..2].copy_from_slice(&0x927cu16.to_le_bytes());
        // Type 7 = UNDEFINED
        entry[2..4].copy_from_slice(&7u16.to_le_bytes());
        // Count = size of MakerNote data
        entry[4..8].copy_from_slice(&(size as u32).to_le_bytes());
        // Offset to MakerNote data
        entry[8..12].copy_from_slice(&(offset as u32).to_le_bytes());
    } else {
        entry[0..2].copy_from_slice(&0x927cu16.to_be_bytes());
        entry[2..4].copy_from_slice(&7u16.to_be_bytes());
        entry[4..8].copy_from_slice(&(size as u32).to_be_bytes());
        entry[8..12].copy_from_slice(&(offset as u32).to_be_bytes());
    }

    entry
}

/// Rebase offsets in an IFD by adding adjustment to all external value offsets
fn rebase_ifd(ifd_data: &[u8], is_little_endian: bool, offset_adjustment: i64) -> Vec<u8> {
    if ifd_data.len() < 2 {
        return ifd_data.to_vec();
    }

    let num_entries = if is_little_endian {
        u16::from_le_bytes([ifd_data[0], ifd_data[1]]) as usize
    } else {
        u16::from_be_bytes([ifd_data[0], ifd_data[1]]) as usize
    };

    let entries_size = 2 + num_entries * 12 + 4; // count + entries + next IFD ptr
    if ifd_data.len() < entries_size {
        return ifd_data.to_vec();
    }

    let mut result = ifd_data.to_vec();

    for i in 0..num_entries {
        let entry_offset = 2 + i * 12;
        if entry_offset + 12 > result.len() {
            break;
        }

        let tag_type = if is_little_endian {
            u16::from_le_bytes([result[entry_offset + 2], result[entry_offset + 3]])
        } else {
            u16::from_be_bytes([result[entry_offset + 2], result[entry_offset + 3]])
        };
        let count = if is_little_endian {
            u32::from_le_bytes([
                result[entry_offset + 4],
                result[entry_offset + 5],
                result[entry_offset + 6],
                result[entry_offset + 7],
            ])
        } else {
            u32::from_be_bytes([
                result[entry_offset + 4],
                result[entry_offset + 5],
                result[entry_offset + 6],
                result[entry_offset + 7],
            ])
        };

        let type_size = match tag_type {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 => 4,
            5 | 10 | 12 => 8,
            _ => 1,
        };
        let total_size = type_size * count as usize;

        if total_size > 4 {
            let old_offset = if is_little_endian {
                u32::from_le_bytes([
                    result[entry_offset + 8],
                    result[entry_offset + 9],
                    result[entry_offset + 10],
                    result[entry_offset + 11],
                ])
            } else {
                u32::from_be_bytes([
                    result[entry_offset + 8],
                    result[entry_offset + 9],
                    result[entry_offset + 10],
                    result[entry_offset + 11],
                ])
            };
            let new_offset = (old_offset as i64 + offset_adjustment) as u32;
            if is_little_endian {
                let bytes = new_offset.to_le_bytes();
                result[entry_offset + 8..entry_offset + 12].copy_from_slice(&bytes);
            } else {
                let bytes = new_offset.to_be_bytes();
                result[entry_offset + 8..entry_offset + 12].copy_from_slice(&bytes);
            }
        }
    }

    result
}
