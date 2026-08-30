use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

use codopsy::analyze::{analyze, AnalyzeOptions};
use codopsy::baseline::{compare_with_baseline, load_baseline, save_baseline};
use codopsy::commands::init::init_action;
use codopsy::commands::print::{
    print_baseline_comparison, print_hotspots, print_summary, print_verbose,
};
use codopsy::config::load_config;
use codopsy::hotspot::detect_hotspots;
use codopsy::reporter::format_report;
use codopsy::types::AnalysisResult;
use codopsy::utils::file::find_source_files_with_config;
use codopsy::utils::git::{get_changed_files, is_git_repository};

#[derive(Parser)]
#[command(name = "codopsy", version, about = "AST-level code quality analyzer for 35 languages")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze source files in a directory
    Analyze(AnalyzeArgs),
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

#[derive(Args)]
struct AnalyzeArgs {
    /// Target directory to analyze
    dir: String,

    /// Output file path (use "-" for stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Output format: json
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Complexity threshold for warnings
    #[arg(long, default_value_t = codopsy::defaults::MAX_COMPLEXITY)]
    max_complexity: usize,

    /// Cognitive complexity threshold for warnings
    #[arg(long, default_value_t = codopsy::defaults::MAX_COGNITIVE_COMPLEXITY)]
    max_cognitive_complexity: usize,

    /// Exit with code 1 if warnings are found
    #[arg(long)]
    fail_on_warning: bool,

    /// Exit with code 1 if errors are found
    #[arg(long)]
    fail_on_error: bool,

