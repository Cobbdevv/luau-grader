use std::collections::HashSet;

use crate::report::Diagnostic;
use crate::rulesets::Rule;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Fix {
    pub description: String,
    pub line: usize,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppliedFix {
    pub rule_id: String,
    pub line: usize,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FixReport {
    pub fixed_source: String,
    pub applied: Vec<AppliedFix>,
    pub unfixable: Vec<String>,
}

pub fn apply_fixes(
    source: &str,
    diagnostics: &[Diagnostic],
    rules: &[Box<dyn Rule>],
) -> FixReport {
    let mut lines: Vec<String> = source.lines().map(String::from).collect();
    let mut applied = Vec::new();
    let mut unfixable = Vec::new();
    let mut fixes: Vec<(usize, Fix, String)> = Vec::new();

    for diag in diagnostics {
        if let Some(rule) = rules.iter().find(|r| r.id() == diag.rule_id) {
            match rule.fix(source, diag) {
                Some(fix) => {
                    let line = diag.span.as_ref().map(|s| s.line).unwrap_or(0);
                    fixes.push((line, fix, diag.rule_id.clone()));
                }
                None => unfixable.push(format!("{}: not auto-fixable", diag.rule_id)),
            }
        }
    }

    fixes.sort_by(|a, b| b.0.cmp(&a.0));

    let mut touched = HashSet::new();
    for (line, fix, rule_id) in fixes {
        if touched.contains(&line) { continue; }
        touched.insert(line);
        if line == 0 {
            lines.insert(0, fix.replacement);
        } else if line <= lines.len() {
            lines[line - 1] = fix.replacement;
        }
        applied.push(AppliedFix { rule_id, line, description: fix.description });
    }

    applied.reverse();
    FixReport { fixed_source: lines.join("\n"), applied, unfixable }
}
