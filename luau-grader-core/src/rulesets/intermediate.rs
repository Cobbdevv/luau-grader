use full_moon::ast::{self, Prefix, Suffix, Call, Index};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

fn span_from_node(node: &impl Node) -> Option<Span> {
    node.start_position().map(|pos| Span { line: pos.line(), column: pos.character() })
}

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
                    message: format!("function is {line_count} lines long - keep functions under {} lines", self.max_lines),
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
        Self { exceptions: vec!["i".into(), "j".into(), "k".into(), "_".into(), "v".into(), "t".into(), "x".into(), "y".into(), "z".into()] }
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
                        message: format!("single-letter variable '{var_name}' - use a descriptive name"),
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
                message: "GetService() inside a loop - cache services at the top of the file".to_string(),
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
                message: "GetService() inside a loop - cache services at the top of the file".to_string(),
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
                    description: format!("hoist GetService({service_name}) - move to file top"),
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
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix
                && method.name().token().to_string() == "GetService" { return true; }
        }
        false
    }
}

#[derive(Debug)] pub struct NumericForWrongStepRule;
impl Rule for NumericForWrongStepRule {
    fn id(&self) -> &'static str { "I004" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Numeric for loop with wrong step direction - loop body never executes" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::NumericFor(num_for) = stmt {
            let start_val = extract_number(num_for.start());
            let end_val = extract_number(num_for.end());
            let step_val = num_for.step().and_then(|s| extract_number(s));

            if let (Some(start), Some(end), Some(step)) = (start_val, end_val, step_val) {
                if step == 0.0 {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "for loop step is 0 - infinite loop".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("use a positive or negative step value".to_string()),
                        fixable: false,
                    }];
                }
                if (start < end && step < 0.0) || (start > end && step > 0.0) {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("for loop from {start} to {end} with step {step} - loop body will never execute"),
                        span: span_from_node(stmt),
                        suggestion: Some(format!("use step {} instead", -step)),
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
        if let Some(suggestion) = &diagnostic.suggestion {
            if let Some(step_str) = suggestion.strip_prefix("use step ") {
                let step_str = step_str.strip_suffix(" instead")?;
                let msg = &diagnostic.message;
                if let Some(pos) = msg.find("with step ") {
                    let rest = &msg[pos + "with step ".len()..];
                    let old_step = rest.split(' ').next()?;
                    return Some(Fix {
                        description: format!("change step from {old_step} to {step_str}"),
                        line: line_num,
                        replacement: line.replacen(old_step, step_str, 1),
                    });
                }
            }
        }
        None
    }
}

