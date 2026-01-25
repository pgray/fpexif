//! WebAssembly bindings for fpexif
//!
//! This module provides JavaScript-compatible bindings for the fpexif library
//! when compiled to WebAssembly.

use wasm_bindgen::prelude::*;

/// Parse EXIF data from a byte array
///
/// # Arguments
/// * `data` - The image file data as a byte array
///
/// # Returns
/// A JSON string containing the parsed EXIF data, or an error message
#[wasm_bindgen]
pub fn parse_exif(data: &[u8]) -> Result<String, JsValue> {
    let parser = crate::ExifParser::new();

    match parser.parse_bytes(data) {
        Ok(exif_data) => {
            // Convert EXIF data to a JSON-serializable format
            let mut result = serde_json::Map::new();
            result.insert(
                "tag_count".to_string(),
                serde_json::Value::from(exif_data.len()),
            );

            let mut tags = serde_json::Map::new();
            for (tag_id, value) in exif_data.iter() {
                let tag_name = format!("{:?}", tag_id);
                let tag_value = format!("{:?}", value);
                tags.insert(tag_name, serde_json::Value::String(tag_value));
            }
            result.insert("tags".to_string(), serde_json::Value::Object(tags));

            serde_json::to_string(&result)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&format!("EXIF parsing error: {}", e))),
    }
}

/// Parse EXIF data and return exiftool-compatible JSON format
///
/// This format matches exiftool's `-json` output for easy comparison.
///
/// # Arguments
/// * `data` - The image file data as a byte array
///
/// # Returns
/// A JSON string in exiftool format (array with single object containing tags)
#[wasm_bindgen]
pub fn parse_exif_json(data: &[u8]) -> Result<String, JsValue> {
    let parser = crate::ExifParser::new();
    let file_size = data.len() as u64;

    match parser.parse_bytes(data) {
        Ok(exif_data) => {
            let json_value = crate::output::to_exiftool_json(&exif_data, None, Some(file_size));
            serde_json::to_string_pretty(&json_value)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&format!("EXIF parsing error: {}", e))),
    }
}

/// Parse EXIF data and return a specific tag value (case-insensitive)
///
/// This is a convenience function for extracting single tags without parsing the full JSON.
/// Tag names are case-insensitive and underscore-insensitive.
///
/// # Arguments
/// * `data` - The image file data as a byte array
/// * `tag_name` - The tag name to extract (e.g., "make", "Make", "shutter_speed", "ShutterSpeed")
///
/// # Returns
/// The tag value as a JSON string, or an error if tag not found
///
/// # Examples
/// ```javascript
/// // All these work:
/// parse_exif_get_tag(data, "make")
/// parse_exif_get_tag(data, "Make")
/// parse_exif_get_tag(data, "shutter_speed")
/// parse_exif_get_tag(data, "ShutterSpeed")
/// ```
#[wasm_bindgen]
pub fn parse_exif_get_tag(data: &[u8], tag_name: &str) -> Result<String, JsValue> {
    let parser = crate::ExifParser::new();
    let file_size = data.len() as u64;

    match parser.parse_bytes(data) {
        Ok(exif_data) => {
            let json = crate::output::to_exiftool_json(&exif_data, None, Some(file_size));

            match crate::output::get_tag_value(&json, tag_name) {
                Some(value) => {
                    // Convert JSON value to string
                    let result = match value {
                        serde_json::Value::String(s) => s,
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => serde_json::to_string(&value).unwrap_or_default(),
                    };
                    Ok(result)
                }
                None => Err(JsValue::from_str(&format!("Tag '{}' not found", tag_name))),
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("Parse error: {}", e))),
    }
}

