// Example demonstrating the define_tag_decoder! macro
// This shows how the macro simplifies tag decoder definitions

use fpexif::define_tag_decoder;

// ============================================================================
// Example 1: Different mappings for ExifTool and exiv2
// ============================================================================

// This is a real example from Panasonic's ImageQuality tag
define_tag_decoder! {
    image_quality,
    exiftool: {
        1 => "TIFF",
        2 => "High",
        3 => "Standard",
        6 => "Very High",
        7 => "RAW",
        9 => "Motion Picture",
        11 => "Full HD Movie",
        12 => "4K Movie",
    },
    exiv2: {
        1 => "TIFF",
        2 => "High",
        3 => "Normal",        // Different!
        6 => "Very High",
        7 => "Raw",           // Different capitalization!
        9 => "Motion Picture",
        11 => "Full HD Movie",
        12 => "4k Movie",     // Different capitalization!
    }
}

// ============================================================================
// Example 2: Identical mappings (use 'both')
// ============================================================================

// This is a real example from Panasonic's FocusMode tag
define_tag_decoder! {
    focus_mode,
    both: {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
    }
}

// ============================================================================
// Example 3: Complex enumeration (Canon LensType)
// ============================================================================

define_tag_decoder! {
    lens_type,
    exiftool: {
        0 => "n/a",
        1 => "Canon EF 50mm f/1.8",
        2 => "Canon EF 28mm f/2.8",
        3 => "Canon EF 135mm f/2.8 Soft",
        4 => "Canon EF 35-105mm f/3.5-4.5",
        5 => "Canon EF 35-70mm f/3.5-4.5",
        6 => "Canon EF 28-70mm f/3.5-4.5",
    },
    exiv2: {
        0 => "n/a",
        1 => "Canon EF 50mm f/1.8",
        2 => "Canon EF 28mm f/2.8",
        3 => "Canon EF 135mm f/2.8 Soft",
        4 => "Canon EF 35-105mm f/3.5-4.5",
        5 => "Canon EF 35-70mm f/3.5-4.5",
        6 => "Canon EF 28-70mm f/3.5-4.5",
    }
}

// ============================================================================
// Example 4: Simple boolean-like values
// ============================================================================

define_tag_decoder! {
    hdr,
    both: {
        0 => "Off",
        1 => "On (mode 1)",
        2 => "On (mode 2)",
        3 => "On (mode 3)",
    }
}

// ============================================================================
// Demo function showing usage
// ============================================================================

fn main() {
    println!("=== Tag Decoder Macro Demo ===\n");

    // Example 1: Different mappings
    println!("ImageQuality (value=3):");
    println!("  ExifTool: {}", decode_image_quality_exiftool(3));
    println!("  exiv2:    {}", decode_image_quality_exiv2(3));
    println!();

    // Example 2: Same mapping
    println!("FocusMode (value=6):");
    println!("  ExifTool: {}", decode_focus_mode_exiftool(6));
    println!("  exiv2:    {}", decode_focus_mode_exiv2(6));
    println!();

    // Example 3: Lens type
    println!("LensType (value=1):");
    println!("  ExifTool: {}", decode_lens_type_exiftool(1));
    println!("  exiv2:    {}", decode_lens_type_exiv2(1));
    println!();

    // Example 4: Boolean-like
    println!("HDR (value=1):");
    println!("  ExifTool: {}", decode_hdr_exiftool(1));
    println!("  exiv2:    {}", decode_hdr_exiv2(1));
    println!();

    // Unknown values
    println!("Unknown value (999):");
    println!("  ExifTool: {}", decode_focus_mode_exiftool(999));
    println!("  exiv2:    {}", decode_focus_mode_exiv2(999));
}
