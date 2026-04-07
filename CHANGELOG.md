# Changelog

## 1.2.0

### Scoring Engine Overhaul — Calibrated Against 630 Production Scripts

The entire scoring engine has been recalibrated against a 630-script production codebase to eliminate score inflation and systematic bias. Every change was validated against 18 expert-graded reference scripts to ensure accuracy.

#### Length-Agnostic Scoring
- Removed all file-length-based penalties — long files are no longer penalized for being long
- Removed tiny-file Performance inflation — scripts under 30 lines no longer auto-receive 80 in Performance
- Scoring is now purely quality-based regardless of file size

#### Stricter Baselines
- Performance baseline lowered from 92 to 82 for scripts with no detected issues
- API Correctness baseline lowered from 80 to 75 for scripts with no API surface
- Security baseline lowered from 80 to 75 for scripts with no security surface
- Error Handling baseline lowered from 100 to 90
- Performance with issues now deducts from 82 (not 100), preventing scripts with issues from scoring higher than clean ones

#### Quality Ceiling
- Scripts missing all quality evidence (no `--!strict`, no type annotations, no pcall) are now capped at 75 overall
- Prevents low-effort scripts from coasting on dimensions with no violations

#### Strict Mode Enforcement
- Missing `--!strict` changed from a +3 bonus to a **-10 penalty**
- Strict mode is now treated as a baseline expectation, not a reward

#### Universal Good Practices
- Good practices checks (strict mode, type annotations, pcall, module pattern) now apply to **all files with functions**, not just small files
- Missing all 4 practices: -20 penalty (was -15)
- Missing 3: -15 penalty
- Missing 2: -8 penalty

#### Error Handling Hardening
- New tiered penalty system for missing pcall:
  - Uses DataStoreService/HttpService/MarketplaceService with no pcall: -35
  - Has risky operations (3+ services or functions) with no pcall: -30 server / -25 client
  - Has functions but no pcall anywhere: -20 server / -15 client
- Removed redundant duplicate "no error handling in any function" penalty that was double-counting

#### Dimension Floors
- All dimensions now have a minimum score floor (10-15) to prevent extreme 0-scores on large functional files

#### Readability Tightening
- Single-letter variable penalty increased from 3 to 5 points per occurrence
- Naming quality thresholds raised: <3.0 avg = -30, <3.5 = -15, <4.0 = -8 (was <2.0/-2.5/-3.0)
- Cognitive complexity penalties now capped at -15 total (prevents runaway deductions on large files)
- Vague names list refined to reduce false positives (removed "val", "value", "item", "args", "params", "input", "output", "buf", "arr", "list", "map")

### New Rules

#### I033 — Abbreviated Variable Names
- Detects 37 common Roblox abbreviations: `plr`, `hrp`, `hum`, `ts`, `uis`, `cas`, `btn`, `conn`, `cfg`, `mgr`, `pos`, `vel`, `dir`, `rot`, `cf`, `vec`, and more
- Provides specific suggestions for each abbreviation (e.g., `plr` → `player`, `hrp` → `humanoidRootPart`)
- Has a built-in exception list for universally understood short names (`ok`, `id`, `dt`, `hp`, `ai`, `ui`, etc.)
- Impacts Readability score: -3 per occurrence, capped at -20

#### A029 — Global Table (`_G`) Usage
- Detects both reads and writes to the `_G` global table
- Flags implicit cross-script dependencies that make code harder to test and reason about
- Suggests using ModuleScripts and `require()` instead
- Impacts Readability score: -5 per occurrence, capped at -20

### Bug Fixes
- Fixed InconsistentReturnRule (I023) false positives: guard clauses (`if not x then return end`) are no longer counted as bare returns
- Fixed VariableShadowingRule (I027) false positives: keyword matching inside string literals no longer causes incorrect scope tracking (uses `strip_string_contents` + `count_keyword_occurrences`)
- Fixed Performance scoring backwards: scripts with 1 minor issue could previously score higher (97) than scripts with zero issues (92)

### Calibration Results
- 93 rules across 5 tiers (was 91 in v1.1.0)
- 159 integration tests, zero failures
- Validated against 630 production scripts from a front-page Roblox game
- 13/18 expert-graded scripts within ±5 points of expert assessment
- Average scoring gap: 3.7 points (was 8.8 in v1.1.0)

## 1.1.0

### Scoring Engine Recalibration
- Fixed hollow dimension scoring: API Correctness, Performance, and Security now return neutral 80 instead of perfect 100 when no relevant code exists to evaluate
- Added type annotation coverage to Readability and Safety scoring dimensions
- Reduced over-generous baseline bonuses that rewarded expected behavior rather than excellence
- Removed dead I025 (magic numbers) scorer reference that could never fire
- Adjusted grade thresholds to account for recalibrated raw scores

### New Features
- Inline suppression comments: `-- luau-grader: ignore RULE_ID` suppresses diagnostics on the next line
- Trailing suppression comments: `code() -- luau-grader: ignore RULE_ID` suppresses on the same line
- Multi-rule suppression: `-- luau-grader: ignore B001, B002`
- Type annotation counting via full_moon AST (parameters, return types, typed locals)
- New `type_annotation_ratio` metric tracking annotation coverage percentage

### Bug Fixes
- `type_annotation_count` metric now actually counts annotations (was always 0)

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
- 159 integration tests with zero failures
- Clippy clean with zero warnings
