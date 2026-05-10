use std::collections::HashMap;

use crate::defaults;
use crate::types::{
    to_grade, AnalysisResult, FileAnalysis, FileScore, Grade, ProjectScore, Severity,
};

/// Rules excluded from the "issues" scoring component
/// because they are already accounted for in complexity or structure scoring.
const EXCLUDED_FROM_ISSUES: &[&str] = &[
    "max-lines",
    "max-depth",
    "max-params",
    "max-complexity",
    "max-cognitive-complexity",
];

fn clamp_min_0(value: f64) -> f64 {
    value.max(0.0)
}

/// Complexity component (max weight: WEIGHT_COMPLEXITY = 35).
/// Penalizes functions that exceed CC/cognitive thresholds.
/// Penalty per function is capped to prevent a single outlier from dominating.
fn score_complexity(analysis: &FileAnalysis) -> f64 {
    let mut penalty = 0.0;
    for func in &analysis.complexity.functions {
        let cc_excess = (func.complexity as f64 - defaults::CC_THRESHOLD).max(0.0);
        let cog_excess = (func.cognitive_complexity as f64 - defaults::COG_THRESHOLD).max(0.0);
        penalty += (cc_excess * defaults::CC_PENALTY_RATE).min(defaults::CC_PENALTY_CAP);
        penalty += (cog_excess * defaults::COG_PENALTY_RATE).min(defaults::COG_PENALTY_CAP);
    }
    clamp_min_0(defaults::WEIGHT_COMPLEXITY - penalty)
}

/// Issues component (max weight: WEIGHT_ISSUES = 40).
/// Errors are penalized linearly; warnings use sub-linear scaling
/// (exponent 0.7) so that many minor warnings don't outweigh a few errors.
/// Info-level issues use sqrt scaling for minimal impact.
fn score_issues(analysis: &FileAnalysis) -> f64 {
    let mut rule_groups: HashMap<&str, (Severity, usize)> = HashMap::new();

    for issue in &analysis.issues {
        if EXCLUDED_FROM_ISSUES.contains(&issue.rule.as_str()) {
            continue;
        }
        let entry = rule_groups
            .entry(&issue.rule)
            .or_insert((issue.severity, 0));
        entry.1 += 1;
    }

    let mut penalty = 0.0;
    for &(severity, count) in rule_groups.values() {
        let count_f = count as f64;
        match severity {
            Severity::Error => penalty += defaults::ERROR_PENALTY * count_f,
            Severity::Warning => penalty += defaults::WARNING_PENALTY * count_f.powf(defaults::WARNING_EXPONENT),
            Severity::Info => penalty += count_f.sqrt(),
        }
    }

    clamp_min_0((defaults::WEIGHT_ISSUES - penalty).round())
}

/// Structure component (max weight: WEIGHT_STRUCTURE = 25).
/// Penalizes threshold violations (max-lines, max-depth, max-params).
fn score_structure(analysis: &FileAnalysis) -> f64 {
    let mut score = defaults::WEIGHT_STRUCTURE;
    for &(rule, per_violation, cap) in defaults::STRUCTURE_PENALTIES {
        let count = analysis
            .issues
            .iter()
            .filter(|i| i.rule == rule)
            .count() as f64;
        if count > 0.0 {
            score -= (per_violation * count).min(cap);
        }
    }
    clamp_min_0(score)
}

pub fn calculate_file_score(analysis: &FileAnalysis) -> FileScore {
    let raw = score_complexity(analysis) + score_issues(analysis) + score_structure(analysis);
    let score = raw.round() as i32;
    FileScore {
        score,
        grade: to_grade(score),
    }
}

