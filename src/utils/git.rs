use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

pub fn is_git_repository(dir: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_changed_files(dir: &Path, base: &str) -> Vec<String> {
    let merge_base = Command::new("git")
        .args(["merge-base", base, "HEAD"])
        .current_dir(dir)
        .output()
        .map_err(|e| eprintln!("Warning: git merge-base failed: {e}"))
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                eprintln!(
                    "Warning: git merge-base exited with {}: {}",
                    o.status,
                    String::from_utf8_lossy(&o.stderr).trim()
                );
                None
            }
        });

    let merge_base = match merge_base {
        Some(mb) => mb,
        None => return vec![],
    };

    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=ACMR", &merge_base])
        .current_dir(dir)
        .output()
        .map_err(|e| eprintln!("Warning: git diff failed: {e}"))
        .ok();

    let repo_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .map_err(|e| eprintln!("Warning: git rev-parse failed: {e}"))
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let (output, repo_root) = match (output, repo_root) {
        (Some(o), Some(r)) if o.status.success() => (o, r),
        _ => return vec![],
    };

    let root = Path::new(&repo_root);
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|f| root.join(f).to_string_lossy().into_owned())
        .collect()
}

#[derive(Debug, Clone)]
pub struct ChurnStats {
    pub commits: usize,
    pub authors: usize,
}

pub fn get_file_churn_stats(dir: &Path, since: &str) -> HashMap<String, ChurnStats> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("--since={since}"),
            "--format=%H %aN",
            "--name-only",
        ])
        .current_dir(dir)
        .output()
        .map_err(|e| eprintln!("Warning: git log for churn stats failed: {e}"))
        .ok();

    let output = match output {
        Some(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut stats: HashMap<String, (HashSet<String>, HashSet<String>)> = HashMap::new();
    let mut current_hash = String::new();
    let mut current_author = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Header line: 40-char hex hash + space + author name
        if trimmed.len() > 41
            && trimmed.as_bytes()[40] == b' '
            && trimmed[..40].bytes().all(|b| b.is_ascii_hexdigit())
        {
            current_hash = trimmed[..40].to_string();
            current_author = trimmed[41..].to_string();
            continue;
        }

        // File path line
        let entry = stats.entry(trimmed.to_string()).or_default();
        entry.0.insert(current_hash.clone());
        entry.1.insert(current_author.clone());
    }

    stats
        .into_iter()
        .map(|(file, (commits, authors))| {
            (
                file,
                ChurnStats {
                    commits: commits.len(),
                    authors: authors.len(),
                },
            )
        })
        .collect()
}
