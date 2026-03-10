use ignore::WalkBuilder;
use std::path::Path;

use crate::analyzer::ast_utils::get_language;

/// Path fragments that should always be skipped.
const SKIP_DIRS: &[&str] = &[
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
const SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "package.json",
    "tsconfig.json",
    "composer.json",
    "Cargo.lock",
    ".d.ts",
    ".d.tsx",
];

pub fn find_source_files(target_dir: &Path) -> Vec<String> {
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

        if should_skip(&path_str) {
            continue;
        }

        files.push(path_str.into_owned());
    }

    files.sort();
    files
}

fn should_skip(path: &str) -> bool {
    SKIP_DIRS.iter().any(|d| path.contains(d))
        || SKIP_FILES.iter().any(|f| path.ends_with(f))
        || path.ends_with("/dist")
}
