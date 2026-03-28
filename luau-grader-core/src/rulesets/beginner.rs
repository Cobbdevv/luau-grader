use full_moon::ast::{self, Prefix, Suffix, Call};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

fn span_from_node(node: &impl Node) -> Option<Span> {
    node.start_position().map(|pos| Span { line: pos.line(), column: pos.character() })
}

fn check_deprecated_global(call: &ast::FunctionCall, target: &str, rule_id: &str, severity: Severity, suggestion: &str, category: &str) -> Option<Diagnostic> {
    if let Prefix::Name(name) = call.prefix()
        && name.token().to_string() == target {
            let suffixes: Vec<_> = call.suffixes().collect();
            if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                return Some(Diagnostic {
                    rule_id: rule_id.to_string(), severity, category: category.to_string(),
                    message: format!("use task.{target}() instead of the deprecated {target}()"),
                    span: span_from_node(call), suggestion: Some(suggestion.to_string()),
                    fixable: true,
                });
            }
        }
    None
}

fn check_method_call_name(call: &ast::FunctionCall, method_name: &str) -> bool {
    for suffix in call.suffixes() {
        if let Suffix::Call(Call::MethodCall(method)) = suffix
            && method.name().token().to_string() == method_name { return true; }
    }
    false
}

fn fix_replace_on_line(source: &str, diagnostic: &Diagnostic, find: &str, replace: &str) -> Option<Fix> {
    let line_num = diagnostic.span.as_ref()?.line;
    let line = source.lines().nth(line_num - 1)?;
    Some(Fix {
        description: format!("replace {find} with {replace}"),
        line: line_num,
        replacement: line.replacen(find, replace, 1),
    })
}

#[derive(Debug)] pub struct DeprecatedWaitRule;
impl Rule for DeprecatedWaitRule {
    fn id(&self) -> &'static str { "B001" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "Deprecated wait() — use task.wait()" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt && let Some(d) = check_deprecated_global(call, "wait", self.id(), self.severity(), "task.wait()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr && let Some(d) = check_deprecated_global(call, "wait", self.id(), self.severity(), "task.wait()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        fix_replace_on_line(source, diagnostic, "wait(", "task.wait(")
    }
}

#[derive(Debug)] pub struct DeprecatedSpawnRule;
impl Rule for DeprecatedSpawnRule {
    fn id(&self) -> &'static str { "B002" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "Deprecated spawn() — use task.spawn()" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt && let Some(d) = check_deprecated_global(call, "spawn", self.id(), self.severity(), "task.spawn()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr && let Some(d) = check_deprecated_global(call, "spawn", self.id(), self.severity(), "task.spawn()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        fix_replace_on_line(source, diagnostic, "spawn(", "task.spawn(")
    }
}

#[derive(Debug)] pub struct DeprecatedDelayRule;
impl Rule for DeprecatedDelayRule {
    fn id(&self) -> &'static str { "B003" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "Deprecated delay() — use task.delay()" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt && let Some(d) = check_deprecated_global(call, "delay", self.id(), self.severity(), "task.delay()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr && let Some(d) = check_deprecated_global(call, "delay", self.id(), self.severity(), "task.delay()", self.category()) { return vec![d]; }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        fix_replace_on_line(source, diagnostic, "delay(", "task.delay(")
    }
}

#[derive(Debug)] pub struct InvokeClientRule;
impl Rule for InvokeClientRule {
    fn id(&self) -> &'static str { "B004" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Networking" }
    fn description(&self) -> &'static str { "InvokeClient permanently yields if client disconnects" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt
            && check_method_call_name(call, "InvokeClient") {
                return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "never use InvokeClient — it permanently yields if the client disconnects".to_string(),
                    span: span_from_node(call), suggestion: Some("use RemoteEvent:FireClient() instead".to_string()),
                    fixable: false,
                }];
            }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct WaitForChildTimeoutRule {
    pub default_timeout: u64,
}
impl WaitForChildTimeoutRule {
    pub fn new(default_timeout: u64) -> Self { Self { default_timeout } }
}
impl Default for WaitForChildTimeoutRule {
    fn default() -> Self { Self { default_timeout: 5 } }
}
impl Rule for WaitForChildTimeoutRule {
    fn id(&self) -> &'static str { "B005" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "WaitForChild() missing timeout parameter" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        if let Some(wfc_pos) = line.find("WaitForChild(") {
            let after_open = wfc_pos + "WaitForChild(".len();
            let rest = &line[after_open..];
            let mut depth = 1u32;
            for (i, ch) in rest.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => { depth -= 1; if depth == 0 {
                        let insert = after_open + i;
                        let mut fixed = line.to_string();
                        fixed.insert_str(insert, &format!(", {}", self.default_timeout));
                        return Some(Fix {
                            description: format!("add timeout of {} seconds", self.default_timeout),
                            line: line_num,
                            replacement: fixed,
                        });
                    }}
                    _ => {}
                }
            }
        }
        None
    }
}
impl WaitForChildTimeoutRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix
                && method.name().token().to_string() == "WaitForChild" {
                    let arg_count = match method.args() {
                        ast::FunctionArgs::Parentheses { arguments, .. } => arguments.len(),
                        _ => 0,
                    };
                    if arg_count <= 1 {
                        return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "WaitForChild() without a timeout — will yield forever if child never exists".to_string(),
                            span: span_from_node(call), suggestion: Some(format!(":WaitForChild(\"Name\", {})", self.default_timeout)),
                            fixable: true,
                        }];
                    }
                }
        }
        Vec::new()
    }
}