use crate::types::AnalysisResult;

pub fn format_json(result: &AnalysisResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".to_string())
}
