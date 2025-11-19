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
                // Non-JSON output: process each file
                for file in files {
                    match parser.parse_file(file) {
                        Ok(exif_data) => {
                            println!("======== {} ========", file.display());
                            print_exif_data(&exif_data, false);
                            println!();
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
}
