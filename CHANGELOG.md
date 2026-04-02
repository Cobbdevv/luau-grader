# Changelog

## 1.0.0

### Grading Engine
- 7-dimensional code scoring: Structure, API Correctness, Error Handling, Performance, Readability, Safety, Security
- Per-function grades with cyclomatic and cognitive complexity analysis
- Technical debt estimation in minutes with category breakdown
- Improvement projection with prioritized fixes and effort estimates
- Script type auto-detection: ServerScript, ClientScript, ModuleScript, SharedModule, Plugin
- Pattern recognition: Debounce, Cooldown, Data Save/Load, Character Added Handler, Module Pattern, Observer, Cleanup/Janitor

### Rules
- 77 rules across 5 tiers and 6 categories
- 12 Beginner rules for deprecated APIs and common mistakes
- 22 Intermediate rules for code quality, naming, and style
- 20 Advanced rules for performance, memory, and deprecated patterns
- 15 Front Page rules for architecture and strict standards
- 8 Security rules for remote event safety, rate limiting, and data integrity
- 11 rules with auto-fix support

### CLI
- New `grade` command for full grade reports with dimension bars and improvement paths
- `check` command for diagnostic-only output
- `check-dir` for recursive multi-file analysis with aggregate grades
- `fix` command with `--dry-run` preview mode
- `list-rules` command showing all 77 rules grouped by tier
- JSON output for CI pipeline integration
- Exit codes: 0 = clean, 1 = warnings, 2 = errors

### Desktop Application
- Grade dashboard with animated score ring and 7 dimension bars
- Per-function grade table with click-to-jump navigation
- Technical debt visualization with category breakdown
- Improvement path with point impact and effort estimates
- Strengths and detected patterns display
- Code editor with Luau syntax highlighting
- Diagnostic cards with severity chips and line references
- Auto-fix with one-click "FIX ALL" button
- Rule settings sidebar with per-rule toggle
- Drag-and-drop file upload

### Configuration
- `.luaugraderrc` JSON config file with auto-discovery
- Tier override, disabled rules, severity overrides, and parameter tuning
- Custom rule definitions with function_call and method_call pattern matching

### Infrastructure
- Cross-platform CI: Windows and Ubuntu
- Automated release pipeline with CLI exe, Tauri MSI, and NSIS installer
- 136 integration tests with zero failures
- Clippy clean with zero warnings
