use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::types::FileAnalysis;
use crate::utils::git::get_file_churn_stats;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotspotInfo {
    pub file: String,
    pub commits: usize,
    pub authors: usize,
    pub complexity: usize,
    pub cognitive_complexity: usize,
    pub score: f64,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotResult {
    pub period: String,
    pub hotspots: Vec<HotspotInfo>,
}

fn classify_risk(score: f64) -> &'static str {
    if score > 100.0 {
        "high"
    } else if score > 30.0 {
        "medium"
    } else {
        "low"
    }
}

pub fn detect_hotspots(
    target_dir: &Path,
    file_analyses: &[FileAnalysis],
    months: usize,
    top: usize,
) -> HotspotResult {
    let since = format!("{months} months ago");
    let git_stats = get_file_churn_stats(target_dir, &since);

    let mut hotspots: Vec<HotspotInfo> = file_analyses
        .iter()
        .filter_map(|fa| {
            let rel_path = Path::new(&fa.file)
                .strip_prefix(target_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| fa.file.clone());

            let churn = git_stats.get(&rel_path)?;
            if churn.commits == 0 {
                return None;
            }

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

            let score =
                churn.commits as f64 * (max_cyclomatic as f64 + max_cognitive as f64 * 0.5);

            Some(HotspotInfo {
                file: rel_path,
                commits: churn.commits,
                authors: churn.authors,
                complexity: max_cyclomatic,
                cognitive_complexity: max_cognitive,
                score,
                risk: classify_risk(score).to_string(),
            })
        })
        .collect();

    hotspots.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hotspots.truncate(top);

    HotspotResult {
        period: format!("{months} months"),
        hotspots,
    }
}
