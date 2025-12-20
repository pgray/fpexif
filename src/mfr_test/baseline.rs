//! Baseline save/load functionality

use super::ManufacturerTestResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const BASELINES_DIR: &str = ".mfr-baselines";

/// Metadata about a saved baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub manufacturer: String,
    pub created_at: String,
    pub git_commit: String,
    pub git_branch: String,
    pub description: Option<String>,
}

/// A complete baseline with metadata and results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub metadata: BaselineMetadata,
    pub result: ManufacturerTestResult,
}

/// Get the baseline directory path for a manufacturer
fn get_baseline_dir(manufacturer: &str) -> PathBuf {
    PathBuf::from(BASELINES_DIR).join(manufacturer.to_lowercase())
}

/// Get the baseline file path for a manufacturer
fn get_baseline_path(manufacturer: &str) -> PathBuf {
    get_baseline_dir(manufacturer).join("baseline.json")
}

/// Get current git commit hash (short)
pub fn get_git_commit() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get current git branch name
pub fn get_git_branch() -> String {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get current timestamp in ISO format
fn get_timestamp() -> String {
    // Simple timestamp without chrono dependency
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Save a baseline for a manufacturer
pub fn save_baseline(
    manufacturer: &str,
    result: ManufacturerTestResult,
    description: Option<&str>,
) -> Result<PathBuf, String> {
    let baseline_dir = get_baseline_dir(manufacturer);
    fs::create_dir_all(&baseline_dir)
        .map_err(|e| format!("Failed to create baseline directory: {}", e))?;

    let metadata = BaselineMetadata {
        manufacturer: manufacturer.to_string(),
        created_at: get_timestamp(),
        git_commit: get_git_commit(),
        git_branch: get_git_branch(),
        description: description.map(String::from),
    };

    let baseline = Baseline { metadata, result };

    let baseline_path = get_baseline_path(manufacturer);
    let json = serde_json::to_string_pretty(&baseline)
        .map_err(|e| format!("Failed to serialize baseline: {}", e))?;

    fs::write(&baseline_path, json).map_err(|e| format!("Failed to write baseline file: {}", e))?;

    Ok(baseline_path)
}

/// Load a baseline for a manufacturer
pub fn load_baseline(manufacturer: &str) -> Result<Baseline, String> {
    let baseline_path = get_baseline_path(manufacturer);

    if !baseline_path.exists() {
        return Err(format!(
            "No baseline found for '{}'. Run with --save-baseline first.",
            manufacturer
        ));
    }

    let json = fs::read_to_string(&baseline_path)
        .map_err(|e| format!("Failed to read baseline: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse baseline: {}", e))
}

/// Check if a baseline exists for a manufacturer
pub fn baseline_exists(manufacturer: &str) -> bool {
    get_baseline_path(manufacturer).exists()
}

/// List all manufacturers with saved baselines
pub fn list_baselines() -> Vec<(String, BaselineMetadata)> {
    let baselines_dir = PathBuf::from(BASELINES_DIR);
    if !baselines_dir.exists() {
        return Vec::new();
    }

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(baselines_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(mfr) = entry.file_name().to_str() {
                    if let Ok(baseline) = load_baseline(mfr) {
                        results.push((mfr.to_string(), baseline.metadata));
                    }
                }
            }
        }
    }

    results
}