pub fn calculate_project_score(result: &AnalysisResult) -> ProjectScore {
    if result.files.is_empty() {
        let mut distribution = HashMap::new();
        for g in ["A", "B", "C", "D", "F"] {
            distribution.insert(g.to_string(), 0);
        }
        return ProjectScore {
            overall: 100,
            grade: Grade::A,
            distribution,
        };
    }

    let file_scores: Vec<FileScore> = result.files.iter().map(|f| calculate_file_score(f)).collect();

    let mut distribution: HashMap<String, usize> = HashMap::new();
    for g in ["A", "B", "C", "D", "F"] {
        distribution.insert(g.to_string(), 0);
    }
    for fs in &file_scores {
        *distribution.entry(fs.grade.to_string()).or_default() += 1;
    }

    // Weighted average: files with more functions carry more weight (sqrt scaling).
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for (i, file) in result.files.iter().enumerate() {
        let func_count = file.complexity.functions.len() as f64;
        let weight = (func_count + 1.0).sqrt();
        weighted_sum += file_scores[i].score as f64 * weight;
        total_weight += weight;
    }

    let base_score = if total_weight > 0.0 {
        (weighted_sum / total_weight).round() as i32
    } else {
        100
    };

    // Issue density penalty: penalizes projects with many scattered issues.
    let total_issues: usize = result.files.iter().map(|f| f.issues.len()).sum();
    let density_penalty = ((total_issues as f64).sqrt() * defaults::DENSITY_PENALTY_RATE)
        .round()
        .min(defaults::DENSITY_PENALTY_CAP) as i32;

    let score = (base_score - density_penalty).max(0);

    ProjectScore {
        overall: score,
        grade: to_grade(score),
        distribution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComplexityResult, FunctionComplexity, Issue};

    fn make_analysis(functions: Vec<FunctionComplexity>, issues: Vec<Issue>) -> FileAnalysis {
        FileAnalysis {
            file: "test.ts".to_string(),
            complexity: ComplexityResult {
                cyclomatic: functions.iter().map(|f| f.complexity).max().unwrap_or(0),
                cognitive: functions.iter().map(|f| f.cognitive_complexity).max().unwrap_or(0),
                functions,
            },
            issues,
            score: None,
        }
    }

    fn make_issue(rule: &str, severity: Severity) -> Issue {
        Issue {
            file: "test.ts".to_string(),
            line: 1,
            column: 1,
            severity,
            rule: rule.to_string(),
            message: "test".to_string(),
        }
    }

    #[test]
    fn perfect_score_for_clean_file() {
        let analysis = make_analysis(
            vec![FunctionComplexity {
                name: "main".to_string(),
                line: 1,
                complexity: 1,
                cognitive_complexity: 0,
            }],
            vec![],
        );
        let score = calculate_file_score(&analysis);
        assert_eq!(score.score, 100);
        assert_eq!(score.grade, Grade::A);
    }

    #[test]
    fn high_complexity_reduces_score() {
        let analysis = make_analysis(
            vec![FunctionComplexity {
                name: "complex".to_string(),
                line: 1,
                complexity: 25,
                cognitive_complexity: 30,
            }],
            vec![],
        );
        let score = calculate_file_score(&analysis);
        assert!(score.score < 80, "Score should be reduced: {}", score.score);
    }

    #[test]
    fn errors_penalize_more_than_warnings() {
        let error_analysis = make_analysis(vec![], vec![make_issue("no-eval", Severity::Error)]);
        let warning_analysis = make_analysis(vec![], vec![make_issue("no-var", Severity::Warning)]);

        let error_score = calculate_file_score(&error_analysis);
        let warning_score = calculate_file_score(&warning_analysis);
        assert!(error_score.score < warning_score.score);
    }

    #[test]
    fn threshold_issues_excluded_from_issue_scoring() {
        let analysis = make_analysis(
            vec![],
            vec![make_issue("max-complexity", Severity::Warning)],
        );
        // max-complexity is excluded from issue scoring, only affects structure
        let score = score_issues(&analysis);
        assert_eq!(score, defaults::WEIGHT_ISSUES);
    }

    #[test]
    fn empty_project_gets_perfect_score() {
        let result = AnalysisResult {
            timestamp: "test".to_string(),
            target_dir: ".".to_string(),
            files: vec![],
            summary: crate::types::Summary {
                total_files: 0,
                total_issues: 0,
                issues_by_severity: HashMap::new(),
                average_complexity: 0.0,
                max_complexity: None,
            },
            score: None,
        };
        let project_score = calculate_project_score(&result);
        assert_eq!(project_score.overall, 100);
        assert_eq!(project_score.grade, Grade::A);
    }
}
