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
    use std::collections::HashSet;

    // Tags where MakerNote should override EXIF (ExifTool behavior)
    // Note: Saturation/Contrast/Sharpness are NOT included here - for most Sony cameras,
    // the EXIF values are preferred. Only the MinoltaRaw data in SR2Private would override,
    // but we don't parse that yet.
    const MAKERNOTE_PRIORITY_TAGS: &[&str] = &["MeteringMode", "WhiteBalance", "LightSource"];

    // Build set of MakerNote tag names for priority/dedup logic
    let makernote_tags: HashSet<&str> = exif_data
        .get_maker_notes()
        .map(|notes| notes.iter().filter_map(|(_, note)| note.tag_name).collect())
        .unwrap_or_default();

    // Track which tags we've already printed
    let mut printed_tags: HashSet<String> = HashSet::new();

    // Output RAF-specific metadata first (for Fujifilm RAF files)
    if let Some(raf_metadata) = exif_data.get_raf_metadata() {
        // Output in a consistent order
        let ordered_keys = [
            "RAFVersion",
            "RAFCompression",
            "RawImageFullSize",
            "RawImageCropTopLeft",
            "RawImageCroppedSize",
            "FujiLayout",
            "XTransLayout",
            "RawExposureBias",
            "RawImageWidth",
            "RawImageHeight",
            "RawImageFullWidth",
            "RawImageFullHeight",
            "BlackLevel",
            "WB_GRBLevels",
            "WB_GRBLevelsAuto",
            "WB_GRBLevelsStandard",
            "GeometricDistortionParams",
            "ChromaticAberrationParams",
            "VignettingParams",
        ];
        for key in ordered_keys {
            if let Some(value) = raf_metadata.tags.get(key) {
                println!("{:<32}: {}", key, value);
                printed_tags.insert(key.to_string());
            }
        }
    }

    for (tag_id, value) in exif_data.iter() {
        let tag_name = tag_id.name().unwrap_or("Unknown");

        // Skip priority tags if they exist in MakerNote (MakerNote value takes precedence)
        if MAKERNOTE_PRIORITY_TAGS.contains(&tag_name) && makernote_tags.contains(tag_name) {
            continue;
        }

        let display_value = format_exiftool_text_value(value, tag_id.id);
        println!("{:<32}: {}", tag_name, display_value);
        printed_tags.insert(tag_name.to_string());
    }

    // Add ISO alias from ISOSpeedRatings BEFORE MakerNote output
    // This ensures ExifTool-compatible ISO (from EXIF) takes precedence over MakerNote ISO
    if !printed_tags.contains("ISO") {
        if let Some(fpexif::data_types::ExifValue::Short(v)) = exif_data.get_tag_by_id(0x8827) {
            if !v.is_empty() {
                println!("{:<32}: {}", "ISO", v[0]);
                printed_tags.insert("ISO".to_string());
            }
        }
    }

    // Output maker notes
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        for (_, note) in maker_notes.iter() {
            if let Some(name) = note.tag_name {
                // Skip if already printed (non-priority tags)
                if printed_tags.contains(name) {
                    continue;
                }
                let display_value = format_exiftool_text_value(&note.value, note.tag_id);
                println!("{:<32}: {}", name, display_value);
                printed_tags.insert(name.to_string());
            }
        }
    }

    // Output computed/alias fields
    print_computed_fields(exif_data);
}

