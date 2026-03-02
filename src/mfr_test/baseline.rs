//! Baseline save/load functionality

use super::ManufacturerTestResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const BASELINES_DIR: &str = ".mfr-baselines";
const BASELINES_DIR_EXIV2: &str = ".mfr-baselines-exiv2";
const BASELINES_DIR_LFS: &str = ".mfr-baselines-lfs";
const BASELINES_DIR_LFS_EXIV2: &str = ".mfr-baselines-lfs-exiv2";

/// Type of baseline (exiftool or exiv2 reference)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaselineType {
    Exiftool,
    Exiv2,
}

/// Dataset type (raws or data.lfs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataSet {
    #[default]
    Raws,
    DataLfs,
}

impl DataSet {
    /// Get the directory suffix for this dataset
    pub fn dir_suffix(&self) -> &'static str {
        match self {
            DataSet::Raws => "",
            DataSet::DataLfs => "-lfs",
        }
    }

    /// Get display name for this dataset
    pub fn display_name(&self) -> &'static str {
        match self {
            DataSet::Raws => "raws",
            DataSet::DataLfs => "data.lfs",
        }
    }
}

impl BaselineType {
    /// Get the directory name for this baseline type and dataset
    pub fn dir_name_for_dataset(&self, dataset: DataSet) -> &'static str {
        match (self, dataset) {
            (BaselineType::Exiftool, DataSet::Raws) => BASELINES_DIR,
            (BaselineType::Exiv2, DataSet::Raws) => BASELINES_DIR_EXIV2,
            (BaselineType::Exiftool, DataSet::DataLfs) => BASELINES_DIR_LFS,
            (BaselineType::Exiv2, DataSet::DataLfs) => BASELINES_DIR_LFS_EXIV2,
        }
    }

    /// Get the directory name for this baseline type (default dataset)
    pub fn dir_name(&self) -> &'static str {
        self.dir_name_for_dataset(DataSet::Raws)
    }

    /// Get the flag name for CLI messages
    pub fn flag_name(&self) -> &'static str {
        match self {
            BaselineType::Exiftool => "--save-baseline",
            BaselineType::Exiv2 => "--save-baseline-exiv2",
        }
    }
}

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

/// Get the baseline directory path for a manufacturer (with dataset support)
fn get_baseline_dir_with_dataset(
    manufacturer: &str,
    baseline_type: BaselineType,
    dataset: DataSet,
) -> PathBuf {
    PathBuf::from(baseline_type.dir_name_for_dataset(dataset)).join(manufacturer.to_lowercase())
}

/// Get the baseline file path for a manufacturer (with dataset support)
fn get_baseline_path_with_dataset(
    manufacturer: &str,
    baseline_type: BaselineType,
    dataset: DataSet,
) -> PathBuf {
    get_baseline_dir_with_dataset(manufacturer, baseline_type, dataset).join("baseline.json")
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

/// Save a baseline for a manufacturer (with full options)
pub fn save_baseline_full(
    manufacturer: &str,
    result: ManufacturerTestResult,
    description: Option<&str>,
    baseline_type: BaselineType,
    dataset: DataSet,
) -> Result<PathBuf, String> {
    let baseline_dir = get_baseline_dir_with_dataset(manufacturer, baseline_type, dataset);
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

    let baseline_path = get_baseline_path_with_dataset(manufacturer, baseline_type, dataset);
    let json = serde_json::to_string_pretty(&baseline)
        .map_err(|e| format!("Failed to serialize baseline: {}", e))?;

    fs::write(&baseline_path, json).map_err(|e| format!("Failed to write baseline file: {}", e))?;

    Ok(baseline_path)
}

/// Save a baseline for a manufacturer (typed version - backward compatible)
pub fn save_baseline_typed(
    manufacturer: &str,
    result: ManufacturerTestResult,
    description: Option<&str>,
    baseline_type: BaselineType,
) -> Result<PathBuf, String> {
    save_baseline_full(
        manufacturer,
        result,
        description,
        baseline_type,
        DataSet::Raws,
    )
}

/// Save a baseline for a manufacturer (exiftool - backward compatible)
pub fn save_baseline(
    manufacturer: &str,
    result: ManufacturerTestResult,
    description: Option<&str>,
) -> Result<PathBuf, String> {
    save_baseline_typed(manufacturer, result, description, BaselineType::Exiftool)
}

/// Load a baseline for a manufacturer (with full options)
pub fn load_baseline_full(
    manufacturer: &str,
    baseline_type: BaselineType,
    dataset: DataSet,
) -> Result<Baseline, String> {
    let baseline_path = get_baseline_path_with_dataset(manufacturer, baseline_type, dataset);

    if !baseline_path.exists() {
        return Err(format!(
            "No baseline found for '{}' ({} dataset). Run with {} first.",
            manufacturer,
            dataset.display_name(),
            baseline_type.flag_name()
        ));
    }

    let json = fs::read_to_string(&baseline_path)
        .map_err(|e| format!("Failed to read baseline: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse baseline: {}", e))
}

/// Load a baseline for a manufacturer (typed version - backward compatible)
pub fn load_baseline_typed(
    manufacturer: &str,
    baseline_type: BaselineType,
) -> Result<Baseline, String> {
    load_baseline_full(manufacturer, baseline_type, DataSet::Raws)
}

/// Load a baseline for a manufacturer (exiftool - backward compatible)
pub fn load_baseline(manufacturer: &str) -> Result<Baseline, String> {
    load_baseline_typed(manufacturer, BaselineType::Exiftool)
}

/// Check if a baseline exists for a manufacturer (with full options)
pub fn baseline_exists_full(
    manufacturer: &str,
    baseline_type: BaselineType,
    dataset: DataSet,
) -> bool {
    get_baseline_path_with_dataset(manufacturer, baseline_type, dataset).exists()
}

/// Check if a baseline exists for a manufacturer (typed version - backward compatible)
pub fn baseline_exists_typed(manufacturer: &str, baseline_type: BaselineType) -> bool {
    baseline_exists_full(manufacturer, baseline_type, DataSet::Raws)
}

/// Check if a baseline exists for a manufacturer (exiftool - backward compatible)
pub fn baseline_exists(manufacturer: &str) -> bool {
    baseline_exists_typed(manufacturer, BaselineType::Exiftool)
}

/// List all manufacturers with saved baselines (with full options)
pub fn list_baselines_full(
    baseline_type: BaselineType,
    dataset: DataSet,
) -> Vec<(String, BaselineMetadata)> {
    let baselines_dir = PathBuf::from(baseline_type.dir_name_for_dataset(dataset));
    if !baselines_dir.exists() {
        return Vec::new();
    }

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(baselines_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir()
                && let Some(mfr) = entry.file_name().to_str()
                && let Ok(baseline) = load_baseline_full(mfr, baseline_type, dataset)
            {
                results.push((mfr.to_string(), baseline.metadata));
            }
        }
    }

    results
}

/// List all manufacturers with saved baselines (typed version - backward compatible)
pub fn list_baselines_typed(baseline_type: BaselineType) -> Vec<(String, BaselineMetadata)> {
    list_baselines_full(baseline_type, DataSet::Raws)
}

/// List all manufacturers with saved baselines (exiftool - backward compatible)
pub fn list_baselines() -> Vec<(String, BaselineMetadata)> {
    list_baselines_typed(BaselineType::Exiftool)
}
