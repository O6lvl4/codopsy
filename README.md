# codopsy

AST-level code quality analyzer for 24 languages. Uses [tree-sitter](https://tree-sitter.github.io/) to parse source code into ASTs and analyzes complexity, lint issues, and structural quality — without executing code.

## Supported Languages

| Language | Extensions | Lint Rules | Complexity |
|----------|-----------|------------|------------|
| TypeScript | `.ts` | 11 rules | CC + Cognitive |
| TSX | `.tsx` | 11 rules | CC + Cognitive |
| JavaScript | `.js` `.jsx` `.mjs` `.cjs` | 11 rules | CC + Cognitive |
| Rust | `.rs` | 6 rules | CC + Cognitive |
| Go | `.go` | threshold | CC + Cognitive |
| Python | `.py` `.pyi` | threshold | CC + Cognitive |
| C | `.c` `.h` | threshold | CC + Cognitive |
| C++ | `.cpp` `.cc` `.cxx` `.hpp` `.hxx` | threshold | CC + Cognitive |
| Java | `.java` | threshold | CC + Cognitive |
| Ruby | `.rb` | threshold | CC + Cognitive |
| C# | `.cs` | threshold | CC + Cognitive |
| PHP | `.php` | threshold | CC + Cognitive |
| Scala | `.scala` `.sc` | threshold | CC + Cognitive |
| Haskell | `.hs` | threshold | CC + Cognitive |
| Bash | `.sh` `.bash` `.zsh` | threshold | CC + Cognitive |
| HTML | `.html` `.htm` | threshold | structure |
| CSS | `.css` | threshold | structure |
| JSON | `.json` | threshold | structure |
| OCaml | `.ml` `.mli` | threshold | CC + Cognitive |
| Swift | `.swift` | threshold | CC + Cognitive |
| Lua | `.lua` | threshold | CC + Cognitive |
| Zig | `.zig` | threshold | CC + Cognitive |
| Elixir | `.ex` `.exs` | threshold | CC + Cognitive |
| YAML | `.yml` `.yaml` | threshold | structure |

## Install

```bash
cargo install --git https://github.com/O6lvl4/codopsy.git
```

## Usage

```bash
# Analyze a project
codopsy analyze ./src

# Verbose output (per-file details)
codopsy analyze ./src -v

# Output to stdout as JSON
codopsy analyze ./src -o -

# Only analyze changed files (vs main branch)
codopsy analyze ./src --diff main

# Show complexity hotspots (requires git)
codopsy analyze ./src --hotspots

# Save baseline for regression tracking
codopsy analyze ./src --save-baseline

# Fail CI if quality degrades
codopsy analyze ./src --no-degradation --fail-on-warning

# Initialize config
codopsy init
```

## Quality Score

Projects are graded A–F based on three components:

| Component | Weight | What it measures |
|-----------|--------|-----------------|
| Complexity | 35% | Average & max cyclomatic complexity |
| Issues | 40% | Lint violations per file |
| Structure | 25% | File count & function distribution |

| Grade | Score |
|-------|-------|
| A | 90–100 |
| B | 80–89 |
| C | 70–79 |
| D | 60–69 |
| F | 0–59 |

## Configuration

Create `.codopsyrc.json` in your project root (or run `codopsy init`):

```json
{
  "rules": {
    "no-console": "warning",
    "no-debugger": "error",
    "no-eval": "error",
    "max-lines": { "severity": "warning", "max": 300 },
    "max-depth": { "severity": "warning", "max": 4 },
    "max-params": { "severity": "warning", "max": 4 },
    "max-complexity": { "severity": "warning", "max": 10 },
    "max-cognitive-complexity": { "severity": "warning", "max": 15 },
    "no-println": false
  }
}
```

Rules can be set to `"warning"`, `"error"`, `"info"`, or `false` (disabled). Threshold rules accept `{ "severity": ..., "max": N }`.

Config is searched upward from the target directory to the home directory.

## Rules

### JS/TS Rules

| Rule | Default | Description |
|------|---------|-------------|
| `no-any` | warning | Disallow `any` type |
| `no-console` | warning | Disallow `console.*` calls |
| `no-var` | warning | Disallow `var` declarations |
| `eqeqeq` | warning | Require `===`/`!==` over `==`/`!=` |
| `no-empty-function` | warning | Disallow empty function bodies |
| `no-nested-ternary` | warning | Disallow nested ternary expressions |
| `no-debugger` | error | Disallow `debugger` statements |
| `no-duplicate-case` | error | Disallow duplicate switch cases |
| `no-self-assign` | warning | Disallow self-assignment |
| `no-eval` | error | Disallow `eval()` |
| `no-unreachable` | error | Detect unreachable code after return/throw |

### Rust Rules

| Rule | Default | Description |
|------|---------|-------------|
| `no-unsafe` | warning | Disallow `unsafe` blocks |
| `no-unwrap` | warning | Disallow `.unwrap()` |
| `no-dbg` | warning | Disallow `dbg!()` macro |
| `no-todo` | warning | Disallow `todo!()`/`unimplemented!()` |
| `no-println` | info | Disallow `println!()`/`print!()` etc. |
| `no-empty-function` | warning | Disallow empty function bodies |

### Threshold Rules (all languages)

| Rule | Default | Description |
|------|---------|-------------|
| `max-lines` | 300 | Maximum lines per file |
| `max-depth` | 4 | Maximum nesting depth |
| `max-params` | 4 | Maximum function parameters |
| `max-complexity` | 10 | Maximum cyclomatic complexity per function |
| `max-cognitive-complexity` | 15 | Maximum cognitive complexity per function |

## How It Works

1. **Parse**: tree-sitter converts source code into a language-agnostic AST
2. **Analyze**: Walk the AST to compute cyclomatic/cognitive complexity and detect lint violations
3. **Score**: Weighted scoring across complexity, issues, and structure
4. **Report**: JSON output with per-file and per-function details

All analysis is static — no code execution required. Files are analyzed in parallel via [rayon](https://github.com/rayon-rs/rayon).

## License

MIT
