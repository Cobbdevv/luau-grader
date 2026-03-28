use full_moon::ast;
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

#[derive(Debug)]
pub struct FunctionTooLongRule {
    pub max_lines: usize,
}
impl FunctionTooLongRule {
    pub fn new(max_lines: usize) -> Self { Self { max_lines } }
}
impl Default for FunctionTooLongRule {
    fn default() -> Self { Self { max_lines: 50 } }
}
impl Rule for FunctionTooLongRule {
    fn id(&self) -> &'static str { "I001" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Function exceeds line limit" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_function_body(&self, body: &ast::FunctionBody, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let (Some(start), Some(end)) = (body.start_position(), body.end_position()) {
            let line_count = end.line().saturating_sub(start.line()) + 1;
            if line_count > self.max_lines {
                return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("function is {line_count} lines long — keep functions under {} lines", self.max_lines),
                    span: Some(Span { line: start.line(), column: start.character() }),
                    suggestion: Some("break this function into smaller, focused helper functions".to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SingleLetterVariableRule {
    pub exceptions: Vec<String>,
}
impl SingleLetterVariableRule {
    pub fn new(exceptions: Vec<String>) -> Self { Self { exceptions } }
}
impl Default for SingleLetterVariableRule {
    fn default() -> Self {
        Self { exceptions: vec!["i".into(), "j".into(), "k".into(), "_".into()] }
    }
}
impl Rule for SingleLetterVariableRule {
    fn id(&self) -> &'static str { "I002" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Single-letter variable names" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        if let ast::Stmt::LocalAssignment(local) = stmt {
            for name in local.names() {
                let var_name = name.token().to_string();
                if var_name.len() == 1 && !self.exceptions.iter().any(|e| e == &var_name) {
                    results.push(Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("single-letter variable '{var_name}' — use a descriptive name"),
                        span: name.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                        suggestion: None, fixable: false,
                    });
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct GetServiceInLoopRule;
impl Rule for GetServiceInLoopRule {
    fn id(&self) -> &'static str { "I003" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "GetService() called inside a loop or hot path" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Stmt::FunctionCall(call) = stmt && self.has_get_service(call) {
            return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: "GetService() inside a loop — cache services at the top of the file".to_string(),
                span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                suggestion: Some("local Players = game:GetService(\"Players\") -- at file top".to_string()),
                fixable: true,
            }];
        }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr && self.has_get_service(call) {
            return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: "GetService() inside a loop — cache services at the top of the file".to_string(),
                span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                suggestion: Some("local Players = game:GetService(\"Players\") -- at file top".to_string()),
                fixable: true,
            }];
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        let trimmed = line.trim();
        let indent = &line[..line.len() - trimmed.len()];
        if let Some(gs_pos) = trimmed.find(":GetService(") {
            let prefix = trimmed[..gs_pos].trim();
            let after_gs = &trimmed[gs_pos + ":GetService(".len()..];
            if let Some(close) = after_gs.find(')') {
                let service_name = after_gs[..close].trim().trim_matches('"').trim_matches('\'');
                return Some(Fix {
                    description: format!("hoist GetService({service_name}) — move to file top"),
                    line: line_num,
                    replacement: format!("{indent}-- TODO: move to file top: local {service_name} = {prefix}:GetService(\"{service_name}\")"),
                });
            }
        }
        None
    }
}
impl GetServiceInLoopRule {
    fn has_get_service(&self, call: &ast::FunctionCall) -> bool {
        use full_moon::ast::{Suffix, Call};
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix
                && method.name().token().to_string() == "GetService" { return true; }
        }
        false
    }
}