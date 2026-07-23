use std::collections::HashMap;
use std::path::Path;

use rayon::prelude::*;

use crate::analyzer::analyze_file;
use crate::config::CodopsyConfig;
use crate::scorer::{calculate_file_score, calculate_project_score};
use crate::types::{
    AnalysisResult, FileAnalysis, MaxComplexityInfo, ScoringThresholds, Severity, Summary,
};

pub struct AnalyzeOptions {
    pub max_complexity: usize,
    pub max_cognitive_complexity: usize,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            max_complexity: crate::defaults::MAX_COMPLEXITY,
            max_cognitive_complexity: crate::defaults::MAX_COGNITIVE_COMPLEXITY,
        }
    }
}

/// Resolve the thresholds that will govern BOTH the `max-complexity` /
/// `max-cognitive-complexity` issue rules AND the `score_complexity`
/// component, from the same config + CLI-flag inputs. Keeping this as one
/// function is what guarantees the score can never silently diverge from a
/// project's configured tolerance again.
pub fn resolve_scoring_thresholds(config: &CodopsyConfig, opts: &AnalyzeOptions) -> ScoringThresholds {
    ScoringThresholds {
        cyclomatic_complexity: config.resolve_threshold("max-complexity", opts.max_complexity),
        cognitive_complexity: config
            .resolve_threshold("max-cognitive-complexity", opts.max_cognitive_complexity),
    }
}

fn check_max_complexity(analysis: &mut FileAnalysis, config: &CodopsyConfig, threshold: Option<usize>) {
    let Some(threshold) = threshold else { return };
    let severity = config
        .get_rule_severity("max-complexity")
        .unwrap_or(Severity::Warning);

    let file_path = analysis.file.clone();
    for func in &analysis.complexity.functions {
        if func.complexity > threshold {
            analysis.issues.push(crate::types::Issue {
                file: file_path.clone(),
                line: func.line,
                column: 1,
                severity,
                rule: "max-complexity".to_string(),
                message: format!(
                    "Function \"{}\" has a cyclomatic complexity of {} (threshold: {})",
                    func.name, func.complexity, threshold
                ),
            });
        }
    }
}

fn check_max_cognitive_complexity(
    analysis: &mut FileAnalysis,
    config: &CodopsyConfig,
    threshold: Option<usize>,
) {
    let Some(threshold) = threshold else { return };
    let severity = config
        .get_rule_severity("max-cognitive-complexity")
        .unwrap_or(Severity::Warning);

    let file_path = analysis.file.clone();
    for func in &analysis.complexity.functions {
        if func.cognitive_complexity > threshold {
            analysis.issues.push(crate::types::Issue {
                file: file_path.clone(),
                line: func.line,
                column: 1,
                severity,
                rule: "max-cognitive-complexity".to_string(),
                message: format!(
                    "Function \"{}\" has a cognitive complexity of {} (threshold: {})",
                    func.name, func.cognitive_complexity, threshold
                ),
            });
        }
    }
}

pub fn analyze_files(
    files: &[String],
    config: &CodopsyConfig,
    opts: &AnalyzeOptions,
) -> Vec<FileAnalysis> {
    let thresholds = resolve_scoring_thresholds(config, opts);
    files
        .par_iter()
        .map(|file_path| {
            let mut analysis = analyze_file(file_path, config);
            check_max_complexity(&mut analysis, config, thresholds.cyclomatic_complexity);
            check_max_cognitive_complexity(&mut analysis, config, thresholds.cognitive_complexity);
            analysis
        })
        .collect()
}

pub fn build_analysis_result(
    mut file_analyses: Vec<FileAnalysis>,
    files: &[String],
    target_dir: &str,
    thresholds: ScoringThresholds,
) -> AnalysisResult {
    let all_issues_count: usize = file_analyses.iter().map(|f| f.issues.len()).sum();

    let mut issues_by_severity = HashMap::new();
    issues_by_severity.insert("error".to_string(), 0usize);
    issues_by_severity.insert("warning".to_string(), 0usize);
    issues_by_severity.insert("info".to_string(), 0usize);

    for fa in &file_analyses {
        for issue in &fa.issues {
            let key = match issue.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            };
            *issues_by_severity.entry(key.to_string()).or_insert(0) += 1;
        }
    }

    let all_functions: Vec<_> = file_analyses
        .iter()
        .flat_map(|f| &f.complexity.functions)
        .collect();

    let avg_complexity = if !all_functions.is_empty() {
        all_functions.iter().map(|f| f.complexity as f64).sum::<f64>() / all_functions.len() as f64
    } else {
        0.0
    };

    let max_complexity = file_analyses
        .iter()
        .flat_map(|fa| {
            fa.complexity.functions.iter().map(move |func| MaxComplexityInfo {
                file: fa.file.clone(),
                function: func.name.clone(),
                complexity: func.complexity,
            })
        })
        .max_by_key(|m| m.complexity);

    // Attach per-file scores, using the SAME thresholds the issue rules above
    // were evaluated against — see `resolve_scoring_thresholds`.
    for fa in &mut file_analyses {
        fa.score = Some(calculate_file_score(fa, thresholds));
    }

    let mut result = AnalysisResult {
        timestamp: chrono::Utc::now().to_rfc3339(),
        target_dir: target_dir.to_string(),
        files: file_analyses,
        summary: Summary {
            total_files: files.len(),
            total_issues: all_issues_count,
            issues_by_severity,
            average_complexity: avg_complexity,
            max_complexity,
        },
        scoring_thresholds: thresholds,
        score: None,
    };

    let project_score = calculate_project_score(&result);
    result.score = Some(project_score);

    result
}

pub fn analyze(
    target_dir: &Path,
    files: &[String],
    config: &CodopsyConfig,
    opts: &AnalyzeOptions,
) -> AnalysisResult {
    let thresholds = resolve_scoring_thresholds(config, opts);
    let file_analyses = analyze_files(files, config, opts);
    build_analysis_result(file_analyses, files, &target_dir.to_string_lossy(), thresholds)
}
