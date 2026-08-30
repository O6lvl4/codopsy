use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

pub fn to_grade(score: i32) -> Grade {
    use crate::defaults;
    if score >= defaults::GRADE_A_MIN {
        Grade::A
    } else if score >= defaults::GRADE_B_MIN {
        Grade::B
    } else if score >= defaults::GRADE_C_MIN {
        Grade::C
    } else if score >= defaults::GRADE_D_MIN {
        Grade::D
    } else {
        Grade::F
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonStatus {
    Improved,
    Degraded,
    Unchanged,
}

impl std::fmt::Display for ComparisonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Improved => write!(f, "improved"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unchanged => write!(f, "unchanged"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults;

    #[test]
    fn grade_boundaries() {
        assert_eq!(to_grade(100), Grade::A);
        assert_eq!(to_grade(defaults::GRADE_A_MIN), Grade::A);
        assert_eq!(to_grade(defaults::GRADE_A_MIN - 1), Grade::B);
        assert_eq!(to_grade(defaults::GRADE_B_MIN), Grade::B);
        assert_eq!(to_grade(defaults::GRADE_B_MIN - 1), Grade::C);
        assert_eq!(to_grade(defaults::GRADE_C_MIN), Grade::C);
        assert_eq!(to_grade(defaults::GRADE_C_MIN - 1), Grade::D);
        assert_eq!(to_grade(defaults::GRADE_D_MIN), Grade::D);
        assert_eq!(to_grade(defaults::GRADE_D_MIN - 1), Grade::F);
        assert_eq!(to_grade(0), Grade::F);
    }

    #[test]
    fn grade_display() {
        assert_eq!(Grade::A.to_string(), "A");
        assert_eq!(Grade::F.to_string(), "F");
    }

    #[test]
    fn comparison_status_display() {
        assert_eq!(ComparisonStatus::Improved.to_string(), "improved");
        assert_eq!(ComparisonStatus::Degraded.to_string(), "degraded");
        assert_eq!(ComparisonStatus::Unchanged.to_string(), "unchanged");
    }

    #[test]
    fn severity_serialization() {
        let json = serde_json::to_string(&Severity::Error).unwrap();
        assert_eq!(json, "\"error\"");
        let parsed: Severity = serde_json::from_str("\"warning\"").unwrap();
        assert_eq!(parsed, Severity::Warning);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionComplexity {
    pub name: String,
    pub line: usize,
    pub complexity: usize,
    pub cognitive_complexity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    pub cyclomatic: usize,
    pub cognitive: usize,
    pub functions: Vec<FunctionComplexity>,
}

/// The per-function complexity thresholds actually used to compute
/// `score_complexity`. Mirrors whatever governs the `max-complexity` /
/// `max-cognitive-complexity` issue rules (CLI flag or `.codopsyrc.json`),
/// so the score and the visible issues are always driven by the same numbers.
/// `None` means the corresponding rule is disabled: that dimension never
/// penalizes the score.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoringThresholds {
    pub cyclomatic_complexity: Option<usize>,
    pub cognitive_complexity: Option<usize>,
}

/// Raw points (before rounding) contributed by each of the three scoring
/// components, out of their respective weights (complexity: 35, issues: 40,
/// structure: 25). Exposed so a degraded score is explainable without
/// reverse-engineering the formula.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreBreakdown {
    pub complexity: f64,
    pub issues: f64,
    pub structure: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScore {
    pub score: i32,
    pub grade: Grade,
    pub breakdown: ScoreBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file: String,
    pub complexity: ComplexityResult,
    pub issues: Vec<Issue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<FileScore>,
    /// True when the grammar could not read enough of the file to score it.
    /// Such files are left unscored (`score: None`) and excluded from the
    /// project average and grade distribution. Omitted from JSON when false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unanalyzed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxComplexityInfo {
    pub file: String,
    pub function: String,
    pub complexity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total_files: usize,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<String, usize>,
    pub average_complexity: f64,
    pub max_complexity: Option<MaxComplexityInfo>,
    /// Files the grammar could not read enough of to score. They are counted
    /// among `total_files` but excluded from the score. Defaults to 0 so older
    /// reports still deserialize.
    #[serde(default)]
    pub unanalyzed_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScore {
    pub overall: i32,
    pub grade: Grade,
    pub distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub timestamp: String,
    pub target_dir: String,
    pub files: Vec<FileAnalysis>,
    pub summary: Summary,
    #[serde(default = "default_scoring_thresholds")]
    pub scoring_thresholds: ScoringThresholds,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<ProjectScore>,
}

fn default_scoring_thresholds() -> ScoringThresholds {
    ScoringThresholds {
        cyclomatic_complexity: Some(crate::defaults::MAX_COMPLEXITY),
        cognitive_complexity: Some(crate::defaults::MAX_COGNITIVE_COMPLEXITY),
    }
}
