<p align="center">
  <img src="assets/logo.png" width="120" alt="Luau Grader logo">
</p>

<h1 align="center">Luau Grader</h1>

<p align="center">
  <a href="https://github.com/Cobbdevv/luau-grader/actions/workflows/ci.yml"><img src="https://github.com/Cobbdevv/luau-grader/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI"></a>
  <img src="https://img.shields.io/badge/rules-77-blue" alt="77 Rules">
  <img src="https://img.shields.io/badge/tests-136-brightgreen" alt="136 Tests">
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Rust">
  <img src="https://img.shields.io/github/license/Cobbdevv/luau-grader" alt="License">
</p>

Luau Grader is a professional-grade static analysis and grading engine for Luau code. It uses AST parsing to thoroughly analyze your code against 77 rules across 5 tiers, then scores it across 7 dimensions to produce a letter grade, technical debt estimate, and prioritized improvement path.

Built with Rust and Tauri. Ships as both a CLI tool and a desktop application.

---

## Features

- **77 rules** across Beginner, Intermediate, Advanced, Front Page, and Security tiers
- **7-dimensional grading** evaluating Structure, API Correctness, Error Handling, Performance, Readability, Safety, and Security
- **Per-function grades** with cyclomatic and cognitive complexity analysis
- **Technical debt estimation** in minutes with category breakdown
- **Improvement projection** showing exactly what to fix to raise your grade
- **Script type detection** identifying ServerScript, ClientScript, ModuleScript, SharedModule, and Plugin
- **Pattern recognition** detecting debounce, cooldown, data persistence, cleanup, observer, and module patterns
- **11 auto-fixable rules** applied with a single command or button click
- **Custom rules** defined via `.luaugraderrc` with function and method call pattern matching
- **AST-based analysis** using full_moon for accurate Luau parsing

---

## CLI Usage

```bash
luau-grader grade script.luau
luau-grader grade script.luau --format json

luau-grader check script.luau --tier advanced
luau-grader check-dir src/ --recursive --tier front_page

luau-grader fix script.luau --dry-run
luau-grader fix script.luau

luau-grader list-rules
```

Exit codes: `0` = clean, `1` = warnings only, `2` = errors found.

### Grade Report

```
  script.luau - ServerScript

  Grade: B+ (82/100)

  Dimensions
    Structure            ████████░░  78
    API Correctness      ██████████  95
    Error Handling       ██████░░░░  60
    Performance          █████████░  88
    Readability          ████████░░  75
    Safety               █████████░  90
    Security             ██████████ 100

  Per-Function Grades
    onPlayerAdded (line 12) B   complexity:8  lines:34
    saveData (line 48)      A-  complexity:4  lines:18

  Technical Debt: 45 minutes
    Error Handling     25 min  (3 issues)
    Performance        15 min  (2 issues)

  Improvement Path B+ -> A-
    1. Add pcall around DataStore calls  +8 pts ~15 min
    2. Cache FindFirstChild results      +4 pts ~5 min

  Strengths
    + Uses --!strict mode
    + Clean parameter signatures
```

---

## Rules

### Beginner (12 rules)

| ID | Category | Description | Fix |
|:---|:---|:---|:---:|
| B001 | Performance | Deprecated `wait()` usage | Y |
| B002 | Performance | Deprecated `spawn()` usage | Y |
| B003 | Performance | Deprecated `delay()` usage | Y |
| B004 | Networking | `InvokeClient` deadlock risk | |
| B005 | Common Bugs | `WaitForChild()` without timeout | Y |
| B006 | Common Bugs | `Instance.new()` with parent argument | |
| B007 | API Deprecation | Deprecated lowercase method aliases | Y |
| B008 | API Deprecation | Deprecated `table.foreach/foreachi/getn` | |
| B009 | Code Style | `game.Workspace` instead of `workspace` | Y |
| B010 | Common Bugs | Constructor argument count validation | |
| B011 | Common Bugs | Method argument count validation | |
| B012 | Common Bugs | Standard library argument count validation | |

### Intermediate (22 rules)

| ID | Category | Description | Fix |
|:---|:---|:---|:---:|
| I001 | Code Style | Function exceeds line limit | |
| I002 | Code Style | Single-letter variable names | |
| I003 | Performance | `GetService()` inside loops | Y |
| I004 | Common Bugs | Numeric for loop wrong step direction | |
| I005 | Code Quality | Empty if block | |
| I006 | Code Quality | Redundant `tostring` on string | |
| I007 | Code Quality | Redundant `tonumber` on number | |
| I008 | Code Style | Deep nesting exceeds threshold | |
| I009 | Code Hygiene | Debug `print`/`warn` left in code | |
| I010 | Common Bugs | `table.sort()` result assigned (returns nil) | |
| I011 | Common Bugs | `type()` vs `typeof()` for Roblox types | |
| I012 | Common Bugs | `Color3.new()` with values > 1 | |
| I014 | Code Quality | Self-assignment (`x = x`) | |
| I016 | Code Quality | Empty function body | |
| I017 | Code Quality | Duplicate key in table constructor | |
| I018 | Code Quality | `#` length operator on dictionary table | |
| I019 | Performance | `while wait() do` anti-pattern | Y |
| I020 | Code Quality | Explicit nil comparison simplification | |
| I021 | Code Quality | Negated if condition with else block | |
| I022 | Code Quality | Comparison with `math.huge` | |
| I024 | Code Quality | Inconsistent return values across paths | |
| I025 | Code Quality | Magic numbers without named constants | |

