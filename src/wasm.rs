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

/// Get the version of the fpexif library
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
