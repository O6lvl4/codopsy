pub mod json;

use crate::types::AnalysisResult;

pub fn format_report(result: &AnalysisResult, format: &str) -> String {
    match format {
        "json" => json::format_json(result),
        _ => json::format_json(result),
    }
}
