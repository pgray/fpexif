// Test essential tags across multiple file formats and manufacturers

use fpexif::ExifParser;
#[cfg(feature = "serde")]
use fpexif::output::{get_tag_value, to_exiftool_json};
use std::fs;
use std::path::{Path, PathBuf};

const ESSENTIAL_TAGS: &[&str] = &[
    "Make",
    "Model",
    "ISO",
    "Aperture",
    "ShutterSpeed",
    "FocalLength",
    "CreateDate",
    "FileSize",
];

const RAW_EXTENSIONS: &[&str] = &[
    "RAF", "CR2", "CR3", "NEF", "ARW", "DNG", "ORF", "PEF", "RW2", "MRW", "X3F", "SRW", "KDC",
    "NRW", "3FR", "SR2", "ERF", "DCR", "MOS", "SRF", "MEF",
];

fn find_test_files(dir: &Path, max_files: usize) -> Vec<PathBuf> {
    fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>, max_files: usize) {
        if files.len() >= max_files {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_upper = ext.to_string_lossy().to_uppercase();
                        if RAW_EXTENSIONS.contains(&ext_upper.as_str()) {
                            files.push(path);
                            if files.len() >= max_files {
                                return;
                            }
                        }
                    }
                } else if path.is_dir() {
                    scan_dir(&path, files, max_files);
                }
            }
        }
    }

    let mut files = Vec::new();

    if !dir.exists() {
        return files;
    }

    scan_dir(dir, &mut files, max_files);
    files.sort();
    files
}

#[cfg(feature = "serde")]
fn test_file_essential_tags(file_path: &Path) -> (usize, usize, bool) {
    let parser = ExifParser::new();

    let file_data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("  ✗ Failed to read file: {}", e);
            return (0, 0, false);
        }
    };

    let file_size = file_data.len() as u64;

    match parser.parse_bytes(&file_data) {
        Ok(exif_data) => {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            let mut found = 0;
            let mut missing = 0;
            let mut has_filesize = false;

            for &tag in ESSENTIAL_TAGS {
                if get_tag_value(&json, tag).is_some() {
                    found += 1;
                    if tag == "FileSize" {
                        has_filesize = true;
                    }
                } else {
                    missing += 1;
                }
            }

            (found, missing, has_filesize)
        }
        Err(e) => {
            println!("  ✗ Parse error: {}", e);
            (0, ESSENTIAL_TAGS.len(), false)
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_essential_tags_raws_directory() {
    let raws_dir = Path::new("/fpexif/raws");

    if !raws_dir.exists() {
        println!("Skipping test: /fpexif/raws directory not found");
        return;
    }

    println!("\n=== Testing Essential Tags in /fpexif/raws ===");

    let files = find_test_files(raws_dir, 20); // Test first 20 files

    if files.is_empty() {
        println!("No RAW files found in /fpexif/raws");
        return;
    }

    println!("Testing {} files:", files.len());

    let mut total_found = 0;
    let mut total_possible = 0;
    let mut files_with_filesize = 0;

    for file in &files {
        let filename = file.file_name().unwrap().to_string_lossy();
        let (found, _missing, has_filesize) = test_file_essential_tags(file);

        total_found += found;
        total_possible += ESSENTIAL_TAGS.len();

        if has_filesize {
            files_with_filesize += 1;
        }

        let status = if has_filesize { "✓" } else { "✗" };
        println!(
            "  {} {} - {}/{} tags (FileSize: {})",
            status,
            filename,
            found,
            ESSENTIAL_TAGS.len(),
            if has_filesize { "✓" } else { "✗" }
        );

        // FileSize should ALWAYS be present when we provide it
        assert!(
            has_filesize,
            "FileSize should always be present for {}",
            filename
        );
    }

    println!("\nSummary:");
    println!(
        "  Total tags found: {}/{} ({:.1}%)",
        total_found,
        total_possible,
        (total_found as f64 / total_possible as f64) * 100.0
    );
    println!(
        "  Files with FileSize: {}/{} (should be 100%)",
        files_with_filesize,
        files.len()
    );

    // All files should have FileSize
    assert_eq!(
        files_with_filesize,
        files.len(),
        "All files should have FileSize tag"
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_essential_tags_data_lfs_directory() {
    let lfs_dir = Path::new("/fpexif/data.lfs");

    if !lfs_dir.exists() {
        println!("Skipping test: /fpexif/data.lfs directory not found");
        return;
    }

    println!("\n=== Testing Essential Tags in /fpexif/data.lfs ===");

    let files = find_test_files(lfs_dir, 20); // Test first 20 files

    if files.is_empty() {
        println!("No RAW files found in /fpexif/data.lfs");
        return;
    }

    println!("Testing {} files:", files.len());

    let mut total_found = 0;
    let mut total_possible = 0;
    let mut files_with_filesize = 0;

    for file in &files {
        let filename = file.file_name().unwrap().to_string_lossy();
        let (found, _missing, has_filesize) = test_file_essential_tags(file);

        total_found += found;
        total_possible += ESSENTIAL_TAGS.len();

        if has_filesize {
            files_with_filesize += 1;
        }

        let status = if has_filesize { "✓" } else { "✗" };
        println!(
            "  {} {} - {}/{} tags (FileSize: {})",
            status,
            filename,
            found,
            ESSENTIAL_TAGS.len(),
            if has_filesize { "✓" } else { "✗" }
        );

        // FileSize should ALWAYS be present when we provide it
        assert!(
            has_filesize,
            "FileSize should always be present for {}",
            filename
        );
    }

    println!("\nSummary:");
    println!(
        "  Total tags found: {}/{} ({:.1}%)",
        total_found,
        total_possible,
        (total_found as f64 / total_possible as f64) * 100.0
    );
    println!(
        "  Files with FileSize: {}/{} (should be 100%)",
        files_with_filesize,
        files.len()
    );

    // All files should have FileSize
    assert_eq!(
        files_with_filesize,
        files.len(),
        "All files should have FileSize tag"
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_case_insensitive_across_formats() {
    // Test case-insensitive tag access works across different manufacturers
    let test_files = vec![
        "/fpexif/raws/RAW_CANON_1000D.CR2",
        "/fpexif/raws/RAW_NIKON_D3.NEF",
        "/fpexif/raws/RAW_SONY_A100.ARW",
    ];

    let parser = ExifParser::new();

    for test_file in test_files {
        let path = Path::new(test_file);
        if !path.exists() {
            continue;
        }

        let file_data = fs::read(path).expect("Failed to read file");
        let file_size = file_data.len() as u64;

        if let Ok(exif_data) = parser.parse_bytes(&file_data) {
            let json = to_exiftool_json(&exif_data, None, Some(file_size));

            let filename = path.file_name().unwrap().to_string_lossy();
            println!("\nTesting case-insensitive access: {}", filename);

            // Test different casings for Make
            let variants = vec![
                ("Make", "make", "MAKE"),
                ("Model", "model", "MODEL"),
                ("FileSize", "file_size", "FILE_SIZE"),
            ];

            for (pascal, lower, upper) in variants {
                let v1 = get_tag_value(&json, pascal);
                let v2 = get_tag_value(&json, lower);
                let v3 = get_tag_value(&json, upper);

                if v1.is_some() {
                    assert_eq!(v1, v2, "  {} and {} should match", pascal, lower);
                    assert_eq!(v1, v3, "  {} and {} should match", pascal, upper);
                    println!("  ✓ {} case-insensitive access works", pascal);
                }
            }
        }
    }
}
