use full_moon::ast::{self, Suffix, Index, Prefix, Call};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

fn span_from_node(node: &impl Node) -> Option<Span> {
    node.start_position().map(|pos| Span { line: pos.line(), column: pos.character() })
}

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
                message: "--!strict is missing - add it to the top of every script".to_string(),
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
    fn description(&self) -> &'static str { "Parent = nil without :Destroy() - causes memory leaks" }
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
                                            message: "setting Parent = nil without :Destroy() - use :Destroy() to properly clean up".to_string(),
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
                        message: "require() inside a loop - cache modules at the top of the file".to_string(),
                        span: call.start_position().map(|p| Span { line: p.line(), column: p.character() }),
                        suggestion: Some("local Module = require(...) -- at file top".to_string()),
                        fixable: false,
                    }];
                }
        Vec::new()
    }
}

#[derive(Debug)] pub struct GetServiceWorkspaceRule;
impl Rule for GetServiceWorkspaceRule {
    fn id(&self) -> &'static str { "F004" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "game:GetService(\"Workspace\") - use the `workspace` global" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn is_fixable(&self) -> bool { true }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        let fixed = line
            .replace("game:GetService(\"Workspace\")", "workspace")
            .replace("game:GetService('Workspace')", "workspace");
        Some(Fix {
            description: "replace game:GetService(\"Workspace\") with workspace".to_string(),
            line: line_num,
            replacement: fixed,
        })
    }
}
impl GetServiceWorkspaceRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "game" {
                for suffix in call.suffixes() {
                    if let Suffix::Call(Call::MethodCall(method)) = suffix {
                        if method.name().token().to_string() == "GetService" {
                            if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                                if let Some(arg) = arguments.iter().next() {
                                    if let ast::Expression::String(s) = arg {
                                        let val = s.token().to_string();
                                        let unquoted = val.trim_matches('"').trim_matches('\'');
                                        if unquoted == "Workspace" {
                                            return vec![Diagnostic {
                                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                message: "game:GetService(\"Workspace\") is redundant - use the `workspace` global".to_string(),
                                                span: span_from_node(call),
                                                suggestion: Some("workspace".to_string()),
                                                fixable: true,
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
        Vec::new()
    }
}

#[derive(Debug)] pub struct FindFirstChildChainRule;
impl Rule for FindFirstChildChainRule {
    fn id(&self) -> &'static str { "F005" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Chained call on FindFirstChild result - will error if child is nil" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl FindFirstChildChainRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        let suffixes: Vec<_> = call.suffixes().collect();
        for (i, suffix) in suffixes.iter().enumerate() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let name = method.name().token().to_string();
                if name == "FindFirstChild" || name == "FindFirstChildOfClass" || name == "FindFirstChildWhichIsA" {
                    if i + 1 < suffixes.len() {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!(":{name}() result used directly in a chain - will error if the child doesn't exist (returns nil)"),
                            span: span_from_node(call),
                            suggestion: Some(format!("store the result in a variable and check for nil first:\nlocal child = obj:{name}(\"Name\")\nif child then ... end")),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedRemoveRule;
impl Rule for DeprecatedRemoveRule {
    fn id(&self) -> &'static str { "F006" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { ":Remove() is deprecated - use :Destroy() instead" }
    fn tier(&self) -> &'static str { "Front Page" }
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
        Some(Fix {
            description: "replace :Remove() with :Destroy()".to_string(),
            line: line_num,
            replacement: line.replacen(":Remove(", ":Destroy(", 1),
        })
    }
}
impl DeprecatedRemoveRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "Remove" {
                    let arg_count = match method.args() {
                        ast::FunctionArgs::Parentheses { arguments, .. } => arguments.len(),
                        _ => 1,
                    };
                    if arg_count == 0 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: ":Remove() is deprecated - use :Destroy() to properly clean up instances".to_string(),
                            span: span_from_node(call),
                            suggestion: Some("instance:Destroy()".to_string()),
                            fixable: true,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct StringSubZeroIndexRule;
impl Rule for StringSubZeroIndexRule {
    fn id(&self) -> &'static str { "F007" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "string.sub() with index 0 - Lua/Luau strings are 1-indexed" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "string" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if suffixes.len() >= 2 {
                        if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                            if method.token().to_string() == "sub" {
                                if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                    if let Some(second_arg) = arguments.iter().nth(1) {
                                        if let ast::Expression::Number(n) = second_arg {
                                            if n.token().to_string().trim() == "0" {
                                                return vec![Diagnostic {
                                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                    message: "string.sub() with index 0 - Lua strings are 1-indexed, 0 is treated as 1".to_string(),
                                                    span: span_from_node(expr),
                                                    suggestion: Some("use index 1 for the start of the string".to_string()),
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

#[derive(Debug)] pub struct TaskWaitNegativeRule;
impl Rule for TaskWaitNegativeRule {
    fn id(&self) -> &'static str { "F008" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "task.wait() with negative delay - probably a bug" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl TaskWaitNegativeRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "task" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if suffixes.len() >= 2 {
                    if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                        let method_name = method.token().to_string();
                        if method_name == "wait" || method_name == "delay" {
                            if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                if let Some(first_arg) = arguments.iter().next() {
                                    if let ast::Expression::UnaryOperator { unop, expression } = first_arg {
                                        if matches!(unop, ast::UnOp::Minus(_)) {
                                            if matches!(expression.as_ref(), ast::Expression::Number(_)) {
                                                return vec![Diagnostic {
                                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                    message: format!("task.{method_name}() with negative delay - this is probably a bug"),
                                                    span: span_from_node(call),
                                                    suggestion: Some("use a positive delay value".to_string()),
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

#[derive(Debug)] pub struct InstanceNewEmptyStringRule;
impl Rule for InstanceNewEmptyStringRule {
    fn id(&self) -> &'static str { "F009" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Instance.new(\"\") - empty className will error at runtime" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl InstanceNewEmptyStringRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "Instance" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if suffixes.len() >= 2 {
                    if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                        if method.token().to_string() == "new" {
                            if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                if let Some(first_arg) = arguments.iter().next() {
                                    if let ast::Expression::String(s) = first_arg {
                                        let val = s.token().to_string();
                                        let unquoted = val.trim_matches('"').trim_matches('\'');
                                        if unquoted.is_empty() {
                                            return vec![Diagnostic {
                                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                message: "Instance.new(\"\") - empty className will error at runtime".to_string(),
                                                span: span_from_node(call),
                                                suggestion: Some("provide a valid class name: Instance.new(\"Part\")".to_string()),
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
        Vec::new()
    }
}

#[derive(Debug)] pub struct RenderSteppedOnServerRule;
impl Rule for RenderSteppedOnServerRule {
    fn id(&self) -> &'static str { "F010" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "RenderStepped only fires on client - this will never run on server" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::Var(ast::Var::Expression(var_expr)) = expr {
            let full = format!("{var_expr}");
            if full.contains("RenderStepped") {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "RenderStepped only fires on the client - on a server script this event will never fire".to_string(),
                    span: span_from_node(expr),
                    suggestion: Some("use Heartbeat or Stepped for server-side per-frame logic".to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct WaitReturnValueRule;
impl Rule for WaitReturnValueRule {
    fn id(&self) -> &'static str { "F011" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "task.wait() return value captured but rarely needed" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::LocalAssignment(local) = stmt {
            for expr in local.expressions() {
                if let ast::Expression::FunctionCall(call) = expr {
                    let full = format!("{call}");
                    let trimmed = full.trim();
                    if trimmed == "task.wait()" || trimmed.starts_with("task.wait(") {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "task.wait() return value is the actual elapsed time - this is rarely useful".to_string(),
                            span: span_from_node(stmt),
                            suggestion: Some("call task.wait() as a standalone statement unless you need the elapsed time".to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct ConnectWithNonFunctionRule;
impl Rule for ConnectWithNonFunctionRule {
    fn id(&self) -> &'static str { "F012" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { ":Connect() called with a non-function argument" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl ConnectWithNonFunctionRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let ast::Suffix::Call(ast::Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "Connect" {
                    if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                        if let Some(first_arg) = arguments.iter().next() {
                            match first_arg {
                                ast::Expression::Function(_) => {},
                                ast::Expression::Var(_) => {},
                                ast::Expression::FunctionCall(_) => {
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: ":Connect() received a function call result instead of a function reference".to_string(),
                                        span: span_from_node(call),
                                        suggestion: Some("pass a function reference: event:Connect(handler) not event:Connect(handler())".to_string()),
                                        fixable: false,
                                    }];
                                },
                                ast::Expression::String(_) | ast::Expression::Number(_) => {
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: ":Connect() expects a function but received a literal value".to_string(),
                                        span: span_from_node(call),
                                        suggestion: Some("pass a function: event:Connect(function() ... end)".to_string()),
                                        fixable: false,
                                    }];
                                },
                                _ => {},
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct TodoCommentRule;
impl Rule for TodoCommentRule {
    fn id(&self) -> &'static str { "F015" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "TODO/FIXME/HACK/XXX comment found" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let markers = ["TODO", "FIXME", "HACK", "XXX", "TEMP", "BUG"];
        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                let comment_body = trimmed.trim_start_matches('-').trim();
                for marker in &markers {
                    if comment_body.starts_with(marker) {
                        results.push(Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("{marker} comment found - resolve before shipping"),
                            span: Some(Span { line: i + 1, column: 1 }),
                            suggestion: Some("address the TODO or remove it if no longer relevant".to_string()),
                            fixable: false,
                        });
                        break;
                    }
                }
            }
        }
        results
    }
}

#[derive(Debug)]
pub struct LargeFileRule {
    pub max_lines: usize,
}
impl LargeFileRule {
    pub fn new(max_lines: usize) -> Self { Self { max_lines } }
}
impl Default for LargeFileRule {
    fn default() -> Self { Self { max_lines: 500 } }
}
impl Rule for LargeFileRule {
    fn id(&self) -> &'static str { "F016" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "File is very large - consider splitting into modules" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let line_count = ctx.source.lines().count();
        if line_count > self.max_lines {
            return vec![Diagnostic {
                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                message: format!("file is {line_count} lines long (max {}) - consider splitting into modules", self.max_lines),
                span: Some(Span { line: 1, column: 1 }),
                suggestion: Some("extract related functionality into ModuleScripts and use require()".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedFilteringEnabledRule;
impl Rule for DeprecatedFilteringEnabledRule {
    fn id(&self) -> &'static str { "B013" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "FilteringEnabled is always true and reading it is pointless" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        for (i, line) in ctx.source.lines().enumerate() {
            if line.contains("FilteringEnabled") && !line.trim().starts_with("--") {
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: "FilteringEnabled is always true since 2018 - checking it is pointless".to_string(),
                    span: Some(Span { line: i + 1, column: 1 }),
                    suggestion: Some("remove the FilteringEnabled check entirely".to_string()),
                    fixable: false,
                });
            }
        }
        results
    }
}

#[derive(Debug)] pub struct MissingTypeAnnotationRule;
impl Rule for MissingTypeAnnotationRule {
    fn id(&self) -> &'static str { "F017" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Function with 3+ parameters has no type annotations" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            let is_func = trimmed.starts_with("function ")
                || trimmed.starts_with("local function ")
                || trimmed.contains("= function(")
                || (trimmed.contains("function(") && (trimmed.contains(":") || trimmed.contains(".")));
            if !is_func { continue; }
            if let Some(open) = trimmed.find('(') {
                if let Some(close) = trimmed[open..].find(')') {
                    let params = &trimmed[open + 1..open + close];
                    if params.trim().is_empty() { continue; }
                    let param_parts: Vec<&str> = params.split(',').collect();
                    if param_parts.len() >= 3 {
                        let has_any_annotation = params.contains(':');
                        if !has_any_annotation {
                            results.push(Diagnostic {
                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                message: format!("function with {} parameters has no type annotations", param_parts.len()),
                                span: Some(Span { line: i + 1, column: 1 }),
                                suggestion: Some("add type annotations: function(param: Type, ...)".to_string()),
                                fixable: false,
                            });
                        }
                    }
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct HardcodedInstancePathRule;
impl Rule for HardcodedInstancePathRule {
    fn id(&self) -> &'static str { "F018" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Hardcoded instance path via long dot-chain is fragile" }
    fn tier(&self) -> &'static str { "Front Page" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::Var(ast::Var::Expression(var_expr)) = expr {
            if let Prefix::Name(name) = var_expr.prefix() {
                let prefix_name = name.token().to_string();
                if prefix_name != "workspace" && prefix_name != "game" { return Vec::new(); }

                let suffixes: Vec<_> = var_expr.suffixes().collect();
                let mut dot_count = 0;
                let mut has_method_call = false;
                for suffix in &suffixes {
                    match suffix {
                        Suffix::Index(Index::Dot { .. }) => dot_count += 1,
                        Suffix::Call(_) => { has_method_call = true; break; }
                        _ => break,
                    }
                }

                if dot_count >= 3 && !has_method_call {
                    let chain_parts: Vec<String> = std::iter::once(prefix_name.clone())
                        .chain(suffixes.iter().filter_map(|s| {
                            if let Suffix::Index(Index::Dot { name: prop, .. }) = s {
                                Some(prop.token().to_string())
                            } else {
                                None
                            }
                        }))
                        .collect();
                    let chain = chain_parts.join(".");

                    if chain.contains("GetService") { return Vec::new(); }

                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("hardcoded instance path '{chain}' is fragile and breaks if anything is renamed"),
                        span: span_from_node(expr),
                        suggestion: Some("use FindFirstChild/WaitForChild, CollectionService tags, or a configuration module".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