### Advanced (20 rules)

| ID | Category | Description | Fix |
|:---|:---|:---|:---:|
| A001 | Performance | `Instance.new()` inside loops | |
| A002 | Memory Management | Connection not stored for cleanup | |
| A003 | Performance | String concatenation in loops | |
| A004 | Data Persistence | `SetAsync` instead of `UpdateAsync` | Y |
| A005 | Performance | `while true do` without yield | |
| A006 | Performance | `table.insert` at position 1 in loop | |
| A007 | Performance | `:Connect()` inside loop | |
| A008 | Error Handling | `pcall`/`xpcall` result not checked | |
| A009 | Memory Management | `Clone()` result not stored | |
| A010 | API Deprecation | Deprecated `LoadAnimation` on Humanoid | |
| A011 | API Deprecation | Deprecated `SetPrimaryPartCFrame` | |
| A012 | API Deprecation | Deprecated `GetMouse` | |
| A013 | Code Quality | Unreachable code after return/break | |
| A014 | Common Bugs | `table.remove` in forward loop | |
| A017 | API Deprecation | Deprecated `tick()` | Y |
| A018 | API Deprecation | Deprecated `TweenPosition`/`TweenSize` | |
| A019 | Common Bugs | `Debris:AddItem()` with zero/negative lifetime | |
| A020 | Common Bugs | `string.format` specifier/argument mismatch | |
| A021 | Performance | `FindFirstChild` inside loop | |
| A022 | Code Quality | Global writes without `local` keyword | |

### Front Page (15 rules)

| ID | Category | Description | Fix |
|:---|:---|:---|:---:|
| F001 | Code Style | Missing `--!strict` directive | Y |
| F002 | Memory Management | `Parent = nil` without `:Destroy()` | |
| F003 | Module Architecture | `require()` inside loops | |
| F004 | Code Style | `GetService("Workspace")` instead of `workspace` | |
| F005 | Error Handling | `FindFirstChild` result chained without nil check | |
| F006 | API Deprecation | Deprecated `:Remove()` | |
| F007 | Common Bugs | `string.sub()` with index 0 | |
| F008 | Common Bugs | `task.wait()` with negative delay | |
| F009 | Common Bugs | `Instance.new("")` empty class name | |
| F010 | Common Bugs | `RenderStepped` on server | |
| F011 | Code Quality | `task.wait()` return value captured | |
| F012 | Common Bugs | `:Connect()` with non-function argument | |
| F015 | Code Hygiene | TODO/FIXME/HACK comments | |
| F016 | Code Quality | File too large (>500 lines) | |
| B013 | API Deprecation | `FilteringEnabled` check (always true since 2018) | |

### Security (8 rules)

| ID | Category | Description |
|:---|:---|:---|
| S001 | Security | `OnServerEvent` without argument validation |
| S002 | Security | Remote handler without rate limiting |
| S003 | Security | `FireServer` sending Position/CFrame data |
| S004 | Security | `loadstring` usage |
| S005 | Security | HTTP requests without `pcall` |
| S006 | Data Persistence | DataStore without session locking |
| S007 | Security | `game:Destroy()` |
| S008 | Security | Remote handler without type checking |

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

---

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, MSVC toolchain on Windows)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) (WebView2 on Windows)

### Desktop App

```bash
git clone https://github.com/Cobbdevv/luau-grader.git
cd luau-grader
cargo install tauri-cli --version "^2"
cargo tauri build
```

### CLI Only

```bash
cargo build --release -p luau-grader-cli
```

The binary will be at `target/release/luau-grader.exe`.

### Running Tests

```bash
cargo test -p luau-grader-core
```

---

## Project Structure

```
luau-grader/
  Cargo.toml                   Workspace manifest
  luau-grader-core/            Grading engine library
    src/
      analyzer/                AST walker and analysis context
      rulesets/                Rule implementations by tier
        beginner.rs            12 rules
        intermediate.rs        22 rules
        advanced.rs            20 rules
        front_page.rs          15 rules
        security.rs            8 rules
        mod.rs                 Rule registration and tier mapping
      config.rs                Tier enum
      report.rs                Diagnostic and Report types
      errors.rs                Error types
      fixer.rs                 Auto-fix engine
      batch.rs                 Multi-file analysis
      metrics.rs               AST-based complexity and pattern analysis
      scorer.rs                7-dimensional weighted scoring engine
      grade.rs                 Grade report structures and thresholds
      ruleset_config.rs        .luaugraderrc config parsing
      lib.rs                   Library root
    tests/
      rules.rs                 136 integration tests
  luau-grader-cli/             Standalone CLI binary
    src/
      main.rs                  check, check-dir, fix, grade, list-rules
  src-tauri/                   Tauri desktop application
  src/                         Frontend (HTML/CSS/JS)
```

---

## Adding Rules

See [CONTRIBUTING.md](CONTRIBUTING.md) for a guide on implementing new rules using the `Rule` trait.

---

## License

MIT License. See [LICENSE](LICENSE).

---

Built by Cobb_Dev.