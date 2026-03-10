use ignore::WalkBuilder;
use std::path::Path;

const JS_TS_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx"];
const RUST_EXTENSIONS: &[&str] = &["rs"];

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

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };

        if JS_TS_EXTENSIONS.contains(&ext) {
            // JS/TS: skip node_modules, dist, .d.ts
            if path_str.contains("node_modules")
                || path_str.contains("/dist/")
                || path_str.ends_with("/dist")
                || path_str.ends_with(".d.ts")
                || path_str.ends_with(".d.tsx")
            {
                continue;
            }
            files.push(path_str.into_owned());
        } else if RUST_EXTENSIONS.contains(&ext) {
            // Rust: skip target/ dir and build scripts
            if path_str.contains("/target/") {
                continue;
            }
            files.push(path_str.into_owned());
        }
    }

    files.sort();
    files
}
