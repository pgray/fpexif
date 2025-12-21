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

        /// Output in exiftool-compatible JSON format (same as --json)
        #[arg(long)]
        exiftool_json: bool,
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

    /// Exiftool-compatible interface
    #[command(name = "exiftool")]
    Exiftool {
        /// Output in JSON format (exiftool -j)
        #[arg(short = 'j', long = "json")]
        json: bool,

        /// Path to the image file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Exiv2-compatible interface
    #[command(name = "exiv2")]
    Exiv2 {
        /// Path to the image file(s)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List {
            file,
            verbose,
            json,
            exiftool_json,
        } => {
            let parser = ExifParser::new().verbose(*verbose);
            match parser.parse_file(file) {
                Ok(exif_data) => {
                    if *json || *exiftool_json {
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
        Commands::Exiftool { json, files } => {
            let parser = ExifParser::new();

            if *json {
                // Process all files and output as JSON array
                let mut all_results = Vec::new();

                for file in files {
                    match parser.parse_file(file) {
                        Ok(exif_data) => {
                            // Get the filename as a string
                            let filename = file.to_string_lossy().to_string();
                            let json_obj =
                                fpexif::output::to_exiftool_json(&exif_data, Some(&filename));

                            // Extract the single object from the array
                            if let serde_json::Value::Array(mut arr) = json_obj {
                                if let Some(obj) = arr.pop() {
                                    all_results.push(obj);
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Error parsing {}: {}", file.display(), err);
                            std::process::exit(1);
                        }
                    }
                }

                // Output as JSON array
                let result = serde_json::Value::Array(all_results);
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                // Non-JSON output: exiftool-style format
                for file in files {
                    match parser.parse_file(file) {
                        Ok(exif_data) => {
                            print_exif_data_exiftool(&exif_data);
                        }
                        Err(err) => {
                            eprintln!("Error parsing {}: {}", file.display(), err);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Ok(())
        }
        Commands::Exiv2 { files } => {
            let parser = ExifParser::new();

            for file in files {
                match parser.parse_file(file) {
                    Ok(exif_data) => {
                        print_exif_data_exiv2(&exif_data);
                    }
                    Err(err) => {
                        eprintln!("Error parsing {}: {}", file.display(), err);
                        std::process::exit(1);
                    }
                }
            }
            Ok(())
        }
    }
}

/// Print all EXIF data in exiftool text format: `Tag Name                        : Value`
fn print_exif_data_exiftool(exif_data: &ExifData) {
    for (tag_id, value) in exif_data.iter() {
        let tag_name = tag_id.name().unwrap_or("Unknown");
        let display_value = format_exiftool_text_value(value, tag_id.id);
        println!("{:<32}: {}", tag_name, display_value);
    }
}

/// Format an ExifValue for exiftool-style text output
fn format_exiftool_text_value(value: &fpexif::data_types::ExifValue, tag_id: u16) -> String {
    use fpexif::data_types::ExifValue;

    match value {
        ExifValue::Ascii(s) => {
            let cleaned = s.trim_end_matches('\0').trim();
            // Return empty string if it's all null/whitespace
            if cleaned.is_empty() {
                String::new()
            } else {
                cleaned.to_string()
            }
        }
        ExifValue::Byte(v) => format_values_space(v),
        ExifValue::Short(v) => {
            // Use tag-specific descriptions for known tags
            if v.len() == 1 {
                match tag_id {
                    0x0112 => tags::get_orientation_description(v[0]).to_string(),
                    0x0103 => tags::get_compression_description(v[0]).to_string(),
                    0x0128 => tags::get_resolution_unit_description(v[0]).to_string(),
                    0x0213 => tags::get_ycbcr_positioning_description(v[0]).to_string(),
                    0x8822 => tags::get_exposure_program_description(v[0]).to_string(),
                    0x9207 => tags::get_metering_mode_description(v[0]).to_string(),
                    0x9208 => tags::get_light_source_description(v[0]).to_string(),
                    0x9209 => tags::get_flash_description(v[0]).to_string(),
                    0xA001 => tags::get_color_space_description(v[0]).to_string(),
                    0xA210 => match v[0] {
                        2 => "inches".to_string(),
                        3 => "cm".to_string(),
                        _ => v[0].to_string(),
                    },
                    0xA402 => tags::get_exposure_mode_description(v[0]).to_string(),
                    0xA403 => tags::get_white_balance_description(v[0]).to_string(),
                    0xA406 => tags::get_scene_capture_type_description(v[0]).to_string(),
                    0xA407 => tags::get_gain_control_description(v[0]).to_string(),
                    // Contrast, Saturation, Sharpness - exiftool outputs raw numeric values
                    0xA408..=0xA40A => v[0].to_string(),
                    0xA40C => tags::get_subject_distance_range_description(v[0]).to_string(),
                    0xA401 => tags::get_custom_rendered_description(v[0]).to_string(),
                    0xA217 => tags::get_sensing_method_description(v[0]).to_string(),
                    0x8830 => tags::get_sensitivity_type_description(v[0]).to_string(),
                    _ => format_values_space(v),
                }
            } else {
                format_values_space(v)
            }
        }
        ExifValue::Long(v) => format_values_space(v),
        ExifValue::Rational(v) => {
            if v.len() == 1 {
                let (n, d) = v[0];
                if d != 0 {
                    match tag_id {
                        0x920A => {
                            // FocalLength - format with decimal and mm unit
                            let focal_length = n as f64 / d as f64;
                            format!("{:.1} mm", focal_length)
                        }
                        _ => {
                            let val = n as f64 / d as f64;
                            if val == val.floor() {
                                format!("{}", val as u32)
                            } else {
                                format!("{:.6}", val)
                                    .trim_end_matches('0')
                                    .trim_end_matches('.')
                                    .to_string()
                            }
                        }
                    }
                } else {
                    format!("{}/{}", n, d)
                }
            } else {
                v.iter()
                    .map(|(n, d)| format!("{}/{}", n, d))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
        ExifValue::SByte(v) => format_values_space(v),
        ExifValue::SShort(v) => format_values_space(v),
        ExifValue::SLong(v) => format_values_space(v),
        ExifValue::SRational(v) => {
            if v.len() == 1 {
                let (n, d) = v[0];
                if d != 0 {
                    format!("{:.6}", n as f64 / d as f64)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                } else {
                    format!("{}/{}", n, d)
                }
            } else {
                v.iter()
                    .map(|(n, d)| format!("{}/{}", n, d))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
        ExifValue::Float(v) => format_values_space(v),
        ExifValue::Double(v) => format_values_space(v),
        ExifValue::Undefined(v) => {
            // Handle UserComment and other undefined tags
            match tag_id {
                0x9286 => {
                    // UserComment - EXIF spec says first 8 bytes are character code
                    // followed by the actual comment. Common codes: "ASCII\0\0\0", "UNICODE\0", etc.
                    if v.len() >= 8 {
                        // Skip the 8-byte character code
                        let comment_data = &v[8..];
                        // Check if the comment is empty (all nulls, spaces, or a repeated fill pattern)
                        let is_empty = comment_data
                            .iter()
                            .all(|&b| b == 0 || b == b' ' || b == b'A');
                        if is_empty {
                            String::new()
                        } else {
                            // Try to decode as ASCII/UTF-8
                            let cleaned = comment_data
                                .iter()
                                .copied()
                                .take_while(|&b| b != 0)
                                .collect::<Vec<u8>>();
                            String::from_utf8(cleaned)
                                .ok()
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_default()
                        }
                    } else {
                        // Malformed UserComment or empty
                        String::new()
                    }
                }
                _ => format!("(Binary data {} bytes)", v.len()),
            }
        }
    }
}

fn format_values_space<T: std::fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Print all EXIF data in exiv2 format: `Exif.Image.Make  Ascii  6  Canon`
fn print_exif_data_exiv2(exif_data: &ExifData) {
    for (tag_id, value) in exif_data.iter() {
        let group = match tag_id.ifd {
            tags::TagGroup::Main => "Exif.Image",
            tags::TagGroup::Exif => "Exif.Photo",
            tags::TagGroup::Gps => "Exif.GPSInfo",
            tags::TagGroup::Thumbnail => "Exif.Thumbnail",
            tags::TagGroup::Interop => "Exif.Iop",
        };
        let tag_name = tag_id.name().unwrap_or("Unknown");
        let key = format!("{}.{}", group, tag_name);

        let (type_name, count, display_value) = format_exiv2_value(value);

        println!(
            "{:<44} {:12} {:>4}  {}",
            key, type_name, count, display_value
        );
    }
}

/// Format an ExifValue for exiv2-style output
fn format_exiv2_value(value: &fpexif::data_types::ExifValue) -> (&'static str, usize, String) {
    use fpexif::data_types::ExifValue;

    match value {
        ExifValue::Ascii(s) => {
            let cleaned = s.trim_end_matches('\0');
            ("Ascii", cleaned.len(), cleaned.to_string())
        }
        ExifValue::Byte(v) => ("Byte", v.len(), format_values_space(v)),
        ExifValue::Short(v) => ("Short", v.len(), format_values_space(v)),
        ExifValue::Long(v) => ("Long", v.len(), format_values_space(v)),
        ExifValue::Rational(v) => (
            "Rational",
            v.len(),
            v.iter()
                .map(|(n, d)| format!("{}/{}", n, d))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        ExifValue::SByte(v) => ("SByte", v.len(), format_values_space(v)),
        ExifValue::SShort(v) => ("SShort", v.len(), format_values_space(v)),
        ExifValue::SLong(v) => ("SLong", v.len(), format_values_space(v)),
        ExifValue::SRational(v) => (
            "SRational",
            v.len(),
            v.iter()
                .map(|(n, d)| format!("{}/{}", n, d))
                .collect::<Vec<_>>()
                .join(" "),
        ),
        ExifValue::Float(v) => ("Float", v.len(), format_values_space(v)),
        ExifValue::Double(v) => ("Double", v.len(), format_values_space(v)),
        ExifValue::Undefined(v) => ("Undefined", v.len(), format!("{} bytes", v.len())),
    }
}

/// Print all EXIF data in JSON format (exiftool-compatible)
fn print_exif_data_json(exif_data: &ExifData) -> Result<(), Box<dyn std::error::Error>> {
    use std::env;

    // Get the source file from command line args
    let source_file = env::args().nth(2);

    // Use the library's output module to format as JSON
    let json = fpexif::output::to_exiftool_json(exif_data, source_file.as_deref());

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
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

    // Print maker notes if available
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        if !maker_notes.is_empty() {
            println!("\nMaker Notes ({} tags):", maker_notes.len());
            let mut sorted_notes: Vec<_> = maker_notes.iter().collect();
            sorted_notes.sort_by_key(|(id, _)| *id);
            for (tag_id, tag) in sorted_notes {
                let tag_name = tag.tag_name.unwrap_or("Unknown");
                println!("  {} (0x{:04X}): {}", tag_name, tag_id, tag.value);
            }
        }
    }
}
