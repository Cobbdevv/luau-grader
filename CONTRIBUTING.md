# Contributing

This guide explains how to add new rules to the Luau Auto-Grader.

---

## Architecture

The grading engine is in `luau-grader-core/`. Every rule implements the `Rule` trait, which exposes hooks into the AST walker. The walker visits every statement, expression, and function body in the parsed code and delegates to each active rule.

### Rule Trait

```rust
pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;          // e.g. "B001"
    fn severity(&self) -> Severity;        // Error, Warning, or Info
    fn category(&self) -> &'static str;    // Category name
    fn description(&self) -> &'static str; // Short human-readable summary
    fn tier(&self) -> &'static str;        // Beginner, Intermediate, Advanced, Front Page
    fn is_fixable(&self) -> bool;          // true if fix() returns Some

    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic>;
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic>;
    fn check_function_body(&self, body: &ast::FunctionBody, ctx: &AnalysisContext) -> Vec<Diagnostic>;
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic>;
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix>;
}
```

### Adding a Rule

1. Pick a tier file: `beginner.rs`, `intermediate.rs`, `advanced.rs`, or `front_page.rs`.
2. Create a struct and implement `Rule`. Return diagnostics from whichever hook matches your detection.
3. Set `fixable: true` in the Diagnostic if the rule supports auto-fix, and implement `fix()` + `is_fixable()`.
4. Register the rule in `rulesets/mod.rs` inside `all_rules_with_config()`.
5. Write a test in `luau-grader-core/tests/rules.rs`.
6. Run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings`.

### Implementing Auto-Fix

The `fix()` method receives the full source and the diagnostic, and returns a `Fix`:

```rust
pub struct Fix {
    pub description: String, // what the fix does
    pub line: usize,         // 0 = prepend to file, >0 = replace that line
    pub replacement: String, // the new line content
}
```

The fixer applies fixes from bottom to top so line numbers don't shift.

### Parameterizable Rules

Rules can accept parameters from `.luaugraderrc`. Use struct fields instead of a unit struct:

```rust
pub struct FunctionTooLongRule {
    pub max_lines: usize,
}
```

Then read the parameter in `all_rules_with_config()`:

```rust
let max_lines: usize = config.get_param("I001", "max_lines").unwrap_or(50);
Box::new(intermediate::FunctionTooLongRule::new(max_lines)),
```

### ID Convention

- `B` prefix for Beginner tier rules.
- `I` prefix for Intermediate.
- `A` prefix for Advanced.
- `F` prefix for Front Page.
- Increment the number from the last rule in that tier.

### Category Names

- Code Style
- Module Architecture
- Memory Management
- Performance
- Networking
- Data Persistence
- Common Bugs

### The AnalysisContext

The `AnalysisContext` tracks state as the walker traverses the AST:

- `ctx.in_loop()` — true when inside a loop body (while, for, repeat).
- `ctx.scope_depth` — current lexical scope depth.
- `ctx.source` — the raw source string for rules that need text analysis.

---

## Code Style

- No comments unless they explain a non-obvious decision.
- Functions do one thing. Modules handle one system.
- Test every rule with at least one positive and one negative case.

---

## Pull Requests

- One PR per rule or per feature.
- Include tests.
- Run `cargo check --workspace` and `cargo test --workspace` before submitting.