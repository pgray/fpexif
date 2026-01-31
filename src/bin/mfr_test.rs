//! Manufacturer-specific EXIF testing CLI
//!
//! Usage:
//!   mfr-test <manufacturer> --save-baseline       # Save exiftool baseline
//!   mfr-test <manufacturer> --check               # Compare against exiftool baseline
//!   mfr-test <manufacturer> --vs-exiftool         # Compare against exiftool
//!   mfr-test <manufacturer> --save-baseline-exiv2 # Save exiv2 baseline
//!   mfr-test <manufacturer> --check-exiv2         # Compare against exiv2 baseline
//!   mfr-test <manufacturer> --vs-exiv2            # Compare against exiv2
//!   mfr-test <manufacturer> --full-report         # Full comparison report
//!   mfr-test --list-baselines                     # List exiftool baselines
//!   mfr-test --list-baselines-exiv2               # List exiv2 baselines

use clap::Parser;
use fpexif::mfr_test::{
    baseline::{self, BaselineType, DataSet},
    comparison, get_formats_for_manufacturer, get_supported_manufacturers, output,
};

#[derive(Parser)]
#[command(name = "mfr-test")]
#[command(about = "Manufacturer-specific EXIF tag testing tool")]
#[command(version)]
struct Cli {
    /// Manufacturer to test (canon, nikon, sony, fujifilm, panasonic, olympus, pentax, minolta, kodak, sigma, samsung, ricoh, leica, hasselblad, dng)
    manufacturer: Option<String>,

    /// Save current fpexif state as baseline
    #[arg(long)]
    save_baseline: bool,

    /// Compare current state against saved baseline
    #[arg(long)]
    check: bool,

    /// Compare against exiftool output (ground truth)
    #[arg(long)]
    vs_exiftool: bool,

    /// Run full report (baseline + exiftool comparison)
    #[arg(long)]
    full_report: bool,

    /// Show saved baseline details for manufacturer
    #[arg(long)]
    show_baseline: bool,

    /// List all saved baselines
    #[arg(long)]
    list_baselines: bool,

    /// Compare against exiv2 output (secondary reference)
    #[arg(long)]
    vs_exiv2: bool,

    /// Save current fpexif state as exiv2 baseline
    #[arg(long)]
    save_baseline_exiv2: bool,

    /// Compare current state against saved exiv2 baseline
    #[arg(long)]
    check_exiv2: bool,

    /// Show saved exiv2 baseline details for manufacturer
    #[arg(long)]
    show_baseline_exiv2: bool,

    /// List all saved exiv2 baselines
    #[arg(long)]
    list_baselines_exiv2: bool,

    /// Verbose output (show per-file details)
    #[arg(short, long)]
    verbose: bool,

    /// Use /fpexif/data.lfs directory (large test dataset)
    #[arg(long)]
    data_lfs: bool,
}

