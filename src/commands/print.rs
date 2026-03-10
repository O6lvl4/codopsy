use crate::baseline::BaselineComparison;
use crate::hotspot::HotspotResult;
use crate::types::{AnalysisResult, FileAnalysis};

pub fn print_summary(result: &AnalysisResult) {
    eprintln!();
    eprintln!("\x1b[1m=== Analysis Summary ===\x1b[0m");
    if let Some(score) = &result.score {
        eprintln!(
            "  Quality Score:  {} ({}/100)",
            grade_colored(&score.grade.to_string()),
            score.overall
        );
    }
    eprintln!(
        "  Files analyzed: \x1b[1m{}\x1b[0m",
        result.summary.total_files
    );
    eprintln!(
        "  Total issues:   \x1b[1m{}\x1b[0m",
        result.summary.total_issues
    );
    eprintln!(
        "    \x1b[31mError:\x1b[0m   {}",
        result.summary.issues_by_severity.get("error").unwrap_or(&0)
    );
    eprintln!(
        "    \x1b[33mWarning:\x1b[0m {}",
        result
            .summary
            .issues_by_severity
            .get("warning")
            .unwrap_or(&0)
    );
    eprintln!(
        "    \x1b[34mInfo:\x1b[0m    {}",
        result.summary.issues_by_severity.get("info").unwrap_or(&0)
    );
    eprintln!(
        "  Avg complexity: {:.1}",
        result.summary.average_complexity
    );
    if let Some(max) = &result.summary.max_complexity {
        eprintln!(
            "  Max complexity: {} ({} in \x1b[36m{}\x1b[0m)",
            max.complexity, max.function, max.file
        );
    }
    eprintln!();
}

pub fn print_verbose(analysis: &FileAnalysis) {
    let issue_count = analysis.issues.len();
    let max_cc = analysis
        .complexity
        .functions
        .iter()
        .map(|f| f.complexity)
        .max()
        .unwrap_or(0);
    let max_cog = analysis
        .complexity
        .functions
        .iter()
        .map(|f| f.cognitive_complexity)
        .max()
        .unwrap_or(0);

    if issue_count == 0 {
        eprintln!(
            "  \x1b[32m✓ \x1b[36m{}\x1b[0m (complexity: {}, cognitive: {}, issues: 0)",
            analysis.file, max_cc, max_cog
        );
    } else {
        let errors = analysis
            .issues
            .iter()
            .filter(|i| matches!(i.severity, crate::types::Severity::Error))
            .count();
        let warnings = analysis
            .issues
            .iter()
            .filter(|i| matches!(i.severity, crate::types::Severity::Warning))
            .count();
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(format!(
                "{} error{}",
                errors,
                if errors > 1 { "s" } else { "" }
            ));
        }
        if warnings > 0 {
            parts.push(format!(
                "{} warning{}",
                warnings,
                if warnings > 1 { "s" } else { "" }
            ));
        }
        eprintln!(
            "  \x1b[31m✗ \x1b[36m{}\x1b[0m (complexity: {}, cognitive: {}, issues: {})",
            analysis.file,
            max_cc,
            max_cog,
            parts.join(", ")
        );
    }
}

pub fn print_hotspots(result: &HotspotResult) {
    if result.hotspots.is_empty() {
        return;
    }
    eprintln!(
        "\x1b[1m=== Hotspot Analysis (last {}) ===\x1b[0m",
        result.period
    );
    for h in &result.hotspots {
        let risk_label = match h.risk.as_str() {
            "high" => "\x1b[31mHIGH  \x1b[0m",
            "medium" => "\x1b[33mMEDIUM\x1b[0m",
            _ => "\x1b[32mLOW   \x1b[0m",
        };
        eprintln!(
            "  {} \x1b[36m{}\x1b[0m ({} commits, {} authors, complexity: {})",
            risk_label, h.file, h.commits, h.authors, h.complexity
        );
    }
    eprintln!();
}

pub fn print_baseline_comparison(comparison: &BaselineComparison) {
    let status_label = match comparison.status.as_str() {
        "improved" => "\x1b[32mIMPROVED\x1b[0m",
        "degraded" => "\x1b[31mDEGRADED\x1b[0m",
        _ => "\x1b[34mUNCHANGED\x1b[0m",
    };

    let arrow = if comparison.overall.score_delta > 0 {
        "\x1b[32m↑\x1b[0m"
    } else if comparison.overall.score_delta < 0 {
        "\x1b[31m↓\x1b[0m"
    } else {
        "→"
    };

    eprintln!("\x1b[1m=== Baseline Comparison ===\x1b[0m");
    eprintln!("  Status: {status_label}");
    eprintln!(
        "  Score:  {} → {} ({arrow} {}{}) ",
        comparison.overall.grade_before,
        comparison.overall.grade_after,
        if comparison.overall.score_delta >= 0 {
            "+"
        } else {
            ""
        },
        comparison.overall.score_delta
    );
    eprintln!(
        "  Issues: {}{}",
        if comparison.overall.issues_delta >= 0 {
            "+"
        } else {
            ""
        },
        comparison.overall.issues_delta
    );
    if !comparison.degraded_files.is_empty() {
        eprintln!("  Degraded: {}", comparison.degraded_files.join(", "));
    }
    if !comparison.improved_files.is_empty() {
        eprintln!("  Improved: {}", comparison.improved_files.join(", "));
    }
    eprintln!();
}

fn grade_colored(grade: &str) -> String {
    match grade {
        "A" => format!("\x1b[32m\x1b[1m{grade}\x1b[0m"),
        "B" => format!("\x1b[32m{grade}\x1b[0m"),
        "C" => format!("\x1b[33m{grade}\x1b[0m"),
        "D" => format!("\x1b[31m{grade}\x1b[0m"),
        "F" => format!("\x1b[31m\x1b[1m{grade}\x1b[0m"),
        _ => grade.to_string(),
    }
}
