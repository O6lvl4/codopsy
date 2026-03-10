use std::collections::HashMap;

use crate::types::{
    to_grade, AnalysisResult, FileAnalysis, FileScore, Grade, ProjectScore, Severity,
};

const STRUCTURE_RULES: &[(&str, f64, f64)] = &[
    ("max-lines", 10.0, 12.0),
    ("max-depth", 4.0, 12.0),
    ("max-params", 3.0, 10.0),
];

const EXCLUDED_FROM_ISSUES: &[&str] = &[
    "max-lines",
    "max-depth",
    "max-params",
    "max-complexity",
    "max-cognitive-complexity",
];

fn clamp_min_0(value: f64) -> f64 {
    if value < 0.0 { 0.0 } else { value }
}

fn score_complexity(analysis: &FileAnalysis) -> f64 {
    let mut penalty = 0.0;
    for func in &analysis.complexity.functions {
        let cc_excess = (func.complexity as f64 - 10.0).max(0.0);
        let cog_excess = (func.cognitive_complexity as f64 - 15.0).max(0.0);
        penalty += (cc_excess * 2.0).min(15.0);
        penalty += (cog_excess * 1.5).min(12.0);
    }
    clamp_min_0(35.0 - penalty)
}

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
            Severity::Error => penalty += 8.0 * count_f,
            Severity::Warning => penalty += 4.0 * count_f.powf(0.7),
            Severity::Info => penalty += count_f.sqrt(),
        }
    }

    clamp_min_0((40.0 - penalty).round())
}

fn score_structure(analysis: &FileAnalysis) -> f64 {
    let mut score = 25.0;
    for &(rule, per_violation, cap) in STRUCTURE_RULES {
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

    // Issue density penalty
    let total_issues: usize = result.files.iter().map(|f| f.issues.len()).sum();
    let density_penalty = ((total_issues as f64).sqrt() * 0.8).round().min(15.0) as i32;

    let score = (base_score - density_penalty).max(0);

    ProjectScore {
        overall: score,
        grade: to_grade(score),
        distribution,
    }
}
