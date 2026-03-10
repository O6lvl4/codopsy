use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use codopsy::analyze::{analyze, AnalyzeOptions};
use codopsy::baseline::{compare_with_baseline, load_baseline, save_baseline};
use codopsy::commands::init::init_action;
use codopsy::commands::print::{
    print_baseline_comparison, print_hotspots, print_summary, print_verbose,
};
use codopsy::config::load_config;
use codopsy::hotspot::detect_hotspots;
use codopsy::reporter::format_report;
use codopsy::utils::file::find_source_files;
use codopsy::utils::git::{get_changed_files, is_git_repository};

#[derive(Parser)]
#[command(name = "codopsy", version, about = "AST-level code quality analyzer for TypeScript, JavaScript & Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze source files in a directory
    Analyze {
        /// Target directory to analyze
        dir: String,

        /// Output file path (use "-" for stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Output format: json
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Complexity threshold for warnings
        #[arg(long, default_value = "10")]
        max_complexity: usize,

        /// Cognitive complexity threshold for warnings
        #[arg(long, default_value = "15")]
        max_cognitive_complexity: usize,

        /// Exit with code 1 if warnings are found
        #[arg(long)]
        fail_on_warning: bool,

        /// Exit with code 1 if errors are found
        #[arg(long)]
        fail_on_error: bool,

        /// Show summary only
        #[arg(short, long)]
        quiet: bool,

        /// Show per-file analysis results
        #[arg(short, long)]
        verbose: bool,

        /// Only analyze files changed from base ref
        #[arg(long)]
        diff: Option<String>,

        /// Save current results as baseline
        #[arg(long)]
        save_baseline: bool,

        /// Path to baseline file
        #[arg(long, default_value = ".codopsy-baseline.json")]
        baseline_path: String,

        /// Exit 1 if quality degrades vs baseline
        #[arg(long)]
        no_degradation: bool,

        /// Show hotspot analysis
        #[arg(long)]
        hotspots: bool,
    },
    /// Create a .codopsyrc.json configuration file
    Init {
        /// Target directory
        #[arg(default_value = ".")]
        dir: String,

        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            dir,
            output,
            format,
            max_complexity,
            max_cognitive_complexity,
            fail_on_warning,
            fail_on_error,
            quiet,
            verbose,
            diff,
            save_baseline: do_save_baseline,
            baseline_path,
            no_degradation,
            hotspots,
        } => {
            let target_dir = PathBuf::from(&dir).canonicalize().unwrap_or_else(|_| {
                eprintln!("Error: directory \"{dir}\" does not exist.");
                std::process::exit(1);
            });

            let is_stdout = output.as_deref() == Some("-");
            let config = load_config(&target_dir);

            if !quiet && !is_stdout {
                eprintln!("Analyzing {} ...", target_dir.display());
            }

            let mut files = find_source_files(&target_dir);

            if let Some(base_ref) = &diff {
                if !is_git_repository(&target_dir) {
                    eprintln!("Error: --diff requires a git repository.");
                    std::process::exit(1);
                }
                let changed: std::collections::HashSet<String> =
                    get_changed_files(&target_dir, base_ref)
                        .into_iter()
                        .collect();
                files.retain(|f| changed.contains(f));
            }

            if files.is_empty() {
                if !quiet {
                    eprintln!("No source files found.");
                }
                return;
            }

            if !quiet && !is_stdout {
                eprintln!("Found {} source file(s).", files.len());
            }

            let opts = AnalyzeOptions {
                max_complexity,
                max_cognitive_complexity,
            };

            let result = analyze(&target_dir, &files, &config, &opts);

            // Verbose output
            if verbose && !is_stdout {
                for analysis in &result.files {
                    print_verbose(analysis);
                }
            }

            // Hotspots
            if hotspots && !is_stdout && is_git_repository(&target_dir) {
                let hotspot_result = detect_hotspots(&target_dir, &result.files, 6, 10);
                print_hotspots(&hotspot_result);
            }

            // Output report
            if is_stdout {
                print!("{}", format_report(&result, &format));
            } else {
                let output_path = output.unwrap_or_else(|| format!("codopsy-report.{format}"));
                let report = format_report(&result, &format);
                std::fs::write(&output_path, &report).unwrap_or_else(|e| {
                    eprintln!("Error writing report: {e}");
                    std::process::exit(1);
                });
                print_summary(&result);
                eprintln!("Report written to: {output_path}");
            }

            // Baseline handling
            let baseline_file = Path::new(&baseline_path);
            if do_save_baseline {
                if let Err(e) = save_baseline(&result, baseline_file) {
                    eprintln!("Error saving baseline: {e}");
                }
                if !is_stdout {
                    eprintln!("Baseline saved to: {baseline_path}");
                }
            } else if let Some(baseline) = load_baseline(baseline_file) {
                if !is_stdout {
                    let comparison = compare_with_baseline(&result, &baseline);
                    print_baseline_comparison(&comparison);
                    if no_degradation && comparison.status == "degraded" {
                        std::process::exit(1);
                    }
                }
            }

            // Fail conditions
            if fail_on_warning
                && result
                    .summary
                    .issues_by_severity
                    .get("warning")
                    .copied()
                    .unwrap_or(0)
                    > 0
            {
                std::process::exit(1);
            }
            if fail_on_error
                && result
                    .summary
                    .issues_by_severity
                    .get("error")
                    .copied()
                    .unwrap_or(0)
                    > 0
            {
                std::process::exit(1);
            }
        }
        Commands::Init { dir, force } => {
            let target_dir = PathBuf::from(&dir);
            if let Err(e) = init_action(&target_dir, force) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}