fn main() {
    let cli = Cli::parse();

    // Determine dataset based on --data-lfs flag
    let dataset = if cli.data_lfs {
        DataSet::DataLfs
    } else {
        DataSet::Raws
    };

    // Set test directory if --data-lfs flag is used
    if cli.data_lfs {
        std::env::set_var("FPEXIF_TEST_FILES", "/fpexif/data.lfs");
    }

    // Handle --list-baselines (no manufacturer required)
    if cli.list_baselines {
        let baselines = baseline::list_baselines_full(BaselineType::Exiftool, dataset);
        if baselines.is_empty() && dataset == DataSet::DataLfs {
            println!("No data.lfs baselines saved.");
            println!("Run: ./bin/mfr-test <manufacturer> --data-lfs --save-baseline");
        } else {
            output::print_baselines_list(&baselines);
        }
        return;
    }

    // Handle --list-baselines-exiv2 (no manufacturer required)
    if cli.list_baselines_exiv2 {
        let baselines = baseline::list_baselines_full(BaselineType::Exiv2, dataset);
        if baselines.is_empty() {
            if dataset == DataSet::DataLfs {
                println!("No data.lfs exiv2 baselines saved.");
                println!("Run: ./bin/mfr-test <manufacturer> --data-lfs --save-baseline-exiv2");
            } else {
                println!("No exiv2 baselines saved.");
                println!("Run: ./bin/mfr-test <manufacturer> --save-baseline-exiv2");
            }
        } else {
            println!("Saved Exiv2 Baselines ({}):", dataset.display_name());
            println!("{}", "-".repeat(60));
            for (mfr, meta) in &baselines {
                println!(
                    "  {:12} {} (commit {}, branch {})",
                    mfr,
                    &meta.created_at[..10],
                    meta.git_commit,
                    meta.git_branch
                );
            }
        }
        return;
    }

    // All other commands require a manufacturer
    let manufacturer = match &cli.manufacturer {
        Some(m) => m.to_lowercase(),
        None => {
            eprintln!("Error: manufacturer is required");
            eprintln!();
            eprintln!(
                "Supported manufacturers: {}",
                get_supported_manufacturers().join(", ")
            );
            eprintln!();
            eprintln!("Usage: mfr-test <manufacturer> [OPTIONS]");
            eprintln!("       mfr-test --list-baselines");
            std::process::exit(1);
        }
    };

    // Validate manufacturer
    let formats = match get_formats_for_manufacturer(&manufacturer) {
        Some(f) => f,
        None => {
            eprintln!("Error: Unknown manufacturer '{}'", manufacturer);
            eprintln!();
            eprintln!(
                "Supported manufacturers: {}",
                get_supported_manufacturers().join(", ")
            );
            std::process::exit(1);
        }
    };

    // Handle --show-baseline
    if cli.show_baseline {
        match baseline::load_baseline_full(&manufacturer, BaselineType::Exiftool, dataset) {
            Ok(b) => {
                output::print_baseline_details(&b.metadata, &b.result);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Handle --show-baseline-exiv2
    if cli.show_baseline_exiv2 {
        match baseline::load_baseline_full(&manufacturer, BaselineType::Exiv2, dataset) {
            Ok(b) => {
                println!(
                    "Exiv2 Baseline for: {}",
                    b.result.manufacturer.to_uppercase()
                );
                println!("{}", "-".repeat(40));
                println!("  Created:    {}", b.metadata.created_at);
                println!("  Commit:     {}", b.metadata.git_commit);
                println!("  Branch:     {}", b.metadata.git_branch);
                if let Some(desc) = &b.metadata.description {
                    println!("  Description: {}", desc);
                }
                println!();
                println!("  Files tested:    {}", b.result.files_tested);
                println!("  Matching tags:   {}", b.result.total_matching_tags);
                println!("  Mismatched:      {}", b.result.total_mismatched_tags);
                println!("  Missing:         {}", b.result.total_missing_tags);
                println!("  Extra:           {}", b.result.total_extra_tags);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Handle exiv2-specific commands
    if cli.vs_exiv2 || cli.save_baseline_exiv2 || cli.check_exiv2 {
        // Run exiv2 comparison
        let current_result =
            match comparison::run_exiv2_comparison(&manufacturer, formats, cli.verbose) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

        // Handle --save-baseline-exiv2
        if cli.save_baseline_exiv2 {
            match baseline::save_baseline_full(
                &manufacturer,
                current_result.clone(),
                None,
                BaselineType::Exiv2,
                dataset,
            ) {
                Ok(path) => {
                    println!("Exiv2 baseline saved to: {}", path.display());
                    println!();
                    output::print_exiv2_summary(&current_result);
                }
                Err(e) => {
                    eprintln!("Error saving baseline: {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }

        // Handle --check-exiv2 (compare against exiv2 baseline)
        if cli.check_exiv2 {
            match baseline::load_baseline_full(&manufacturer, BaselineType::Exiv2, dataset) {
                Ok(b) => {
                    let diff = comparison::compare_with_baseline(&current_result, &b);
                    output::print_baseline_comparison(&current_result, &b.metadata, &diff);
                    // NOTE: Don't exit with error for exiv2 regressions (informational only)
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            return;
        }

        // Default for exiv2: --vs-exiv2
        output::print_exiv2_summary(&current_result);
        return;
    }

    // We always need to run exiftool comparison for any action
    // (save_baseline, check, vs_exiftool, full_report all require current comparison data)

    // Run exiftool comparison
    let current_result =
        match comparison::run_exiftool_comparison(&manufacturer, formats, cli.verbose) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

    // Handle --save-baseline
    if cli.save_baseline {
        match baseline::save_baseline_full(
            &manufacturer,
            current_result.clone(),
            None,
            BaselineType::Exiftool,
            dataset,
        ) {
            Ok(path) => {
                println!("Baseline saved to: {}", path.display());
                println!();
                output::print_exiftool_summary(&current_result);
            }
            Err(e) => {
                eprintln!("Error saving baseline: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Handle --check (compare against baseline)
    if cli.check {
        match baseline::load_baseline_full(&manufacturer, BaselineType::Exiftool, dataset) {
            Ok(b) => {
                let diff = comparison::compare_with_baseline(&current_result, &b);
                output::print_baseline_comparison(&current_result, &b.metadata, &diff);

                // Exit with error if regressions detected
                if !diff.regressions.is_empty() {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Handle --full-report
    if cli.full_report {
        let baseline_data =
            baseline::load_baseline_full(&manufacturer, BaselineType::Exiftool, dataset).ok();
        let diff = baseline_data
            .as_ref()
            .map(|b| comparison::compare_with_baseline(&current_result, b));

        output::print_full_report(
            &current_result,
            baseline_data.as_ref().map(|b| &b.metadata),
            diff.as_ref(),
        );

        // Exit with error if regressions detected
        if let Some(d) = diff {
            if !d.regressions.is_empty() {
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: --vs-exiftool
    if cli.verbose {
        output::print_verbose(&current_result);
    } else {
        output::print_exiftool_summary(&current_result);
    }
}