/// Get the version of the fpexif library
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// List embedded JPEGs in a RAW file
///
/// # Arguments
/// * `data` - The RAW file data as a byte array
///
/// # Returns
/// A JSON string containing information about embedded JPEGs
#[wasm_bindgen]
pub fn list_jpegs(data: &[u8]) -> Result<String, JsValue> {
    use crate::extract::{extract_jpegs, JpegType};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    match extract_jpegs(cursor, JpegType::All) {
        Ok(jpegs) => {
            let infos: Vec<serde_json::Value> = jpegs
                .iter()
                .map(|(info, _)| {
                    let mut obj = serde_json::Map::new();
                    obj.insert("offset".to_string(), serde_json::Value::from(info.offset));
                    obj.insert("length".to_string(), serde_json::Value::from(info.length));
                    obj.insert(
                        "description".to_string(),
                        serde_json::Value::String(info.description.clone()),
                    );
                    if let Some((w, h)) = info.dimensions {
                        obj.insert("width".to_string(), serde_json::Value::from(w));
                        obj.insert("height".to_string(), serde_json::Value::from(h));
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();

            serde_json::to_string(&infos)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&format!("JPEG extraction error: {}", e))),
    }
}

/// Extract the largest embedded JPEG (preview) from a RAW file
///
/// # Arguments
/// * `data` - The RAW file data as a byte array
///
/// # Returns
/// The JPEG data as a byte array, or an error
#[wasm_bindgen]
pub fn extract_preview(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    use crate::extract::{extract_jpegs, JpegType};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    match extract_jpegs(cursor, JpegType::Preview) {
        Ok(mut jpegs) => {
            if let Some((_, jpeg_data)) = jpegs.pop() {
                Ok(jpeg_data)
            } else {
                Err(JsValue::from_str("No preview JPEG found"))
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("JPEG extraction error: {}", e))),
    }
}

/// Extract the smallest embedded JPEG (thumbnail) from a RAW file
///
/// # Arguments
/// * `data` - The RAW file data as a byte array
///
/// # Returns
/// The JPEG data as a byte array, or an error
#[wasm_bindgen]
pub fn extract_thumbnail(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    use crate::extract::{extract_jpegs, JpegType};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    match extract_jpegs(cursor, JpegType::Thumbnail) {
        Ok(mut jpegs) => {
            if let Some((_, jpeg_data)) = jpegs.pop() {
                Ok(jpeg_data)
            } else {
                Err(JsValue::from_str("No thumbnail JPEG found"))
            }
        }
        Err(e) => Err(JsValue::from_str(&format!("JPEG extraction error: {}", e))),
    }
}

/// Extract all embedded JPEGs from a RAW file
///
/// # Arguments
/// * `data` - The RAW file data as a byte array
///
/// # Returns
/// A JSON string with base64-encoded JPEG data for each embedded image
#[wasm_bindgen]
pub fn extract_all_jpegs(data: &[u8]) -> Result<String, JsValue> {
    use crate::extract::{extract_jpegs, JpegType};
    use base64::{engine::general_purpose::STANDARD, Engine};
    use std::io::Cursor;

    let cursor = Cursor::new(data);
    match extract_jpegs(cursor, JpegType::All) {
        Ok(jpegs) => {
            let results: Vec<serde_json::Value> = jpegs
                .iter()
                .map(|(info, jpeg_data)| {
                    let mut obj = serde_json::Map::new();
                    obj.insert(
                        "description".to_string(),
                        serde_json::Value::String(info.description.clone()),
                    );
                    obj.insert("length".to_string(), serde_json::Value::from(info.length));
                    if let Some((w, h)) = info.dimensions {
                        obj.insert("width".to_string(), serde_json::Value::from(w));
                        obj.insert("height".to_string(), serde_json::Value::from(h));
                    }
                    obj.insert(
                        "data".to_string(),
                        serde_json::Value::String(STANDARD.encode(jpeg_data)),
                    );
                    serde_json::Value::Object(obj)
                })
                .collect();

            serde_json::to_string(&results)
                .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
        }
        Err(e) => Err(JsValue::from_str(&format!("JPEG extraction error: {}", e))),
    }
}
