// Canon maker notes tests

use fpexif::data_types::ExifValue;
use fpexif::makernotes::canon::{
    decode_af_info2, decode_camera_settings, decode_file_info, decode_focal_length,
    decode_shot_info,
};
use fpexif::ExifParser;
use std::path::Path;

#[test]
fn test_decode_camera_settings() {
    // Sample CameraSettings array from Canon PowerShot S90
    let camera_settings = vec![
        96, 2, 0, 4, 0, 1, 0, 4, 65535, 7, 65535, 1, 0, 0, 0, 0, 16464, 3, 1, 8197, 3, 32767,
        65535, 22500, 6000, 1000, 95, 192, 65535, 0, 0, 0, 0, 0, 2, 0, 3648, 3648, 0, 0, 65535, 0,
        32767, 32767, 0, 0, 0, 100,
    ];

    let decoded = decode_camera_settings(&camera_settings);

    // Check macro mode
    assert!(decoded.contains_key("MacroMode"));
    if let Some(ExifValue::Ascii(mode)) = decoded.get("MacroMode") {
        assert_eq!(mode, "Normal");
    } else {
        panic!("MacroMode should be Ascii");
    }

    // Check quality
    assert!(decoded.contains_key("Quality"));
    if let Some(ExifValue::Ascii(quality)) = decoded.get("Quality") {
        assert_eq!(quality, "RAW");
    } else {
        panic!("Quality should be Ascii");
    }

    // Check flash mode (renamed to CanonFlashMode to match ExifTool)
    assert!(decoded.contains_key("CanonFlashMode"));
    if let Some(ExifValue::Ascii(flash)) = decoded.get("CanonFlashMode") {
        assert_eq!(flash, "Off");
    } else {
        panic!("CanonFlashMode should be Ascii");
    }

    // Check continuous drive (formerly called DriveMode, renamed to match ExifTool)
    assert!(decoded.contains_key("ContinuousDrive"));
    if let Some(ExifValue::Ascii(drive)) = decoded.get("ContinuousDrive") {
        assert_eq!(drive, "Continuous Shooting");
    } else {
        panic!("ContinuousDrive should be Ascii");
    }

    // Check focus mode
    assert!(decoded.contains_key("FocusMode"));

    // Note: AFAssistBeam is in CustomFunctions, not CameraSettings

    // Check metering mode
    assert!(decoded.contains_key("MeteringMode"));
    if let Some(ExifValue::Ascii(metering)) = decoded.get("MeteringMode") {
        assert_eq!(metering, "Evaluative");
    } else {
        panic!("MeteringMode should be Ascii");
    }

    // Check focus range
    assert!(decoded.contains_key("FocusRange"));

    // Check exposure mode (renamed to CanonExposureMode to match ExifTool)
    assert!(decoded.contains_key("CanonExposureMode"));

    // Check focal length values
    assert!(decoded.contains_key("MaxFocalLength"));
    assert!(decoded.contains_key("MinFocalLength"));
}

#[test]
fn test_decode_shot_info() {
    // Sample ShotInfo array
    let shot_info = vec![
        68, 0, 149, 266, 159, 223, 12, 0, 0, 1, 3, 0, 0, 0, 0, 0, 0, 0, 1, 612, 0, 160, 219, 0, 0,
        65535, 250, 0, 0, 0, 0, 0, 0, 0,
    ];

    let decoded = decode_shot_info(&shot_info);

    // Check that we got some decoded values
    assert!(decoded.contains_key("AutoISO"));
    assert!(decoded.contains_key("BaseISO"));
    assert!(decoded.contains_key("MeasuredEV"));
    assert!(decoded.contains_key("TargetAperture"));
    assert!(decoded.contains_key("TargetExposureTime"));
    assert!(decoded.contains_key("WhiteBalance"));

    // Check white balance value
    if let Some(ExifValue::Ascii(wb)) = decoded.get("WhiteBalance") {
        assert_eq!(wb, "Auto");
    } else {
        panic!("WhiteBalance should be Ascii");
    }

    // Check sequence number
    assert!(decoded.contains_key("SequenceNumber"));
    if let Some(ExifValue::Short(seq)) = decoded.get("SequenceNumber") {
        assert_eq!(seq[0], 1);
    } else {
        panic!("SequenceNumber should be Short");
    }
}

#[test]
fn test_decode_focal_length() {
    // Sample FocalLength array
    let focal_length = vec![2, 8557, 299, 224];

    let decoded = decode_focal_length(&focal_length);

    // Check focal type
    assert!(decoded.contains_key("FocalType"));
    if let Some(ExifValue::Ascii(focal_type)) = decoded.get("FocalType") {
        assert_eq!(focal_type, "Zoom");
    } else {
        panic!("FocalType should be Ascii");
    }

    // Check focal length value (formatted as "X.Y mm" string)
    assert!(decoded.contains_key("FocalLength"));
    if let Some(ExifValue::Ascii(fl)) = decoded.get("FocalLength") {
        assert_eq!(fl, "8557.0 mm");
    } else {
        panic!("FocalLength should be Ascii");
    }

    // Note: FocalPlaneXSize and FocalPlaneYSize are derived fields that exiftool
    // calculates from sensor dimensions, not from this array, so we don't test them here.
}

