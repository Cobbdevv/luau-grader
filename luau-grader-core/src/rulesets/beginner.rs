use full_moon::ast::{self, Prefix, Suffix, Call, Index};
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
    fn description(&self) -> &'static str { "Deprecated wait() - use task.wait()" }
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
    fn description(&self) -> &'static str { "Deprecated spawn() - use task.spawn()" }
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
    fn description(&self) -> &'static str { "Deprecated delay() - use task.delay()" }
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
                    message: "never use InvokeClient - it permanently yields if the client disconnects".to_string(),
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
        let mut search_start = 0;
        while let Some(rel_pos) = line[search_start..].find("WaitForChild(") {
            let wfc_pos = search_start + rel_pos;
            let after_open = wfc_pos + "WaitForChild(".len();
            let rest = &line[after_open..];
            let mut depth = 1u32;
            let mut has_comma = false;
            for (i, ch) in rest.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            if !has_comma {
                                let insert = after_open + i;
                                let mut fixed = line.to_string();
                                fixed.insert_str(insert, &format!(", {}", self.default_timeout));
                                return Some(Fix {
                                    description: format!("add timeout of {} seconds", self.default_timeout),
                                    line: line_num,
                                    replacement: fixed,
                                });
                            }
                            break;
                        }
                    }
                    ',' if depth == 1 => has_comma = true,
                    _ => {}
                }
            }
            search_start = after_open;
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
                            message: "WaitForChild() without a timeout - will yield forever if child never exists".to_string(),
                            span: span_from_node(call), suggestion: Some(format!(":WaitForChild(\"Name\", {})", self.default_timeout)),
                            fixable: true,
                        }];
                    }
                }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct InstanceNewParentArgRule;
impl Rule for InstanceNewParentArgRule {
    fn id(&self) -> &'static str { "B006" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "Instance.new() with parent argument - deprecated, set Parent separately" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Some(d) = self.check_call(call) { return vec![d]; }
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Some(d) = self.check_call(call) { return vec![d]; }
        }
        Vec::new()
    }
}
impl InstanceNewParentArgRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Option<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "Instance" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if suffixes.len() >= 2 {
                    if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                        if method.token().to_string() == "new" {
                            if let Some(Suffix::Call(Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }))) = suffixes.get(1) {
                                if arguments.len() >= 2 {
                                    return Some(Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: "Instance.new() with parent argument is deprecated - set .Parent separately for better performance".to_string(),
                                        span: span_from_node(call),
                                        suggestion: Some("local obj = Instance.new(\"ClassName\")\nobj.Parent = parent".to_string()),
                                        fixable: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

const DEPRECATED_LOWERCASE_METHODS: &[(&str, &str)] = &[
    ("connect", "Connect"),
    ("disconnect", "Disconnect"),
    ("children", "GetChildren"),
    ("getChildren", "GetChildren"),
    ("isA", "IsA"),
    ("findFirstChild", "FindFirstChild"),
    ("isDescendantOf", "IsDescendantOf"),
    ("isAncestorOf", "IsAncestorOf"),
    ("clone", "Clone"),
    ("destroy", "Destroy"),
    ("remove", "Destroy"),
    ("clearAllChildren", "ClearAllChildren"),
];

#[derive(Debug)] pub struct DeprecatedLowercaseMethodRule;
impl Rule for DeprecatedLowercaseMethodRule {
    fn id(&self) -> &'static str { "B007" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "Deprecated lowercase method alias - use PascalCase version" }
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
        for (deprecated, correct) in DEPRECATED_LOWERCASE_METHODS.iter() {
            let search = format!(":{deprecated}(");
            if line.contains(&search) {
                return Some(Fix {
                    description: format!("rename :{deprecated}() to :{correct}()"),
                    line: line_num,
                    replacement: line.replacen(&search, &format!(":{correct}("), 1),
                });
            }
        }
        None
    }
}
impl DeprecatedLowercaseMethodRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let method_name = method.name().token().to_string();
                if let Some((deprecated, correct)) = DEPRECATED_LOWERCASE_METHODS.iter().find(|(d, _)| *d == method_name) {
                    results.push(Diagnostic {
                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                        message: format!("deprecated lowercase method: :{deprecated}() - use :{correct}() instead"),
                        span: span_from_node(call),
                        suggestion: Some(format!(":{correct}()")),
                        fixable: true,
                    });
                }
            }
        }
        results
    }
}

