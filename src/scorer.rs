use std::collections::HashMap;

use crate::defaults;
use crate::types::{
    to_grade, AnalysisResult, FileAnalysis, FileScore, Grade, ProjectScore, ScoreBreakdown,
    ScoringThresholds, Severity,
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
/// Penalizes functions that exceed the CC/cognitive thresholds — the SAME
/// thresholds that govern whether `max-complexity` / `max-cognitive-complexity`
/// issues are emitted (see `analyze::resolve_scoring_thresholds`). A rule with
/// no threshold (disabled) never penalizes that dimension.
/// Penalty per function is capped to prevent a single outlier from dominating.
fn score_complexity(analysis: &FileAnalysis, thresholds: ScoringThresholds) -> f64 {
    let mut penalty = 0.0;
    for func in &analysis.complexity.functions {
        if let Some(cc_threshold) = thresholds.cyclomatic_complexity {
            let cc_excess = (func.complexity as f64 - cc_threshold as f64).max(0.0);
            penalty += (cc_excess * defaults::CC_PENALTY_RATE).min(defaults::CC_PENALTY_CAP);
        }
        if let Some(cog_threshold) = thresholds.cognitive_complexity {
            let cog_excess = (func.cognitive_complexity as f64 - cog_threshold as f64).max(0.0);
            penalty += (cog_excess * defaults::COG_PENALTY_RATE).min(defaults::COG_PENALTY_CAP);
        }
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

pub fn calculate_file_score(analysis: &FileAnalysis, thresholds: ScoringThresholds) -> FileScore {
    let complexity = score_complexity(analysis, thresholds);
    let issues = score_issues(analysis);
    let structure = score_structure(analysis);
    let score = (complexity + issues + structure).round() as i32;
    FileScore {
        score,
        grade: to_grade(score),
        breakdown: ScoreBreakdown {
            complexity,
            issues,
            structure,
        },
    }
}

/// Aggregates the project score from each file's ALREADY-COMPUTED `score`
/// (set by `analyze::build_analysis_result` before this runs) rather than
/// recomputing it, so there is exactly one place — `calculate_file_score` —
/// that turns thresholds + analysis into a score.
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

    let mut distribution: HashMap<String, usize> = HashMap::new();
    for g in ["A", "B", "C", "D", "F"] {
        distribution.insert(g.to_string(), 0);
    }

    // Weighted average: files with more functions carry more weight (sqrt scaling).
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for file in &result.files {
        let Some(fs) = &file.score else {
            continue;
        };
        *distribution.entry(fs.grade.to_string()).or_default() += 1;
        let func_count = file.complexity.functions.len() as f64;
        let weight = (func_count + 1.0).sqrt();
        weighted_sum += fs.score as f64 * weight;
        total_weight += weight;
    }

    let base_score = if total_weight > 0.0 {
        (weighted_sum / total_weight).round() as i32
    } else {
        100
    };

    // Issue density penalty: penalizes projects with many scattered issues.
    // Unanalyzed files are excluded — their issues (a lone `syntax-error`, plus
    // whatever noise a shredded tree produced) are not reliable signal, and the
    // file is already kept out of the average.
    let total_issues: usize = result
        .files
        .iter()
        .filter(|f| !f.unanalyzed)
        .map(|f| f.issues.len())
        .sum();
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
            unanalyzed: false,
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

    /// The historical hardcoded thresholds (10/15), for tests that don't
    /// care about threshold configuration itself.
    fn default_thresholds() -> ScoringThresholds {
        ScoringThresholds {
            cyclomatic_complexity: Some(10),
            cognitive_complexity: Some(15),
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
        let score = calculate_file_score(&analysis, default_thresholds());
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
        let score = calculate_file_score(&analysis, default_thresholds());
        assert!(score.score < 80, "Score should be reduced: {}", score.score);
    }

    #[test]
    fn errors_penalize_more_than_warnings() {
        let error_analysis = make_analysis(vec![], vec![make_issue("no-eval", Severity::Error)]);
        let warning_analysis = make_analysis(vec![], vec![make_issue("no-var", Severity::Warning)]);

        let error_score = calculate_file_score(&error_analysis, default_thresholds());
        let warning_score = calculate_file_score(&warning_analysis, default_thresholds());
        assert!(error_score.score < warning_score.score);
    }

    #[test]
    fn score_breakdown_sums_to_score() {
        let analysis = make_analysis(
            vec![FunctionComplexity {
                name: "complex".to_string(),
                line: 1,
                complexity: 25,
                cognitive_complexity: 30,
            }],
            vec![make_issue("no-var", Severity::Warning)],
        );
        let fs = calculate_file_score(&analysis, default_thresholds());
        let sum = fs.breakdown.complexity + fs.breakdown.issues + fs.breakdown.structure;
        assert_eq!(sum.round() as i32, fs.score);
    }

    /// The bug this whole module exists to prevent: relaxing the configured
    /// complexity threshold (e.g. `.codopsyrc.json`'s `max-complexity.max`)
    /// MUST move `score_complexity` too — it must never keep silently
    /// penalizing at the old, tighter default once the project has opted
    /// into a looser bar.
    #[test]
    fn relaxed_threshold_improves_complexity_score() {
        let analysis = make_analysis(
            vec![FunctionComplexity {
                name: "mid".to_string(),
                line: 1,
                complexity: 18,
                cognitive_complexity: 25,
            }],
            vec![],
        );
        let strict = score_complexity(&analysis, default_thresholds());
        let relaxed = score_complexity(
            &analysis,
            ScoringThresholds {
                cyclomatic_complexity: Some(20),
                cognitive_complexity: Some(30),
            },
        );
        assert!(
            relaxed > strict,
            "relaxed threshold should score higher: strict={strict} relaxed={relaxed}"
        );
        assert_eq!(relaxed, defaults::WEIGHT_COMPLEXITY, "under the relaxed threshold this function has no excess at all");
    }

    #[test]
    fn disabled_complexity_rule_never_penalizes() {
        let analysis = make_analysis(
            vec![FunctionComplexity {
                name: "huge".to_string(),
                line: 1,
                complexity: 500,
                cognitive_complexity: 500,
            }],
            vec![],
        );
        let score = score_complexity(
            &analysis,
            ScoringThresholds {
                cyclomatic_complexity: None,
                cognitive_complexity: None,
            },
        );
        assert_eq!(score, defaults::WEIGHT_COMPLEXITY);
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

    /// The bug Stage 0 exists to prevent: a file the grammar could not read
    /// yields no functions and no scored issues, which used to score ~A(99) and
    /// count as a clean file. An unanalyzed file must be left unscored and kept
    /// out of both the weighted average and the grade distribution.
    #[test]
    fn unanalyzed_file_is_excluded_from_project_score() {
        let clean = FileAnalysis {
            score: Some(FileScore {
                score: 100,
                grade: Grade::A,
                breakdown: ScoreBreakdown {
                    complexity: 35.0,
                    issues: 40.0,
                    structure: 25.0,
                },
            }),
            ..make_analysis(
                vec![FunctionComplexity {
                    name: "f".to_string(),
                    line: 1,
                    complexity: 1,
                    cognitive_complexity: 0,
                }],
                vec![],
            )
        };
        // A file the grammar could not read: unscored, flagged unanalyzed,
        // carrying only the informational syntax-error.
        let unreadable = FileAnalysis {
            unanalyzed: true,
            ..make_analysis(vec![], vec![make_issue("syntax-error", Severity::Info)])
        };

        let result = AnalysisResult {
            timestamp: "test".to_string(),
            target_dir: ".".to_string(),
            files: vec![clean, unreadable],
            summary: crate::types::Summary {
                total_files: 2,
                total_issues: 1,
                issues_by_severity: HashMap::new(),
                average_complexity: 0.0,
                max_complexity: None,
                unanalyzed_files: 1,
            },
            scoring_thresholds: default_thresholds(),
            score: None,
        };

        let project = calculate_project_score(&result);
        // Only the one clean file counts toward the distribution.
        assert_eq!(project.distribution.get("A"), Some(&1));
        assert_eq!(
            project.distribution.values().sum::<usize>(),
            1,
            "the unanalyzed file must not appear in any grade bucket"
        );
        // And it does not drag the average up as a phantom A.
        assert_eq!(project.overall, 100);
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
                unanalyzed_files: 0,
            },
            scoring_thresholds: default_thresholds(),
            score: None,
        };
        let project_score = calculate_project_score(&result);
        assert_eq!(project_score.overall, 100);
        assert_eq!(project_score.grade, Grade::A);
    }
}
