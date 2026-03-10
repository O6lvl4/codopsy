use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::types::AnalysisResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineEntry {
    pub file: String,
    pub issue_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub max_cyclomatic: usize,
    pub max_cognitive: usize,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineOverall {
    #[serde(rename = "totalIssues")]
    pub total_issues: usize,
    #[serde(rename = "totalErrors")]
    pub total_errors: usize,
    #[serde(rename = "totalWarnings")]
    pub total_warnings: usize,
    #[serde(rename = "averageComplexity")]
    pub average_complexity: f64,
    pub score: i32,
    pub grade: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub version: u32,
    pub timestamp: String,
    pub overall: BaselineOverall,
    pub files: Vec<BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineComparison {
    pub status: String, // "improved" | "degraded" | "unchanged"
    pub overall: BaselineOverallComparison,
    pub new_files: usize,
    pub removed_files: usize,
    pub degraded_files: Vec<String>,
    pub improved_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineOverallComparison {
    pub issues_delta: i64,
    pub score_delta: i64,
    pub grade_before: String,
    pub grade_after: String,
}

pub fn create_baseline(result: &AnalysisResult) -> Baseline {
    let target_dir = Path::new(&result.target_dir);

    let mut files: Vec<BaselineEntry> = result
        .files
        .iter()
        .map(|fa| {
            let rel_path = Path::new(&fa.file)
                .strip_prefix(target_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| fa.file.clone());

            let errors = fa
                .issues
                .iter()
                .filter(|i| matches!(i.severity, crate::types::Severity::Error))
                .count();
            let warnings = fa
                .issues
                .iter()
                .filter(|i| matches!(i.severity, crate::types::Severity::Warning))
                .count();
            let max_cyclomatic = fa
                .complexity
                .functions
                .iter()
                .map(|f| f.complexity)
                .max()
                .unwrap_or(0);
            let max_cognitive = fa
                .complexity
                .functions
                .iter()
                .map(|f| f.cognitive_complexity)
                .max()
                .unwrap_or(0);

            BaselineEntry {
                file: rel_path,
                issue_count: fa.issues.len(),
                error_count: errors,
                warning_count: warnings,
                max_cyclomatic,
                max_cognitive,
                score: fa.score.as_ref().map(|s| s.score).unwrap_or(100),
            }
        })
        .collect();

    files.sort_by(|a, b| a.file.cmp(&b.file));

    Baseline {
        version: 1,
        timestamp: result.timestamp.clone(),
        overall: BaselineOverall {
            total_issues: result.summary.total_issues,
            total_errors: *result
                .summary
                .issues_by_severity
                .get("error")
                .unwrap_or(&0),
            total_warnings: *result
                .summary
                .issues_by_severity
                .get("warning")
                .unwrap_or(&0),
            average_complexity: (result.summary.average_complexity * 10.0).round() / 10.0,
            score: result.score.as_ref().map(|s| s.overall).unwrap_or(100),
            grade: result
                .score
                .as_ref()
                .map(|s| s.grade.to_string())
                .unwrap_or_else(|| "A".to_string()),
        },
        files,
    }
}

pub fn save_baseline(result: &AnalysisResult, output_path: &Path) -> anyhow::Result<()> {
    let baseline = create_baseline(result);
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(&baseline)?;
    std::fs::write(output_path, format!("{json}\n"))?;
    Ok(())
}

pub fn load_baseline(path: &Path) -> Option<Baseline> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn compare_with_baseline(result: &AnalysisResult, baseline: &Baseline) -> BaselineComparison {
    let current = create_baseline(result);

    let base_map: HashMap<&str, &BaselineEntry> =
        baseline.files.iter().map(|f| (f.file.as_str(), f)).collect();
    let current_map: HashMap<&str, &BaselineEntry> =
        current.files.iter().map(|f| (f.file.as_str(), f)).collect();

    let mut new_files = 0usize;
    let mut degraded_files = Vec::new();
    let mut improved_files = Vec::new();

    for (file, cur) in &current_map {
        if let Some(base) = base_map.get(file) {
            if cur.score < base.score {
                degraded_files.push(file.to_string());
            }
            if cur.score > base.score {
                improved_files.push(file.to_string());
            }
        } else {
            new_files += 1;
        }
    }

    let removed_files = base_map.keys().filter(|f| !current_map.contains_key(*f)).count();

    let score_delta = current.overall.score as i64 - baseline.overall.score as i64;
    let issues_delta = current.overall.total_issues as i64 - baseline.overall.total_issues as i64;

    let status = if score_delta > 0 || issues_delta < 0 {
        "improved"
    } else if score_delta < 0 || issues_delta > 0 {
        "degraded"
    } else {
        "unchanged"
    };

    BaselineComparison {
        status: status.to_string(),
        overall: BaselineOverallComparison {
            issues_delta,
            score_delta,
            grade_before: baseline.overall.grade.clone(),
            grade_after: current.overall.grade.clone(),
        },
        new_files,
        removed_files,
        degraded_files,
        improved_files,
    }
}
