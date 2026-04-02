use full_moon::ast::{self, Prefix, Suffix, Call, Index, BinOp};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::fixer::Fix;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

fn span_from_node(node: &impl Node) -> Option<Span> {
    node.start_position().map(|pos| Span { line: pos.line(), column: pos.character() })
}

#[derive(Debug)] pub struct InstanceNewInLoopRule;
impl Rule for InstanceNewInLoopRule {
    fn id(&self) -> &'static str { "A001" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "Instance.new() inside loops - use object pooling" }
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
                                    message: "Instance.new() inside a loop - use object pooling instead".to_string(),
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
                        message: "connection created but not stored - will leak memory if not cleaned up".to_string(),
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
                    message: "string concatenation in a loop - use table.insert() and table.concat()".to_string(),
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

#[derive(Debug)] pub struct WhileTrueNoYieldRule;
impl Rule for WhileTrueNoYieldRule {
    fn id(&self) -> &'static str { "A005" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "while true do without yield - will freeze the game" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::While(while_stmt) = stmt {
            if is_literal_true(while_stmt.condition()) {
                if !block_has_yield(while_stmt.block(), ctx) {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "while true do loop without any yield call - this will freeze/crash the game".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("add task.wait() or another yield inside the loop body".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        if let ast::Stmt::Repeat(repeat_stmt) = stmt {
            if is_literal_false(repeat_stmt.until()) {
                if !block_has_yield(repeat_stmt.block(), ctx) {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "repeat ... until false without any yield call - this will freeze/crash the game".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("add task.wait() or another yield inside the loop body".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

fn is_literal_true(expr: &ast::Expression) -> bool {
    matches!(expr, ast::Expression::Symbol(sym) if sym.token().to_string() == "true")
}

fn is_literal_false(expr: &ast::Expression) -> bool {
    matches!(expr, ast::Expression::Symbol(sym) if sym.token().to_string() == "false")
}

fn block_has_yield(block: &ast::Block, ctx: &AnalysisContext) -> bool {
    if let (Some(start), Some(end)) = (block.start_position(), block.end_position()) {
        let lines: Vec<&str> = ctx.source.lines().collect();
        let start_line = start.line().saturating_sub(1);
        let end_line = end.line().min(lines.len());
        for line in &lines[start_line..end_line] {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            if trimmed.contains("task.wait") || trimmed.contains("task.delay")
                || trimmed.contains("task.defer") || trimmed.contains("wait(")
                || trimmed.contains("coroutine.yield") || trimmed.contains(":Wait()")
                || trimmed.contains(":wait()") {
                return true;
            }
        }
    }
    false
}

#[derive(Debug)] pub struct TableInsertFrontInLoopRule;
impl Rule for TableInsertFrontInLoopRule {
    fn id(&self) -> &'static str { "A006" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "table.insert() at position 1 inside a loop - O(n²) performance" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl TableInsertFrontInLoopRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "table" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                    if method.token().to_string() == "insert" {
                        if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                            if arguments.len() == 3 {
                                if let Some(second_arg) = arguments.iter().nth(1) {
                                    if let ast::Expression::Number(n) = second_arg {
                                        if n.token().to_string().trim() == "1" {
                                            return vec![Diagnostic {
                                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                message: "table.insert(t, 1, v) inside a loop - O(n²) performance, consider building in reverse or using table.move()".to_string(),
                                                span: span_from_node(call),
                                                suggestion: Some("build the list in order and reverse at the end, or use a different data structure".to_string()),
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

#[derive(Debug)] pub struct ConnectInLoopRule;
impl Rule for ConnectInLoopRule {
    fn id(&self) -> &'static str { "A007" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Memory Management" }
    fn description(&self) -> &'static str { ":Connect() inside a loop - creates multiple connections" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl ConnectInLoopRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let name = method.name().token().to_string();
                if name == "Connect" || name == "Once" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!(":{name}() inside a loop - creates a new connection every iteration, likely a memory leak"),
                        span: span_from_node(call),
                        suggestion: Some("move the connection outside the loop, or store and manage connections".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct PcallNoCheckRule;
impl Rule for PcallNoCheckRule {
    fn id(&self) -> &'static str { "A008" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Error Handling" }
    fn description(&self) -> &'static str { "pcall/xpcall result not checked - errors silently swallowed" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                let fn_name = name.token().to_string();
                if fn_name == "pcall" || fn_name == "xpcall" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("{fn_name}() called as a statement - the success/error result is silently discarded"),
                            span: span_from_node(stmt),
                            suggestion: Some(format!("local success, err = {fn_name}(fn); if not success then warn(err) end")),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct CloneNotStoredRule;
impl Rule for CloneNotStoredRule {
    fn id(&self) -> &'static str { "A009" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Memory Management" }
    fn description(&self) -> &'static str { ":Clone() result not stored - cloned instance immediately garbage collected" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            let suffixes: Vec<_> = call.suffixes().collect();
            if let Some(Suffix::Call(Call::MethodCall(method))) = suffixes.last() {
                if method.name().token().to_string() == "Clone" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: ":Clone() called but result not stored - the cloned instance is immediately lost".to_string(),
                        span: span_from_node(stmt),
                        suggestion: Some("local cloned = original:Clone()".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedLoadAnimationRule;
impl Rule for DeprecatedLoadAnimationRule {
    fn id(&self) -> &'static str { "A010" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { ":LoadAnimation() on Humanoid is deprecated - use Animator:LoadAnimation()" }
    fn tier(&self) -> &'static str { "Advanced" }
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
        if line.contains(":LoadAnimation(") {
            return Some(Fix {
                description: "update LoadAnimation to use Animator".to_string(),
                line: line_num,
                replacement: line.replacen(":LoadAnimation(", ":FindFirstChildOfClass(\"Animator\"):LoadAnimation(", 1),
            });
        }
        None
    }
}
impl DeprecatedLoadAnimationRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "LoadAnimation" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: ":LoadAnimation() is deprecated - use Animator:LoadAnimation() from the Humanoid or AnimationController".to_string(),
                        span: span_from_node(call),
                        suggestion: Some("humanoid:FindFirstChildOfClass(\"Animator\"):LoadAnimation(anim)".to_string()),
                        fixable: true,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedSetPrimaryPartCFrameRule;
impl Rule for DeprecatedSetPrimaryPartCFrameRule {
    fn id(&self) -> &'static str { "A011" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { ":SetPrimaryPartCFrame() is deprecated - use :PivotTo()" }
    fn tier(&self) -> &'static str { "Advanced" }
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
            description: "replace SetPrimaryPartCFrame with PivotTo".to_string(),
            line: line_num,
            replacement: line.replacen("SetPrimaryPartCFrame", "PivotTo", 1),
        })
    }
}
impl DeprecatedSetPrimaryPartCFrameRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "SetPrimaryPartCFrame" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: ":SetPrimaryPartCFrame() is deprecated - use :PivotTo() instead".to_string(),
                        span: span_from_node(call),
                        suggestion: Some("model:PivotTo(cframe)".to_string()),
                        fixable: true,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedMouseApiRule;
impl Rule for DeprecatedMouseApiRule {
    fn id(&self) -> &'static str { "A012" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { ":GetMouse() is deprecated - use UserInputService" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl DeprecatedMouseApiRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "GetMouse" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: ":GetMouse() is deprecated - use UserInputService or ContextActionService instead".to_string(),
                        span: span_from_node(call),
                        suggestion: Some("game:GetService(\"UserInputService\")".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct UnreachableCodeRule;
impl Rule for UnreachableCodeRule {
    fn id(&self) -> &'static str { "A013" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Code after return, break, or continue is unreachable" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut prev_was_terminal = false;
        let mut terminal_line = 0;
        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") || trimmed.is_empty() {
                continue;
            }
            if prev_was_terminal {
                if trimmed == "end" || trimmed == "else" || trimmed == "elseif"
                    || trimmed.starts_with("end)") || trimmed.starts_with("end,")
                    || trimmed == "until" {
                    prev_was_terminal = false;
                    continue;
                }
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("this code is unreachable after line {}", terminal_line),
                    span: Some(Span { line: i + 1, column: 1 }),
                    suggestion: Some("remove the unreachable code or restructure the control flow".to_string()),
                    fixable: false,
                });
                prev_was_terminal = false;
            }
            if trimmed.starts_with("return") || trimmed == "break" || trimmed == "continue"
                || trimmed.starts_with("error(") {
                prev_was_terminal = true;
                terminal_line = i + 1;
            }
        }
        results
    }
}

#[derive(Debug)] pub struct TableRemoveForwardLoopRule;
impl Rule for TableRemoveForwardLoopRule {
    fn id(&self) -> &'static str { "A014" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "table.remove() in a forward for loop skips elements" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "table" {
                    for suffix in call.suffixes() {
                        if let Suffix::Index(Index::Dot { name: method, .. }) = suffix {
                            if method.token().to_string() == "remove" {
                                return vec![Diagnostic {
                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                    message: "table.remove() in a forward loop will skip elements - iterate in reverse instead".to_string(),
                                    span: span_from_node(call),
                                    suggestion: Some("use `for i = #t, 1, -1 do` when removing elements".to_string()),
                                    fixable: false,
                                }];
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DeprecatedTickRule;
impl Rule for DeprecatedTickRule {
    fn id(&self) -> &'static str { "A017" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "tick() is deprecated - use os.clock() or workspace:GetServerTimeNow()" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn is_fixable(&self) -> bool { true }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "tick" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "tick() is deprecated - use os.clock() for benchmarking or workspace:GetServerTimeNow() for timestamps".to_string(),
                            span: span_from_node(call),
                            suggestion: Some("os.clock()".to_string()),
                            fixable: true,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "tick" {
                    let suffixes: Vec<_> = call.suffixes().collect();
                    if matches!(suffixes.first(), Some(Suffix::Call(Call::AnonymousCall(_)))) && suffixes.len() == 1 {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: "tick() is deprecated - use os.clock() for benchmarking or workspace:GetServerTimeNow() for timestamps".to_string(),
                            span: span_from_node(call),
                            suggestion: Some("os.clock()".to_string()),
                            fixable: true,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
    fn fix(&self, source: &str, diagnostic: &Diagnostic) -> Option<Fix> {
        let line_num = diagnostic.span.as_ref()?.line;
        let line = source.lines().nth(line_num - 1)?;
        Some(Fix {
            description: "replace tick() with os.clock()".to_string(),
            line: line_num,
            replacement: line.replacen("tick()", "os.clock()", 1),
        })
    }
}

#[derive(Debug)] pub struct DeprecatedTweenSizeRule;
impl Rule for DeprecatedTweenSizeRule {
    fn id(&self) -> &'static str { "A018" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { ":TweenPosition/:TweenSize is deprecated - use TweenService" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl DeprecatedTweenSizeRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let name = method.name().token().to_string();
                if name == "TweenPosition" || name == "TweenSize" || name == "TweenSizeAndPosition" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!(":{name}() is deprecated - use TweenService:Create() instead"),
                        span: span_from_node(call),
                        suggestion: Some("TweenService:Create(guiObject, TweenInfo.new(...), {Size = ...})".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct DebrisNegativeTimeRule;
impl Rule for DebrisNegativeTimeRule {
    fn id(&self) -> &'static str { "A019" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Debris:AddItem() with zero or negative lifetime" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl DebrisNegativeTimeRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "AddItem" {
                    if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                        if arguments.len() >= 2 {
                            if let Some(time_expr) = arguments.iter().nth(1) {
                                if let Some(val) = extract_number_from_expr(time_expr) {
                                    if val <= 0.0 {
                                        return vec![Diagnostic {
                                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                            message: format!("Debris:AddItem() with lifetime {val} - object will be destroyed immediately or never"),
                                            span: span_from_node(call),
                                            suggestion: Some("use a positive lifetime value".to_string()),
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
        Vec::new()
    }
}

#[derive(Debug)] pub struct StringFormatMismatchRule;
impl Rule for StringFormatMismatchRule {
    fn id(&self) -> &'static str { "A020" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "string.format() specifier count does not match argument count" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl StringFormatMismatchRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "string" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                    if method.token().to_string() == "format" {
                        if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                            let arg_count = arguments.len();
                            if arg_count >= 1 {
                                if let Some(ast::Expression::String(fmt_str)) = arguments.iter().next() {
                                    let fmt = fmt_str.token().to_string();
                                    let specifier_count = self.count_specifiers(&fmt);
                                    let value_count = arg_count - 1;
                                    if specifier_count != value_count {
                                        return vec![Diagnostic {
                                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                            message: format!("string.format has {specifier_count} specifiers but {value_count} values provided"),
                                            span: span_from_node(call),
                                            suggestion: Some("ensure the format string specifiers match the number of arguments".to_string()),
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
        Vec::new()
    }

    fn count_specifiers(&self, fmt: &str) -> usize {
        let mut count = 0;
        let mut chars = fmt.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '%' {
                if let Some(&next) = chars.peek() {
                    if next == '%' {
                        chars.next();
                    } else if "diouxXeEfgGqsc".contains(next) {
                        count += 1;
                        chars.next();
                    } else if next.is_ascii_digit() || next == '-' || next == '+' || next == ' ' || next == '.' {
                        chars.next();
                        while let Some(&c) = chars.peek() {
                            if "diouxXeEfgGqsc".contains(c) {
                                count += 1;
                                chars.next();
                                break;
                            } else if c.is_ascii_digit() || c == '.' {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
        count
    }
}

#[derive(Debug)] pub struct FindFirstChildInLoopRule;
impl Rule for FindFirstChildInLoopRule {
    fn id(&self) -> &'static str { "A021" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Performance" }
    fn description(&self) -> &'static str { "FindFirstChild() inside a loop - cache the result" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
    fn check_expression(&self, expr: &ast::Expression, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if !ctx.in_loop() { return Vec::new(); }
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
}
impl FindFirstChildInLoopRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let name = method.name().token().to_string();
                if name == "FindFirstChild" || name == "FindFirstChildOfClass" || name == "FindFirstChildWhichIsA" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!(":{name}() inside a loop - cache the result before the loop"),
                        span: span_from_node(call),
                        suggestion: Some("move the FindFirstChild call before the loop and store in a local".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct GlobalWriteRule;
impl Rule for GlobalWriteRule {
    fn id(&self) -> &'static str { "A022" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Writing to global scope without local keyword" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::Assignment(assignment) = stmt {
            for var in assignment.variables() {
                if let ast::Var::Name(name) = var {
                    let var_name = name.token().to_string();
                    let known_globals = ["game", "workspace", "script", "plugin", "shared", "_G"];
                    if !known_globals.contains(&var_name.as_str()) {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("'{var_name}' assigned without local keyword - this writes to global scope"),
                            span: span_from_node(stmt),
                            suggestion: Some(format!("use `local {var_name} = ...` instead")),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

fn extract_number_from_expr(expr: &ast::Expression) -> Option<f64> {
    match expr {
        ast::Expression::Number(token) => token.token().to_string().parse::<f64>().ok(),
        ast::Expression::UnaryOperator { unop, expression } => {
            if matches!(unop, ast::UnOp::Minus(_)) {
                extract_number_from_expr(expression).map(|v| -v)
            } else {
                None
            }
        }
        _ => None,
    }
}

const DEPRECATED_BODY_MOVERS: &[(&str, &str)] = &[
    ("BodyVelocity", "LinearVelocity"),
    ("BodyAngularVelocity", "AngularVelocity"),
    ("BodyGyro", "AlignOrientation"),
    ("BodyForce", "VectorForce"),
    ("BodyPosition", "AlignPosition"),
    ("BodyThrust", "VectorForce"),
    ("RocketPropulsion", "LinearVelocity with AlignOrientation"),
];

#[derive(Debug)] pub struct DeprecatedBodyMoverRule;
impl Rule for DeprecatedBodyMoverRule {
    fn id(&self) -> &'static str { "A023" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "Deprecated BodyMover constraint - use modern constraint equivalents" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl DeprecatedBodyMoverRule {
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
                                        let class_name = s.token().to_string();
                                        let class_name = class_name.trim_matches('"').trim_matches('\'');
                                        for (deprecated, replacement) in DEPRECATED_BODY_MOVERS {
                                            if class_name == *deprecated {
                                                return vec![Diagnostic {
                                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                                    message: format!("{deprecated} is deprecated - use {replacement} instead"),
                                                    span: span_from_node(call),
                                                    suggestion: Some(format!("Instance.new(\"{replacement}\")")),
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

#[derive(Debug)] pub struct DirectHealthSetRule;
impl Rule for DirectHealthSetRule {
    fn id(&self) -> &'static str { "A024" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Direct Health assignment - consider using Humanoid:TakeDamage()" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }
            if trimmed.contains(".Health") && trimmed.contains("=") && trimmed.contains(".Health -") {
                if !trimmed.contains("local") && !trimmed.starts_with("--") {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "setting Health directly bypasses ForceField and armor systems - use :TakeDamage() instead".to_string(),
                        span: Some(Span { line: i + 1, column: 1 }),
                        suggestion: Some("humanoid:TakeDamage(amount)".to_string()),
                        fixable: false,
                    });
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct NestedPcallRule;
impl Rule for NestedPcallRule {
    fn id(&self) -> &'static str { "A025" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { "Nested pcall detected - restructure error handling" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut pcall_depth = 0i32;
        let mut pcall_start_lines: Vec<usize> = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }

            if trimmed.contains("pcall(") || trimmed.contains("xpcall(") {
                if pcall_depth > 0 {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "nested pcall/xpcall - consider restructuring error handling into separate functions".to_string(),
                        span: Some(Span { line: i + 1, column: 1 }),
                        suggestion: Some("extract the inner operation into its own function with its own error handling".to_string()),
                        fixable: false,
                    });
                }
                pcall_depth += 1;
                pcall_start_lines.push(i);
            }

            let ends = trimmed.matches("end").count();
            for _ in 0..ends {
                if pcall_depth > 0 {
                    pcall_depth -= 1;
                    pcall_start_lines.pop();
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct SetAsyncInPcallRule;
impl Rule for SetAsyncInPcallRule {
    fn id(&self) -> &'static str { "A026" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Data Persistence" }
    fn description(&self) -> &'static str { "SetAsync inside pcall wrapper - still unsafe for concurrent writes" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut in_pcall = false;
        let mut pcall_depth = 0i32;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }

            if trimmed.contains("pcall(function") || trimmed.contains("xpcall(function") {
                in_pcall = true;
                pcall_depth = 0;
            }

            if in_pcall {
                pcall_depth += trimmed.matches("function").count() as i32;
                pcall_depth -= trimmed.matches("end").count() as i32;

                if trimmed.contains(":SetAsync(") || trimmed.contains(":SetAsync (") {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: "SetAsync inside pcall - wrapping in pcall does not make SetAsync safe for concurrent access".to_string(),
                        span: Some(Span { line: i + 1, column: 1 }),
                        suggestion: Some("use :UpdateAsync() which handles concurrency with a transform function".to_string()),
                        fixable: false,
                    });
                }

                if pcall_depth <= 0 {
                    in_pcall = false;
                }
            }
        }
        results
    }
}

#[derive(Debug)] pub struct PcallErrorSwallowedRule;
impl Rule for PcallErrorSwallowedRule {
    fn id(&self) -> &'static str { "A027" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Error Handling" }
    fn description(&self) -> &'static str { "pcall error captured but never logged or used" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }

            if !trimmed.contains("pcall(") && !trimmed.contains("xpcall(") { continue; }
            if !trimmed.contains("local ") { continue; }

            let before_pcall = if let Some(eq_pos) = trimmed.find('=') {
                trimmed[..eq_pos].trim_start_matches("local").trim()
            } else {
                continue;
            };

            let parts: Vec<&str> = before_pcall.split(',').collect();
            if parts.len() < 2 { continue; }

            let err_var = parts[1].trim().split(':').next().unwrap_or("").trim();
            if err_var.is_empty() || err_var == "_" { continue; }

            let search_end = (i + 15).min(lines.len());
            let mut err_used = false;
            for check_line in &lines[i + 1..search_end] {
                let check_trimmed = check_line.trim();
                if check_trimmed.starts_with("--") { continue; }
                if Self::line_uses_var(check_trimmed, err_var) {
                    if check_trimmed.contains("warn(") || check_trimmed.contains("error(")
                        || check_trimmed.contains("print(") || check_trimmed.contains(&format!(".. {err_var}"))
                        || check_trimmed.contains(&format!("..{err_var}"))
                        || check_trimmed.contains(&format!(", {err_var}"))
                        || check_trimmed.contains(&format!("({err_var}"))
                        || check_trimmed.contains(&format!("tostring({err_var}"))
                    {
                        err_used = true;
                        break;
                    }
                }
            }

            if !err_used {
                results.push(Diagnostic {
                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                    message: format!("pcall error variable '{err_var}' is captured but never logged or used"),
                    span: Some(Span { line: i + 1, column: 1 }),
                    suggestion: Some(format!("log the error: warn(\"{err_var}:\", {err_var}) in the failure branch")),
                    fixable: false,
                });
            }
        }
        results
    }
}
impl PcallErrorSwallowedRule {
    fn line_uses_var(line: &str, var: &str) -> bool {
        let mut search_from = 0;
        while let Some(pos) = line[search_from..].find(var) {
            let abs_pos = search_from + pos;
            let before_ok = abs_pos == 0
                || (!line.as_bytes()[abs_pos - 1].is_ascii_alphanumeric() && line.as_bytes()[abs_pos - 1] != b'_');
            let after_pos = abs_pos + var.len();
            let after_ok = after_pos >= line.len()
                || (!line.as_bytes()[after_pos].is_ascii_alphanumeric() && line.as_bytes()[after_pos] != b'_');
            if before_ok && after_ok {
                return true;
            }
            search_from = abs_pos + 1;
        }
        false
    }
}

#[derive(Debug)] pub struct ConnectWhenOnceSufficesRule;
impl Rule for ConnectWhenOnceSufficesRule {
    fn id(&self) -> &'static str { "A028" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Quality" }
    fn description(&self) -> &'static str { ":Connect() callback immediately disconnects itself, use :Once() instead" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") { continue; }

            if !trimmed.contains(":Connect(function") { continue; }

            let conn_var = if let Some(eq_pos) = trimmed.find('=') {
                let before = trimmed[..eq_pos].trim();
                let name = before.trim_start_matches("local").trim();
                if name.contains(' ') || name.is_empty() { continue; }
                name.to_string()
            } else {
                continue;
            };

            let search_end = (i + 20).min(lines.len());
            for check_line in &lines[i + 1..search_end] {
                let check_trimmed = check_line.trim();
                if check_trimmed.starts_with("--") { continue; }
                if check_trimmed == "end)" || check_trimmed == "end" { break; }
                if check_trimmed.contains(&format!("{conn_var}:Disconnect()"))
                    || check_trimmed.contains(&format!("{conn_var}:disconnect()"))
                {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("connection '{conn_var}' is disconnected inside its own callback, use :Once() instead"),
                        span: Some(Span { line: i + 1, column: 1 }),
                        suggestion: Some("replace :Connect() with :Once() and remove the manual :Disconnect() call".to_string()),
                        fixable: false,
                    });
                    break;
                }
            }
        }
        results
    }
}