#[test]
fn test_decode_file_info() {
    // Sample FileInfo array: [count, file_number(skipped), reserved, bracket_mode, bracket_value, bracket_shot_number]
    // FileNumber at index 1 is skipped - it requires complex model-specific bit manipulation
    // and is redundant with the main FileNumber tag (0x0008)
    let file_info = vec![0, 123, 0, 0, 0, 0];

    let decoded = decode_file_info(&file_info);

    // Check bracket mode (index 3)
    assert!(decoded.contains_key("BracketMode"));
    if let Some(ExifValue::Ascii(mode)) = decoded.get("BracketMode") {
        assert_eq!(mode, "Off");
    } else {
        panic!("BracketMode should be Ascii");
    }

    // Check bracket value (index 4)
    assert!(decoded.contains_key("BracketValue"));

    // Check bracket shot number (index 5)
    assert!(decoded.contains_key("BracketShotNumber"));
}

#[test]
fn test_canon_s90_real_file() {
    let test_file = Path::new("test-data/RAW_CANON_S90.CR2");
    if !test_file.exists() {
        eprintln!("Skipping test: test file not found");
        return;
    }

    let parser = ExifParser::new();
    let exif_data = parser.parse_file(test_file).expect("Failed to parse file");

    // Check that we have basic Canon maker notes
    let maker_notes = exif_data
        .get_maker_notes()
        .expect("Should have maker notes");
    assert!(!maker_notes.is_empty(), "Should have Canon maker notes");

    // Check for decoded sub-fields from CameraSettings
    let has_quality = maker_notes
        .values()
        .any(|tag| tag.tag_name == Some("Quality"));
    assert!(
        has_quality,
        "Should have Quality tag (decoded from CameraSettings)"
    );

    let has_continuous_drive = maker_notes
        .values()
        .any(|tag| tag.tag_name == Some("ContinuousDrive"));
    assert!(
        has_continuous_drive,
        "Should have ContinuousDrive tag (decoded from CameraSettings)"
    );

    // Check for decoded sub-fields from FocalLength
    let has_focal_type = maker_notes
        .values()
        .any(|tag| tag.tag_name == Some("FocalType"));
    assert!(
        has_focal_type,
        "Should have FocalType tag (decoded from FocalLength)"
    );

    // Check for decoded sub-fields from ShotInfo
    let has_white_balance = maker_notes
        .values()
        .any(|tag| tag.tag_name == Some("WhiteBalance"));
    assert!(
        has_white_balance,
        "Should have WhiteBalance tag (decoded from ShotInfo)"
    );
}

#[test]
fn test_canon_50d_real_file() {
    let test_file = Path::new("test-data/RAW_CANON_50D.CR2");
    if !test_file.exists() {
        eprintln!("Skipping test: test file not found");
        return;
    }

    let parser = ExifParser::new();
    let exif_data = parser.parse_file(test_file).expect("Failed to parse file");

    // Check that we have basic Canon maker notes
    let maker_notes = exif_data
        .get_maker_notes()
        .expect("Should have maker notes");
    assert!(!maker_notes.is_empty(), "Should have Canon maker notes");

    // Check for model ID
    let has_model_id = maker_notes
        .values()
        .any(|tag| tag.tag_name == Some("CanonModelID"));
    assert!(has_model_id, "Should have CanonModelID tag");
}

#[test]
fn test_decode_camera_settings_exposure_modes() {
    // Test different exposure modes
    let mut settings = vec![0u16; 25];

    // Program AE
    settings[20] = 1;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("CanonExposureMode") {
        assert_eq!(mode, "Program AE");
    }

    // Shutter priority
    settings[20] = 2;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("CanonExposureMode") {
        assert_eq!(mode, "Shutter speed priority AE");
    }

    // Aperture priority
    settings[20] = 3;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("CanonExposureMode") {
        assert_eq!(mode, "Aperture-priority AE");
    }

    // Manual
    settings[20] = 4;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("CanonExposureMode") {
        assert_eq!(mode, "Manual");
    }
}

#[test]
fn test_decode_shot_info_white_balance_modes() {
    // Test different white balance modes
    let mut shot_info = vec![0u16; 10];

    // Daylight
    shot_info[7] = 1;
    let decoded = decode_shot_info(&shot_info);
    if let Some(ExifValue::Ascii(wb)) = decoded.get("WhiteBalance") {
        assert_eq!(wb, "Daylight");
    }

    // Cloudy
    shot_info[7] = 2;
    let decoded = decode_shot_info(&shot_info);
    if let Some(ExifValue::Ascii(wb)) = decoded.get("WhiteBalance") {
        assert_eq!(wb, "Cloudy");
    }

    // Tungsten
    shot_info[7] = 3;
    let decoded = decode_shot_info(&shot_info);
    if let Some(ExifValue::Ascii(wb)) = decoded.get("WhiteBalance") {
        assert_eq!(wb, "Tungsten");
    }

    // Flash
    shot_info[7] = 5;
    let decoded = decode_shot_info(&shot_info);
    if let Some(ExifValue::Ascii(wb)) = decoded.get("WhiteBalance") {
        assert_eq!(wb, "Flash");
    }
}

