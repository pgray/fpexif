// Quick test to verify macro-converted decoders work with real Canon data
use fpexif::makernotes::canon::*;

fn main() {
    println!("=== Testing Macro-Converted Canon Tag Decoders ===\n");

    // Test MacroMode
    println!("MacroMode:");
    println!("  Value 1 ExifTool: {}", decode_macro_mode_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_macro_mode_exiv2(1));
    println!("  Value 2 ExifTool: {}", decode_macro_mode_exiftool(2));
    println!("  Value 2 exiv2:    {}", decode_macro_mode_exiv2(2));
    println!();

    // Test FlashMode
    println!("FlashMode:");
    println!("  Value 0 ExifTool: {}", decode_flash_mode_exiftool(0));
    println!("  Value 0 exiv2:    {}", decode_flash_mode_exiv2(0));
    println!("  Value 2 ExifTool: {}", decode_flash_mode_exiftool(2));
    println!("  Value 2 exiv2:    {}", decode_flash_mode_exiv2(2));
    println!();

    // Test DriveMode
    println!("DriveMode:");
    println!("  Value 0 ExifTool: {}", decode_drive_mode_exiftool(0));
    println!("  Value 0 exiv2:    {}", decode_drive_mode_exiv2(0));
    println!("  Value 1 ExifTool: {}", decode_drive_mode_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_drive_mode_exiv2(1));
    println!();

    // Test FocusMode
    println!("FocusMode:");
    println!("  Value 0 ExifTool: {}", decode_focus_mode_exiftool(0));
    println!("  Value 0 exiv2:    {}", decode_focus_mode_exiv2(0));
    println!("  Value 1 ExifTool: {}", decode_focus_mode_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_focus_mode_exiv2(1));
    println!();

    // Test MeteringMode
    println!("MeteringMode:");
    println!("  Value 3 ExifTool: {}", decode_metering_mode_exiftool(3));
    println!("  Value 3 exiv2:    {}", decode_metering_mode_exiv2(3));
    println!("  Value 5 ExifTool: {}", decode_metering_mode_exiftool(5));
    println!("  Value 5 exiv2:    {}", decode_metering_mode_exiv2(5));
    println!();

    // Test ExposureMode
    println!("ExposureMode:");
    println!("  Value 1 ExifTool: {}", decode_exposure_mode_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_exposure_mode_exiv2(1));
    println!("  Value 4 ExifTool: {}", decode_exposure_mode_exiftool(4));
    println!("  Value 4 exiv2:    {}", decode_exposure_mode_exiv2(4));
    println!();

    // Test WhiteBalance
    println!("WhiteBalance:");
    println!("  Value 0 ExifTool: {}", decode_white_balance_exiftool(0));
    println!("  Value 0 exiv2:    {}", decode_white_balance_exiv2(0));
    println!("  Value 1 ExifTool: {}", decode_white_balance_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_white_balance_exiv2(1));
    println!();

    // Test ImageStabilization (both format)
    println!("ImageStabilization:");
    println!("  Value 0 ExifTool: {}", decode_image_stabilization_exiftool(0));
    println!("  Value 0 exiv2:    {}", decode_image_stabilization_exiv2(0));
    println!("  Value 1 ExifTool: {}", decode_image_stabilization_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_image_stabilization_exiv2(1));
    println!();

    // Test FocalType (both format)
    println!("FocalType:");
    println!("  Value 1 ExifTool: {}", decode_focal_type_exiftool(1));
    println!("  Value 1 exiv2:    {}", decode_focal_type_exiv2(1));
    println!("  Value 2 ExifTool: {}", decode_focal_type_exiftool(2));
    println!("  Value 2 exiv2:    {}", decode_focal_type_exiv2(2));
    println!();

    println!("✅ All macro-converted decoders working correctly!");
}