const DEPRECATED_TABLE_FUNCS: &[(&str, &str)] = &[
    ("foreach", "use `for k, v in pairs(t) do` instead"),
    ("foreachi", "use `for i, v in ipairs(t) do` instead"),
    ("getn", "use the `#` length operator instead"),
];

#[derive(Debug)] pub struct DeprecatedTableFunctionRule;
impl Rule for DeprecatedTableFunctionRule {
    fn id(&self) -> &'static str { "B008" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "API Deprecation" }
    fn description(&self) -> &'static str { "Deprecated table function - table.foreach/foreachi/getn" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl DeprecatedTableFunctionRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "table" {
                let suffixes: Vec<_> = call.suffixes().collect();
                if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                    let method_name = method.token().to_string();
                    if let Some((_, suggestion)) = DEPRECATED_TABLE_FUNCS.iter().find(|(n, _)| *n == method_name) {
                        return vec![Diagnostic {
                            rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                            message: format!("table.{method_name}() is deprecated - {suggestion}"),
                            span: span_from_node(call),
                            suggestion: Some(suggestion.to_string()),
                            fixable: false,
                        }];
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)] pub struct GameDotWorkspaceRule;
impl Rule for GameDotWorkspaceRule {
    fn id(&self) -> &'static str { "B009" }
    fn severity(&self) -> Severity { Severity::Info }
    fn category(&self) -> &'static str { "Code Style" }
    fn description(&self) -> &'static str { "game.Workspace - use the `workspace` global instead" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn is_fixable(&self) -> bool { true }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::Var(ast::Var::Expression(var_expr)) = expr {
            if let Prefix::Name(name) = var_expr.prefix() {
                if name.token().to_string() == "game" {
                    let suffixes: Vec<_> = var_expr.suffixes().collect();
                    if let Some(Suffix::Index(Index::Dot { name: prop, .. })) = suffixes.first() {
                        if prop.token().to_string() == "Workspace" {
                            return vec![Diagnostic {
                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                message: "use the `workspace` global instead of `game.Workspace`".to_string(),
                                span: span_from_node(expr),
                                suggestion: Some("workspace".to_string()),
                                fixable: true,
                            }];
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
        Some(Fix {
            description: "replace game.Workspace with workspace".to_string(),
            line: line_num,
            replacement: line.replacen("game.Workspace", "workspace", 1),
        })
    }
}

const CONSTRUCTORS: &[(&str, &str, &[usize])] = &[
    ("Vector3", "new", &[0, 3]),
    ("Vector2", "new", &[0, 2]),
    ("CFrame", "new", &[0, 1, 2, 3, 7, 12]),
    ("CFrame", "lookAt", &[2, 3]),
    ("CFrame", "Angles", &[3]),
    ("CFrame", "fromEulerAnglesXYZ", &[3]),
    ("CFrame", "fromEulerAnglesYXZ", &[3]),
    ("CFrame", "fromAxisAngle", &[2]),
    ("CFrame", "fromMatrix", &[3, 4]),
    ("Color3", "new", &[0, 3]),
    ("Color3", "fromRGB", &[3]),
    ("Color3", "fromHSV", &[3]),
    ("Color3", "fromHex", &[1]),
    ("UDim", "new", &[2]),
    ("UDim2", "new", &[2, 4]),
    ("UDim2", "fromScale", &[2]),
    ("UDim2", "fromOffset", &[2]),
    ("TweenInfo", "new", &[0, 1, 2, 3, 4, 5, 6]),
    ("NumberRange", "new", &[1, 2]),
    ("NumberSequenceKeypoint", "new", &[2, 3]),
    ("ColorSequenceKeypoint", "new", &[2]),
    ("Rect", "new", &[2, 4]),
    ("Region3", "new", &[2]),
    ("Ray", "new", &[2]),
    ("Random", "new", &[0, 1]),
    ("Instance", "new", &[1, 2]),
    ("BrickColor", "new", &[1]),
    ("RaycastParams", "new", &[0]),
    ("OverlapParams", "new", &[0]),
    ("PhysicalProperties", "new", &[1, 3, 5]),
];

#[derive(Debug)] pub struct ConstructorArgCountRule;
impl Rule for ConstructorArgCountRule {
    fn id(&self) -> &'static str { "B010" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Constructor called with wrong number of arguments" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl ConstructorArgCountRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            let prefix_name = name.token().to_string();
            let suffixes: Vec<_> = call.suffixes().collect();
            if suffixes.len() >= 2 {
                if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                    let method_name = method.token().to_string();
                    if let Some(Suffix::Call(Call::AnonymousCall(args))) = suffixes.get(1) {
                        let arg_count = count_function_args(args);
                        for (pn, mn, valid) in CONSTRUCTORS {
                            if *pn == prefix_name && *mn == method_name {
                                if !valid.is_empty() && !valid.contains(&arg_count) {
                                    let valid_str = valid.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: format!("{prefix_name}.{method_name}() expects {valid_str} argument(s), got {arg_count}"),
                                        span: span_from_node(call),
                                        suggestion: None,
                                        fixable: false,
                                    }];
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

const KNOWN_METHODS: &[(&str, usize, usize)] = &[
    ("FindFirstChild", 1, 2),
    ("FindFirstChildOfClass", 1, 1),
    ("FindFirstChildWhichIsA", 1, 2),
    ("FindFirstAncestor", 1, 1),
    ("FindFirstAncestorOfClass", 1, 1),
    ("FindFirstAncestorWhichIsA", 1, 1),
    ("WaitForChild", 1, 2),
    ("GetChildren", 0, 0),
    ("GetDescendants", 0, 0),
    ("IsA", 1, 1),
    ("IsDescendantOf", 1, 1),
    ("IsAncestorOf", 1, 1),
    ("Destroy", 0, 0),
    ("Clone", 0, 0),
    ("ClearAllChildren", 0, 0),
    ("GetPropertyChangedSignal", 1, 1),
    ("GetAttribute", 1, 1),
    ("SetAttribute", 2, 2),
    ("GetAttributes", 0, 0),
    ("GetService", 1, 1),
    ("Connect", 1, 1),
    ("Once", 1, 1),
    ("Disconnect", 0, 0),
    ("Kick", 0, 1),
    ("MoveTo", 1, 1),
    ("GetPivot", 0, 0),
    ("PivotTo", 1, 1),
    ("Lerp", 2, 2),
    ("Raycast", 2, 3),
];

#[derive(Debug)] pub struct MethodArgCountRule;
impl Rule for MethodArgCountRule {
    fn id(&self) -> &'static str { "B011" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Method called with wrong number of arguments" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl MethodArgCountRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                let method_name = method.name().token().to_string();
                for (name, min, max) in KNOWN_METHODS {
                    if *name == method_name {
                        let arg_count = match method.args() {
                            ast::FunctionArgs::Parentheses { arguments, .. } => arguments.len(),
                            ast::FunctionArgs::String(_) => 1,
                            ast::FunctionArgs::TableConstructor(_) => 1,
                            _ => continue,
                        };
                        if arg_count < *min || arg_count > *max {
                            let expected = if min == max {
                                format!("{min}")
                            } else {
                                format!("{min}-{max}")
                            };
                            return vec![Diagnostic {
                                rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                message: format!(":{method_name}() expects {expected} argument(s), got {arg_count}"),
                                span: span_from_node(call),
                                suggestion: None,
                                fixable: false,
                            }];
                        }
                        break;
                    }
                }
            }
        }
        Vec::new()
    }
}

const STDLIB_FUNCTIONS: &[(&str, &str, usize, usize)] = &[
    ("string", "byte", 1, 3),
    ("string", "char", 1, usize::MAX),
    ("string", "find", 2, 4),
    ("string", "format", 1, usize::MAX),
    ("string", "gmatch", 2, 2),
    ("string", "gsub", 3, 4),
    ("string", "len", 1, 1),
    ("string", "lower", 1, 1),
    ("string", "upper", 1, 1),
    ("string", "match", 2, 3),
    ("string", "rep", 2, 3),
    ("string", "reverse", 1, 1),
    ("string", "sub", 2, 3),
    ("string", "split", 1, 2),
    ("table", "insert", 2, 3),
    ("table", "remove", 1, 2),
    ("table", "sort", 1, 2),
    ("table", "concat", 1, 4),
    ("table", "create", 1, 2),
    ("table", "find", 2, 3),
    ("table", "move", 4, 5),
    ("table", "clear", 1, 1),
    ("table", "clone", 1, 1),
    ("table", "freeze", 1, 1),
    ("table", "isfrozen", 1, 1),
    ("table", "pack", 0, usize::MAX),
    ("table", "unpack", 1, 3),
    ("math", "abs", 1, 1),
    ("math", "ceil", 1, 1),
    ("math", "floor", 1, 1),
    ("math", "sqrt", 1, 1),
    ("math", "log", 1, 2),
    ("math", "max", 1, usize::MAX),
    ("math", "min", 1, usize::MAX),
    ("math", "clamp", 3, 3),
    ("math", "noise", 1, 3),
    ("math", "round", 1, 1),
    ("math", "sign", 1, 1),
    ("math", "random", 0, 2),
    ("math", "randomseed", 1, 1),
    ("math", "atan2", 2, 2),
    ("math", "sin", 1, 1),
    ("math", "cos", 1, 1),
    ("math", "tan", 1, 1),
    ("math", "asin", 1, 1),
    ("math", "acos", 1, 1),
    ("math", "atan", 1, 1),
    ("math", "exp", 1, 1),
    ("math", "pow", 2, 2),
];

const GLOBAL_FUNCTIONS: &[(&str, usize, usize)] = &[
    ("pcall", 1, usize::MAX),
    ("xpcall", 2, usize::MAX),
    ("select", 1, usize::MAX),
    ("rawget", 2, 2),
    ("rawset", 3, 3),
    ("rawequal", 2, 2),
    ("rawlen", 1, 1),
    ("setmetatable", 2, 2),
    ("getmetatable", 1, 1),
    ("assert", 1, 2),
    ("error", 1, 2),
    ("ipairs", 1, 1),
    ("pairs", 1, 1),
    ("next", 1, 2),
    ("unpack", 1, 3),
    ("require", 1, 1),
    ("type", 1, 1),
    ("typeof", 1, 1),
    ("tostring", 1, 1),
    ("tonumber", 1, 2),
];

#[derive(Debug)] pub struct StdLibArgCountRule;
impl Rule for StdLibArgCountRule {
    fn id(&self) -> &'static str { "B012" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Common Bugs" }
    fn description(&self) -> &'static str { "Standard library function called with wrong number of arguments" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr { return self.check_call(call); }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt { return self.check_call(call); }
        Vec::new()
    }
}
impl StdLibArgCountRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            let prefix_name = name.token().to_string();
            let suffixes: Vec<_> = call.suffixes().collect();

            if suffixes.len() >= 2 {
                if let Some(Suffix::Index(Index::Dot { name: method, .. })) = suffixes.first() {
                    let method_name = method.token().to_string();
                    if let Some(Suffix::Call(Call::AnonymousCall(args))) = suffixes.get(1) {
                        let arg_count = count_function_args(args);
                        for (lib, func, min, max) in STDLIB_FUNCTIONS {
                            if *lib == prefix_name && *func == method_name {
                                if arg_count < *min || (*max != usize::MAX && arg_count > *max) {
                                    let expected = if *max == usize::MAX {
                                        format!("at least {min}")
                                    } else if min == max {
                                        format!("{min}")
                                    } else {
                                        format!("{min}-{max}")
                                    };
                                    return vec![Diagnostic {
                                        rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                        message: format!("{prefix_name}.{method_name}() expects {expected} argument(s), got {arg_count}"),
                                        span: span_from_node(call),
                                        suggestion: None,
                                        fixable: false,
                                    }];
                                }
                                break;
                            }
                        }
                    }
                }
            }

            if suffixes.len() == 1 {
                if let Some(Suffix::Call(Call::AnonymousCall(args))) = suffixes.first() {
                    let arg_count = count_function_args(args);
                    for (func, min, max) in GLOBAL_FUNCTIONS {
                        if *func == prefix_name {
                            if arg_count < *min || (*max != usize::MAX && arg_count > *max) {
                                let expected = if *max == usize::MAX {
                                    format!("at least {min}")
                                } else if min == max {
                                    format!("{min}")
                                } else {
                                    format!("{min}-{max}")
                                };
                                return vec![Diagnostic {
                                    rule_id: self.id().to_string(), severity: self.severity(), category: self.category().to_string(),
                                    message: format!("{prefix_name}() expects {expected} argument(s), got {arg_count}"),
                                    span: span_from_node(call),
                                    suggestion: None,
                                    fixable: false,
                                }];
                            }
                            break;
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

pub fn count_function_args(args: &ast::FunctionArgs) -> usize {
    match args {
        ast::FunctionArgs::Parentheses { arguments, .. } => arguments.len(),
        ast::FunctionArgs::String(_) => 1,
        ast::FunctionArgs::TableConstructor(_) => 1,
        _ => 0,
    }
}