#[test]
fn test_decode_af_info2() {
    // Sample AFInfo2 array with 3 AF points
    let af_info = vec![
        2, // AF area mode: Single-point AF
        3, // Number of AF points
        10, 12, 14, // Widths for 3 points
        20, 22, 24, // Heights for 3 points
        100, 110, 120, // X positions for 3 points
        200, 210, 220, // Y positions for 3 points
    ];

    let decoded = decode_af_info2(&af_info);

    // Check AF area mode
    assert!(decoded.contains_key("AFAreaMode"));
    if let Some(ExifValue::Ascii(mode)) = decoded.get("AFAreaMode") {
        assert_eq!(mode, "Single-point AF");
    } else {
        panic!("AFAreaMode should be Ascii");
    }

    // Check number of AF points
    assert!(decoded.contains_key("NumAFPoints"));
    if let Some(ExifValue::Short(num)) = decoded.get("NumAFPoints") {
        assert_eq!(num[0], 3);
    } else {
        panic!("NumAFPoints should be Short");
    }

    // Check AF area widths
    assert!(decoded.contains_key("AFAreaWidths"));
    if let Some(ExifValue::Short(widths)) = decoded.get("AFAreaWidths") {
        assert_eq!(widths.len(), 3);
        assert_eq!(widths[0], 10);
        assert_eq!(widths[1], 12);
        assert_eq!(widths[2], 14);
    } else {
        panic!("AFAreaWidths should be Short array");
    }

    // Check AF area heights
    assert!(decoded.contains_key("AFAreaHeights"));
    if let Some(ExifValue::Short(heights)) = decoded.get("AFAreaHeights") {
        assert_eq!(heights.len(), 3);
        assert_eq!(heights[0], 20);
        assert_eq!(heights[1], 22);
        assert_eq!(heights[2], 24);
    } else {
        panic!("AFAreaHeights should be Short array");
    }

    // Check AF area X positions
    assert!(decoded.contains_key("AFAreaXPositions"));
    if let Some(ExifValue::Short(x_pos)) = decoded.get("AFAreaXPositions") {
        assert_eq!(x_pos.len(), 3);
        assert_eq!(x_pos[0], 100);
        assert_eq!(x_pos[1], 110);
        assert_eq!(x_pos[2], 120);
    } else {
        panic!("AFAreaXPositions should be Short array");
    }

    // Check AF area Y positions
    assert!(decoded.contains_key("AFAreaYPositions"));
    if let Some(ExifValue::Short(y_pos)) = decoded.get("AFAreaYPositions") {
        assert_eq!(y_pos.len(), 3);
        assert_eq!(y_pos[0], 200);
        assert_eq!(y_pos[1], 210);
        assert_eq!(y_pos[2], 220);
    } else {
        panic!("AFAreaYPositions should be Short array");
    }
}

#[test]
fn test_decode_af_info2_different_modes() {
    // Test different AF area modes
    let mut af_info = vec![0u16; 2];

    // Manual focus
    af_info[0] = 0;
    let decoded = decode_af_info2(&af_info);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("AFAreaMode") {
        assert_eq!(mode, "Off (Manual Focus)");
    }

    // Face Detect AF
    af_info[0] = 5;
    let decoded = decode_af_info2(&af_info);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("AFAreaMode") {
        assert_eq!(mode, "Face Detect AF");
    }

    // Zone AF
    af_info[0] = 7;
    let decoded = decode_af_info2(&af_info);
    if let Some(ExifValue::Ascii(mode)) = decoded.get("AFAreaMode") {
        assert_eq!(mode, "Zone AF");
    }
}

#[test]
fn test_camera_settings_af_assist_beam() {
    // Test AF assist beam settings
    let mut settings = vec![0u16; 12];

    // AF assist beam off
    settings[10] = 0;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(af_assist)) = decoded.get("AFAssistBeam") {
        assert_eq!(af_assist, "Off");
    }

    // AF assist beam on (auto)
    settings[10] = 1;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(af_assist)) = decoded.get("AFAssistBeam") {
        assert_eq!(af_assist, "On (Auto)");
    }

    // AF assist beam on
    settings[10] = 2;
    let decoded = decode_camera_settings(&settings);
    if let Some(ExifValue::Ascii(af_assist)) = decoded.get("AFAssistBeam") {
        assert_eq!(af_assist, "On");
    }
}
