use ignore::WalkBuilder;
use std::path::Path;

use crate::analyzer::ast_utils::get_language;
use crate::config::CodopsyConfig;

/// Path fragments that should always be skipped.
const DEFAULT_SKIP_DIRS: &[&str] = &[
    "node_modules",
    "/dist/",
    "/target/",
    "/vendor/",
    "/__pycache__/",
    "/.venv/",
    "/venv/",
    "/site-packages/",
    "/build/",
    "/bin/obj/",
    "/bundle/",
    "/.git/",
    "/generated/",
];

/// File names that should be skipped (exact suffix match).
const DEFAULT_SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "package.json",
    "tsconfig.json",
    "composer.json",
    "Cargo.lock",
    ".d.ts",
    ".d.tsx",
];

pub fn find_source_files(target_dir: &Path) -> Vec<String> {
    find_source_files_with_config(target_dir, &CodopsyConfig::default())
}

pub fn find_source_files_with_config(target_dir: &Path, config: &CodopsyConfig) -> Vec<String> {
    let extra_skip_dirs: Vec<&str> = config
        .skip_dirs
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let extra_skip_files: Vec<&str> = config
        .skip_files
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default();

    let mut files = Vec::new();

    let walker = WalkBuilder::new(target_dir)
        .hidden(false)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();
        let path_str = path.to_string_lossy();

        if get_language(&path_str).is_none() {
            continue;
        }

        if should_skip(&path_str, &extra_skip_dirs, &extra_skip_files) {
            continue;
        }

        files.push(path_str.into_owned());
    }

    files.sort();
    files
}

fn should_skip(path: &str, extra_dirs: &[&str], extra_files: &[&str]) -> bool {
    DEFAULT_SKIP_DIRS.iter().any(|d| path.contains(d))
        || extra_dirs.iter().any(|d| path.contains(d))
        || DEFAULT_SKIP_FILES.iter().any(|f| path.ends_with(f))
        || extra_files.iter().any(|f| path.ends_with(f))
        || path.ends_with("/dist")
}
