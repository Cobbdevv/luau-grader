use full_moon::ast::{self, Suffix, Index};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

#[derive(Debug)] pub struct NoStrictModeRule;
impl Rule for NoStrictModeRule {
    fn id(&self) -> &'static str { "F001" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Missing --!strict at top of script" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn is_fixable(&self) -> bool { true }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let first_line = ctx.source.lines().next().unwrap_or("");
        if !first_line.trim().starts_with("--!strict") {
            return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: "--!strict is missing — add it to the top of every script".to_string(),
                span: Some(Span { line: 1, column: 1 }), suggestion: Some("--!strict".to_string()),
                fixable: true,
            }];
        }
        Vec::new()
    }
    fn fix(&self, _source: &str, _diagnostic: &Diagnostic) -> Option<Fix> {
        Some(Fix {
            description: "prepend --!strict directive".to_string(),
            line: 0,
            replacement: "--!strict".to_string(),
        })
    }
}

#[derive(Debug)] pub struct ParentNilWithoutDestroyRule;
impl Rule for ParentNilWithoutDestroyRule {
    fn id(&self) -> &'static str { "F002" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Memory Management" }
    fn description(&self) -> &'static str { "Parent = nil without :Destroy() — causes memory leaks" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::Assignment(assign) = stmt {
            for var in assign.variables() {
                if let ast::Var::Expression(var_expr) = var {
                    let suffixes: Vec<_> = var_expr.suffixes().collect();
                    if let Some(Suffix::Index(Index::Dot { name, .. })) = suffixes.last()
                        && name.token().to_string() == "Parent" {
                            for expr in assign.expressions() {
                                if let ast::Expression::Symbol(sym) = expr
                                    && sym.token().to_string() == "nil" {
                                        return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                            message: "setting Parent = nil without :Destroy() — use :Destroy() to properly clean up".to_string(),
                                            span: stmt.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                                            suggestion: Some("instance:Destroy()".to_string()),
                                            fixable: true,
                                        }];
                                    }
                            }
                        }
                }
            }
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        let trimmed = line.trim();
        if let Some(dot_pos) = trimmed.find(".Parent") {
            let obj_name = trimmed[..dot_pos].trim();
            let indent = &line[..line.len() - line.trim_start().len()];
            return Some(Fix {
                description: "replace Parent = nil with :Destroy()".to_string(),
                line: line_num,
                replacement: format!("{indent}{obj_name}:Destroy()"),
            });
        }
        None
    }
}

#[derive(Debug)] pub struct RequireInLoopRule;
impl Rule for RequireInLoopRule {
    fn id(&self) -> &'static str { "F003" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Module Architecture" }
    fn description(&self) -> &'static str { "require() called inside a loop or hot path" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr
            && let ast::Prefix::Name(name) = call.prefix()
                && name.token().to_string() == "require" {
                    return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "require() inside a loop — cache modules at the top of the file".to_string(),
                        span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                        suggestion: Some("local Module = require(...) -- at file top".to_string()),
                        fixable: false,
                    }];
                }
        Vec::new()
    }
}