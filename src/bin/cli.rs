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

    /// Extract embedded JPEG preview from RAW file
    #[command(name = "extract-jpeg")]
    ExtractJpeg {
        /// Path to the RAW file (or ignored if --test-dir is used)
        #[arg(required_unless_present = "test_dir")]
        file: Option<PathBuf>,

        /// Output file path (default: input_preview.jpg)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Extract thumbnail instead of preview
        #[arg(short, long)]
        thumbnail: bool,

        /// List all embedded JPEGs without extracting
        #[arg(short, long)]
        list: bool,

        /// Extract all embedded JPEGs (outputs to input_0.jpg, input_1.jpg, etc.)
        #[arg(short, long)]
        all: bool,

        /// Test JPEG extraction across all files in a directory
        #[arg(long)]
        test_dir: Option<PathBuf>,

        /// Validate extracted JPEGs by parsing their structure
        #[arg(short, long)]
        validate: bool,
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
                            let source = file.to_str();
                            print_exif_data_exiftool(&exif_data, source);
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
        Commands::ExtractJpeg {
            file,
            output,
            thumbnail,
            list,
            all,
            test_dir,
            validate,
        } => {
            use fpexif::extract::{extract_jpegs, validate_jpeg, JpegType};
            use std::fs::File;
            use std::io::BufReader;

            // Handle --test-dir mode
            if let Some(dir) = test_dir {
                return run_jpeg_extraction_test(dir, *validate);
            }

            let file = file
                .as_ref()
                .expect("file is required when not using --test-dir");
            let reader = BufReader::new(File::open(file)?);

            let jpeg_type = if *all || *list {
                JpegType::All
            } else if *thumbnail {
                JpegType::Thumbnail
            } else {
                JpegType::Preview
            };

            match extract_jpegs(reader, jpeg_type) {
                Ok(jpegs) => {
                    if jpegs.is_empty() {
                        eprintln!("No embedded JPEGs found in {}", file.display());
                        std::process::exit(1);
                    }

                    if *list {
                        // List mode - just show info
                        println!("Embedded JPEGs in {}:", file.display());
                        for (i, (info, jpeg_data)) in jpegs.iter().enumerate() {
                            let dims = info
                                .dimensions
                                .map(|(w, h)| format!(" ({}x{})", w, h))
                                .unwrap_or_default();

                            if *validate {
                                let validation = validate_jpeg(jpeg_data);
                                let status = if validation.valid { "OK" } else { "INVALID" };
                                let parsed_dims = match (validation.width, validation.height) {
                                    (Some(w), Some(h)) => format!(" {}x{}", w, h),
                                    _ => String::new(),
                                };
                                let eoi = if validation.has_eoi { "" } else { " [no EOI]" };
                                println!(
                                    "  {}: {} - {} bytes at offset {}{} [{}{}{}]",
                                    i,
                                    info.description,
                                    info.length,
                                    info.offset,
                                    dims,
                                    status,
                                    parsed_dims,
                                    eoi
                                );
                            } else {
                                println!(
                                    "  {}: {} - {} bytes at offset {}{}",
                                    i, info.description, info.length, info.offset, dims
                                );
                            }
                        }
                    } else {
                        // Extract mode
                        let base_name = file
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("output");

                        for (i, (info, data)) in jpegs.iter().enumerate() {
                            let out_path = if let Some(ref out) = output {
                                if *all && jpegs.len() > 1 {
                                    // Multiple files: add index
                                    let stem = out
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("output");
                                    let ext =
                                        out.extension().and_then(|s| s.to_str()).unwrap_or("jpg");
                                    PathBuf::from(format!("{}_{}.{}", stem, i, ext))
                                } else {
                                    out.clone()
                                }
                            } else {
                                // Default naming
                                let suffix = if *all && jpegs.len() > 1 {
                                    format!("_{}", i)
                                } else if *thumbnail {
                                    "_thumb".to_string()
                                } else {
                                    "_preview".to_string()
                                };
                                PathBuf::from(format!("{}{}.jpg", base_name, suffix))
                            };

                            std::fs::write(&out_path, data)?;
                            let dims = info
                                .dimensions
                                .map(|(w, h)| format!(" ({}x{})", w, h))
                                .unwrap_or_default();
                            println!(
                                "Extracted {} ({} bytes){} -> {}",
                                info.description,
                                data.len(),
                                dims,
                                out_path.display()
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error extracting JPEG: {}", e);
                    std::process::exit(1);
                }
            }
            Ok(())
        }
    }
}

/// Print all EXIF data in exiftool text format: `Tag Name                        : Value`
fn print_exif_data_exiftool(exif_data: &ExifData, source_file: Option<&str>) {
    // Use to_exiftool_json to get consistent output with JSON mode
    // This ensures tag filtering (removing duplicates, IFD pointers, raw binary) is applied
    // source_file is needed for format-specific overrides (e.g., Pentax DNG vs PEF)
    let json = fpexif::output::to_exiftool_json(exif_data, source_file);

    // Extract the object from the array
    if let serde_json::Value::Array(arr) = json {
        if let Some(serde_json::Value::Object(obj)) = arr.into_iter().next() {
            // Sort keys for consistent output
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();

            for key in keys {
                if let Some(value) = obj.get(key) {
                    let display_value = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => "".to_string(),
                        _ => value.to_string(),
                    };
                    println!("{:<32}: {}", key, display_value);
                }
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
    // First output standard EXIF tags
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

    // Then output MakerNote tags
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        // Detect manufacturer from Make tag
        let make = exif_data
            .get_tag_by_name("Make")
            .and_then(|v| match v {
                fpexif::data_types::ExifValue::Ascii(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("");
        let manufacturer_prefix = get_exiv2_manufacturer_prefix(make);

        let mut sorted_tags: Vec<_> = maker_notes.iter().collect();
        sorted_tags.sort_by_key(|(id, _)| *id);

        for (_tag_id, tag) in sorted_tags {
            let tag_name = tag.tag_name.unwrap_or("Unknown");

            // Use exiv2 group/name if available, otherwise generate from manufacturer
            let key = if let (Some(group), Some(name)) = (tag.exiv2_group, tag.exiv2_name) {
                format!("Exif.{}.{}", group, name)
            } else {
                format!("Exif.{}.{}", manufacturer_prefix, tag_name)
            };

            // Use raw value if available, otherwise use decoded value
            let value_to_format = tag.raw_value.as_ref().unwrap_or(&tag.value);
            let (type_name, count, display_value) = format_exiv2_value(value_to_format);

            println!(
                "{:<44} {:12} {:>4}  {}",
                key, type_name, count, display_value
            );
        }
    }
}

/// Get the exiv2 manufacturer prefix from the Make string
fn get_exiv2_manufacturer_prefix(make: &str) -> &'static str {
    let make_lower = make.to_lowercase();
    if make_lower.contains("canon") {
        "Canon"
    } else if make_lower.contains("nikon") {
        "Nikon3"
    } else if make_lower.contains("sony") {
        "Sony1"
    } else if make_lower.contains("fuji") {
        "Fujifilm"
    } else if make_lower.contains("panasonic") {
        "Panasonic"
    } else if make_lower.contains("olympus") || make_lower.contains("om digital") {
        "Olympus"
    } else if make_lower.contains("pentax") {
        "Pentax"
    } else if make_lower.contains("minolta") {
        "Minolta"
    } else if make_lower.contains("kodak") {
        "Kodak"
    } else if make_lower.contains("samsung") {
        "Samsung2"
    } else {
        "MakerNote"
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
        ExifValue::Undefined(v) => (
            "Undefined",
            v.len(),
            v.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        ),
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

/// Run JPEG extraction test across all files in a directory
fn run_jpeg_extraction_test(
    dir: &PathBuf,
    validate: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use fpexif::extract::{extract_jpegs, validate_jpeg, JpegType};
    use std::fs::{self, File};
    use std::io::BufReader;

    let mut total = 0;
    let mut with_jpegs = 0;
    let mut failed = 0;
    let mut no_jpegs = 0;
    let mut invalid_jpegs = 0;
    let mut failures: Vec<(PathBuf, String)> = Vec::new();
    let mut invalid_files: Vec<(PathBuf, String)> = Vec::new();

    // Collect and sort files
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();

        // Skip non-image files
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let is_raw = matches!(
            ext.as_str(),
            "nef"
                | "cr2"
                | "cr3"
                | "arw"
                | "orf"
                | "raf"
                | "rw2"
                | "dng"
                | "pef"
                | "srw"
                | "nrw"
                | "raw"
                | "mrw"
                | "3fr"
                | "mef"
                | "mos"
                | "iiq"
                | "rwl"
                | "kdc"
                | "dcr"
                | "erf"
                | "sr2"
                | "srf"
                | "crw"
        );

        if !is_raw {
            continue;
        }

        total += 1;

        let reader = match File::open(&path) {
            Ok(f) => BufReader::new(f),
            Err(e) => {
                failed += 1;
                failures.push((path.clone(), format!("open error: {}", e)));
                continue;
            }
        };

        match extract_jpegs(reader, JpegType::All) {
            Ok(jpegs) => {
                if jpegs.is_empty() {
                    no_jpegs += 1;
                    println!(
                        "  {} - no JPEGs",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    );
                } else {
                    with_jpegs += 1;
                    let mut all_valid = true;
                    let sizes: Vec<String> = jpegs
                        .iter()
                        .map(|(info, jpeg_data)| {
                            if validate {
                                let v = validate_jpeg(jpeg_data);
                                if !v.valid {
                                    all_valid = false;
                                }
                                match (v.width, v.height) {
                                    (Some(w), Some(h)) => format!("{}x{}", w, h),
                                    _ => format!("{}KB INVALID", info.length / 1024),
                                }
                            } else if let Some((w, h)) = info.dimensions {
                                format!("{}x{}", w, h)
                            } else {
                                format!("{}KB", info.length / 1024)
                            }
                        })
                        .collect();

                    if validate && !all_valid {
                        invalid_jpegs += 1;
                        invalid_files.push((path.clone(), "Contains invalid JPEG".to_string()));
                    }

                    println!(
                        "  {} - {} JPEGs: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        jpegs.len(),
                        sizes.join(", ")
                    );
                }
            }
            Err(e) => {
                failed += 1;
                failures.push((path.clone(), e.to_string()));
            }
        }
    }

    println!("\n=== JPEG Extraction Test Results ===");
    println!("Total RAW files:    {}", total);
    println!(
        "With embedded JPEG: {} ({:.1}%)",
        with_jpegs,
        if total > 0 {
            with_jpegs as f64 / total as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("No JPEG found:      {}", no_jpegs);
    println!("Failed:             {}", failed);
    if validate {
        println!("Invalid JPEGs:      {}", invalid_jpegs);
    }

    if !failures.is_empty() {
        println!("\nExtraction Failures:");
        for (path, err) in &failures {
            println!(
                "  {} - {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                err
            );
        }
    }

    if validate && !invalid_files.is_empty() {
        println!("\nInvalid JPEG Files:");
        for (path, err) in &invalid_files {
            println!(
                "  {} - {}",
                path.file_name().unwrap_or_default().to_string_lossy(),
                err
            );
        }
    }

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
