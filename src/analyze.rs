use std::collections::HashMap;
use std::path::Path;

use rayon::prelude::*;

use crate::analyzer::analyze_file;
use crate::config::CodopsyConfig;
use crate::scorer::{calculate_file_score, calculate_project_score};
use crate::types::{AnalysisResult, FileAnalysis, MaxComplexityInfo, Severity, Summary};

pub struct AnalyzeOptions {
    pub max_complexity: usize,
    pub max_cognitive_complexity: usize,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            max_complexity: 10,
            max_cognitive_complexity: 15,
        }
    }
}

fn check_max_complexity(
    analysis: &mut FileAnalysis,
    config: &CodopsyConfig,
    default_max: usize,
) {
    if config.is_rule_disabled("max-complexity") {
        return;
    }

    let threshold = config
        .get_rule_max("max-complexity")
        .unwrap_or(default_max);
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
    default_max: usize,
) {
    if config.is_rule_disabled("max-cognitive-complexity") {
        return;
    }

    let threshold = config
        .get_rule_max("max-cognitive-complexity")
        .unwrap_or(default_max);
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
    files
        .par_iter()
        .map(|file_path| {
            let mut analysis = analyze_file(file_path, config);
            check_max_complexity(&mut analysis, config, opts.max_complexity);
            check_max_cognitive_complexity(
                &mut analysis,
                config,
                opts.max_cognitive_complexity,
            );
            analysis
        })
        .collect()
}

pub fn build_analysis_result(
    mut file_analyses: Vec<FileAnalysis>,
    files: &[String],
    target_dir: &str,
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

    // Attach per-file scores
    for fa in &mut file_analyses {
        let fs = calculate_file_score(fa);
        fa.score = Some(crate::types::FileScore {
            score: fs.score,
            grade: fs.grade,
        });
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
    let file_analyses = analyze_files(files, config, opts);
    build_analysis_result(file_analyses, files, &target_dir.to_string_lossy())
}
