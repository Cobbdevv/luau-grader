<p align="center">
  <img src="assets/logo.png" width="120" alt="Luau Grader logo">
</p>

<h1 align="center">Luau Auto-Grader</h1>

<p align="center">
  <a href="https://github.com/Cobbdevv/luau-grader/actions/workflows/ci.yml"><img src="https://github.com/Cobbdevv/luau-grader/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI"></a>
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/badge/platform-Windows-blue" alt="Windows">
  <img src="https://img.shields.io/github/license/Cobbdevv/luau-grader" alt="License">
</p>

A high-performance desktop application and CLI for grading Luau code against professional development standards. Built with Rust and Tauri.

The grading engine performs static analysis on Luau source code using AST parsing, checking against a curated ruleset derived from production-grade Roblox development practices. Rules are organized into four tiers (Beginner through Front Page) and seven categories, giving developers clear, actionable feedback on their code quality.

---

## Features

- **AST-based analysis** — Uses full_moon to parse Luau into a complete syntax tree. No regex hacks.
- **Tier system** — Four tiers from Beginner to Front Page. Each higher tier includes all rules below it.
- **Auto-fix** — 9 rules support automatic fixes. Apply one-click fixes in the GUI or run `luau-grader fix` from the CLI.
- **CLI mode** — Grade files, fix code, and scan directories from the command line. CI-friendly exit codes.
- **Multi-file analysis** — Grade entire directories of `.luau`/`.lua` files with aggregate reporting.
- **Custom rules** — Define your own rules via `.luaugraderrc` using pattern matching (function calls, method calls).
- **Configurable** — Per-rule parameters, severity overrides, and disabled rules via `.luaugraderrc` JSON config.
- **Letter grading** — A+ through F scoring with per-issue point deductions.
- **Syntax highlighting** — Built-in Luau tokenizer with keyword, string, number, comment, and builtin coloring.
- **Diagnostic line markers** — Issue lines highlighted directly in the editor.
- **Click-to-jump** — Click any diagnostic card to scroll to the offending line.
- **Settings sidebar** — Toggle individual rules on/off, grouped by category.

---

## Rules

### Beginner (5 rules)

| ID | Category | Description | Auto-fix |
|:---|:---|:---|:---:|
| B001 | Performance | Deprecated `wait()` usage | ✓ |
| B002 | Performance | Deprecated `spawn()` usage | ✓ |
| B003 | Performance | Deprecated `delay()` usage | ✓ |
| B004 | Networking | `InvokeClient` deadlock risk | |
| B005 | Common Bugs | `WaitForChild()` without timeout | ✓ |

### Intermediate (3 rules)

| ID | Category | Description | Auto-fix |
|:---|:---|:---|:---:|
| I001 | Code Style | Function exceeds line limit (configurable) | |
| I002 | Code Style | Single-letter variable names (configurable exceptions) | |
| I003 | Performance | `GetService()` inside loops | ✓ |

### Advanced (4 rules)

| ID | Category | Description | Auto-fix |
|:---|:---|:---|:---:|
| A001 | Performance | `Instance.new()` inside loops | |
| A002 | Memory Management | Connection not stored for cleanup | ✓ |
| A003 | Performance | String concatenation in loops | |
| A004 | Data Persistence | `SetAsync` instead of `UpdateAsync` | ✓ |

### Front Page (3 rules)

| ID | Category | Description | Auto-fix |
|:---|:---|:---|:---:|
| F001 | Code Style | Missing `--!strict` directive | ✓ |
| F002 | Memory Management | `Parent = nil` without `:Destroy()` | ✓ |
| F003 | Module Architecture | `require()` inside loops | |

---

## CLI Usage

```bash
# Grade a single file
luau-grader check script.luau --tier front_page

# Grade a directory recursively
luau-grader check-dir src/ --recursive --tier advanced

# Auto-fix a file (preview first with --dry-run)
luau-grader fix script.luau --dry-run
luau-grader fix script.luau

# JSON output for CI pipelines
luau-grader check script.luau --format json

# List all available rules
luau-grader list-rules
```

Exit codes: `0` = clean, `1` = warnings only, `2` = errors found.

---

## Configuration (`.luaugraderrc`)

Create a `.luaugraderrc` file in your project root. The CLI auto-discovers it by walking up parent directories.

```json
{
    "tier": "advanced",
    "disabled_rules": ["I002"],
    "severity_overrides": {
        "B001": "Error"
    },
    "params": {
        "I001": { "max_lines": 40 },
        "I002": { "exceptions": ["i", "j", "k", "v", "_"] },
        "B005": { "default_timeout": 10 }
    },
    "custom_rules": [
        {
            "id": "C001",
            "description": "No print() in production code",
            "severity": "Warning",
            "category": "Code Hygiene",
            "tier": "Beginner",
            "pattern": { "type": "function_call", "name": "print" },
            "message": "remove print() calls before shipping",
            "suggestion": "use a logging module instead"
        }
    ]
}
```

### Custom Rule Patterns

| Pattern Type | Matches | Example |
|:---|:---|:---|
| `function_call` | Global function calls | `print()`, `warn()`, `error()` |
| `method_call` | Method calls on any object | `:Clone()`, `:Destroy()`, `:Fire()` |

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, MSVC toolchain on Windows)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2 on Windows)

### Desktop App

```bash
git clone https://github.com/Cobb-Dev/luau-grader.git
cd luau-grader
cargo install tauri-cli --version "^2"
cargo tauri build
```

### CLI Only

```bash
cargo build --release --package luau-grader-cli
```

The binary will be at `target/release/luau-grader-cli.exe`.

### Development

```bash
cargo tauri dev
```

### Running Tests

```bash
cargo test --workspace
```

---

## Project Structure

```
luau-grader/
  Cargo.toml                # Workspace manifest
  luau-grader-core/          # Grading engine library
    src/
      analyzer/              # AST walker and analysis context
      rulesets/               # Rule implementations by tier
      config.rs              # Tier enum
      report.rs              # Diagnostic and Report types
      errors.rs              # Error types
      fixer.rs               # Auto-fix engine
      batch.rs               # Multi-file analysis
      ruleset_config.rs      # .luaugraderrc config parsing
      lib.rs                 # Library root
    tests/
      rules.rs               # Integration tests (39 tests)
  luau-grader-cli/           # Standalone CLI binary
    src/
      main.rs                # check, check-dir, fix, list-rules
  src-tauri/                 # Tauri desktop application
    src/
      lib.rs                 # Tauri commands
      main.rs                # Desktop entry point
    tauri.conf.json
  src/                       # Frontend (HTML/CSS/JS)
    index.html
    styles.css
    main.js
```

---

## Adding Rules

See [CONTRIBUTING.md](CONTRIBUTING.md) for a guide on implementing new rules using the `Rule` trait.

---

## License

MIT License. See [LICENSE](LICENSE).

---

Built by Cobb_Dev.