// Comprehensive FileSize test across ALL manufacturers and formats

use fpexif::ExifParser;
#[cfg(feature = "serde")]
use fpexif::output::{get_tag_value, to_exiftool_json};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const RAW_EXTENSIONS: &[&str] = &[
    "RAF", "CR2", "CR3", "CRW", "NEF", "ARW", "DNG", "ORF", "PEF", "RW2", "MRW", "X3F", "SRW",
    "KDC", "NRW", "3FR", "SR2", "ERF", "DCR", "MOS", "SRF", "MEF", "MDC", "RAW", "IIQ", "RWL",
];

fn find_all_raw_files(dir: &Path) -> Vec<PathBuf> {
    fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_upper = ext.to_string_lossy().to_uppercase();
                        if RAW_EXTENSIONS.contains(&ext_upper.as_str()) {
                            files.push(path);
                        }
                    }
                } else if path.is_dir() {
                    scan_dir(&path, files);
                }
            }
        }
    }

    let mut files = Vec::new();
    if dir.exists() {
        scan_dir(dir, &mut files);
        files.sort();
    }
    files
}

#[derive(Default)]
struct TestStats {
    total: usize,
    passed: usize,
    failed: usize,
    parse_errors: usize,
    read_errors: usize,
}

impl TestStats {
    fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(feature = "serde")]
fn test_file_filesize(file_path: &Path, parser: &ExifParser) -> Result<(), String> {
    let file_data = fs::read(file_path).map_err(|e| format!("Read error: {}", e))?;

    let file_size = file_data.len() as u64;

    let exif_data = parser
        .parse_bytes(&file_data)
        .map_err(|e| format!("Parse error: {}", e))?;

    let json = to_exiftool_json(&exif_data, None, Some(file_size));

    // Check FileSize presence
    let filesize_value =
        get_tag_value(&json, "FileSize").ok_or_else(|| "FileSize tag missing".to_string())?;

    // Check FileSizeBytes presence
    let filesize_bytes_value = get_tag_value(&json, "FileSizeBytes")
        .ok_or_else(|| "FileSizeBytes tag missing".to_string())?;

    // Verify FileSizeBytes is correct
    if let Value::Number(bytes) = filesize_bytes_value {
        let actual_bytes = bytes
            .as_u64()
            .ok_or_else(|| "FileSizeBytes not a valid u64".to_string())?;

        if actual_bytes != file_size {
            return Err(format!(
                "FileSizeBytes mismatch: expected {}, got {}",
                file_size, actual_bytes
            ));
        }
    } else {
        return Err("FileSizeBytes is not a number".to_string());
    }

    // Verify FileSize is a string with units
    if let Value::String(size_str) = filesize_value {
        if !size_str.contains("bytes")
            && !size_str.contains("kB")
            && !size_str.contains("MB")
            && !size_str.contains("GB")
        {
            return Err(format!("FileSize missing units: '{}'", size_str));
        }
    } else {
        return Err("FileSize is not a string".to_string());
    }

    Ok(())
}

