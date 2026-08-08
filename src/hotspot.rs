use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::defaults;
use crate::types::FileAnalysis;
use crate::utils::git::{get_file_churn_stats, repo_root};

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
    if score > defaults::HOTSPOT_RISK_HIGH {
        "high"
    } else if score > defaults::HOTSPOT_RISK_MEDIUM {
        "medium"
    } else {
        "low"
    }
}

/// Key under which `git log --name-only` reports this file: its path relative
/// to the repository root. Stripping the *analysed* directory instead would
/// silently match nothing whenever that directory is not the repo root.
fn churn_key(file: &str, root: Option<&Path>) -> String {
    let path = Path::new(file);
    root.and_then(|r| path.strip_prefix(r).ok())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| file.to_string())
}

pub fn detect_hotspots(
    target_dir: &Path,
    file_analyses: &[FileAnalysis],
    months: usize,
    top: usize,
) -> HotspotResult {
    let since = format!("{months} months ago");
    let git_stats = get_file_churn_stats(target_dir, &since);
    let root = repo_root(target_dir);

    let mut hotspots: Vec<HotspotInfo> = file_analyses
        .iter()
        .filter_map(|fa| {
            let rel_path = churn_key(&fa.file, root.as_deref());

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

            // Hotspot score: churn * (cyclomatic + weighted cognitive).
            // Cognitive is weighted at 0.5x because it often correlates with
            // cyclomatic and we want to avoid double-counting.
            let score = churn.commits as f64
                * (max_cyclomatic as f64 + max_cognitive as f64 * defaults::HOTSPOT_COGNITIVE_WEIGHT);

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

    hotspots.sort_by(|a, b| b.score.total_cmp(&a.score));
    hotspots.truncate(top);

    HotspotResult {
        period: format!("{months} months"),
        hotspots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn churn_key_is_relative_to_the_repo_root() {
        let root = Path::new("/repo");
        assert_eq!(
            churn_key("/repo/src/analyzer/linter.rs", Some(root)),
            "src/analyzer/linter.rs"
        );
    }

    #[test]
    fn churn_key_falls_back_to_the_full_path_outside_a_repo() {
        assert_eq!(churn_key("/elsewhere/a.rs", None), "/elsewhere/a.rs");
        assert_eq!(
            churn_key("/elsewhere/a.rs", Some(Path::new("/repo"))),
            "/elsewhere/a.rs"
        );
    }
}