#[derive(Debug)] pub struct EmptyIfBodyRule;
impl Rule for EmptyIfBodyRule {
    fn id(&self) -> &'static str { "I005" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Empty if block - likely unfinished code" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::If(if_stmt) = stmt {
            if if_stmt.block().stmts().count() == 0 {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "empty if block - this may be unfinished code".to_string(),
                    span: span_from_node(stmt),
                    suggestion: Some("add the intended logic or remove the empty if".to_string()),
                    fixable: false,
                }];
            }
            if let Some(else_ifs) = if_stmt.else_if() {
                for branch in else_ifs {
                    if branch.block().stmts().count() == 0 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "empty elseif block - this may be unfinished code".to_string(),
                            span: span_from_node(stmt),
                            suggestion: Some("add the intended logic or remove the empty branch".to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
            if let Some(else_block) = if_stmt.else_block() {
                if else_block.stmts().count() == 0 {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "empty else block - this may be unfinished code".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("add the intended logic or remove the empty else".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct RedundantTostringRule;
impl Rule for RedundantTostringRule {
    fn id(&self) -> &'static str { "I006" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Redundant tostring() on a string literal" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn is_fixable(&self) -> bool { true }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "tostring" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.first() {
                        if arguments.len() == 1 {
                            if let Some(arg) = arguments.iter().next() {
                                if matches!(arg, ast::Expression::String(_)) {
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: "tostring() called on a string literal - this is redundant".to_string(),
                                        span: span_from_node(expr),
                                        suggestion: Some("remove the tostring() wrapper".to_string()),
                                        fixable: true,
                                    }];
                                }
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
        if let Some(start) = line.find("tostring(") {
            let after = &line[start + "tostring(".len()..];
            let mut depth = 1u32;
            for (i, ch) in after.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => { depth -= 1; if depth == 0 {
                        let inner = &after[..i];
                        let mut fixed = line.to_string();
                        fixed.replace_range(start..start + "tostring(".len() + i + 1, inner);
                        return Some(Fix {
                            description: "remove redundant tostring() wrapper".to_string(),
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

#[derive(Debug)] pub struct RedundantTonumberRule;
impl Rule for RedundantTonumberRule {
    fn id(&self) -> &'static str { "I007" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Redundant tonumber() on a number literal" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn is_fixable(&self) -> bool { true }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "tonumber" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.first() {
                        if arguments.len() == 1 {
                            if let Some(arg) = arguments.iter().next() {
                                if matches!(arg, ast::Expression::Number(_)) {
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: "tonumber() called on a number literal - this is redundant".to_string(),
                                        span: span_from_node(expr),
                                        suggestion: Some("remove the tonumber() wrapper".to_string()),
                                        fixable: true,
                                    }];
                                }
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
        if let Some(start) = line.find("tonumber(") {
            let after = &line[start + "tonumber(".len()..];
            let mut depth = 1u32;
            for (i, ch) in after.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => { depth -= 1; if depth == 0 {
                        let inner = &after[..i];
                        let mut fixed = line.to_string();
                        fixed.replace_range(start..start + "tonumber(".len() + i + 1, inner);
                        return Some(Fix {
                            description: "remove redundant tonumber() wrapper".to_string(),
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

#[derive(Debug)]
pub struct DeepNestingRule {
    pub max_depth: usize,
}
impl DeepNestingRule {
    pub fn new(max_depth: usize) -> Self { Self { max_depth } }
}
impl Default for DeepNestingRule {
    fn default() -> Self { Self { max_depth: 5 } }
}
impl Rule for DeepNestingRule {
    fn id(&self) -> &'static str { "I008" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Code is nested too deeply" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let is_block_stmt = matches!(stmt,
            ast::Stmt::If(_) | ast::Stmt::While(_) | ast::Stmt::Repeat(_) |
            ast::Stmt::NumericFor(_) | ast::Stmt::GenericFor(_)
        );
        if is_block_stmt && ctx.scope_depth > self.max_depth {
            return vec![Diagnostic {
                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: format!("code nested {} levels deep (max {}) - consider extracting into functions", ctx.scope_depth, self.max_depth),
                span: span_from_node(stmt),
                suggestion: Some("extract deeply-nested logic into helper functions".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DebugPrintWarnRule;
impl Rule for DebugPrintWarnRule {
    fn id(&self) -> &'static str { "I009" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "print() / warn() calls left in code" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                let fn_name = name.token().to_string();
                if fn_name == "print" || fn_name == "warn" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("{fn_name}() call - consider removing debug statements before shipping"),
                            span: span_from_node(stmt),
                            suggestion: Some("remove or replace with a proper logging system".to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct TableSortResultRule;
impl Rule for TableSortResultRule {
    fn id(&self) -> &'static str { "I010" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "table.sort() result used in assignment - it returns nil" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::LocalAssignment(local) = stmt {
            for expr in local.expressions() {
                if self.is_table_sort_call(expr) {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "table.sort() returns nil - assigning its result is a bug".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("table.sort(t) sorts in place; use it as a statement, then reference t".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        if let ast::Stmt::Assignment(assign) = stmt {
            for expr in assign.expressions() {
                if self.is_table_sort_call(expr) {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "table.sort() returns nil - assigning its result is a bug".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("table.sort(t) sorts in place; use it as a statement, then reference t".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}
impl TableSortResultRule {
    fn is_table_sort_call(&self, expr: &ast::Expression) -> bool {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "table" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                        return method.token().to_string() == "sort";
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug)] pub struct TypeVsTypeofRule;
impl Rule for TypeVsTypeofRule {
    fn id(&self) -> &'static str { "I011" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "type() returns 'userdata' for Roblox types - use typeof() instead" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "type" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "type() returns 'userdata' for Roblox types - consider typeof() for more specific type info".to_string(),
                            span: span_from_node(expr),
                            suggestion: Some("typeof(value) returns specific types like 'Vector3', 'Instance', etc.".to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct Color3NewLargeValuesRule;
impl Rule for Color3NewLargeValuesRule {
    fn id(&self) -> &'static str { "I012" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Color3.new() with values > 1 - likely meant Color3.fromRGB()" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "Color3" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if suffixes.len() >= 2 {
                        if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                            if method.token().to_string() == "new" {
                                if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                    for arg in arguments.iter() {
                                        if let Some(v) = extract_number(arg) {
                                            if v > 1.0 {
                                                return vec![Diagnostic {
                                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                    message: "Color3.new() takes values 0-1; did you mean Color3.fromRGB() (0-255)?".to_string(),
                                                    span: span_from_node(expr),
                                                    suggestion: Some("Color3.fromRGB(r, g, b) for 0-255 values".to_string()),
                                                    fixable: false,
                                                }];
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

fn extract_number(expr: &ast::Expression) -> Option<f64> {
    match expr {
        ast::Expression::Number(token) => token.token().to_string().parse::<f64>().ok(),
        ast::Expression::UnaryOperator { unop, expression } => {
            if matches!(unop, ast::UnOp::Minus(_)) {
                extract_number(expression).map(|v| -v)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[derive(Debug)] pub struct SelfAssignmentRule;
impl Rule for SelfAssignmentRule {
    fn id(&self) -> &'static str { "I014" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Variable assigned to itself" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::Assignment(assignment) = stmt {
            let vars: Vec<String> = assignment.variables().iter().map(|v| format!("{v}").trim().to_string()).collect();
            let exprs: Vec<String> = assignment.expressions().iter().map(|e| format!("{e}").trim().to_string()).collect();
            for (i, var) in vars.iter().enumerate() {
                if let Some(expr) = exprs.get(i) {
                    if var == expr && !var.is_empty() {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("'{var}' is assigned to itself"),
                            span: span_from_node(stmt),
                            suggestion: Some("remove the self-assignment or fix the right-hand side".to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct EmptyFunctionBodyRule;
impl Rule for EmptyFunctionBodyRule {
    fn id(&self) -> &'static str { "I016" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Empty function body" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_function_body(&self, body: &ast::FunctionBody, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let block = body.block();
        if block.stmts().next().is_none() && block.last_stmt().is_none() {
            return vec![Diagnostic {
                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: "empty function body - might be unfinished code".to_string(),
                span: span_from_node(body),
                suggestion: Some("add implementation or a comment explaining why it is empty".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DuplicateTableKeyRule;
impl Rule for DuplicateTableKeyRule {
    fn id(&self) -> &'static str { "I017" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Duplicate key in table constructor" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::TableConstructor(table) = expr {
            let mut seen_keys: Vec<String> = Vec::new();
            for field in table.fields() {
                if let ast::Field::NameKey { key, .. } = field {
                    let key_name = key.token().to_string();
                    if seen_keys.contains(&key_name) {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("duplicate key '{key_name}' in table constructor - only the last value will be kept"),
                            span: span_from_node(expr),
                            suggestion: Some(format!("remove the duplicate '{key_name}' or rename one of them")),
                            fixable: false,
                        }];
                    }
                    seen_keys.push(key_name);
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct HashLengthOnDictRule;
impl Rule for HashLengthOnDictRule {
    fn id(&self) -> &'static str { "I018" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "# length operator on a dictionary always returns 0" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let source = &ctx.source;
        let mut results = Vec::new();
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("#") && !trimmed.starts_with("--") {
                if let Some(hash_pos) = trimmed.find('#') {
                    let after = &trimmed[hash_pos + 1..];
                    let var_name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
                    if !var_name.is_empty() && self.is_likely_dict(source, &var_name) {
                        results.push(Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("#{var_name} on a dictionary-style table will always return 0"),
                            span: Some(Span { line: i + 1, column: 1 }),
                            suggestion: Some("use a counter variable or iterate with pairs() to count entries".to_string()),
                            fixable: false,
                        });
                    }
                }
            }
        }
        results
    }
}
impl HashLengthOnDictRule {
    fn is_likely_dict(&self, source: &str, var_name: &str) -> bool {
        let pattern = format!("{var_name} = {{");
        for line in source.lines() {
            if line.contains(&pattern) {
                let after_brace = line.split('{').nth(1).unwrap_or("");
                if after_brace.contains('=') && !after_brace.trim().is_empty() {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug)] pub struct WhileWaitDoRule;
impl Rule for WhileWaitDoRule {
    fn id(&self) -> &'static str { "I019" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "while wait() do is an anti-pattern" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn is_fixable(&self) -> bool { true }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::While(while_loop) = stmt {
            let cond = format!("{}", while_loop.condition());
            let trimmed = cond.trim();
            if trimmed == "wait()" || trimmed == "wait( )" {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "while wait() do is an anti-pattern - use while true do with task.wait() inside".to_string(),
                    span: span_from_node(stmt),
                    suggestion: Some("while true do\\n    task.wait()\\n    ...".to_string()),
                    fixable: true,
                }];
            }
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        let replacement = line.replacen("while wait() do", "while true do", 1);
        Some(Fix {
            description: "replace while wait() do with while true do".to_string(),
            line: line_num,
            replacement,
        })
    }
}

#[derive(Debug)] pub struct NilComparisonRule;
impl Rule for NilComparisonRule {
    fn id(&self) -> &'static str { "I020" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Explicit nil comparison can be simplified" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::If(if_stmt) = stmt {
            let cond = format!("{}", if_stmt.condition());
            let trimmed = cond.trim();
            if trimmed.contains("== nil") || trimmed.contains("~= nil") {
                let suggestion = if trimmed.contains("== nil") {
                    "use `if not x then` instead of `if x == nil then`"
                } else {
                    "use `if x then` instead of `if x ~= nil then`"
                };
                return vec![Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "explicit nil comparison can be simplified".to_string(),
                    span: span_from_node(stmt),
                    suggestion: Some(suggestion.to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct NegatedConditionRule;
impl Rule for NegatedConditionRule {
    fn id(&self) -> &'static str { "I021" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Negated if condition with else block can be flipped" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::If(if_stmt) = stmt {
            if if_stmt.else_block().is_some() && if_stmt.else_if().is_none() {
                let cond = format!("{}", if_stmt.condition());
                if cond.trim().starts_with("not ") || cond.trim().starts_with("not(") {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "if not ... then ... else ... end can be simplified by flipping the condition".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("remove the `not` and swap the if/else bodies".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct MathHugeComparisonRule;
impl Rule for MathHugeComparisonRule {
    fn id(&self) -> &'static str { "I022" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Comparison with math.huge" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::If(if_stmt) = stmt {
            let cond = format!("{}", if_stmt.condition());
            if cond.contains("math.huge") && (cond.contains("==") || cond.contains("~=")) {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "direct comparison with math.huge is usually a sign of a logic issue".to_string(),
                    span: span_from_node(stmt),
                    suggestion: Some("for NaN checks use x ~= x, for infinity use math.abs(x) == math.huge".to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct InconsistentReturnRule;
impl Rule for InconsistentReturnRule {
    fn id(&self) -> &'static str { "I024" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Function has inconsistent return - some paths return values, some do not" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_function_body(&self, body: &ast::FunctionBody, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let block = body.block();
        let mut returns_with_value = 0;
        let mut returns_without_value = 0;

        self.count_returns(block, &mut returns_with_value, &mut returns_without_value);

        if returns_with_value > 0 && returns_without_value > 0 {
            return vec![Diagnostic {
                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: format!("function has {} returns with values and {} bare returns - this is inconsistent",
                    returns_with_value, returns_without_value),
                span: span_from_node(body),
                suggestion: Some("ensure all return paths return a value, or none do".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}
impl InconsistentReturnRule {
    fn count_returns(&self, block: &ast::Block, with_val: &mut usize, without_val: &mut usize) {
        if let Some(last) = block.last_stmt() {
            if let ast::LastStmt::Return(ret) = last {
                if ret.returns().is_empty() {
                    *without_val += 1;
                } else {
                    *with_val += 1;
                }
            }
        }
        for stmt in block.stmts() {
            if let ast::Stmt::If(if_stmt) = stmt {
                self.count_returns(if_stmt.block(), with_val, without_val);
                if let Some(else_block) = if_stmt.else_block() {
                    self.count_returns(else_block, with_val, without_val);
                }
                if let Some(else_ifs) = if_stmt.else_if() {
                    for else_if in else_ifs {
                        self.count_returns(else_if.block(), with_val, without_val);
                    }
                }
            }
        }
    }
}



#[derive(Debug)] pub struct VariableShadowingRule;
impl Rule for VariableShadowingRule {
    fn id(&self) -> &'static str { "I026" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Local variable shadows a variable from an outer scope" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut scope_stack: Vec<Vec<String>> = vec![Vec::new()];
        let mut depth: i32 = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }

            let opens = trimmed.matches("function").count()
                + trimmed.matches("do").count()
                + trimmed.matches("then").count()
                + trimmed.matches("repeat").count();
            let closes = trimmed.matches("end").count()
                + trimmed.matches("until").count();

            for name in Self::extract_local_names(trimmed) {
                if name == "_" || name == "self" { continue; }
                let shadowed = scope_stack.iter().take(depth as usize).any(|scope| scope.contains(&name));
                if shadowed {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("'{name}' shadows a variable from an outer scope"),
                        span: Some(Span { line: i + 1, column: 1 }),
                        suggestion: Some(format!("rename to a more specific name like '{name}2' or '{name}Result'")),
                        fixable: false,
                    });
                }
                if let Some(current) = scope_stack.get_mut(depth as usize) {
                    if !current.contains(&name) {
                        current.push(name);
                    }
                }
            }

            for _ in 0..opens {
                depth += 1;
                if scope_stack.len() <= depth as usize {
                    scope_stack.push(Vec::new());
                }
            }
            for _ in 0..closes {
                if depth > 0 {
                    if let Some(scope) = scope_stack.get_mut(depth as usize) {
                        scope.clear();
                    }
                    depth -= 1;
                }
            }
        }
        results
    }
}
impl VariableShadowingRule {
    fn extract_local_names(line: &str) -> Vec<String> {
        let mut names = Vec::new();
        let trimmed = line.trim();
        if !trimmed.starts_with("local ") { return names; }
        let after_local = &trimmed[6..];
        if after_local.starts_with("function ") { return names; }
        let decl_part = if let Some(eq_pos) = after_local.find(" = ") {
            &after_local[..eq_pos]
        } else if let Some(eq_pos) = after_local.find('=') {
            &after_local[..eq_pos]
        } else {
            after_local
        };
        for part in decl_part.split(',') {
            let name = part.trim().split(':').next().unwrap_or("").trim();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                names.push(name.to_string());
            }
        }
        names
    }
}

#[derive(Debug)] pub struct UnusedLocalRule;
impl Rule for UnusedLocalRule {
    fn id(&self) -> &'static str { "I027" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Local variable is assigned but never used" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut declarations: Vec<(String, usize)> = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            if !trimmed.starts_with("local ") { continue; }
            let after_local = &trimmed[6..];
            if after_local.starts_with("function ") { continue; }
            let decl_part = if let Some(eq_pos) = after_local.find('=') {
                &after_local[..eq_pos]
            } else {
                after_local
            };
            for part in decl_part.split(',') {
                let name = part.trim().split(':').next().unwrap_or("").trim().to_string();
                if !name.is_empty() && name != "_" && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    declarations.push((name, i + 1));
                }
            }
        }

        for (name, decl_line) in &declarations {
            let mut used = false;
            for (i, line) in lines.iter().enumerate() {
                if i + 1 == *decl_line { continue; }
                let trimmed = line.trim();
                if trimmed.starts_with("--") { continue; }
                if Self::line_references(trimmed, name) {
                    used = true;
                    break;
                }
            }
            if !used {
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("'{name}' is assigned but never used"),
                    span: Some(Span { line: *decl_line, column: 1 }),
                    suggestion: Some(format!("remove the unused variable or prefix with _ as '_{name}'")),
                    fixable: false,
                });
            }
        }
        results
    }
}
impl UnusedLocalRule {
    fn line_references(line: &str, name: &str) -> bool {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(name) {
            let abs_pos = search_from + pos;
            let before_ok = abs_pos == 0 || !line.as_bytes()[abs_pos - 1].is_ascii_alphanumeric() && line.as_bytes()[abs_pos - 1] != b'_';
            let after_pos = abs_pos + name.len();
            let after_ok = after_pos >= line.len() || !line.as_bytes()[after_pos].is_ascii_alphanumeric() && line.as_bytes()[after_pos] != b'_';
            if before_ok && after_ok {
                return true;
            }
            search_from = abs_pos + 1;
        }
        false
    }
}

#[derive(Debug)] pub struct RepeatedAccessChainRule;
impl Rule for RepeatedAccessChainRule {
    fn id(&self) -> &'static str { "I028" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Deep property access chain repeated 3+ times - cache in a local" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut chains: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            for chain in Self::extract_chains(trimmed) {
                chains.entry(chain).or_default().push(i + 1);
            }
        }

        let mut reported: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (chain, locations) in &chains {
            if locations.len() >= 3 && !reported.contains(chain) {
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("'{chain}' accessed {} times - cache in a local variable", locations.len()),
                    span: Some(Span { line: locations[0], column: 1 }),
                    suggestion: Some(format!("local cached = {chain}")),
                    fixable: false,
                });
                reported.insert(chain.clone());
            }
        }
        results
    }
}
impl RepeatedAccessChainRule {
    fn extract_chains(line: &str) -> Vec<String> {
        let mut chains = Vec::new();
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
                let start = i;
                let mut dots = 0;
                while i < len {
                    if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
                        i += 1;
                    } else if bytes[i] == b'.' && i + 1 < len && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_') {
                        dots += 1;
                        i += 1;
                    } else if bytes[i] == b'[' {
                        dots += 1;
                        let bracket_start = i;
                        i += 1;
                        let mut depth = 1;
                        while i < len && depth > 0 {
                            if bytes[i] == b'[' { depth += 1; }
                            if bytes[i] == b']' { depth -= 1; }
                            i += 1;
                        }
                        if depth > 0 { i = bracket_start + 1; break; }
                    } else {
                        break;
                    }
                }
                if dots >= 2 {
                    let chain = &line[start..i];
                    if !chain.starts_with("game:") && !chain.starts_with("self.") {
                        chains.push(chain.to_string());
                    }
                }
            } else {
                i += 1;
            }
        }
        chains
    }
}

const VAGUE_NAMES: &[&str] = &[
    "temp", "obj", "val", "value", "stuff",
    "thing", "item", "tbl", "str", "num", "func", "ret", "tmp", "res",
    "args", "params", "input", "output", "buf", "arr", "list", "map",
    "cb", "fn", "proc", "ref",
];

#[derive(Debug)] pub struct VagueVariableNameRule;
impl Rule for VagueVariableNameRule {
    fn id(&self) -> &'static str { "I029" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Variable name is vague and does not describe its contents" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        if let ast::Stmt::LocalAssignment(local) = stmt {
            for name in local.names() {
                let var_name = name.token().to_string();
                let lower = var_name.to_lowercase();
                if VAGUE_NAMES.contains(&lower.as_str()) {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("'{var_name}' is a vague name that does not describe what it holds"),
                        span: name.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                        suggestion: Some("use a descriptive name that tells the reader what the variable contains".to_string()),
                        fixable: false,
                    });
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct RedundantBooleanComparisonRule;
impl Rule for RedundantBooleanComparisonRule {
    fn id(&self) -> &'static str { "I030" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "Redundant comparison with true or false" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::If(if_stmt) = stmt {
            if let Some(diag) = self.check_condition(if_stmt.condition(), stmt) {
                return vec![diag];
            }
        }
        Vec::new()
    }
}
impl RedundantBooleanComparisonRule {
    fn check_condition(&self, expr: &ast::Expression, stmt: &ast::Stmt) -> Option<Diagnostic> {
        if let ast::Expression::BinaryOperator { lhs, binop, rhs } = expr {
            let is_eq = matches!(binop, ast::BinOp::TwoEqual(_));
            if !is_eq { return None; }
            let rhs_is_bool = self.is_boolean_literal(rhs);
            let lhs_is_bool = self.is_boolean_literal(lhs);
            if rhs_is_bool || lhs_is_bool {
                let bool_val = if rhs_is_bool {
                    format!("{rhs}").trim().to_string()
                } else {
                    format!("{lhs}").trim().to_string()
                };
                let suggestion = if bool_val == "true" {
                    "use `if x then` instead".to_string()
                } else {
                    "use `if not x then` instead".to_string()
                };
                return Some(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("redundant comparison with {bool_val} using =="),
                    span: span_from_node(stmt),
                    suggestion: Some(suggestion),
                    fixable: false,
                });
            }
        }
        None
    }

    fn is_boolean_literal(&self, expr: &ast::Expression) -> bool {
        if let ast::Expression::Symbol(sym) = expr {
            let val = sym.token().to_string();
            return val == "true" || val == "false";
        }
        false
    }
}

#[derive(Debug)] pub struct TaskSpawnClosureWrappingRule;
impl Rule for TaskSpawnClosureWrappingRule {
    fn id(&self) -> &'static str { "I031" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "task.spawn/defer wrapping a single call in a closure" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "task" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if suffixes.len() >= 2 {
                        if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                            let method_name = method.token().to_string();
                            if method_name == "spawn" || method_name == "defer" {
                                if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                    if arguments.len() == 1 {
                                        if let Some(ast::Expression::Function(anon_fn)) = arguments.iter().next() {
                                            let block = anon_fn.body().block();
                                            let stmt_count = block.stmts().count();
                                            let has_return = block.last_stmt().is_some();
                                            if stmt_count == 1 && !has_return {
                                                if let Some(ast::Stmt::FunctionCall(_)) = block.stmts().next() {
                                                    return vec![Diagnostic {
                                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                        message: format!("task.{method_name}(function() singleCall() end) creates an unnecessary closure"),
                                                        span: span_from_node(call),
                                                        suggestion: Some(format!("pass the function directly: task.{method_name}(myFunction, arg1, arg2)")),
                                                        fixable: false,
                                                    }];
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "task" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if suffixes.len() >= 2 {
                        if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                            let method_name = method.token().to_string();
                            if method_name == "spawn" || method_name == "defer" {
                                if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                    if arguments.len() == 1 {
                                        if let Some(ast::Expression::Function(anon_fn)) = arguments.iter().next() {
                                            let block = anon_fn.body().block();
                                            let stmt_count = block.stmts().count();
                                            let has_return = block.last_stmt().is_some();
                                            if stmt_count == 1 && !has_return {
                                                if let Some(ast::Stmt::FunctionCall(_)) = block.stmts().next() {
                                                    return vec![Diagnostic {
                                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                        message: format!("task.{method_name}(function() singleCall() end) creates an unnecessary closure"),
                                                        span: span_from_node(stmt),
                                                        suggestion: Some(format!("pass the function directly: task.{method_name}(myFunction, arg1, arg2)")),
                                                        fixable: false,
                                                    }];
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DuplicateGetServiceRule;
impl Rule for DuplicateGetServiceRule {
    fn id(&self) -> &'static str { "I032" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Same service obtained via GetService() more than once" }
    fn tier(&self) -> &'static str { "Intermediate" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut service_lines: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            if let Some(gs_pos) = trimmed.find("GetService(") {
                let after = &trimmed[gs_pos + "GetService(".len()..];
                if let Some(close) = after.find(')') {
                    let service = after[..close].trim().trim_matches('"').trim_matches('\'');
                    if !service.is_empty() {
                        service_lines.entry(service.to_string()).or_default().push(i + 1);
                    }
                }
            }
        }

        for (service, lines) in &service_lines {
            if lines.len() >= 2 {
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("GetService(\"{service}\") called {} times, declare it once at the top of the file", lines.len()),
                    span: Some(Span { line: lines[1], column: 1 }),
                    suggestion: Some(format!("local {service} = game:GetService(\"{service}\") at the top, then reuse the variable")),
                    fixable: false,
                });
            }
        }
        results
    }
}