#[cfg(feature = "serde")]
#[test]
fn test_all_files_in_raws() {
    let raws_dir = Path::new("/fpexif/raws");

    if !raws_dir.exists() {
        println!("Skipping test: /fpexif/raws directory not found");
        return;
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  COMPREHENSIVE FILESIZE TEST: /fpexif/raws                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let files = find_all_raw_files(raws_dir);
    println!("Found {} RAW files to test\n", files.len());

    let parser = ExifParser::new();
    let mut stats_by_ext: HashMap<String, TestStats> = HashMap::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    for (i, file) in files.iter().enumerate() {
        let filename = file.file_name().unwrap().to_string_lossy().to_string();
        let ext = file
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_uppercase()
            .to_string();

        let stats = stats_by_ext.entry(ext.clone()).or_default();
        stats.total += 1;

        match test_file_filesize(file, &parser) {
            Ok(()) => {
                stats.passed += 1;
                if (i + 1) % 50 == 0 {
                    println!("  Processed {} files...", i + 1);
                }
            }
            Err(e) => {
                if e.contains("Read error") {
                    stats.read_errors += 1;
                } else if e.contains("Parse error") {
                    stats.parse_errors += 1;
                } else {
                    stats.failed += 1;
                }
                failures.push((filename, e));
            }
        }
    }

    // Print summary by extension
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  RESULTS BY FORMAT                                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut extensions: Vec<_> = stats_by_ext.keys().collect();
    extensions.sort();

    let mut grand_total = 0;
    let mut grand_passed = 0;

    for ext in extensions {
        let stats = &stats_by_ext[ext];
        grand_total += stats.total;
        grand_passed += stats.passed;

        let status = if stats.passed == stats.total {
            "✓"
        } else {
            "✗"
        };

        println!(
            "  {} {:<6} - {:4}/{:4} files ({:5.1}%) - {} parse errors, {} read errors",
            status,
            ext,
            stats.passed,
            stats.total,
            stats.success_rate(),
            stats.parse_errors,
            stats.read_errors
        );
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  OVERALL SUMMARY                                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("  Total files tested:  {}", grand_total);
    println!(
        "  Passed:              {} ({:.1}%)",
        grand_passed,
        (grand_passed as f64 / grand_total as f64) * 100.0
    );
    println!("  Failed:              {}", grand_total - grand_passed);

    if !failures.is_empty() {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  FAILURES                                                    ║");
        println!("╚══════════════════════════════════════════════════════════════╝\n");
        for (filename, error) in &failures {
            println!("  ✗ {} - {}", filename, error);
        }
    }

    // Calculate actual FileSize failures (not parse errors)
    let mut actual_filesize_failures = 0;
    for stats in stats_by_ext.values() {
        actual_filesize_failures += stats.failed;
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  CRITICAL ASSERTION                                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!(
        "  Files that parsed successfully: {}",
        grand_total
            - stats_by_ext
                .values()
                .map(|s| s.parse_errors + s.read_errors)
                .sum::<usize>()
    );
    println!("  Files with FileSize tag:        {}", grand_passed);
    println!(
        "  FileSize failures (actual):     {}",
        actual_filesize_failures
    );
    println!();

    // CRITICAL: All SUCCESSFULLY PARSED files must have FileSize tag
    // Parse errors are expected for unsupported formats (CHDK CRW, etc.)
    assert_eq!(
        actual_filesize_failures,
        0,
        "ALL successfully parsed files must have FileSize tag! {} FileSize failures found (not counting {} parse/read errors)",
        actual_filesize_failures,
        grand_total - grand_passed - actual_filesize_failures
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_all_files_in_data_lfs() {
    let lfs_dir = Path::new("/fpexif/data.lfs");

    if !lfs_dir.exists() {
        println!("Skipping test: /fpexif/data.lfs directory not found");
        return;
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  COMPREHENSIVE FILESIZE TEST: /fpexif/data.lfs              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let files = find_all_raw_files(lfs_dir);
    println!("Found {} RAW files to test\n", files.len());

    let parser = ExifParser::new();
    let mut stats_by_ext: HashMap<String, TestStats> = HashMap::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    for (i, file) in files.iter().enumerate() {
        let filename = file.file_name().unwrap().to_string_lossy().to_string();
        let ext = file
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_uppercase()
            .to_string();

        let stats = stats_by_ext.entry(ext.clone()).or_default();
        stats.total += 1;

        match test_file_filesize(file, &parser) {
            Ok(()) => {
                stats.passed += 1;
                if (i + 1) % 100 == 0 {
                    println!("  Processed {} files...", i + 1);
                }
            }
            Err(e) => {
                if e.contains("Read error") {
                    stats.read_errors += 1;
                } else if e.contains("Parse error") {
                    stats.parse_errors += 1;
                } else {
                    stats.failed += 1;
                }
                failures.push((filename, e));
            }
        }
    }

    // Print summary by extension
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  RESULTS BY FORMAT                                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut extensions: Vec<_> = stats_by_ext.keys().collect();
    extensions.sort();

    let mut grand_total = 0;
    let mut grand_passed = 0;

    for ext in extensions {
        let stats = &stats_by_ext[ext];
        grand_total += stats.total;
        grand_passed += stats.passed;

        let status = if stats.passed == stats.total {
            "✓"
        } else {
            "✗"
        };

        println!(
            "  {} {:<6} - {:4}/{:4} files ({:5.1}%) - {} parse errors, {} read errors",
            status,
            ext,
            stats.passed,
            stats.total,
            stats.success_rate(),
            stats.parse_errors,
            stats.read_errors
        );
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  OVERALL SUMMARY                                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!("  Total files tested:  {}", grand_total);
    println!(
        "  Passed:              {} ({:.1}%)",
        grand_passed,
        (grand_passed as f64 / grand_total as f64) * 100.0
    );
    println!("  Failed:              {}", grand_total - grand_passed);

    if !failures.is_empty() {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  FAILURES                                                    ║");
        println!("╚══════════════════════════════════════════════════════════════╝\n");
        for (filename, error) in &failures {
            println!("  ✗ {} - {}", filename, error);
        }
    }

    // Calculate actual FileSize failures (not parse errors)
    let mut actual_filesize_failures = 0;
    for stats in stats_by_ext.values() {
        actual_filesize_failures += stats.failed;
    }

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  CRITICAL ASSERTION                                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    println!(
        "  Files that parsed successfully: {}",
        grand_total
            - stats_by_ext
                .values()
                .map(|s| s.parse_errors + s.read_errors)
                .sum::<usize>()
    );
    println!("  Files with FileSize tag:        {}", grand_passed);
    println!(
        "  FileSize failures (actual):     {}",
        actual_filesize_failures
    );
    println!();

    // CRITICAL: All SUCCESSFULLY PARSED files must have FileSize tag
    // Parse errors are expected for unsupported formats (CHDK CRW, etc.)
    assert_eq!(
        actual_filesize_failures,
        0,
        "ALL successfully parsed files must have FileSize tag! {} FileSize failures found (not counting {} parse/read errors)",
        actual_filesize_failures,
        grand_total - grand_passed - actual_filesize_failures
    );
}