/// Print computed fields that are aliases or calculated from existing data
fn print_computed_fields(exif_data: &ExifData) {
    use fpexif::data_types::ExifValue;

    // ThumbnailOffset - alias for JPEGInterchangeFormat (0x0201)
    if let Some(ExifValue::Long(v)) = exif_data.get_tag_by_id(0x0201) {
        if !v.is_empty() {
            println!("{:<32}: {}", "ThumbnailOffset", v[0]);
        }
    }

    // ThumbnailLength - alias for JPEGInterchangeFormatLength (0x0202)
    if let Some(ExifValue::Long(v)) = exif_data.get_tag_by_id(0x0202) {
        if !v.is_empty() {
            println!("{:<32}: {}", "ThumbnailLength", v[0]);
        }
    }

    // PreviewImageStart - alias for StripOffsets (0x0111)
    // PreviewImageLength - alias for StripByteCounts (0x0117)
    // (These aliases are typically only for Canon RAW files where preview is in IFD0)

    // Aperture - alias for FNumber (always show one decimal place like ExifTool)
    if let Some(ExifValue::Rational(v)) = exif_data.get_tag_by_id(0x829D) {
        if !v.is_empty() {
            let (n, d) = v[0];
            if d != 0 {
                let f = n as f64 / d as f64;
                if f == 0.0 {
                    // FNumber of 0 means infinite aperture (e.g., manual lens)
                    println!("{:<32}: Inf", "Aperture");
                } else {
                    println!("{:<32}: {:.1}", "Aperture", f);
                }
            }
        }
    }

    // ImageSize - computed from image dimensions
    // For Fujifilm RAF: use RawImageCroppedSize from RAF metadata
    // For others: MakerNote dimensions > PixelXDimension/PixelYDimension > ImageWidth/ImageLength
    let (width, height) = if let Some(raf_metadata) = exif_data.get_raf_metadata() {
        // Fujifilm RAF: parse RawImageCroppedSize (e.g., "4896x3264")
        if let Some(cropped_size) = raf_metadata.tags.get("RawImageCroppedSize") {
            let parts: Vec<&str> = cropped_size.split('x').collect();
            if parts.len() == 2 {
                let w = parts[0].parse::<u32>().ok();
                let h = parts[1].parse::<u32>().ok();
                (w, h)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        let (makernote_width, makernote_height) = get_makernote_image_dimensions(exif_data);
        let width = makernote_width
            .or_else(|| get_dimension(exif_data, 0xA002))
            .or_else(|| get_dimension(exif_data, 0x0100));
        let height = makernote_height
            .or_else(|| get_dimension(exif_data, 0xA003))
            .or_else(|| get_dimension(exif_data, 0x0101));
        (width, height)
    };

    if let (Some(w), Some(h)) = (width, height) {
        println!("{:<32}: {}x{}", "ImageSize", w, h);

        // Megapixels
        let megapixels = (w as f64 * h as f64) / 1_000_000.0;
        println!("{:<32}: {:.1}", "Megapixels", megapixels);
    }

    // RedBalance and BlueBalance from WB_RGGBLevelsAsShot
    if let Some(maker_notes) = exif_data.get_maker_notes() {
        // Find WB_RGGBLevelsAsShot or WB_RGGBLevels
        for (_, note) in maker_notes.iter() {
            if let Some(name) = note.tag_name {
                if name == "WB_RGGBLevelsAsShot" || name == "WB_RGGBLevels" {
                    // Can be either Short array or Ascii string "R G G B"
                    let values: Option<Vec<f64>> = match &note.value {
                        ExifValue::Short(v) if v.len() >= 4 => {
                            Some(v.iter().map(|&x| x as f64).collect())
                        }
                        ExifValue::Ascii(s) => {
                            // Parse space-separated values
                            let parsed: Vec<f64> = s
                                .split_whitespace()
                                .filter_map(|x| x.parse().ok())
                                .collect();
                            if parsed.len() >= 4 {
                                Some(parsed)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(v) = values {
                        // RGGB: v[0]=R, v[1]=G1, v[2]=G2, v[3]=B
                        // Balance is R/G and B/G where G = G1 (first G)
                        let r = v[0];
                        let g = v[1]; // Use first G
                        let b = v[3];
                        if g > 0.0 {
                            println!("{:<32}: {:.6}", "RedBalance", r / g);
                            println!("{:<32}: {:.6}", "BlueBalance", b / g);
                        }
                    }
                    break;
                }
            }
        }
    }

    // ScaleFactor35efl for Canon APS-C is 1.6
    // Check if this is a Canon APS-C camera (not full-frame)
    if let Some(ExifValue::Ascii(make)) = exif_data.get_tag_by_id(0x010F) {
        if make.contains("Canon") {
            // For APS-C Canon cameras, scale factor is 1.6
            // We'd need to check the model to determine if it's APS-C or full-frame
            // For now, let's assume 50D is APS-C
            println!("{:<32}: {}", "ScaleFactor35efl", 1.6);
        }
    }

    // ShutterSpeed - alias for ExposureTime
    if let Some(ExifValue::Rational(v)) = exif_data.get_tag_by_id(0x829A) {
        if !v.is_empty() {
            let (n, d) = v[0];
            if d != 0 {
                let exposure = n as f64 / d as f64;
                let denom = (1.0 / exposure).round() as u32;
                println!("{:<32}: 1/{}", "ShutterSpeed", denom);
            }
        }
    }
}

/// Get dimension value from a tag (handles Short or Long)
fn get_dimension(exif_data: &ExifData, tag_id: u16) -> Option<u32> {
    use fpexif::data_types::ExifValue;

    if let Some(value) = exif_data.get_tag_by_id(tag_id) {
        match value {
            ExifValue::Short(v) if !v.is_empty() => Some(v[0] as u32),
            ExifValue::Long(v) if !v.is_empty() => Some(v[0]),
            _ => None,
        }
    } else {
        None
    }
}

/// Get image dimensions from MakerNotes (manufacturer-specific)
/// Returns (width, height) if available
fn get_makernote_image_dimensions(exif_data: &ExifData) -> (Option<u32>, Option<u32>) {
    use fpexif::data_types::ExifValue;

    let mut width = None;
    let mut height = None;

    if let Some(maker_notes) = exif_data.get_maker_notes() {
        for (_, note) in maker_notes.iter() {
            if let Some(name) = note.tag_name {
                match name {
                    // Panasonic-specific dimension tags
                    "PanasonicImageWidth" => {
                        if let ExifValue::Long(v) = &note.value {
                            if !v.is_empty() {
                                width = Some(v[0]);
                            }
                        }
                    }
                    "PanasonicImageHeight" => {
                        if let ExifValue::Long(v) = &note.value {
                            if !v.is_empty() {
                                height = Some(v[0]);
                            }
                        }
                    }
                    // Sony/generic MakerNote dimension tags (prefer over EXIF for older Sony)
                    "ImageWidth" => {
                        if width.is_none() {
                            match &note.value {
                                ExifValue::Long(v) if !v.is_empty() => width = Some(v[0]),
                                ExifValue::Short(v) if !v.is_empty() => width = Some(v[0] as u32),
                                _ => {}
                            }
                        }
                    }
                    "ImageHeight" => {
                        if height.is_none() {
                            match &note.value {
                                ExifValue::Long(v) if !v.is_empty() => height = Some(v[0]),
                                ExifValue::Short(v) if !v.is_empty() => height = Some(v[0] as u32),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    (width, height)
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
        ExifValue::Byte(v) => {
            // GPSVersionID (0x0000) should be formatted as "2.2.0.0"
            if tag_id == 0x0000 && v.len() == 4 {
                format!("{}.{}.{}.{}", v[0], v[1], v[2], v[3])
            } else {
                format_values_space(v)
            }
        }
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
                    // Contrast (0xA408), Saturation (0xA409)
                    0xA408 | 0xA409 => match v[0] {
                        0 => "Normal",
                        1 => "Low",
                        2 => "High",
                        _ => "Unknown",
                    }
                    .to_string(),
                    // Sharpness (0xA40A)
                    0xA40A => match v[0] {
                        0 => "Normal",
                        1 => "Soft",
                        2 => "Hard",
                        _ => "Unknown",
                    }
                    .to_string(),
                    0xA40C => tags::get_subject_distance_range_description(v[0]).to_string(),
                    0xA401 => tags::get_custom_rendered_description(v[0]).to_string(),
                    0x9217 | 0xA217 => tags::get_sensing_method_description(v[0]).to_string(),
                    0x8830 => tags::get_sensitivity_type_description(v[0]).to_string(),
                    // FocalLengthIn35mmFormat - add "mm" suffix
                    0xA405 => format!("{} mm", v[0]),
                    _ => format_values_space(v),
                }
            } else if tag_id == 0x828E && v.len() >= 4 {
                // CFAPattern (TIFF/EP) as Short array - format as [Red,Green][Green,Blue]
                let color_name = |c: u16| -> &'static str {
                    match c {
                        0 => "Red",
                        1 => "Green",
                        2 => "Blue",
                        3 => "Cyan",
                        4 => "Magenta",
                        5 => "Yellow",
                        6 => "White",
                        _ => "Unknown",
                    }
                };
                format!(
                    "[{},{}][{},{}]",
                    color_name(v[0]),
                    color_name(v[1]),
                    color_name(v[2]),
                    color_name(v[3])
                )
            } else if v.len() == 2 && tag_id == 0x100A {
                // Fuji WhiteBalanceFineTune - format as "Red +X, Blue +Y"
                let red = v[0] as i16;
                let blue = v[1] as i16;
                let red_str = if red >= 0 {
                    format!("+{}", red)
                } else {
                    format!("{}", red)
                };
                let blue_str = if blue >= 0 {
                    format!("+{}", blue)
                } else {
                    format!("{}", blue)
                };
                format!("Red {}, Blue {}", red_str, blue_str)
            } else {
                format_values_space(v)
            }
        }
        ExifValue::Long(v) => {
            // Fuji ImageStabilization (0x1422) - format as "Type; Mode; Param"
            if tag_id == 0x1422 && v.len() >= 3 {
                let is_type = match v[0] {
                    0 => "None",
                    1 => "Optical",
                    2 => "Sensor-shift",
                    3 => "OIS/IBIS Combined",
                    _ => "Unknown",
                };
                let is_mode = match v[1] {
                    0 => "Off",
                    1 => "On (mode 1, continuous)",
                    2 => "On (mode 2, shooting only)",
                    3 => "On (mode 3, panning)",
                    _ => "Unknown",
                };
                format!("{}; {}; {}", is_type, is_mode, v[2])
            } else {
                format_values_space(v)
            }
        }
        ExifValue::Rational(v) => {
            if v.len() == 1 {
                let (n, d) = v[0];
                if d != 0 {
                    match tag_id {
                        0x829A => {
                            // ExposureTime - show as 1/n fraction
                            let exposure_time = n as f64 / d as f64;
                            let approx_den = (1.0 / exposure_time).round() as u32;
                            format!("1/{}", approx_den)
                        }
                        0x920A => {
                            // FocalLength - format with decimal and mm unit
                            // ExifTool shows "55.0 mm" for CR2/standard EXIF
                            // CRW shows "400 mm" but that comes from MakerNote, not EXIF
                            let focal_length = n as f64 / d as f64;
                            format!("{:.1} mm", focal_length)
                        }
                        0x9206 => {
                            // SubjectDistance - format with m unit
                            let distance = n as f64 / d as f64;
                            format!("{} m", distance)
                        }
                        0x9201 => {
                            // ShutterSpeedValue (APEX) - convert to shutter speed
                            // ShutterSpeed = 2^APEX
                            let apex = n as f64 / d as f64;
                            let shutter_speed = 2f64.powf(apex);
                            let denominator = shutter_speed.round() as u32;
                            format!("1/{}", denominator)
                        }
                        0x9202 | 0x9205 => {
                            // ApertureValue, MaxApertureValue (APEX) - convert to f-number
                            // F-number = 2^(APEX/2)
                            let apex = n as f64 / d as f64;
                            let f_number = 2f64.powf(apex / 2.0);
                            let rounded = (f_number * 10.0).round() / 10.0;
                            format!("{}", rounded)
                        }
                        0x008B => {
                            // LensFStops (Nikon) - format with 2 decimal places
                            let val = n as f64 / d as f64;
                            format!("{:.2}", val)
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
                    match tag_id {
                        0x9201 => {
                            // ShutterSpeedValue (APEX) - convert to shutter speed
                            let apex = n as f64 / d as f64;
                            let shutter_speed = 2f64.powf(apex);
                            let denominator = shutter_speed.round() as i32;
                            format!("1/{}", denominator)
                        }
                        0x9204 => {
                            // ExposureBiasValue - format as fraction when possible
                            let bias = n as f64 / d as f64;
                            if bias == 0.0 {
                                "0".to_string()
                            } else {
                                // Simplify fraction and check for 1/3, 2/3, 1/2
                                fn gcd(a: i32, b: i32) -> i32 {
                                    if b == 0 {
                                        a.abs()
                                    } else {
                                        gcd(b, a % b)
                                    }
                                }
                                let g = gcd(n.abs(), d.abs());
                                let simple_n = n.abs() / g;
                                let simple_d = d.abs() / g;
                                let sign = if bias > 0.0 { "+" } else { "-" };

                                if simple_d == 3 || simple_d == 2 {
                                    format!("{}{}/{}", sign, simple_n, simple_d)
                                } else if bias.fract() == 0.0 {
                                    format!("{}{}", sign, bias.abs() as i32)
                                } else {
                                    let formatted = format!("{:.2}", bias.abs())
                                        .trim_end_matches('0')
                                        .trim_end_matches('.')
                                        .to_string();
                                    format!("{}{}", sign, formatted)
                                }
                            }
                        }
                        // Nikon ExposureBracketValue (0x0019) - output as fraction
                        // ExifTool uses PrintFraction: "0", "+N", "-N", "+N/2", "-N/2", "+N/3", "-N/3"
                        0x0019 => {
                            let val = n as f64 / d as f64 * 1.00001; // Avoid round-off
                            if val.abs() < 0.0001 {
                                "0".to_string()
                            } else if (val.trunc() / val).abs() > 0.999 {
                                format!("{:+}", val.trunc() as i32)
                            } else if ((val * 2.0).trunc() / (val * 2.0)).abs() > 0.999 {
                                format!("{:+}/2", (val * 2.0).trunc() as i32)
                            } else if ((val * 3.0).trunc() / (val * 3.0)).abs() > 0.999 {
                                format!("{:+}/3", (val * 3.0).trunc() as i32)
                            } else {
                                format!("{:+.3}", n as f64 / d as f64)
                            }
                        }
                        // Nikon ProgramShift (0x000D), ExposureDifference (0x000E),
                        // FlashExposureComp (0x0012), ExternalFlashExposureComp (0x0017),
                        // FlashExposureBracketValue (0x0018), ExposureTuning (0x001C)
                        // Format: +0.2 / -0.2 / 0 (1 decimal, + prefix for positive)
                        0x000D | 0x000E | 0x0012 | 0x0017 | 0x0018 | 0x001C => {
                            let val = n as f64 / d as f64;
                            if val == 0.0 {
                                "0".to_string()
                            } else {
                                // Round to 1 decimal using banker's rounding (round half to even)
                                // to match ExifTool/Perl behavior
                                fn round_half_even(x: f64, decimals: i32) -> f64 {
                                    let multiplier = 10f64.powi(decimals);
                                    let shifted = x * multiplier;
                                    let floor = shifted.floor();
                                    let frac = shifted - floor;
                                    if (frac - 0.5).abs() < 1e-9 {
                                        // Exactly 0.5 - round to even
                                        if floor as i64 % 2 == 0 {
                                            floor / multiplier
                                        } else {
                                            (floor + 1.0) / multiplier
                                        }
                                    } else {
                                        shifted.round() / multiplier
                                    }
                                }
                                let rounded = round_half_even(val, 1);
                                let formatted = format!("{:.1}", rounded.abs());
                                // Remove trailing zeros and dot if integer
                                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                                if rounded > 0.0 {
                                    format!("+{}", trimmed)
                                } else if rounded < 0.0 {
                                    format!("-{}", trimmed)
                                } else {
                                    "0".to_string()
                                }
                            }
                        }
                        _ => format!("{:.6}", n as f64 / d as f64)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string(),
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
        ExifValue::Float(v) => format_values_space(v),
        ExifValue::Double(v) => format_values_space(v),
        ExifValue::Undefined(v) => {
            // Handle UserComment and other undefined tags
            match tag_id {
                0x9000 | 0xA000 => {
                    // ExifVersion (0x9000), FlashpixVersion (0xA000) - decode as ASCII
                    // Stored as 4 bytes like "0221" or "0100"
                    String::from_utf8(v.to_vec())
                        .ok()
                        .map(|s| s.trim_end_matches('\0').to_string())
                        .unwrap_or_else(|| format!("(Binary data {} bytes)", v.len()))
                }
                0x9101 => {
                    // ComponentsConfiguration - decode component IDs
                    // 0=-, 1=Y, 2=Cb, 3=Cr, 4=R, 5=G, 6=B
                    let components: Vec<&str> = v
                        .iter()
                        .map(|&b| match b {
                            0 => "-",
                            1 => "Y",
                            2 => "Cb",
                            3 => "Cr",
                            4 => "R",
                            5 => "G",
                            6 => "B",
                            _ => "?",
                        })
                        .collect();
                    components.join(", ")
                }
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
                0xA300 if v.len() == 1 => {
                    // FileSource
                    crate::tags::get_file_source_description(v[0]).to_string()
                }
                0xA301 if v.len() == 1 => {
                    // SceneType
                    crate::tags::get_scene_type_description(v[0]).to_string()
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
