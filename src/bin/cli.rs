// src/bin/cli.rs - Command line interface for fpexif
use clap::{Parser, Subcommand};
use fpexif::{tags, ExifData, ExifParser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fpexif")]
#[command(author = "pgray <pgray@file.photo>")]
#[command(version = "0.1.0")]
#[command(about = "A pure Rust EXIF metadata parser and manipulator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all EXIF tags in a file
    List {
        /// Path to the image file
        #[arg(required = true)]
        file: PathBuf,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Output in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Extract a specific tag from a file
    Extract {
        /// Path to the image file
        #[arg(required = true)]
        file: PathBuf,

        /// Tag name to extract (e.g., "DateTimeOriginal")
        #[arg(required = true)]
        tag: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List {
            file,
            verbose,
            json,
        } => {
            let parser = ExifParser::new().verbose(*verbose);
            match parser.parse_file(file) {
                Ok(exif_data) => {
                    if *json {
                        print_exif_data_json(&exif_data)?;
                    } else {
                        print_exif_data(&exif_data, *verbose);
                    }
                    Ok(())
                }
                Err(err) => {
                    eprintln!("Error parsing EXIF data: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Extract { file, tag } => {
            let parser = ExifParser::new();
            match parser.parse_file(file) {
                Ok(exif_data) => match exif_data.get_tag_by_name(tag) {
                    Some(value) => {
                        println!("{}: {}", tag, value);
                        Ok(())
                    }
                    None => {
                        eprintln!("Tag '{}' not found", tag);
                        std::process::exit(1);
                    }
                },
                Err(err) => {
                    eprintln!("Error parsing EXIF data: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Print all EXIF data in JSON format (exiftool-compatible)
fn print_exif_data_json(exif_data: &ExifData) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::{Map, Value};
    use std::env;

    let mut output = Map::new();

    // Add SourceFile field
    if let Some(file_arg) = env::args().nth(2) {
        output.insert("SourceFile".to_string(), Value::String(file_arg));
    }

    // Convert each tag to a key-value pair
    for (tag_id, value) in exif_data.iter() {
        let tag_name = if let Some(name) = tag_id.name() {
            name.to_string()
        } else {
            format!("Tag{}", tag_id.id)
        };
        let json_value = format_exif_value_for_json(value, tag_id.id);
        output.insert(tag_name, json_value);
    }

    // Add maker notes if present
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        for (tag_id, maker_tag) in maker_notes.iter() {
            let tag_name = maker_tag
                .tag_name
                .unwrap_or_else(|| Box::leak(format!("MakerNote{:04X}", tag_id).into_boxed_str()));
            let json_value = format_exif_value_for_json(&maker_tag.value, *tag_id);
            output.insert(tag_name.to_string(), json_value);
        }
    }

    // Wrap in an array like exiftool does
    let result = Value::Array(vec![Value::Object(output)]);

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

/// Convert an ExifValue to a JSON value in exiftool-compatible format
fn format_exif_value_for_json(
    value: &fpexif::data_types::ExifValue,
    tag_id: u16,
) -> serde_json::Value {
    use fpexif::data_types::ExifValue;
    use serde_json::Value;

    match value {
        // ASCII strings - return as plain string
        ExifValue::Ascii(s) => {
            // Remove null terminators and trim
            let cleaned = s.trim_end_matches('\0').trim();
            Value::String(cleaned.to_string())
        }

        // Single-value numeric types - return as number or interpreted string
        ExifValue::Byte(v) if v.len() == 1 => {
            // Check for tags that need interpretation
            match tag_id {
                0xA300 => {
                    // FileSource
                    Value::String(fpexif::tags::get_file_source_description(v[0]).to_string())
                }
                0xA301 => {
                    // SceneType
                    Value::String(fpexif::tags::get_scene_type_description(v[0]).to_string())
                }
                _ => Value::Number(v[0].into()),
            }
        }
        ExifValue::Short(v) if v.len() == 1 => {
            // Check for tags that need human-readable interpretation
            match tag_id {
                0x0112 => {
                    // Orientation
                    Value::String(fpexif::tags::get_orientation_description(v[0]).to_string())
                }
                0x0103 => {
                    // Compression
                    Value::String(fpexif::tags::get_compression_description(v[0]).to_string())
                }
                0x0128 => {
                    // ResolutionUnit
                    Value::String(fpexif::tags::get_resolution_unit_description(v[0]).to_string())
                }
                0x0213 => {
                    // YCbCrPositioning
                    Value::String(fpexif::tags::get_ycbcr_positioning_description(v[0]).to_string())
                }
                0x8822 => {
                    // ExposureProgram
                    Value::String(fpexif::tags::get_exposure_program_description(v[0]).to_string())
                }
                0x9207 => {
                    // MeteringMode
                    Value::String(fpexif::tags::get_metering_mode_description(v[0]).to_string())
                }
                0x9208 => {
                    // LightSource
                    Value::String(fpexif::tags::get_light_source_description(v[0]).to_string())
                }
                0x9209 => {
                    // Flash
                    Value::String(fpexif::tags::get_flash_description(v[0]).to_string())
                }
                0xA001 => {
                    // ColorSpace
                    Value::String(fpexif::tags::get_color_space_description(v[0]).to_string())
                }
                0xA402 => {
                    // ExposureMode
                    Value::String(fpexif::tags::get_exposure_mode_description(v[0]).to_string())
                }
                0xA403 => {
                    // WhiteBalance
                    Value::String(fpexif::tags::get_white_balance_description(v[0]).to_string())
                }
                0xA406 => {
                    // SceneCaptureType
                    Value::String(
                        fpexif::tags::get_scene_capture_type_description(v[0]).to_string(),
                    )
                }
                0xA408 => {
                    // Contrast
                    Value::String(fpexif::tags::get_contrast_description(v[0]).to_string())
                }
                0xA409 => {
                    // Saturation
                    Value::String(fpexif::tags::get_saturation_description(v[0]).to_string())
                }
                0xA40A => {
                    // Sharpness
                    Value::String(fpexif::tags::get_sharpness_description(v[0]).to_string())
                }
                0xA40C => {
                    // SubjectDistanceRange
                    Value::String(
                        fpexif::tags::get_subject_distance_range_description(v[0]).to_string(),
                    )
                }
                0xA401 => {
                    // CustomRendered
                    Value::String(fpexif::tags::get_custom_rendered_description(v[0]).to_string())
                }
                0xA40B => {
                    // GainControl
                    Value::String(fpexif::tags::get_gain_control_description(v[0]).to_string())
                }
                0x041A => {
                    // SensingMethod
                    Value::String(fpexif::tags::get_sensing_method_description(v[0]).to_string())
                }
                _ => Value::Number(v[0].into()),
            }
        }
        ExifValue::Long(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SByte(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SShort(v) if v.len() == 1 => Value::Number(v[0].into()),
        ExifValue::SLong(v) if v.len() == 1 => Value::Number(v[0].into()),

        // Float/Double - return as number
        ExifValue::Float(v) if v.len() == 1 => serde_json::Number::from_f64(v[0] as f64)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v[0].to_string())),
        ExifValue::Double(v) if v.len() == 1 => serde_json::Number::from_f64(v[0])
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v[0].to_string())),

        // Rational - format as decimal number or fraction depending on tag
        ExifValue::Rational(v) if v.len() == 1 => {
            let (num, den) = v[0];
            if den == 0 {
                Value::String("inf".to_string())
            } else {
                // Special handling for certain tags
                match tag_id {
                    0x829A => {
                        // ExposureTime - show as fraction
                        if num == 1 {
                            Value::String(format!("1/{}", den))
                        } else {
                            let decimal = num as f64 / den as f64;
                            Value::String(format!("{:.6}", decimal))
                        }
                    }
                    0x9201 | 0x9202 | 0x9203 | 0x9204 | 0x9205 => {
                        // APEX values - show as decimal
                        let decimal = num as f64 / den as f64;
                        serde_json::Number::from_f64(decimal)
                            .map(Value::Number)
                            .unwrap_or_else(|| Value::String(decimal.to_string()))
                    }
                    _ => {
                        // Default: show as decimal
                        let decimal = num as f64 / den as f64;
                        serde_json::Number::from_f64(decimal)
                            .map(Value::Number)
                            .unwrap_or_else(|| Value::String(decimal.to_string()))
                    }
                }
            }
        }

        // Signed Rational
        ExifValue::SRational(v) if v.len() == 1 => {
            let (num, den) = v[0];
            if den == 0 {
                Value::String("inf".to_string())
            } else {
                let decimal = num as f64 / den as f64;
                serde_json::Number::from_f64(decimal)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(decimal.to_string()))
            }
        }

        // Multi-value arrays - return as JSON array
        ExifValue::Byte(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::Short(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::Long(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SByte(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SShort(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::SLong(v) => Value::Array(v.iter().map(|&n| Value::Number(n.into())).collect()),
        ExifValue::Float(v) => Value::Array(
            v.iter()
                .map(|&f| {
                    serde_json::Number::from_f64(f as f64)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::String(f.to_string()))
                })
                .collect(),
        ),
        ExifValue::Double(v) => Value::Array(
            v.iter()
                .map(|&f| {
                    serde_json::Number::from_f64(f)
                        .map(Value::Number)
                        .unwrap_or_else(|| Value::String(f.to_string()))
                })
                .collect(),
        ),

        // Multi-value rationals - return as array of decimals
        ExifValue::Rational(v) => Value::Array(
            v.iter()
                .map(|&(num, den)| {
                    if den == 0 {
                        Value::String("inf".to_string())
                    } else {
                        let decimal = num as f64 / den as f64;
                        serde_json::Number::from_f64(decimal)
                            .map(Value::Number)
                            .unwrap_or_else(|| Value::String(decimal.to_string()))
                    }
                })
                .collect(),
        ),
        ExifValue::SRational(v) => Value::Array(
            v.iter()
                .map(|&(num, den)| {
                    if den == 0 {
                        Value::String("inf".to_string())
                    } else {
                        let decimal = num as f64 / den as f64;
                        serde_json::Number::from_f64(decimal)
                            .map(Value::Number)
                            .unwrap_or_else(|| Value::String(decimal.to_string()))
                    }
                })
                .collect(),
        ),

        // Undefined - return as base64 or hex string, or interpret special tags
        ExifValue::Undefined(v) => {
            // Special handling for FileSource and SceneType which are often Undefined type
            match tag_id {
                0xA300 if v.len() == 1 => {
                    // FileSource
                    Value::String(fpexif::tags::get_file_source_description(v[0]).to_string())
                }
                0xA301 if v.len() == 1 => {
                    // SceneType
                    Value::String(fpexif::tags::get_scene_type_description(v[0]).to_string())
                }
                _ => {
                    if v.len() <= 32 {
                        // For short undefined data, show as hex
                        Value::String(
                            v.iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                    } else {
                        // For longer data, use base64
                        Value::String(base64_encode(v))
                    }
                }
            }
        }
    }
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[(((b2 & 0x0f) << 2) | (b3 >> 6)) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[(b3 & 0x3f) as usize] as char
        } else {
            '='
        });
    }

    result
}

/// Print all EXIF data in a human-readable format
fn print_exif_data(exif_data: &ExifData, verbose: bool) {
    println!("Found {} EXIF tags", exif_data.len());

    for (tag_id, value) in exif_data.iter() {
        let tag_name = tag_id.name().unwrap_or("Unknown Tag");
        println!("{} (0x{:04X}): {}", tag_name, tag_id.id, value);

        // For certain tags, provide additional human-readable interpretation
        if verbose {
            match tag_id.id {
                // Orientation
                0x0112 => {
                    if let fpexif::data_types::ExifValue::Short(values) = value {
                        if !values.is_empty() {
                            let desc = tags::get_orientation_description(values[0]);
                            println!("  → {}", desc);
                        }
                    }
                }
                // Exposure Program
                0x8822 => {
                    if let fpexif::data_types::ExifValue::Short(values) = value {
                        if !values.is_empty() {
                            let desc = tags::get_exposure_program_description(values[0]);
                            println!("  → {}", desc);
                        }
                    }
                }
                // Metering Mode
                0x9207 => {
                    if let fpexif::data_types::ExifValue::Short(values) = value {
                        if !values.is_empty() {
                            let desc = tags::get_metering_mode_description(values[0]);
                            println!("  → {}", desc);
                        }
                    }
                }
                // Light Source
                0x9208 => {
                    if let fpexif::data_types::ExifValue::Short(values) = value {
                        if !values.is_empty() {
                            let desc = tags::get_light_source_description(values[0]);
                            println!("  → {}", desc);
                        }
                    }
                }
                // Flash
                0x9209 => {
                    if let fpexif::data_types::ExifValue::Short(values) = value {
                        if !values.is_empty() {
                            let desc = tags::get_flash_description(values[0]);
                            println!("  → {}", desc);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
