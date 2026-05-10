use std::path::Path;

const DEFAULT_CONFIG: &str = r#"{
  "rules": {
    "no-any": "warning",
    "no-console": "warning",
    "no-var": "warning",
    "eqeqeq": "warning",
    "no-empty-function": "warning",
    "no-nested-ternary": "warning",
    "no-debugger": "error",
    "no-duplicate-case": "error",
    "no-self-assign": "warning",
    "no-eval": "error",
    "no-unreachable": "error",
    "no-unsafe": "warning",
    "no-unwrap": "warning",
    "no-dbg": "warning",
    "no-todo": "warning",
    "no-println": "info",
    "todo-comment": "info",
    "max-lines": { "severity": "warning", "max": 300 },
    "max-depth": { "severity": "warning", "max": 4 },
    "max-params": { "severity": "warning", "max": 4 },
    "max-complexity": { "severity": "warning", "max": 10 },
    "max-cognitive-complexity": { "severity": "warning", "max": 15 }
  }
}
"#;

pub fn init_action(target_dir: &Path, force: bool) -> anyhow::Result<()> {
    let config_path = target_dir.join(".codopsyrc.json");
    if config_path.exists() && !force {
        anyhow::bail!(
            "Config file already exists: {}. Use --force to overwrite.",
            config_path.display()
        );
    }
    std::fs::write(&config_path, DEFAULT_CONFIG)?;
    eprintln!("Created {}", config_path.display());
    Ok(())
}
