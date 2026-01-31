//! Terminal output formatting for test results

use super::{baseline::BaselineMetadata, BaselineDiff, ManufacturerTestResult};
use std::collections::HashMap;

const LINE_WIDTH: usize = 80;

fn print_header(manufacturer: &str, baseline: Option<&BaselineMetadata>) {
    println!("{}", "=".repeat(LINE_WIDTH));
    println!("  {} EXIF Testing Report", manufacturer.to_uppercase());
    if let Some(meta) = baseline {
        println!(
            "  Baseline: {} (commit {})",
            &meta.created_at[..10],
            meta.git_commit
        );
    }
    println!("{}", "=".repeat(LINE_WIDTH));
    println!();
}

fn format_delta(delta: i64, positive_is_good: bool) -> String {
    if delta == 0 {
        "(0)".to_string()
    } else if delta > 0 {
        if positive_is_good {
            format!("(+{}) [UP]", delta)
        } else {
            format!("(+{}) [UP - BAD]", delta)
        }
    } else if positive_is_good {
        format!("({}) [DOWN - BAD]", delta)
    } else {
        format!("({}) [DOWN - GOOD]", delta)
    }
}

/// Print a summary of exiftool comparison results
pub fn print_exiftool_summary(result: &ManufacturerTestResult) {
    print_header(&result.manufacturer, None);

    // Format breakdown
    let mut format_counts: HashMap<&str, usize> = HashMap::new();
    for file in &result.file_results {
        *format_counts.entry(&file.format).or_insert(0) += 1;
    }
    let format_str: Vec<String> = format_counts
        .iter()
        .map(|(f, c)| format!("{} {}", c, f))
        .collect();

    println!("VS EXIFTOOL (Ground Truth)");
    println!("{}", "-".repeat(40));
    println!("  Files tested:    {}", format_str.join(", "));
    println!("  Matching tags:   {}", result.total_matching_tags);
    println!("  Mismatched:      {}", result.total_mismatched_tags);
    println!("  Missing:         {}", result.total_missing_tags);
    println!("  Extra:           {}", result.total_extra_tags);

    let total_compared =
        result.total_matching_tags + result.total_mismatched_tags + result.total_missing_tags;
    if total_compared > 0 {
        let match_rate = (result.total_matching_tags as f64 / total_compared as f64) * 100.0;
        println!();
        println!("  Match rate: {:.1}%", match_rate);
    }

    println!();
    println!("{}", "=".repeat(LINE_WIDTH));
}

/// Print a summary of exiv2 comparison results
pub fn print_exiv2_summary(result: &ManufacturerTestResult) {
    print_header(&result.manufacturer, None);

    // Format breakdown
    let mut format_counts: HashMap<&str, usize> = HashMap::new();
    for file in &result.file_results {
        *format_counts.entry(&file.format).or_insert(0) += 1;
    }
    let format_str: Vec<String> = format_counts
        .iter()
        .map(|(f, c)| format!("{} {}", c, f))
        .collect();

    println!("VS EXIV2 (Secondary Reference)");
    println!("{}", "-".repeat(40));
    println!("  Files tested:    {}", format_str.join(", "));
    println!("  Matching tags:   {}", result.total_matching_tags);
    println!("  Mismatched:      {}", result.total_mismatched_tags);
    println!("  Missing:         {}", result.total_missing_tags);
    println!("  Extra:           {}", result.total_extra_tags);

    let total_compared =
        result.total_matching_tags + result.total_mismatched_tags + result.total_missing_tags;
    if total_compared > 0 {
        let match_rate = (result.total_matching_tags as f64 / total_compared as f64) * 100.0;
        println!();
        println!("  Match rate: {:.1}%", match_rate);
    }

    println!();
    println!("{}", "=".repeat(LINE_WIDTH));
}

