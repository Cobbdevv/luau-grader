use full_moon::ast::{self, Prefix, Suffix, Call, Index, BinOp};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

#[derive(Debug)] pub struct InstanceNewInLoopRule;
impl Rule for InstanceNewInLoopRule {
    fn id(&self) -> &'static str { "A001" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "Instance.new() inside loops — use object pooling" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr
            && let Prefix::Name(name) = call.prefix()
                && name.token().to_string() == "Instance" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if suffixes.len() >= 2
                        && let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first()
                            && method.token().to_string() == "new" {
                                return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                    message: "Instance.new() inside a loop — use object pooling instead".to_string(),
                                    span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                                    suggestion: Some("pre-create instances and reuse them from a pool".to_string()),
                                    fixable: false,
                                }];
                            }
                }
        Vec::new()
    }
}

#[derive(Debug)] pub struct ConnectWithoutStoreRule;
impl Rule for ConnectWithoutStoreRule {
    fn id(&self) -> &'static str { "A002" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Memory Management" }
    fn description(&self) -> &'static str { "Connection created but never stored for cleanup" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            let suffixes: Vec<_> = call.suffixes().collect();
            if let Some(Suffix::Call(Call::MethodCall(method))) = suffixes.last() {
                let method_name = method.name().token().to_string();
                if method_name == "Connect" || method_name == "connect" {
                    return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "connection created but not stored — will leak memory if not cleaned up".to_string(),
                        span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                        suggestion: Some("local connection = signal:Connect(...)".to_string()),
                        fixable: true,
                    }];
                }
            }
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        Some(Fix {
            description: "store connection in a local variable".to_string(),
            line: line_num,
            replacement: format!("{indent}local conn = {trimmed}"),
        })
    }
}

#[derive(Debug)] pub struct StringConcatInLoopRule;
impl Rule for StringConcatInLoopRule {
    fn id(&self) -> &'static str { "A003" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "String concatenation (..) inside loops" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::BinaryOperator { binop, .. } = expr
            && matches!(binop, BinOp::TwoDots(_)) {
                return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "string concatenation in a loop — use table.insert() and table.concat()".to_string(),
                    span: expr.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                    suggestion: Some("table.insert(parts, str); result = table.concat(parts)".to_string()),
                    fixable: false,
                }];
            }
        Vec::new()
    }
}

#[derive(Debug)] pub struct SetAsyncRule;
impl Rule for SetAsyncRule {
    fn id(&self) -> &'static str { "A004" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Data Persistence" }
    fn description(&self) -> &'static str { "SetAsync used instead of UpdateAsync" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            for suffix in call.suffixes() {
                if let Suffix::Call(Call::MethodCall(method)) = suffix
                    && method.name().token().to_string() == "SetAsync" {
                        return vec![Diagnostic { rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "use UpdateAsync instead of SetAsync for safe concurrent data updates".to_string(),
                            span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                            suggestion: Some("dataStore:UpdateAsync(key, function(old) return newData end)".to_string()),
                            fixable: true,
                        }];
                    }
            }
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        Some(Fix {
            description: "replace SetAsync with UpdateAsync".to_string(),
            line: line_num,
            replacement: line.replacen("SetAsync", "UpdateAsync", 1),
        })
    }
}