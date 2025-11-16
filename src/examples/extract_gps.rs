// examples/extract_gps.rs
use fpexif::{data_types::ExifValue, ExifParser};
use std::path::PathBuf;

/// Example program that extracts GPS coordinates from images
/// and formats them for use with mapping software
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get file path from command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image-file>", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    println!("Extracting GPS coordinates from: {}", file_path.display());

    // Create parser with default settings
    let parser = ExifParser::new();

    // Parse the file
    let exif_data = parser.parse_file(&file_path)?;

    // Look for GPS tags
    let lat_ref = exif_data.get_tag_by_name("GPSLatitudeRef");
    let lat = exif_data.get_tag_by_name("GPSLatitude");
    let lon_ref = exif_data.get_tag_by_name("GPSLongitudeRef");
    let lon = exif_data.get_tag_by_name("GPSLongitude");

    // Check if GPS data exists
    if lat.is_none() || lon.is_none() {
        println!("No GPS coordinates found in this image.");
        return Ok(());
    }

    // Extract and format coordinates
    if let (
        Some(ExifValue::Ascii(lat_ref)),
        Some(ExifValue::Rational(lat_vals)),
        Some(ExifValue::Ascii(lon_ref)),
        Some(ExifValue::Rational(lon_vals)),
    ) = (lat_ref, lat, lon_ref, lon)
    {
        if lat_vals.len() >= 3 && lon_vals.len() >= 3 {
            // Convert the rational values to decimal degrees
            let lat_dec = rational_to_decimal_degrees(&lat_vals);
            let lon_dec = rational_to_decimal_degrees(&lon_vals);

            // Apply the reference (N/S, E/W)
            let lat_dec = if lat_ref.contains('S') {
                -lat_dec
            } else {
                lat_dec
            };
            let lon_dec = if lon_ref.contains('W') {
                -lon_dec
            } else {
                lon_dec
            };

            println!("GPS Coordinates: {:.6}, {:.6}", lat_dec, lon_dec);
            println!(
                "Google Maps URL: https://maps.google.com/?q={:.6},{:.6}",
                lat_dec, lon_dec
            );

            // If the image has a timestamp, include it
            if let Some(date_time) = exif_data.get_tag_by_name("DateTimeOriginal") {
                println!("Photo taken: {}", date_time);
            }

            // Include altitude if available
            if let Some(ExifValue::Rational(alt_vals)) = exif_data.get_tag_by_name("GPSAltitude") {
                if !alt_vals.is_empty() {
                    let (num, den) = alt_vals[0];
                    let altitude = if den != 0 {
                        num as f64 / den as f64
                    } else {
                        0.0
                    };

                    // Check if altitude is above or below sea level
                    let alt_ref = exif_data.get_tag_by_name("GPSAltitudeRef");
                    let altitude = if let Some(ExifValue::Byte(ref_vals)) = alt_ref {
                        if !ref_vals.is_empty() && ref_vals[0] == 1 {
                            -altitude // Below sea level
                        } else {
                            altitude // Above sea level
                        }
                    } else {
                        altitude // Default to above sea level
                    };

                    println!("Altitude: {:.1} meters", altitude);
                }
            }
        } else {
            println!("Incomplete GPS data found.");
        }
    } else {
        println!("GPS data is in an unexpected format.");
    }

    Ok(())
}

/// Convert a set of three rational values (degrees, minutes, seconds) to decimal degrees
fn rational_to_decimal_degrees(dms: &[(u32, u32)]) -> f64 {
    if dms.len() < 3 {
        return 0.0;
    }

    // Calculate degrees
    let (deg_num, deg_den) = dms[0];
    let degrees = if deg_den != 0 {
        deg_num as f64 / deg_den as f64
    } else {
        0.0
    };

    // Calculate minutes
    let (min_num, min_den) = dms[1];
    let minutes = if min_den != 0 {
        min_num as f64 / min_den as f64
    } else {
        0.0
    };

    // Calculate seconds
    let (sec_num, sec_den) = dms[2];
    let seconds = if sec_den != 0 {
        sec_num as f64 / sec_den as f64
    } else {
        0.0
    };

    // Convert to decimal degrees
    degrees + (minutes / 60.0) + (seconds / 3600.0)
}
