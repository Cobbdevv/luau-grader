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

Luau Auto-Grader is a fast desktop app and CLI that helps you grade your Luau code against professional Roblox development standards. It's built with Rust and Tauri to be lightning fast.

Under the hood, our grading engine uses AST parsing to thoroughly analyze your Luau code without relying on fragile regex. It checks your code against a carefully curated set of rules based on actual production-grade Roblox development practices. We've organized these rules into four tiers (from Beginner to Front Page) and seven categories to give you clear, actionable feedback on how to improve your code quality.

---

## Features

- **AST-based analysis**: Uses full_moon to parse your Luau code into a real syntax tree. No messy regex hacks here!
- **Tier system**: We have four tiers starting from Beginner up to Front Page. As you move up a tier, it automatically includes all the rules from the ones below it.
- **Auto-fix support**: We've got 9 rules that support automatic fixes. You can easily apply them with a single click in the GUI or by running `luau-grader fix` in the CLI.
- **CLI mode**: Perfect for your command line workflow! Grade files, fix code, and scan whole directories easily. It even outputs CI-friendly exit codes.
- **Multi-file analysis**: Want to grade a whole project? You can analyze entire directories of `.luau` or `.lua` files and get a nice aggregate report.
- **Custom rules**: Need something specific? You can define your very own rules using `.luaugraderrc` with simple pattern matching for function and method calls.
- **Highly configurable**: You can tweak rule parameters, override severity levels, or disable rules completely using a `.luaugraderrc` JSON file.
- **Letter grading**: We score your code from A+ down to F, with specific point deductions for each issue found.
- **Syntax highlighting**: Your code looks great in our editor thanks to a built-in Luau tokenizer that colors keywords, strings, numbers, comments, and built-ins.
- **Diagnostic line markers**: Any issues we find are highlighted right on the lines in the editor.
- **Click-to-jump**: Just click on any diagnostic card and we'll automatically scroll you right to the line with the issue.
- **Settings sidebar**: Easily toggle individual rules on or off, neatly grouped by category.

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