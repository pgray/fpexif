// tags.rs - EXIF tag definitions and mappings
use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Represents an EXIF tag identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ExifTagId {
    /// The numeric identifier of the tag
    pub id: u16,
    /// The IFD (Image File Directory) the tag belongs to
    pub ifd: TagGroup,
}

/// Different groups of EXIF tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum TagGroup {
    /// Main image tags (IFD0)
    Main,
    /// Thumbnail image tags (IFD1)
    Thumbnail,
    /// EXIF specific tags
    Exif,
    /// GPS tags
    Gps,
    /// Interoperability tags
    Interop,
}

impl ExifTagId {
    /// Create a new tag identifier
    pub fn new(id: u16, ifd: TagGroup) -> Self {
        Self { id, ifd }
    }

    /// Get the name of the tag, if known
    pub fn name(&self) -> Option<&'static str> {
        get_tag_name(*self)
    }
}

impl From<u16> for ExifTagId {
    fn from(id: u16) -> Self {
        // By default, assume it's in the main IFD
        Self::new(id, TagGroup::Main)
    }
}

impl fmt::Display for ExifTagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{} (0x{:04X})", name, self.id)
        } else {
            write!(f, "Unknown Tag (0x{:04X})", self.id)
        }
    }
}

// Common EXIF tag IDs
pub const TAG_NEW_SUBFILE_TYPE: ExifTagId = ExifTagId {
    id: 0x00FE,
    ifd: TagGroup::Main,
};
pub const TAG_IMAGE_WIDTH: ExifTagId = ExifTagId {
    id: 0x0100,
    ifd: TagGroup::Main,
};
pub const TAG_IMAGE_LENGTH: ExifTagId = ExifTagId {
    id: 0x0101,
    ifd: TagGroup::Main,
};
pub const TAG_BITS_PER_SAMPLE: ExifTagId = ExifTagId {
    id: 0x0102,
    ifd: TagGroup::Main,
};
pub const TAG_COMPRESSION: ExifTagId = ExifTagId {
    id: 0x0103,
    ifd: TagGroup::Main,
};
pub const TAG_PHOTOMETRIC_INTERPRETATION: ExifTagId = ExifTagId {
    id: 0x0106,
    ifd: TagGroup::Main,
};
pub const TAG_IMAGE_DESCRIPTION: ExifTagId = ExifTagId {
    id: 0x010E,
    ifd: TagGroup::Main,
};
pub const TAG_MAKE: ExifTagId = ExifTagId {
    id: 0x010F,
    ifd: TagGroup::Main,
};
pub const TAG_MODEL: ExifTagId = ExifTagId {
    id: 0x0110,
    ifd: TagGroup::Main,
};
pub const TAG_ORIENTATION: ExifTagId = ExifTagId {
    id: 0x0112,
    ifd: TagGroup::Main,
};
pub const TAG_SAMPLES_PER_PIXEL: ExifTagId = ExifTagId {
    id: 0x0115,
    ifd: TagGroup::Main,
};
pub const TAG_ROWS_PER_STRIP: ExifTagId = ExifTagId {
    id: 0x0116,
    ifd: TagGroup::Main,
};
pub const TAG_STRIP_OFFSETS: ExifTagId = ExifTagId {
    id: 0x0111,
    ifd: TagGroup::Main,
};
pub const TAG_STRIP_BYTE_COUNTS: ExifTagId = ExifTagId {
    id: 0x0117,
    ifd: TagGroup::Main,
};
pub const TAG_MIN_SAMPLE_VALUE: ExifTagId = ExifTagId {
    id: 0x0118,
    ifd: TagGroup::Main,
};
pub const TAG_X_RESOLUTION: ExifTagId = ExifTagId {
    id: 0x011A,
    ifd: TagGroup::Main,
};
pub const TAG_Y_RESOLUTION: ExifTagId = ExifTagId {
    id: 0x011B,
    ifd: TagGroup::Main,
};
pub const TAG_PLANAR_CONFIGURATION: ExifTagId = ExifTagId {
    id: 0x011C,
    ifd: TagGroup::Main,
};
pub const TAG_RESOLUTION_UNIT: ExifTagId = ExifTagId {
    id: 0x0128,
    ifd: TagGroup::Main,
};
pub const TAG_TRANSFER_FUNCTION: ExifTagId = ExifTagId {
    id: 0x012D,
    ifd: TagGroup::Main,
};
pub const TAG_SOFTWARE: ExifTagId = ExifTagId {
    id: 0x0131,
    ifd: TagGroup::Main,
};
pub const TAG_DATE_TIME: ExifTagId = ExifTagId {
    id: 0x0132,
    ifd: TagGroup::Main,
};
pub const TAG_ARTIST: ExifTagId = ExifTagId {
    id: 0x013B,
    ifd: TagGroup::Main,
};
pub const TAG_WHITE_POINT: ExifTagId = ExifTagId {
    id: 0x013E,
    ifd: TagGroup::Main,
};
pub const TAG_PRIMARY_CHROMATICITIES: ExifTagId = ExifTagId {
    id: 0x013F,
    ifd: TagGroup::Main,
};
pub const TAG_SUB_IFDS: ExifTagId = ExifTagId {
    id: 0x014A,
    ifd: TagGroup::Main,
};
pub const TAG_JPEG_INTERCHANGE_FORMAT: ExifTagId = ExifTagId {
    id: 0x0201,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: ExifTagId = ExifTagId {
    id: 0x0202,
    ifd: TagGroup::Thumbnail,
};
// Additional Thumbnail IFD tags (same IDs as Main but in IFD1)
pub const TAG_THUMBNAIL_COMPRESSION: ExifTagId = ExifTagId {
    id: 0x0103,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_ORIENTATION: ExifTagId = ExifTagId {
    id: 0x0112,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_X_RESOLUTION: ExifTagId = ExifTagId {
    id: 0x011A,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_Y_RESOLUTION: ExifTagId = ExifTagId {
    id: 0x011B,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_RESOLUTION_UNIT: ExifTagId = ExifTagId {
    id: 0x0128,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_YCBCR_POSITIONING: ExifTagId = ExifTagId {
    id: 0x0213,
    ifd: TagGroup::Thumbnail,
};
// Some manufacturers put Main IFD tags in Thumbnail IFD
pub const TAG_THUMBNAIL_NEW_SUBFILE_TYPE: ExifTagId = ExifTagId {
    id: 0x00FE,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_IMAGE_DESCRIPTION: ExifTagId = ExifTagId {
    id: 0x010E,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_MAKE: ExifTagId = ExifTagId {
    id: 0x010F,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_MODEL: ExifTagId = ExifTagId {
    id: 0x0110,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_SOFTWARE: ExifTagId = ExifTagId {
    id: 0x0131,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_DATE_TIME: ExifTagId = ExifTagId {
    id: 0x0132,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_THUMBNAIL_PRINT_IM: ExifTagId = ExifTagId {
    id: 0xC4A5,
    ifd: TagGroup::Thumbnail,
};
pub const TAG_YCBCR_COEFFICIENTS: ExifTagId = ExifTagId {
    id: 0x0211,
    ifd: TagGroup::Main,
};
pub const TAG_YCBCR_SUB_SAMPLING: ExifTagId = ExifTagId {
    id: 0x0212,
    ifd: TagGroup::Main,
};
pub const TAG_YCBCR_POSITIONING: ExifTagId = ExifTagId {
    id: 0x0213,
    ifd: TagGroup::Main,
};
pub const TAG_REFERENCE_BLACK_WHITE: ExifTagId = ExifTagId {
    id: 0x0214,
    ifd: TagGroup::Main,
};
pub const TAG_XMP_METADATA: ExifTagId = ExifTagId {
    id: 0x02BC,
    ifd: TagGroup::Main,
};
pub const TAG_IPTC_NAA: ExifTagId = ExifTagId {
    id: 0x83BB,
    ifd: TagGroup::Main,
};
// Some manufacturers put these EXIF tags in Main IFD instead of EXIF SubIFD
pub const TAG_EXPOSURE_TIME_MAIN: ExifTagId = ExifTagId {
    id: 0x829A,
    ifd: TagGroup::Main,
};
pub const TAG_F_NUMBER_MAIN: ExifTagId = ExifTagId {
    id: 0x829D,
    ifd: TagGroup::Main,
};
pub const TAG_EXPOSURE_PROGRAM_MAIN: ExifTagId = ExifTagId {
    id: 0x8822,
    ifd: TagGroup::Main,
};
pub const TAG_EXPOSURE_BIAS_VALUE_MAIN: ExifTagId = ExifTagId {
    id: 0x9204,
    ifd: TagGroup::Main,
};
pub const TAG_MAX_APERTURE_VALUE_MAIN: ExifTagId = ExifTagId {
    id: 0x9205,
    ifd: TagGroup::Main,
};
pub const TAG_METERING_MODE_MAIN: ExifTagId = ExifTagId {
    id: 0x9207,
    ifd: TagGroup::Main,
};
pub const TAG_LIGHT_SOURCE_MAIN: ExifTagId = ExifTagId {
    id: 0x9208,
    ifd: TagGroup::Main,
};
pub const TAG_FOCAL_LENGTH_MAIN: ExifTagId = ExifTagId {
    id: 0x920A,
    ifd: TagGroup::Main,
};
pub const TAG_IMAGE_NUMBER_MAIN: ExifTagId = ExifTagId {
    id: 0x9211,
    ifd: TagGroup::Main,
};
pub const TAG_SENSING_METHOD_MAIN: ExifTagId = ExifTagId {
    id: 0x9217,
    ifd: TagGroup::Main,
};
pub const TAG_DATE_TIME_ORIGINAL_MAIN: ExifTagId = ExifTagId {
    id: 0x9003,
    ifd: TagGroup::Main,
};
pub const TAG_TIFF_EP_STANDARD_ID_MAIN: ExifTagId = ExifTagId {
    id: 0x9216,
    ifd: TagGroup::Main,
};
// Some manufacturers put basic TIFF structure tags in EXIF SubIFD
pub const TAG_IMAGE_WIDTH_EXIF: ExifTagId = ExifTagId {
    id: 0x0100,
    ifd: TagGroup::Exif,
};
pub const TAG_IMAGE_LENGTH_EXIF: ExifTagId = ExifTagId {
    id: 0x0101,
    ifd: TagGroup::Exif,
};
pub const TAG_BITS_PER_SAMPLE_EXIF: ExifTagId = ExifTagId {
    id: 0x0102,
    ifd: TagGroup::Exif,
};
pub const TAG_PHOTOMETRIC_INTERPRETATION_EXIF: ExifTagId = ExifTagId {
    id: 0x0106,
    ifd: TagGroup::Exif,
};
pub const TAG_STRIP_OFFSETS_EXIF: ExifTagId = ExifTagId {
    id: 0x0111,
    ifd: TagGroup::Exif,
};
pub const TAG_SAMPLES_PER_PIXEL_EXIF: ExifTagId = ExifTagId {
    id: 0x0115,
    ifd: TagGroup::Exif,
};
pub const TAG_ROWS_PER_STRIP_EXIF: ExifTagId = ExifTagId {
    id: 0x0116,
    ifd: TagGroup::Exif,
};
pub const TAG_STRIP_BYTE_COUNTS_EXIF: ExifTagId = ExifTagId {
    id: 0x0117,
    ifd: TagGroup::Exif,
};
pub const TAG_PLANAR_CONFIGURATION_EXIF: ExifTagId = ExifTagId {
    id: 0x011C,
    ifd: TagGroup::Exif,
};
pub const TAG_COPYRIGHT: ExifTagId = ExifTagId {
    id: 0x8298,
    ifd: TagGroup::Main,
};
pub const TAG_ICC_PROFILE: ExifTagId = ExifTagId {
    id: 0x8773,
    ifd: TagGroup::Main,
};
pub const TAG_EXIF_OFFSET: ExifTagId = ExifTagId {
    id: 0x8769,
    ifd: TagGroup::Main,
};
pub const TAG_GPS_INFO: ExifTagId = ExifTagId {
    id: 0x8825,
    ifd: TagGroup::Main,
};
// TIFF/EP Color Filter Array tags
pub const TAG_CFA_REPEAT_PATTERN_DIM: ExifTagId = ExifTagId {
    id: 0x828D,
    ifd: TagGroup::Main,
};
pub const TAG_CFA_PATTERN_TIFFEP: ExifTagId = ExifTagId {
    id: 0x828E,
    ifd: TagGroup::Main,
};
// JPEG tags in Main IFD (some RAW formats)
pub const TAG_JPEG_INTERCHANGE_FORMAT_MAIN: ExifTagId = ExifTagId {
    id: 0x0201,
    ifd: TagGroup::Main,
};
pub const TAG_JPEG_INTERCHANGE_FORMAT_LENGTH_MAIN: ExifTagId = ExifTagId {
    id: 0x0202,
    ifd: TagGroup::Main,
};

// DNG (Adobe Digital Negative) tags
pub const TAG_DNG_VERSION: ExifTagId = ExifTagId {
    id: 0xC612,
    ifd: TagGroup::Main,
};
pub const TAG_DNG_BACKWARD_VERSION: ExifTagId = ExifTagId {
    id: 0xC613,
    ifd: TagGroup::Main,
};
pub const TAG_UNIQUE_CAMERA_MODEL: ExifTagId = ExifTagId {
    id: 0xC614,
    ifd: TagGroup::Main,
};
pub const TAG_LOCALIZED_CAMERA_MODEL: ExifTagId = ExifTagId {
    id: 0xC615,
    ifd: TagGroup::Main,
};
pub const TAG_CFA_PLANE_COLOR: ExifTagId = ExifTagId {
    id: 0xC616,
    ifd: TagGroup::Main,
};
pub const TAG_CFA_LAYOUT: ExifTagId = ExifTagId {
    id: 0xC617,
    ifd: TagGroup::Main,
};
pub const TAG_LINEARIZATION_TABLE: ExifTagId = ExifTagId {
    id: 0xC618,
    ifd: TagGroup::Main,
};
pub const TAG_BLACK_LEVEL_REPEAT_DIM: ExifTagId = ExifTagId {
    id: 0xC619,
    ifd: TagGroup::Main,
};
pub const TAG_BLACK_LEVEL: ExifTagId = ExifTagId {
    id: 0xC61A,
    ifd: TagGroup::Main,
};
pub const TAG_WHITE_LEVEL: ExifTagId = ExifTagId {
    id: 0xC61D,
    ifd: TagGroup::Main,
};
pub const TAG_ORIGINAL_RAW_FILE_NAME: ExifTagId = ExifTagId {
    id: 0xC61E,
    ifd: TagGroup::Main,
};
pub const TAG_ORIGINAL_RAW_FILE_DATA: ExifTagId = ExifTagId {
    id: 0xC61F,
    ifd: TagGroup::Main,
};
pub const TAG_DEFAULT_CROP_SIZE: ExifTagId = ExifTagId {
    id: 0xC620,
    ifd: TagGroup::Main,
};
pub const TAG_COLOR_MATRIX1: ExifTagId = ExifTagId {
    id: 0xC621,
    ifd: TagGroup::Main,
};
pub const TAG_COLOR_MATRIX2: ExifTagId = ExifTagId {
    id: 0xC622,
    ifd: TagGroup::Main,
};
pub const TAG_ANALOG_BALANCE: ExifTagId = ExifTagId {
    id: 0xC627,
    ifd: TagGroup::Main,
};
pub const TAG_AS_SHOT_NEUTRAL: ExifTagId = ExifTagId {
    id: 0xC628,
    ifd: TagGroup::Main,
};
pub const TAG_BASELINE_EXPOSURE: ExifTagId = ExifTagId {
    id: 0xC62A,
    ifd: TagGroup::Main,
};
pub const TAG_BASELINE_NOISE: ExifTagId = ExifTagId {
    id: 0xC62B,
    ifd: TagGroup::Main,
};
pub const TAG_BASELINE_SHARPNESS: ExifTagId = ExifTagId {
    id: 0xC62C,
    ifd: TagGroup::Main,
};
pub const TAG_PREVIEW_COLOR_SPACE: ExifTagId = ExifTagId {
    id: 0xC62D,
    ifd: TagGroup::Main,
};
pub const TAG_LINEAR_RESPONSE_LIMIT: ExifTagId = ExifTagId {
    id: 0xC62E,
    ifd: TagGroup::Main,
};
pub const TAG_CAMERA_SERIAL_NUMBER: ExifTagId = ExifTagId {
    id: 0xC62F,
    ifd: TagGroup::Main,
};
pub const TAG_LENS_INFO: ExifTagId = ExifTagId {
    id: 0xC630,
    ifd: TagGroup::Main,
};
pub const TAG_RAW_DATA_UNIQUE_ID: ExifTagId = ExifTagId {
    id: 0xC65D,
    ifd: TagGroup::Main,
};
pub const TAG_MASKED_AREAS: ExifTagId = ExifTagId {
    id: 0xC632,
    ifd: TagGroup::Main,
};
pub const TAG_SHADOW_SCALE: ExifTagId = ExifTagId {
    id: 0xC633,
    ifd: TagGroup::Main,
};
pub const TAG_CALIBRATION_ILLUMINANT1: ExifTagId = ExifTagId {
    id: 0xC65A,
    ifd: TagGroup::Main,
};
pub const TAG_CALIBRATION_ILLUMINANT2: ExifTagId = ExifTagId {
    id: 0xC65B,
    ifd: TagGroup::Main,
};
pub const TAG_BEST_QUALITY_SCALE: ExifTagId = ExifTagId {
    id: 0xC65C,
    ifd: TagGroup::Main,
};
pub const TAG_ACTIVE_AREA: ExifTagId = ExifTagId {
    id: 0xC68D,
    ifd: TagGroup::Main,
};
pub const TAG_RAW_IMAGE_DIGEST: ExifTagId = ExifTagId {
    id: 0xC71C,
    ifd: TagGroup::Main,
};

// CR2-specific tags
pub const TAG_CR2_CFA_PATTERN: ExifTagId = ExifTagId {
    id: 0xC5E0,
    ifd: TagGroup::Main,
};
pub const TAG_SRAW_TYPE: ExifTagId = ExifTagId {
    id: 0xC6C5,
    ifd: TagGroup::Main,
};

// CRW/CIFF FocalPlaneSize tags (private range)
pub const TAG_FOCAL_PLANE_X_SIZE: ExifTagId = ExifTagId {
    id: 0xC001,
    ifd: TagGroup::Main,
};
pub const TAG_FOCAL_PLANE_Y_SIZE: ExifTagId = ExifTagId {
    id: 0xC002,
    ifd: TagGroup::Main,
};

// Sony ARW SubIFD tags (0x7030-0x7040 range)
pub const TAG_SONY_VIGNETTING_CORRECTION: ExifTagId = ExifTagId {
    id: 0x7030,
    ifd: TagGroup::Main,
};
pub const TAG_SONY_CHROMATIC_ABERRATION_CORRECTION: ExifTagId = ExifTagId {
    id: 0x7034,
    ifd: TagGroup::Main,
};
pub const TAG_SONY_DISTORTION_CORRECTION: ExifTagId = ExifTagId {
    id: 0x7036,
    ifd: TagGroup::Main,
};

// EXIF SubIFD tags
pub const TAG_EXPOSURE_TIME: ExifTagId = ExifTagId {
    id: 0x829A,
    ifd: TagGroup::Exif,
};
pub const TAG_F_NUMBER: ExifTagId = ExifTagId {
    id: 0x829D,
    ifd: TagGroup::Exif,
};
pub const TAG_EXPOSURE_PROGRAM: ExifTagId = ExifTagId {
    id: 0x8822,
    ifd: TagGroup::Exif,
};
pub const TAG_ISO_SPEED_RATINGS: ExifTagId = ExifTagId {
    id: 0x8827,
    ifd: TagGroup::Exif,
};
pub const TAG_SENSITIVITY_TYPE: ExifTagId = ExifTagId {
    id: 0x8830,
    ifd: TagGroup::Exif,
};
pub const TAG_SPECTRAL_SENSITIVITY: ExifTagId = ExifTagId {
    id: 0x8832,
    ifd: TagGroup::Exif,
};
pub const TAG_EXIF_VERSION: ExifTagId = ExifTagId {
    id: 0x9000,
    ifd: TagGroup::Exif,
};
pub const TAG_DATE_TIME_ORIGINAL: ExifTagId = ExifTagId {
    id: 0x9003,
    ifd: TagGroup::Exif,
};
pub const TAG_DATE_TIME_DIGITIZED: ExifTagId = ExifTagId {
    id: 0x9004,
    ifd: TagGroup::Exif,
};
pub const TAG_COMPONENTS_CONFIGURATION: ExifTagId = ExifTagId {
    id: 0x9101,
    ifd: TagGroup::Exif,
};
pub const TAG_COMPRESSED_BITS_PER_PIXEL: ExifTagId = ExifTagId {
    id: 0x9102,
    ifd: TagGroup::Exif,
};
pub const TAG_SHUTTER_SPEED_VALUE: ExifTagId = ExifTagId {
    id: 0x9201,
    ifd: TagGroup::Exif,
};
pub const TAG_APERTURE_VALUE: ExifTagId = ExifTagId {
    id: 0x9202,
    ifd: TagGroup::Exif,
};
pub const TAG_BRIGHTNESS_VALUE: ExifTagId = ExifTagId {
    id: 0x9203,
    ifd: TagGroup::Exif,
};
pub const TAG_EXPOSURE_BIAS_VALUE: ExifTagId = ExifTagId {
    id: 0x9204,
    ifd: TagGroup::Exif,
};
pub const TAG_MAX_APERTURE_VALUE: ExifTagId = ExifTagId {
    id: 0x9205,
    ifd: TagGroup::Exif,
};
pub const TAG_SUBJECT_DISTANCE: ExifTagId = ExifTagId {
    id: 0x9206,
    ifd: TagGroup::Exif,
};
pub const TAG_METERING_MODE: ExifTagId = ExifTagId {
    id: 0x9207,
    ifd: TagGroup::Exif,
};
pub const TAG_LIGHT_SOURCE: ExifTagId = ExifTagId {
    id: 0x9208,
    ifd: TagGroup::Exif,
};
pub const TAG_FLASH: ExifTagId = ExifTagId {
    id: 0x9209,
    ifd: TagGroup::Exif,
};
pub const TAG_FOCAL_LENGTH: ExifTagId = ExifTagId {
    id: 0x920A,
    ifd: TagGroup::Exif,
};
pub const TAG_IMAGE_NUMBER: ExifTagId = ExifTagId {
    id: 0x9211,
    ifd: TagGroup::Exif,
};
pub const TAG_SUBJECT_AREA: ExifTagId = ExifTagId {
    id: 0x9214,
    ifd: TagGroup::Exif,
};
pub const TAG_TIFF_EP_STANDARD_ID: ExifTagId = ExifTagId {
    id: 0x9216,
    ifd: TagGroup::Exif,
};
pub const TAG_MAKER_NOTE: ExifTagId = ExifTagId {
    id: 0x927C,
    ifd: TagGroup::Exif,
};
pub const TAG_USER_COMMENT: ExifTagId = ExifTagId {
    id: 0x9286,
    ifd: TagGroup::Exif,
};
pub const TAG_SUB_SEC_TIME: ExifTagId = ExifTagId {
    id: 0x9290,
    ifd: TagGroup::Exif,
};
pub const TAG_SUB_SEC_TIME_ORIGINAL: ExifTagId = ExifTagId {
    id: 0x9291,
    ifd: TagGroup::Exif,
};
pub const TAG_SUB_SEC_TIME_DIGITIZED: ExifTagId = ExifTagId {
    id: 0x9292,
    ifd: TagGroup::Exif,
};

// More EXIF tags
pub const TAG_FLASHPIX_VERSION: ExifTagId = ExifTagId {
    id: 0xA000,
    ifd: TagGroup::Exif,
};
pub const TAG_COLOR_SPACE: ExifTagId = ExifTagId {
    id: 0xA001,
    ifd: TagGroup::Exif,
};
pub const TAG_PIXEL_X_DIMENSION: ExifTagId = ExifTagId {
    id: 0xA002,
    ifd: TagGroup::Exif,
};
pub const TAG_PIXEL_Y_DIMENSION: ExifTagId = ExifTagId {
    id: 0xA003,
    ifd: TagGroup::Exif,
};
pub const TAG_RELATED_SOUND_FILE: ExifTagId = ExifTagId {
    id: 0xA004,
    ifd: TagGroup::Exif,
};
pub const TAG_INTEROPERABILITY_IFD_POINTER: ExifTagId = ExifTagId {
    id: 0xA005,
    ifd: TagGroup::Exif,
};
pub const TAG_FLASH_ENERGY: ExifTagId = ExifTagId {
    id: 0xA20B,
    ifd: TagGroup::Exif,
};
pub const TAG_SPATIAL_FREQUENCY_RESPONSE: ExifTagId = ExifTagId {
    id: 0xA20C,
    ifd: TagGroup::Exif,
};
pub const TAG_FOCAL_PLANE_X_RESOLUTION: ExifTagId = ExifTagId {
    id: 0xA20E,
    ifd: TagGroup::Exif,
};
pub const TAG_FOCAL_PLANE_Y_RESOLUTION: ExifTagId = ExifTagId {
    id: 0xA20F,
    ifd: TagGroup::Exif,
};
pub const TAG_FOCAL_PLANE_RESOLUTION_UNIT: ExifTagId = ExifTagId {
    id: 0xA210,
    ifd: TagGroup::Exif,
};
pub const TAG_SUBJECT_LOCATION: ExifTagId = ExifTagId {
    id: 0xA214,
    ifd: TagGroup::Exif,
};
pub const TAG_EXPOSURE_INDEX: ExifTagId = ExifTagId {
    id: 0xA215,
    ifd: TagGroup::Exif,
};
pub const TAG_SENSING_METHOD: ExifTagId = ExifTagId {
    id: 0xA217,
    ifd: TagGroup::Exif,
};
pub const TAG_FILE_SOURCE: ExifTagId = ExifTagId {
    id: 0xA300,
    ifd: TagGroup::Exif,
};
pub const TAG_SCENE_TYPE: ExifTagId = ExifTagId {
    id: 0xA301,
    ifd: TagGroup::Exif,
};
pub const TAG_CFA_PATTERN: ExifTagId = ExifTagId {
    id: 0xA302,
    ifd: TagGroup::Exif,
};
pub const TAG_CUSTOM_RENDERED: ExifTagId = ExifTagId {
    id: 0xA401,
    ifd: TagGroup::Exif,
};
pub const TAG_EXPOSURE_MODE: ExifTagId = ExifTagId {
    id: 0xA402,
    ifd: TagGroup::Exif,
};
pub const TAG_WHITE_BALANCE: ExifTagId = ExifTagId {
    id: 0xA403,
    ifd: TagGroup::Exif,
};
pub const TAG_DIGITAL_ZOOM_RATIO: ExifTagId = ExifTagId {
    id: 0xA404,
    ifd: TagGroup::Exif,
};
pub const TAG_FOCAL_LENGTH_IN_35MM_FILM: ExifTagId = ExifTagId {
    id: 0xA405,
    ifd: TagGroup::Exif,
};
pub const TAG_SCENE_CAPTURE_TYPE: ExifTagId = ExifTagId {
    id: 0xA406,
    ifd: TagGroup::Exif,
};
pub const TAG_GAIN_CONTROL: ExifTagId = ExifTagId {
    id: 0xA407,
    ifd: TagGroup::Exif,
};
pub const TAG_CONTRAST: ExifTagId = ExifTagId {
    id: 0xA408,
    ifd: TagGroup::Exif,
};
pub const TAG_SATURATION: ExifTagId = ExifTagId {
    id: 0xA409,
    ifd: TagGroup::Exif,
};
pub const TAG_SHARPNESS: ExifTagId = ExifTagId {
    id: 0xA40A,
    ifd: TagGroup::Exif,
};
pub const TAG_DEVICE_SETTING_DESCRIPTION: ExifTagId = ExifTagId {
    id: 0xA40B,
    ifd: TagGroup::Exif,
};
pub const TAG_SUBJECT_DISTANCE_RANGE: ExifTagId = ExifTagId {
    id: 0xA40C,
    ifd: TagGroup::Exif,
};
pub const TAG_IMAGE_UNIQUE_ID: ExifTagId = ExifTagId {
    id: 0xA420,
    ifd: TagGroup::Exif,
};
pub const TAG_CAMERA_OWNER_NAME: ExifTagId = ExifTagId {
    id: 0xA430,
    ifd: TagGroup::Exif,
};
pub const TAG_BODY_SERIAL_NUMBER: ExifTagId = ExifTagId {
    id: 0xA431,
    ifd: TagGroup::Exif,
};
pub const TAG_LENS_SPECIFICATION: ExifTagId = ExifTagId {
    id: 0xA432,
    ifd: TagGroup::Exif,
};
pub const TAG_LENS_MAKE: ExifTagId = ExifTagId {
    id: 0xA433,
    ifd: TagGroup::Exif,
};
pub const TAG_LENS_MODEL: ExifTagId = ExifTagId {
    id: 0xA434,
    ifd: TagGroup::Exif,
};
pub const TAG_LENS_SERIAL_NUMBER: ExifTagId = ExifTagId {
    id: 0xA435,
    ifd: TagGroup::Exif,
};
pub const TAG_PRINT_IM: ExifTagId = ExifTagId {
    id: 0xC4A5,
    ifd: TagGroup::Main,
};

// GPS tags
pub const TAG_GPS_VERSION_ID: ExifTagId = ExifTagId {
    id: 0x0000,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_LATITUDE_REF: ExifTagId = ExifTagId {
    id: 0x0001,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_LATITUDE: ExifTagId = ExifTagId {
    id: 0x0002,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_LONGITUDE_REF: ExifTagId = ExifTagId {
    id: 0x0003,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_LONGITUDE: ExifTagId = ExifTagId {
    id: 0x0004,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_ALTITUDE_REF: ExifTagId = ExifTagId {
    id: 0x0005,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_ALTITUDE: ExifTagId = ExifTagId {
    id: 0x0006,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_TIMESTAMP: ExifTagId = ExifTagId {
    id: 0x0007,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_SATELLITES: ExifTagId = ExifTagId {
    id: 0x0008,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_STATUS: ExifTagId = ExifTagId {
    id: 0x0009,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_MEASURE_MODE: ExifTagId = ExifTagId {
    id: 0x000A,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_DOP: ExifTagId = ExifTagId {
    id: 0x000B,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_SPEED_REF: ExifTagId = ExifTagId {
    id: 0x000C,
    ifd: TagGroup::Gps,
};
pub const TAG_GPS_SPEED: ExifTagId = ExifTagId {
    id: 0x000D,
    ifd: TagGroup::Gps,
};

// Interoperability IFD tags
pub const TAG_INTEROP_INDEX: ExifTagId = ExifTagId {
    id: 0x0001,
    ifd: TagGroup::Interop,
};
pub const TAG_INTEROP_VERSION: ExifTagId = ExifTagId {
    id: 0x0002,
    ifd: TagGroup::Interop,
};

// A static mapping of tag IDs to names
static TAG_NAMES: OnceLock<HashMap<ExifTagId, &'static str>> = OnceLock::new();

// Initialize tag names map
fn init_tag_names() -> HashMap<ExifTagId, &'static str> {
    let mut map = HashMap::new();

    // Add tag names for common tags
    map.insert(TAG_NEW_SUBFILE_TYPE, "NewSubfileType");
    map.insert(TAG_IMAGE_WIDTH, "ImageWidth");
    map.insert(TAG_IMAGE_LENGTH, "ImageLength");
    map.insert(TAG_BITS_PER_SAMPLE, "BitsPerSample");
    map.insert(TAG_COMPRESSION, "Compression");
    map.insert(TAG_PHOTOMETRIC_INTERPRETATION, "PhotometricInterpretation");
    map.insert(TAG_IMAGE_DESCRIPTION, "ImageDescription");
    map.insert(TAG_MAKE, "Make");
    map.insert(TAG_MODEL, "Model");
    map.insert(TAG_ORIENTATION, "Orientation");
    map.insert(TAG_SAMPLES_PER_PIXEL, "SamplesPerPixel");
    map.insert(TAG_ROWS_PER_STRIP, "RowsPerStrip");
    map.insert(TAG_STRIP_OFFSETS, "StripOffsets");
    map.insert(TAG_STRIP_BYTE_COUNTS, "StripByteCounts");
    map.insert(TAG_MIN_SAMPLE_VALUE, "MinSampleValue");
    map.insert(TAG_X_RESOLUTION, "XResolution");
    map.insert(TAG_Y_RESOLUTION, "YResolution");
    map.insert(TAG_PLANAR_CONFIGURATION, "PlanarConfiguration");
    map.insert(TAG_RESOLUTION_UNIT, "ResolutionUnit");
    map.insert(TAG_TRANSFER_FUNCTION, "TransferFunction");
    map.insert(TAG_SOFTWARE, "Software");
    map.insert(TAG_DATE_TIME, "DateTime");
    map.insert(TAG_ARTIST, "Artist");
    map.insert(TAG_WHITE_POINT, "WhitePoint");
    map.insert(TAG_PRIMARY_CHROMATICITIES, "PrimaryChromaticities");
    map.insert(TAG_JPEG_INTERCHANGE_FORMAT, "JPEGInterchangeFormat");
    map.insert(
        TAG_JPEG_INTERCHANGE_FORMAT_LENGTH,
        "JPEGInterchangeFormatLength",
    );
    // Thumbnail IFD tags with "Thumbnail" prefix
    map.insert(TAG_THUMBNAIL_NEW_SUBFILE_TYPE, "ThumbnailNewSubfileType");
    map.insert(TAG_THUMBNAIL_IMAGE_DESCRIPTION, "ThumbnailImageDescription");
    map.insert(TAG_THUMBNAIL_MAKE, "ThumbnailMake");
    map.insert(TAG_THUMBNAIL_MODEL, "ThumbnailModel");
    map.insert(TAG_THUMBNAIL_SOFTWARE, "ThumbnailSoftware");
    map.insert(TAG_THUMBNAIL_DATE_TIME, "ThumbnailDateTime");
    map.insert(TAG_THUMBNAIL_COMPRESSION, "ThumbnailCompression");
    map.insert(TAG_THUMBNAIL_ORIENTATION, "ThumbnailOrientation");
    map.insert(TAG_THUMBNAIL_X_RESOLUTION, "ThumbnailXResolution");
    map.insert(TAG_THUMBNAIL_Y_RESOLUTION, "ThumbnailYResolution");
    map.insert(TAG_THUMBNAIL_RESOLUTION_UNIT, "ThumbnailResolutionUnit");
    map.insert(TAG_THUMBNAIL_YCBCR_POSITIONING, "ThumbnailYCbCrPositioning");
    map.insert(TAG_THUMBNAIL_PRINT_IM, "ThumbnailPrintIM");
    map.insert(TAG_YCBCR_COEFFICIENTS, "YCbCrCoefficients");
    map.insert(TAG_YCBCR_SUB_SAMPLING, "YCbCrSubSampling");
    map.insert(TAG_YCBCR_POSITIONING, "YCbCrPositioning");
    map.insert(TAG_REFERENCE_BLACK_WHITE, "ReferenceBlackWhite");
    map.insert(TAG_XMP_METADATA, "XMPMetadata");
    map.insert(TAG_SUB_IFDS, "SubIFDs");
    map.insert(TAG_IPTC_NAA, "IPTC/NAA");
    // EXIF tags in Main IFD (non-standard placement)
    map.insert(TAG_EXPOSURE_TIME_MAIN, "ExposureTime");
    map.insert(TAG_F_NUMBER_MAIN, "FNumber");
    map.insert(TAG_EXPOSURE_PROGRAM_MAIN, "ExposureProgram");
    map.insert(TAG_EXPOSURE_BIAS_VALUE_MAIN, "ExposureCompensation");
    map.insert(TAG_MAX_APERTURE_VALUE_MAIN, "MaxApertureValue");
    map.insert(TAG_METERING_MODE_MAIN, "MeteringMode");
    map.insert(TAG_LIGHT_SOURCE_MAIN, "LightSource");
    map.insert(TAG_FOCAL_LENGTH_MAIN, "FocalLength");
    map.insert(TAG_IMAGE_NUMBER_MAIN, "ImageNumber");
    map.insert(TAG_SENSING_METHOD_MAIN, "SensingMethod");
    map.insert(TAG_DATE_TIME_ORIGINAL_MAIN, "DateTimeOriginal");
    map.insert(TAG_TIFF_EP_STANDARD_ID_MAIN, "TIFF-EPStandardID");
    // Basic TIFF structure tags in EXIF SubIFD (non-standard placement)
    map.insert(TAG_IMAGE_WIDTH_EXIF, "ImageWidth");
    map.insert(TAG_IMAGE_LENGTH_EXIF, "ImageLength");
    map.insert(TAG_BITS_PER_SAMPLE_EXIF, "BitsPerSample");
    map.insert(
        TAG_PHOTOMETRIC_INTERPRETATION_EXIF,
        "PhotometricInterpretation",
    );
    map.insert(TAG_STRIP_OFFSETS_EXIF, "StripOffsets");
    map.insert(TAG_SAMPLES_PER_PIXEL_EXIF, "SamplesPerPixel");
    map.insert(TAG_ROWS_PER_STRIP_EXIF, "RowsPerStrip");
    map.insert(TAG_STRIP_BYTE_COUNTS_EXIF, "StripByteCounts");
    map.insert(TAG_PLANAR_CONFIGURATION_EXIF, "PlanarConfiguration");
    map.insert(TAG_COPYRIGHT, "Copyright");
    map.insert(TAG_ICC_PROFILE, "ICC_Profile");
    map.insert(TAG_EXIF_OFFSET, "ExifOffset");
    map.insert(TAG_GPS_INFO, "GPSInfo");
    // TIFF/EP CFA tags
    map.insert(TAG_CFA_REPEAT_PATTERN_DIM, "CFARepeatPatternDim");
    map.insert(TAG_CFA_PATTERN_TIFFEP, "CFAPattern2");
    // JPEG tags in Main IFD
    map.insert(TAG_JPEG_INTERCHANGE_FORMAT_MAIN, "JPEGInterchangeFormat");
    map.insert(
        TAG_JPEG_INTERCHANGE_FORMAT_LENGTH_MAIN,
        "JPEGInterchangeFormatLength",
    );

    // DNG tags
    map.insert(TAG_DNG_VERSION, "DNGVersion");
    map.insert(TAG_DNG_BACKWARD_VERSION, "DNGBackwardVersion");
    map.insert(TAG_UNIQUE_CAMERA_MODEL, "UniqueCameraModel");
    map.insert(TAG_LOCALIZED_CAMERA_MODEL, "LocalizedCameraModel");
    map.insert(TAG_CFA_PLANE_COLOR, "CFAPlaneColor");
    map.insert(TAG_CFA_LAYOUT, "CFALayout");
    map.insert(TAG_LINEARIZATION_TABLE, "LinearizationTable");
    map.insert(TAG_BLACK_LEVEL_REPEAT_DIM, "BlackLevelRepeatDim");
    map.insert(TAG_BLACK_LEVEL, "BlackLevel");
    map.insert(TAG_WHITE_LEVEL, "WhiteLevel");
    map.insert(TAG_ORIGINAL_RAW_FILE_NAME, "OriginalRawFileName");
    map.insert(TAG_ORIGINAL_RAW_FILE_DATA, "OriginalRawFileData");
    map.insert(TAG_DEFAULT_CROP_SIZE, "DefaultCropSize");
    map.insert(TAG_COLOR_MATRIX1, "ColorMatrix1");
    map.insert(TAG_COLOR_MATRIX2, "ColorMatrix2");
    map.insert(TAG_ANALOG_BALANCE, "AnalogBalance");
    map.insert(TAG_AS_SHOT_NEUTRAL, "AsShotNeutral");
    map.insert(TAG_BASELINE_EXPOSURE, "BaselineExposure");
    map.insert(TAG_BASELINE_NOISE, "BaselineNoise");
    map.insert(TAG_BASELINE_SHARPNESS, "BaselineSharpness");
    map.insert(TAG_PREVIEW_COLOR_SPACE, "PreviewColorSpace");
    map.insert(TAG_LINEAR_RESPONSE_LIMIT, "LinearResponseLimit");
    map.insert(TAG_CAMERA_SERIAL_NUMBER, "CameraSerialNumber");
    // ExifTool calls DNG tag 0xC630 "DNGLensInfo" to distinguish from EXIF LensInfo (0xA432)
    map.insert(TAG_LENS_INFO, "DNGLensInfo");
    map.insert(TAG_RAW_DATA_UNIQUE_ID, "RawDataUniqueID");
    map.insert(TAG_MASKED_AREAS, "MaskedAreas");
    map.insert(TAG_SHADOW_SCALE, "ShadowScale");
    map.insert(TAG_CALIBRATION_ILLUMINANT1, "CalibrationIlluminant1");
    map.insert(TAG_CALIBRATION_ILLUMINANT2, "CalibrationIlluminant2");
    map.insert(TAG_BEST_QUALITY_SCALE, "BestQualityScale");
    map.insert(TAG_ACTIVE_AREA, "ActiveArea");
    map.insert(TAG_RAW_IMAGE_DIGEST, "RawImageDigest");

    // CR2-specific tags
    map.insert(TAG_CR2_CFA_PATTERN, "CR2CFAPattern");
    map.insert(TAG_SRAW_TYPE, "SRawType");

    // Sony ARW SubIFD tags
    map.insert(TAG_SONY_VIGNETTING_CORRECTION, "VignettingCorrection");
    map.insert(
        TAG_SONY_CHROMATIC_ABERRATION_CORRECTION,
        "ChromaticAberrationCorrection",
    );
    map.insert(TAG_SONY_DISTORTION_CORRECTION, "DistortionCorrection");

    // EXIF SubIFD tags
    map.insert(TAG_EXPOSURE_TIME, "ExposureTime");
    map.insert(TAG_F_NUMBER, "FNumber");
    map.insert(TAG_EXPOSURE_PROGRAM, "ExposureProgram");
    map.insert(TAG_ISO_SPEED_RATINGS, "ISOSpeedRatings");
    map.insert(TAG_SENSITIVITY_TYPE, "SensitivityType");
    map.insert(TAG_SPECTRAL_SENSITIVITY, "SpectralSensitivity");
    map.insert(TAG_EXIF_VERSION, "ExifVersion");
    map.insert(TAG_DATE_TIME_ORIGINAL, "DateTimeOriginal");
    map.insert(TAG_DATE_TIME_DIGITIZED, "DateTimeDigitized");
    map.insert(TAG_COMPONENTS_CONFIGURATION, "ComponentsConfiguration");
    map.insert(TAG_COMPRESSED_BITS_PER_PIXEL, "CompressedBitsPerPixel");
    map.insert(TAG_SHUTTER_SPEED_VALUE, "ShutterSpeedValue");
    map.insert(TAG_APERTURE_VALUE, "ApertureValue");
    map.insert(TAG_BRIGHTNESS_VALUE, "BrightnessValue");
    map.insert(TAG_EXPOSURE_BIAS_VALUE, "ExposureCompensation");
    map.insert(TAG_MAX_APERTURE_VALUE, "MaxApertureValue");
    map.insert(TAG_SUBJECT_DISTANCE, "SubjectDistance");
    map.insert(TAG_METERING_MODE, "MeteringMode");
    map.insert(TAG_LIGHT_SOURCE, "LightSource");
    map.insert(TAG_FLASH, "Flash");
    map.insert(TAG_FOCAL_LENGTH, "FocalLength");
    map.insert(TAG_IMAGE_NUMBER, "ImageNumber");
    map.insert(TAG_SUBJECT_AREA, "SubjectArea");
    map.insert(TAG_EXPOSURE_INDEX, "ExposureIndex");
    map.insert(TAG_TIFF_EP_STANDARD_ID, "TIFF-EPStandardID");
    map.insert(TAG_MAKER_NOTE, "MakerNote");
    map.insert(TAG_USER_COMMENT, "UserComment");
    map.insert(TAG_SUB_SEC_TIME, "SubSecTime");
    map.insert(TAG_SUB_SEC_TIME_ORIGINAL, "SubSecTimeOriginal");
    map.insert(TAG_SUB_SEC_TIME_DIGITIZED, "SubSecTimeDigitized");
    map.insert(TAG_FLASHPIX_VERSION, "FlashpixVersion");
    map.insert(TAG_COLOR_SPACE, "ColorSpace");
    map.insert(TAG_PIXEL_X_DIMENSION, "PixelXDimension");
    map.insert(TAG_PIXEL_Y_DIMENSION, "PixelYDimension");
    map.insert(TAG_RELATED_SOUND_FILE, "RelatedSoundFile");
    map.insert(
        TAG_INTEROPERABILITY_IFD_POINTER,
        "InteroperabilityIFDPointer",
    );
    map.insert(TAG_FOCAL_PLANE_X_RESOLUTION, "FocalPlaneXResolution");
    map.insert(TAG_FOCAL_PLANE_Y_RESOLUTION, "FocalPlaneYResolution");
    map.insert(TAG_FOCAL_PLANE_RESOLUTION_UNIT, "FocalPlaneResolutionUnit");
    map.insert(TAG_FOCAL_PLANE_X_SIZE, "FocalPlaneXSize");
    map.insert(TAG_FOCAL_PLANE_Y_SIZE, "FocalPlaneYSize");
    map.insert(TAG_SENSING_METHOD, "SensingMethod");
    map.insert(TAG_FILE_SOURCE, "FileSource");
    map.insert(TAG_SCENE_TYPE, "SceneType");
    map.insert(TAG_CFA_PATTERN, "CFAPattern");
    map.insert(TAG_CUSTOM_RENDERED, "CustomRendered");
    map.insert(TAG_EXPOSURE_MODE, "ExposureMode");
    map.insert(TAG_WHITE_BALANCE, "WhiteBalance");
    map.insert(TAG_DIGITAL_ZOOM_RATIO, "DigitalZoomRatio");
    map.insert(TAG_FOCAL_LENGTH_IN_35MM_FILM, "FocalLengthIn35mmFormat");
    map.insert(TAG_SCENE_CAPTURE_TYPE, "SceneCaptureType");
    map.insert(TAG_GAIN_CONTROL, "GainControl");
    map.insert(TAG_CONTRAST, "Contrast");
    map.insert(TAG_SATURATION, "Saturation");
    map.insert(TAG_SHARPNESS, "Sharpness");
    map.insert(TAG_DEVICE_SETTING_DESCRIPTION, "DeviceSettingDescription");
    map.insert(TAG_SUBJECT_DISTANCE_RANGE, "SubjectDistanceRange");
    map.insert(TAG_IMAGE_UNIQUE_ID, "ImageUniqueID");
    map.insert(TAG_CAMERA_OWNER_NAME, "CameraOwnerName");
    map.insert(TAG_BODY_SERIAL_NUMBER, "BodySerialNumber");
    // ExifTool calls this "LensInfo" (official EXIF name is LensSpecification)
    map.insert(TAG_LENS_SPECIFICATION, "LensInfo");
    map.insert(TAG_LENS_MAKE, "LensMake");
    map.insert(TAG_LENS_MODEL, "LensModel");
    map.insert(TAG_LENS_SERIAL_NUMBER, "LensSerialNumber");

    // GPS tags
    map.insert(TAG_GPS_VERSION_ID, "GPSVersionID");
    map.insert(TAG_GPS_LATITUDE_REF, "GPSLatitudeRef");
    map.insert(TAG_GPS_LATITUDE, "GPSLatitude");
    map.insert(TAG_GPS_LONGITUDE_REF, "GPSLongitudeRef");
    map.insert(TAG_GPS_LONGITUDE, "GPSLongitude");
    map.insert(TAG_GPS_ALTITUDE_REF, "GPSAltitudeRef");
    map.insert(TAG_GPS_ALTITUDE, "GPSAltitude");
    map.insert(TAG_GPS_TIMESTAMP, "GPSTimeStamp");
    map.insert(TAG_GPS_SATELLITES, "GPSSatellites");
    map.insert(TAG_GPS_STATUS, "GPSStatus");
    map.insert(TAG_GPS_MEASURE_MODE, "GPSMeasureMode");
    map.insert(TAG_GPS_DOP, "GPSDOP");
    map.insert(TAG_GPS_SPEED_REF, "GPSSpeedRef");
    map.insert(TAG_GPS_SPEED, "GPSSpeed");

    // Interoperability tags
    map.insert(TAG_INTEROP_INDEX, "InteropIndex");
    map.insert(TAG_INTEROP_VERSION, "InteropVersion");

    map
}

/// Get the name of a tag from its ID
pub fn get_tag_name(tag_id: ExifTagId) -> Option<&'static str> {
    let tag_names = TAG_NAMES.get_or_init(init_tag_names);

    // Try exact match first
    if let Some(name) = tag_names.get(&tag_id) {
        return Some(*name);
    }

    // Fall back to checking other tag groups with the same ID
    // This handles cases where synthetic EXIF (e.g., from CRW) places tags in IFD0
    // that are normally defined in EXIF SubIFD
    const GROUPS: [TagGroup; 5] = [
        TagGroup::Main,
        TagGroup::Exif,
        TagGroup::Gps,
        TagGroup::Interop,
        TagGroup::Thumbnail,
    ];

    for group in GROUPS {
        if group != tag_id.ifd {
            let alt_tag = ExifTagId::new(tag_id.id, group);
            if let Some(name) = tag_names.get(&alt_tag) {
                return Some(*name);
            }
        }
    }

    None
}

// Reverse mapping of tag names to IDs
static TAG_IDS_BY_NAME: OnceLock<HashMap<&'static str, ExifTagId>> = OnceLock::new();

// Initialize the reverse mapping
fn init_tag_ids_by_name() -> HashMap<&'static str, ExifTagId> {
    let tag_names = TAG_NAMES.get_or_init(init_tag_names);
    let mut map = HashMap::new();

    for (&tag_id, &name) in tag_names {
        map.insert(name, tag_id);
    }

    map
}

/// Get the tag ID from its name
pub fn get_tag_id_by_name(name: &str) -> Option<ExifTagId> {
    TAG_IDS_BY_NAME
        .get_or_init(init_tag_ids_by_name)
        .get(name)
        .copied()
}

/// Human-readable descriptions for various EXIF orientation values (exiftool-compatible)
pub fn get_orientation_description(value: u16) -> &'static str {
    match value {
        1 => "Horizontal (normal)",
        2 => "Mirror horizontal",
        3 => "Rotate 180",
        4 => "Mirror vertical",
        5 => "Mirror horizontal and rotate 270 CW",
        6 => "Rotate 90 CW",
        7 => "Mirror horizontal and rotate 90 CW",
        8 => "Rotate 270 CW",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for various EXIF exposure program values
pub fn get_exposure_program_description(value: u16) -> &'static str {
    match value {
        0 => "Not Defined",
        1 => "Manual",
        2 => "Program AE",
        3 => "Aperture-priority AE",
        4 => "Shutter speed priority AE",
        5 => "Creative (Slow speed)",
        6 => "Action (High speed)",
        7 => "Portrait",
        8 => "Landscape",
        9 => "Bulb",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for various EXIF metering mode values
pub fn get_metering_mode_description(value: u16) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "Average",
        2 => "Center-weighted average",
        3 => "Spot",
        4 => "Multi-spot",
        5 => "Multi-segment",
        6 => "Partial",
        255 => "Other",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for various EXIF light source values (exiftool-compatible)
pub fn get_light_source_description(value: u16) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "Daylight",
        2 => "Fluorescent",
        3 => "Tungsten (Incandescent)",
        4 => "Flash",
        9 => "Fine Weather",
        10 => "Cloudy",
        11 => "Shade",
        12 => "Daylight Fluorescent",
        13 => "Day White Fluorescent",
        14 => "Cool White Fluorescent",
        15 => "White Fluorescent",
        16 => "Warm White Fluorescent",
        17 => "Standard Light A",
        18 => "Standard Light B",
        19 => "Standard Light C",
        20 => "D55",
        21 => "D65",
        22 => "D75",
        23 => "D50",
        24 => "ISO Studio Tungsten",
        255 => "Other",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for various EXIF flash values (exiftool-compatible)
pub fn get_flash_description(value: u16) -> &'static str {
    match value {
        0x00 => "No Flash",
        0x01 => "Fired",
        0x05 => "Fired, Return not detected",
        0x07 => "Fired, Return detected",
        0x08 => "On, Did not fire",
        0x09 => "On, Fired",
        0x0D => "On, Return not detected",
        0x0F => "On, Return detected",
        0x10 => "Off, Did not fire",
        0x14 => "Off, Did not fire, Return not detected",
        0x18 => "Auto, Did not fire",
        0x19 => "Auto, Fired",
        0x1D => "Auto, Fired, Return not detected",
        0x1F => "Auto, Fired, Return detected",
        0x20 => "No flash function",
        0x30 => "Off, No flash function",
        0x41 => "Fired, Red-eye reduction",
        0x45 => "Fired, Red-eye reduction, Return not detected",
        0x47 => "Fired, Red-eye reduction, Return detected",
        0x49 => "On, Red-eye reduction",
        0x4D => "On, Red-eye reduction, Return not detected",
        0x4F => "On, Red-eye reduction, Return detected",
        0x50 => "Off, Red-eye reduction",
        0x58 => "Auto, Did not fire, Red-eye reduction",
        0x59 => "Auto, Fired, Red-eye reduction",
        0x5D => "Auto, Fired, Red-eye reduction, Return not detected",
        0x5F => "Auto, Fired, Red-eye reduction, Return detected",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for ColorSpace values
pub fn get_color_space_description(value: u16) -> &'static str {
    match value {
        1 => "sRGB",
        2 => "Adobe RGB",
        0xFFFF => "Uncalibrated",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for WhiteBalance values
pub fn get_white_balance_description(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for ExposureMode values
pub fn get_exposure_mode_description(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        2 => "Auto bracket",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for SceneCaptureType values
pub fn get_scene_capture_type_description(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        1 => "Landscape",
        2 => "Portrait",
        3 => "Night scene",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for Contrast values
pub fn get_contrast_description(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for Saturation values
pub fn get_saturation_description(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for Sharpness values
pub fn get_sharpness_description(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Soft",
        2 => "Hard",
        _ => "Unknown",
    }
}

/// Sony ARW SubIFD VignettingCorrection (tag 0x7030) values
/// From ExifTool Exif.pm
pub fn get_sony_vignetting_correction_description(value: u16) -> &'static str {
    match value {
        0 => "Off",
        2 => "Auto",
        0xff => "No correction params available",
        _ => "Unknown",
    }
}

/// Sony ARW SubIFD ChromaticAberrationCorrection (tag 0x7034) values
/// From ExifTool Exif.pm
pub fn get_sony_chromatic_aberration_correction_description(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Auto",
        0xff => "No correction params available",
        _ => "Unknown",
    }
}

/// Sony ARW SubIFD DistortionCorrection (tag 0x7036) values
/// From ExifTool Exif.pm
pub fn get_sony_distortion_correction_description(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Auto",
        17 => "Auto fixed by lens",
        0xff => "No correction params available",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for ResolutionUnit values
pub fn get_resolution_unit_description(value: u16) -> &'static str {
    match value {
        1 => "None",
        2 => "inches",
        3 => "cm",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for YCbCrPositioning values
pub fn get_ycbcr_positioning_description(value: u16) -> &'static str {
    match value {
        1 => "Centered",
        2 => "Co-sited",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for Compression values
pub fn get_compression_description(value: u16) -> &'static str {
    match value {
        1 => "Uncompressed",
        2 => "CCITT 1D",
        3 => "T4/Group 3 Fax",
        4 => "T6/Group 4 Fax",
        5 => "LZW",
        6 => "JPEG (old-style)",
        7 => "JPEG",
        8 => "Adobe Deflate",
        9 => "JBIG B&W",
        10 => "JBIG Color",
        99 => "JPEG",
        262 => "Kodak 262",
        32766 => "Next",
        32767 => "Sony ARW Compressed",
        32769 => "Packed RAW",
        32770 => "Samsung SRW Compressed",
        32771 => "CCIRLEW",
        32773 => "PackBits",
        32809 => "Thunderscan",
        32867 => "Kodak KDC Compressed",
        32895 => "IT8CTPAD",
        32896 => "IT8LW",
        32897 => "IT8MP",
        32898 => "IT8BL",
        32908 => "PixarFilm",
        32909 => "PixarLog",
        32946 => "Deflate",
        32947 => "DCS",
        34661 => "JBIG",
        34676 => "SGILog",
        34677 => "SGILog24",
        34316 => "Panasonic RAW 1",
        34712 => "JPEG 2000",
        34713 => "Nikon NEF Compressed",
        34826 => "Panasonic RAW 2",
        34828 => "Panasonic RAW 3",
        34830 => "Panasonic RAW 4",
        34892 => "JPEG XR",
        65000 => "Kodak DCR Compressed",
        65535 => "Pentax PEF Compressed",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for SensingMethod values
pub fn get_sensing_method_description(value: u16) -> &'static str {
    match value {
        1 => "Not defined",
        2 => "One-chip color area",
        3 => "Two-chip color area",
        4 => "Three-chip color area",
        5 => "Color sequential area",
        7 => "Trilinear",
        8 => "Color sequential linear",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for SensitivityType values
pub fn get_sensitivity_type_description(value: u16) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "Standard Output Sensitivity",
        2 => "Recommended Exposure Index",
        3 => "ISO Speed",
        4 => "Standard Output Sensitivity and Recommended Exposure Index",
        5 => "Standard Output Sensitivity and ISO Speed",
        6 => "Recommended Exposure Index and ISO Speed",
        7 => "Standard Output Sensitivity, Recommended Exposure Index and ISO Speed",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for FileSource values
/// Returns "Unknown (N)" for unknown values to match ExifTool format
pub fn get_file_source_description(value: u8) -> String {
    match value {
        1 => "Film Scanner".to_string(),
        2 => "Reflection Print Scanner".to_string(),
        3 => "Digital Camera".to_string(),
        _ => format!("Unknown ({})", value),
    }
}

/// Human-readable descriptions for SceneType values
/// Returns "Unknown (N)" for unknown values to match ExifTool format
pub fn get_scene_type_description(value: u8) -> String {
    match value {
        1 => "Directly photographed".to_string(),
        _ => format!("Unknown ({})", value),
    }
}

/// Human-readable descriptions for CustomRendered values
pub fn get_custom_rendered_description(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Custom",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for GainControl values
pub fn get_gain_control_description(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Low gain up",
        2 => "High gain up",
        3 => "Low gain down",
        4 => "High gain down",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for SubjectDistanceRange values
pub fn get_subject_distance_range_description(value: u16) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "Macro",
        2 => "Close",
        3 => "Distant",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for PhotometricInterpretation values
pub fn get_photometric_interpretation_description(value: u16) -> &'static str {
    match value {
        0 => "WhiteIsZero",
        1 => "BlackIsZero",
        2 => "RGB",
        3 => "RGB Palette",
        4 => "Transparency Mask",
        5 => "CMYK",
        6 => "YCbCr",
        8 => "CIELab",
        9 => "ICCLab",
        10 => "ITULab",
        32803 => "Color Filter Array",
        34892 => "Linear Raw",
        _ => "Unknown",
    }
}

/// Human-readable descriptions for PlanarConfiguration values
pub fn get_planar_configuration_description(value: u16) -> &'static str {
    match value {
        1 => "Chunky",
        2 => "Planar",
        _ => "Unknown",
    }
}

/// Human-readable description for GPSLatitudeRef values
pub fn get_gps_latitude_ref_description(value: &str) -> String {
    // Trim whitespace and null bytes
    let trimmed = value.trim().trim_matches('\0');
    match trimmed {
        "N" => "North".to_string(),
        "S" => "South".to_string(),
        "" => "Unknown ()".to_string(),
        v => format!("Unknown ({})", v),
    }
}

/// Human-readable description for GPSLongitudeRef values
pub fn get_gps_longitude_ref_description(value: &str) -> String {
    // Trim whitespace and null bytes
    let trimmed = value.trim().trim_matches('\0');
    match trimmed {
        "E" => "East".to_string(),
        "W" => "West".to_string(),
        "" => "Unknown ()".to_string(),
        v => format!("Unknown ({})", v),
    }
}

/// Human-readable description for GPSAltitudeRef values
pub fn get_gps_altitude_ref_description(value: u8) -> &'static str {
    match value {
        0 => "Above Sea Level",
        1 => "Below Sea Level",
        _ => "Unknown",
    }
}

/// Human-readable description for CFALayout values (DNG tag 0xC617)
pub fn get_cfa_layout_description(value: u16) -> &'static str {
    match value {
        1 => "Rectangular",
        2 => "Even columns offset down 1/2 row",
        3 => "Even columns offset up 1/2 row",
        4 => "Even rows offset right 1/2 column",
        5 => "Even rows offset left 1/2 column",
        6 => "Even rows offset up by 1/2 row, even columns offset left by 1/2 column",
        7 => "Even rows offset up by 1/2 row, even columns offset right by 1/2 column",
        8 => "Even rows offset down by 1/2 row, even columns offset left by 1/2 column",
        9 => "Even rows offset down by 1/2 row, even columns offset right by 1/2 column",
        _ => "Unknown",
    }
}

/// Human-readable description for CFAPlaneColor values (DNG tag 0xC616)
/// Converts array of indices to color names: 0=Red, 1=Green, 2=Blue, etc.
pub fn get_cfa_plane_color_description(values: &[u8]) -> String {
    let colors = ["Red", "Green", "Blue", "Cyan", "Magenta", "Yellow", "White"];
    let names: Vec<&str> = values
        .iter()
        .map(|&v| {
            if (v as usize) < colors.len() {
                colors[v as usize]
            } else {
                "Unknown"
            }
        })
        .collect();
    names.join(",")
}
