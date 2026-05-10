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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScore {
    pub score: i32,
    pub grade: Grade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file: String,
    pub complexity: ComplexityResult,
    pub issues: Vec<Issue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<FileScore>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<ProjectScore>,
}