    /// Exit with code 1 if any file could not be parsed (unanalyzed)
    #[arg(long)]
    fail_on_unanalyzed: bool,

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze(args) => run_analyze(args),
        Commands::Init { dir, force } => {
            let target_dir = PathBuf::from(&dir);
            if let Err(e) = init_action(&target_dir, force) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn run_analyze(args: AnalyzeArgs) {
    let target_dir = PathBuf::from(&args.dir).canonicalize().unwrap_or_else(|_| {
        eprintln!("Error: directory \"{}\" does not exist.", args.dir);
        std::process::exit(1);
    });

    let is_stdout = args.output.as_deref() == Some("-");
    let config = load_config(&target_dir);

    if !args.quiet && !is_stdout {
        eprintln!("Analyzing {} ...", target_dir.display());
    }

    let files = resolve_files(&target_dir, args.diff.as_deref(), args.quiet, &config);
    if files.is_empty() {
        return;
    }

    if !args.quiet && !is_stdout {
        eprintln!("Found {} source file(s).", files.len());
    }

    let opts = AnalyzeOptions {
        max_complexity: args.max_complexity,
        max_cognitive_complexity: args.max_cognitive_complexity,
    };

    let result = analyze(&target_dir, &files, &config, &opts);

    if args.verbose && !is_stdout {
        for analysis in &result.files {
            print_verbose(analysis);
        }
    }

    if args.hotspots && !is_stdout && is_git_repository(&target_dir) {
        let hotspot_result = detect_hotspots(&target_dir, &result.files, 6, 10);
        print_hotspots(&hotspot_result);
    }

    write_output(&result, args.output, &args.format, is_stdout);
    handle_baseline(&result, &args.baseline_path, args.save_baseline, args.no_degradation, is_stdout);
    check_fail_conditions(
        &result,
        args.fail_on_warning,
        args.fail_on_error,
        args.fail_on_unanalyzed,
    );
}

fn resolve_files(target_dir: &Path, diff: Option<&str>, quiet: bool, config: &codopsy::config::CodopsyConfig) -> Vec<String> {
    let mut files = find_source_files_with_config(target_dir, config);

    if let Some(base_ref) = diff {
        if !is_git_repository(target_dir) {
            eprintln!("Error: --diff requires a git repository.");
            std::process::exit(1);
        }
        let changed: std::collections::HashSet<String> =
            get_changed_files(target_dir, base_ref).into_iter().collect();
        files.retain(|f| changed.contains(f));
    }

    if files.is_empty() && !quiet {
        eprintln!("No source files found.");
    }

    files
}

fn write_output(result: &AnalysisResult, output: Option<String>, format: &str, is_stdout: bool) {
    if is_stdout {
        print!("{}", format_report(result, format));
    } else {
        let output_path = output.unwrap_or_else(|| format!("codopsy-report.{format}"));
        let report = format_report(result, format);
        std::fs::write(&output_path, &report).unwrap_or_else(|e| {
            eprintln!("Error writing report: {e}");
            std::process::exit(1);
        });
        print_summary(result);
        eprintln!("Report written to: {output_path}");
    }
}

fn handle_baseline(result: &AnalysisResult, path: &str, do_save: bool, no_degradation: bool, is_stdout: bool) {
    let baseline_file = Path::new(path);
    if do_save {
        if let Err(e) = save_baseline(result, baseline_file) {
            eprintln!("Error saving baseline: {e}");
        }
        if !is_stdout {
            eprintln!("Baseline saved to: {path}");
        }
    } else if let Some(baseline) = load_baseline(baseline_file) {
        if !is_stdout {
            let comparison = compare_with_baseline(result, &baseline);
            print_baseline_comparison(&comparison);
            if no_degradation && comparison.status == codopsy::types::ComparisonStatus::Degraded {
                std::process::exit(1);
            }
        }
    }
}

fn check_fail_conditions(
    result: &AnalysisResult,
    fail_on_warning: bool,
    fail_on_error: bool,
    fail_on_unanalyzed: bool,
) {
    if should_fail(result, fail_on_warning, fail_on_error, fail_on_unanalyzed) {
        std::process::exit(1);
    }
}

/// Whether any requested `--fail-on-*` gate trips for this result.
/// Kept pure (no exit) so the policy is unit-testable.
///
/// An unanalyzed file is one the grammar could not read; gating on it lets CI
/// refuse to pass on files codopsy silently skipped rather than scored.
fn should_fail(
    result: &AnalysisResult,
    fail_on_warning: bool,
    fail_on_error: bool,
    fail_on_unanalyzed: bool,
) -> bool {
    let has_severity = |key: &str| {
        result.summary.issues_by_severity.get(key).copied().unwrap_or(0) > 0
    };
    (fail_on_unanalyzed && result.summary.unanalyzed_files > 0)
        || (fail_on_warning && has_severity("warning"))
        || (fail_on_error && has_severity("error"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codopsy::types::{ScoringThresholds, Summary};
    use std::collections::HashMap;

    fn result_with(warnings: usize, errors: usize, unanalyzed: usize) -> AnalysisResult {
        let mut by_sev = HashMap::new();
        by_sev.insert("warning".to_string(), warnings);
        by_sev.insert("error".to_string(), errors);
        by_sev.insert("info".to_string(), 0);
        AnalysisResult {
            timestamp: "t".to_string(),
            target_dir: ".".to_string(),
            files: vec![],
            summary: Summary {
                total_files: 0,
                total_issues: warnings + errors,
                issues_by_severity: by_sev,
                average_complexity: 0.0,
                max_complexity: None,
                unanalyzed_files: unanalyzed,
            },
            scoring_thresholds: ScoringThresholds {
                cyclomatic_complexity: Some(10),
                cognitive_complexity: Some(15),
            },
            score: None,
        }
    }

    #[test]
    fn no_flags_never_fails() {
        let r = result_with(3, 2, 5);
        assert!(!should_fail(&r, false, false, false));
    }

    #[test]
    fn fail_on_unanalyzed_only_trips_on_unanalyzed_files() {
        // Unanalyzed present -> trips.
        let with_unanalyzed = result_with(0, 0, 1);
        assert!(should_fail(&with_unanalyzed, false, false, true));
        // No unanalyzed, even with warnings/errors -> the unanalyzed gate stays quiet.
        let clean_parse = result_with(9, 9, 0);
        assert!(!should_fail(&clean_parse, false, false, true));
    }

    #[test]
    fn unanalyzed_is_independent_of_severity_gates() {
        // Only unanalyzed files, no issues: the warning/error gates don't see it,
        // but the unanalyzed gate does.
        let r = result_with(0, 0, 2);
        assert!(!should_fail(&r, true, true, false));
        assert!(should_fail(&r, false, false, true));
    }

    #[test]
    fn severity_gates_still_work() {
        assert!(should_fail(&result_with(1, 0, 0), true, false, false));
        assert!(should_fail(&result_with(0, 1, 0), false, true, false));
        assert!(!should_fail(&result_with(1, 0, 0), false, true, false));
    }
}