/// Print a comparison against baseline
pub fn print_baseline_comparison(
    result: &ManufacturerTestResult,
    baseline_meta: &BaselineMetadata,
    diff: &BaselineDiff,
) {
    print_header(&result.manufacturer, Some(baseline_meta));

    // Format breakdown
    let mut format_counts: HashMap<&str, usize> = HashMap::new();
    for file in &result.file_results {
        *format_counts.entry(&file.format).or_insert(0) += 1;
    }
    let format_str: Vec<String> = format_counts
        .iter()
        .map(|(f, c)| format!("{} {}", c, f))
        .collect();

    println!("SUMMARY (vs Baseline)");
    println!("{}", "-".repeat(40));
    println!("  Files tested:    {}", format_str.join(", "));
    println!(
        "  Matching tags:   {} {}",
        result.total_matching_tags,
        format_delta(diff.matching_delta, true)
    );
    println!(
        "  Mismatched:      {} {}",
        result.total_mismatched_tags,
        format_delta(diff.mismatched_delta, false)
    );
    println!(
        "  Missing:         {} {}",
        result.total_missing_tags,
        format_delta(diff.missing_delta, false)
    );
    println!(
        "  Extra:           {} {}",
        result.total_extra_tags,
        format_delta(diff.extra_delta, false)
    );

    println!();

    // Improvements
    if !diff.improvements.is_empty() {
        // Group by tag name
        let mut by_tag: HashMap<String, Vec<String>> = HashMap::new();
        for change in &diff.improvements {
            by_tag
                .entry(change.tag.clone())
                .or_default()
                .push(change.file.clone());
        }

        println!("IMPROVEMENTS ({} tags fixed)", diff.improvements.len());
        println!("{}", "-".repeat(40));
        for (tag, files) in by_tag.iter().take(10) {
            println!("  {:20}: {} files now correct", tag, files.len());
        }
        if by_tag.len() > 10 {
            println!("  ... and {} more tags", by_tag.len() - 10);
        }
        println!();
    } else {
        println!("IMPROVEMENTS (0)");
        println!("{}", "-".repeat(40));
        println!("  None");
        println!();
    }

    // Regressions
    println!("REGRESSIONS ({})", diff.regressions.len());
    println!("{}", "-".repeat(40));
    if diff.regressions.is_empty() {
        println!("  None - excellent work!");
    } else {
        // Group by tag name
        let mut by_tag: HashMap<String, Vec<String>> = HashMap::new();
        for change in &diff.regressions {
            by_tag
                .entry(change.tag.clone())
                .or_default()
                .push(change.file.clone());
        }
        for (tag, files) in &by_tag {
            println!("  {:20}: {} files now broken", tag, files.len());
        }
        println!();
        println!("  !! REGRESSIONS DETECTED - Please fix before completing work");
    }

    println!();
    println!("{}", "=".repeat(LINE_WIDTH));
}

/// Print a full report (baseline + exiftool)
pub fn print_full_report(
    result: &ManufacturerTestResult,
    baseline_meta: Option<&BaselineMetadata>,
    diff: Option<&BaselineDiff>,
) {
    print_header(&result.manufacturer, baseline_meta);

    // Format breakdown
    let mut format_counts: HashMap<&str, usize> = HashMap::new();
    for file in &result.file_results {
        *format_counts.entry(&file.format).or_insert(0) += 1;
    }
    let format_str: Vec<String> = format_counts
        .iter()
        .map(|(f, c)| format!("{} {}", c, f))
        .collect();

    // Summary section
    if let (Some(meta), Some(_)) = (baseline_meta, diff) {
        println!(
            "SUMMARY (vs Baseline from {} @ {})",
            &meta.created_at[..10],
            meta.git_commit
        );
    } else {
        println!("SUMMARY");
    }
    println!("{}", "-".repeat(40));
    println!("  Files tested:    {}", format_str.join(", "));

    if let Some(d) = diff {
        println!(
            "  Matching tags:   {} {}",
            result.total_matching_tags,
            format_delta(d.matching_delta, true)
        );
        println!(
            "  Mismatched:      {} {}",
            result.total_mismatched_tags,
            format_delta(d.mismatched_delta, false)
        );
        println!(
            "  Missing:         {} {}",
            result.total_missing_tags,
            format_delta(d.missing_delta, false)
        );
        println!(
            "  Extra:           {} {}",
            result.total_extra_tags,
            format_delta(d.extra_delta, false)
        );
    } else {
        println!("  Matching tags:   {}", result.total_matching_tags);
        println!("  Mismatched:      {}", result.total_mismatched_tags);
        println!("  Missing:         {}", result.total_missing_tags);
        println!("  Extra:           {}", result.total_extra_tags);
    }

    // Match rate
    let total_compared =
        result.total_matching_tags + result.total_mismatched_tags + result.total_missing_tags;
    if total_compared > 0 {
        let match_rate = (result.total_matching_tags as f64 / total_compared as f64) * 100.0;
        println!();
        println!("  Match rate: {:.1}%", match_rate);
    }

    println!();

    // Improvements and regressions if we have baseline diff
    if let Some(d) = diff {
        if !d.improvements.is_empty() {
            let mut by_tag: HashMap<String, Vec<String>> = HashMap::new();
            for change in &d.improvements {
                by_tag
                    .entry(change.tag.clone())
                    .or_default()
                    .push(change.file.clone());
            }

            println!("IMPROVEMENTS ({} tags fixed)", d.improvements.len());
            println!("{}", "-".repeat(40));
            for (tag, files) in by_tag.iter().take(10) {
                println!("  {:20}: {} files", tag, files.len());
            }
            if by_tag.len() > 10 {
                println!("  ... and {} more", by_tag.len() - 10);
            }
            println!();
        }

        println!("REGRESSIONS ({})", d.regressions.len());
        println!("{}", "-".repeat(40));
        if d.regressions.is_empty() {
            println!("  None");
        } else {
            for change in d.regressions.iter().take(10) {
                println!(
                    "  {} / {}: \"{}\" -> \"{}\"",
                    change.file, change.tag, change.was, change.now
                );
            }
            if d.regressions.len() > 10 {
                println!("  ... and {} more", d.regressions.len() - 10);
            }
        }
        println!();
    }

    println!("{}", "=".repeat(LINE_WIDTH));
}

