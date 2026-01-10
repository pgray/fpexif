//! Manufacturer-specific EXIF testing CLI
//!
//! Usage:
//!   mfr-test <manufacturer> --save-baseline    # Save current state
//!   mfr-test <manufacturer> --check            # Compare against baseline
//!   mfr-test <manufacturer> --vs-exiftool      # Compare against exiftool
//!   mfr-test <manufacturer> --full-report      # Full comparison report
//!   mfr-test --list-baselines                  # List saved baselines

use clap::Parser;
use fpexif::mfr_test::{
    baseline, comparison, get_formats_for_manufacturer, get_supported_manufacturers, output,
};

#[derive(Parser)]
#[command(name = "mfr-test")]
#[command(about = "Manufacturer-specific EXIF tag testing tool")]
#[command(version)]
struct Cli {
    /// Manufacturer to test (canon, nikon, sony, fujifilm, panasonic, olympus, pentax, minolta, kodak, sigma, samsung, ricoh, leica, dng)
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

    /// Verbose output (show per-file details)
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Handle --list-baselines (no manufacturer required)
    if cli.list_baselines {
        let baselines = baseline::list_baselines();
        output::print_baselines_list(&baselines);
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
        match baseline::load_baseline(&manufacturer) {
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
        match baseline::save_baseline(&manufacturer, current_result.clone(), None) {
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
        match baseline::load_baseline(&manufacturer) {
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
        let baseline_data = baseline::load_baseline(&manufacturer).ok();
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