/// Print verbose output with per-file details
/// NOTE: This function is deprecated in favor of streaming verbose output
/// during file processing (see comparison.rs). This is kept for backwards
/// compatibility but should only be called when tags data is actually present.
pub fn print_verbose(result: &ManufacturerTestResult) {
    // Check if this is a streaming-mode result (tags are empty because
    // they were already printed during processing)
    if result.file_results.iter().all(|f| f.tags.is_empty()) {
        // Already printed during processing, just print summary
        print_exiftool_summary(result);
        return;
    }

    println!("{}", "=".repeat(LINE_WIDTH));
    println!(
        "  {} EXIF Testing Report - VERBOSE",
        result.manufacturer.to_uppercase()
    );
    println!("{}", "=".repeat(LINE_WIDTH));
    println!();

    for file in &result.file_results {
        println!("FILE: {}", file.file_name);
        println!("{}", "-".repeat(50));

        // Mismatches
        let mismatches: Vec<_> = file
            .tags
            .values()
            .filter(|t| !t.matches && t.fpexif_value.is_some() && t.exiftool_value.is_some())
            .collect();

        if !mismatches.is_empty() {
            println!("  Mismatches ({}):", mismatches.len());
            for tag in mismatches.iter().take(5) {
                println!(
                    "    {}: fpexif=\"{}\" exiftool=\"{}\"",
                    tag.tag_name,
                    tag.fpexif_value.as_deref().unwrap_or("?"),
                    tag.exiftool_value.as_deref().unwrap_or("?")
                );
            }
            if mismatches.len() > 5 {
                println!("    ... and {} more", mismatches.len() - 5);
            }
        }

        // Missing
        let missing: Vec<_> = file
            .tags
            .values()
            .filter(|t| t.fpexif_value.is_none() && t.exiftool_value.is_some())
            .collect();

        if !missing.is_empty() {
            println!("  Missing ({}):", missing.len());
            for tag in missing.iter().take(5) {
                println!(
                    "    {}: \"{}\"",
                    tag.tag_name,
                    tag.exiftool_value.as_deref().unwrap_or("?")
                );
            }
            if missing.len() > 5 {
                println!("    ... and {} more", missing.len() - 5);
            }
        }

        // Summary
        println!(
            "  Summary: {} matching, {} mismatch, {} missing",
            file.matching_tags, file.mismatched_tags, file.missing_tags
        );
        println!();
    }

    println!("{}", "=".repeat(LINE_WIDTH));
}

/// Print list of saved baselines
pub fn print_baselines_list(baselines: &[(String, BaselineMetadata)]) {
    if baselines.is_empty() {
        println!("No baselines saved.");
        println!("Run: ./bin/mfr-test <manufacturer> --save-baseline");
        return;
    }

    println!("Saved Baselines:");
    println!("{}", "-".repeat(60));
    for (mfr, meta) in baselines {
        println!(
            "  {:12} {} (commit {}, branch {})",
            mfr,
            &meta.created_at[..10],
            meta.git_commit,
            meta.git_branch
        );
    }
}

/// Print baseline details for a manufacturer
pub fn print_baseline_details(baseline_meta: &BaselineMetadata, result: &ManufacturerTestResult) {
    println!("Baseline for: {}", result.manufacturer.to_uppercase());
    println!("{}", "-".repeat(40));
    println!("  Created:    {}", baseline_meta.created_at);
    println!("  Commit:     {}", baseline_meta.git_commit);
    println!("  Branch:     {}", baseline_meta.git_branch);
    if let Some(desc) = &baseline_meta.description {
        println!("  Description: {}", desc);
    }
    println!();
    println!("  Files tested:    {}", result.files_tested);
    println!("  Matching tags:   {}", result.total_matching_tags);
    println!("  Mismatched:      {}", result.total_mismatched_tags);
    println!("  Missing:         {}", result.total_missing_tags);
    println!("  Extra:           {}", result.total_extra_tags);
}
