use full_moon::ast::{self, Prefix, Suffix, Call, Index};
use full_moon::node::Node;
use crate::analyzer::context::AnalysisContext;
use crate::report::{Diagnostic, Severity, Span};
use super::Rule;

fn span_from_node(node: &impl Node) -> Option<Span> {
    node.start_position().map(|pos| Span { line: pos.line(), column: pos.character() })
}

#[derive(Debug)]
pub struct UnvalidatedRemoteArgsRule;
impl Rule for UnvalidatedRemoteArgsRule {
    fn id(&self) -> &'static str { "S001" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "OnServerEvent handler does not validate arguments" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            return self.check_call(call);
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            return self.check_call(call);
        }
        Vec::new()
    }
}
impl UnvalidatedRemoteArgsRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        let suffixes: Vec<_> = call.suffixes().collect();
        for (i, suffix) in suffixes.iter().enumerate() {
            if let Suffix::Index(Index::Dot { name, .. }) = suffix {
                let prop_name = name.token().to_string();
                if prop_name == "OnServerEvent" || prop_name == "OnServerInvoke" {
                    if i + 1 < suffixes.len() {
                        if let Some(Suffix::Call(Call::MethodCall(method))) = suffixes.get(i + 1) {
                            if method.name().token().to_string() == "Connect" {
                                return vec![Diagnostic {
                                    rule_id: self.id().to_string(),
                                    severity: self.severity(),
                                    category: self.category().to_string(),
                                    message: "Remote event handler should validate all arguments from the client".to_string(),
                                    span: span_from_node(call),
                                    suggestion: Some("Check types and ranges of all arguments before using them".to_string()),
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

#[derive(Debug)]
pub struct TrustClientPositionRule;
impl Rule for TrustClientPositionRule {
    fn id(&self) -> &'static str { "S003" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "FireServer sending position or CFrame data (trust-the-client)" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            return self.check_call(call);
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            return self.check_call(call);
        }
        Vec::new()
    }
}
impl TrustClientPositionRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        for suffix in call.suffixes() {
            if let Suffix::Call(Call::MethodCall(method)) = suffix {
                if method.name().token().to_string() == "FireServer" {
                    if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                        for arg in arguments.iter() {
                            let arg_text = format!("{arg}");
                            if arg_text.contains("Position") || arg_text.contains("CFrame") || arg_text.contains("HumanoidRootPart") {
                                return vec![Diagnostic {
                                    rule_id: self.id().to_string(),
                                    severity: self.severity(),
                                    category: self.category().to_string(),
                                    message: "Sending position/CFrame data via FireServer. The server should calculate positions itself".to_string(),
                                    span: span_from_node(call),
                                    suggestion: Some("Have the server compute positions from its own data".to_string()),
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

#[derive(Debug)]
pub struct LoadstringUsageRule;
impl Rule for LoadstringUsageRule {
    fn id(&self) -> &'static str { "S004" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "loadstring is a security risk and is disabled by default" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "loadstring" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        category: self.category().to_string(),
                        message: "loadstring is disabled in Roblox by default and is a security risk".to_string(),
                        span: span_from_node(call),
                        suggestion: Some("Use ModuleScripts for dynamic code loading".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            if let Prefix::Name(name) = call.prefix() {
                if name.token().to_string() == "loadstring" {
                    return vec![Diagnostic {
                        rule_id: self.id().to_string(),
                        severity: self.severity(),
                        category: self.category().to_string(),
                        message: "loadstring is disabled in Roblox by default and is a security risk".to_string(),
                        span: span_from_node(call),
                        suggestion: Some("Use ModuleScripts for dynamic code loading".to_string()),
                        fixable: false,
                    }];
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct HttpNoPcallRule;
impl Rule for HttpNoPcallRule {
    fn id(&self) -> &'static str { "S005" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "HTTP requests should be wrapped in pcall" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let has_http_call = ctx.source.contains("RequestAsync")
            || ctx.source.contains("GetAsync")
            || ctx.source.contains("PostAsync")
            || ctx.source.contains("JSONDecode");

        let mentioned_services: Vec<&str> = ["HttpService"]
            .iter()
            .filter(|s| ctx.source.contains(**s))
            .copied()
            .collect();

        if !mentioned_services.is_empty() && has_http_call {
            let has_pcall_http = ctx.source.contains("pcall")
                || ctx.source.contains("xpcall");

            if !has_pcall_http {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    category: self.category().to_string(),
                    message: "HTTP requests can fail and should be wrapped in pcall".to_string(),
                    span: Some(Span { line: 1, column: 1 }),
                    suggestion: Some("Wrap HTTP calls in pcall to handle network failures".to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct NoSessionLockRule;
impl Rule for NoSessionLockRule {
    fn id(&self) -> &'static str { "S006" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Data Persistence" }
    fn description(&self) -> &'static str { "DataStore without session locking can cause data loss" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let has_datastore = ctx.source.contains("DataStoreService")
            && (ctx.source.contains("GetAsync") || ctx.source.contains("SetAsync") || ctx.source.contains("UpdateAsync"));

        if has_datastore {
            let has_session_lock = ctx.source.contains("SessionLock")
                || ctx.source.contains("sessionLock")
                || ctx.source.contains("session_lock")
                || ctx.source.contains("MemoryStoreService")
                || ctx.source.contains("locked");

            if !has_session_lock {
                return vec![Diagnostic {
                    rule_id: self.id().to_string(),
                    severity: self.severity(),
                    category: self.category().to_string(),
                    message: "DataStore usage without session locking. Players joining multiple servers can cause data loss".to_string(),
                    span: Some(Span { line: 1, column: 1 }),
                    suggestion: Some("Implement session locking with MemoryStoreService or a locking library".to_string()),
                    fixable: false,
                }];
            }
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct GameDestroyRule;
impl Rule for GameDestroyRule {
    fn id(&self) -> &'static str { "S007" }
    fn severity(&self) -> Severity { Severity::Error }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "game:Destroy() will break the entire game" }
    fn tier(&self) -> &'static str { "Beginner" }
    fn check_expression(&self, expr: &ast::Expression, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Expression::FunctionCall(call) = expr {
            return self.check_call(call);
        }
        Vec::new()
    }
    fn check_stmt(&self, stmt: &ast::Stmt, _ctx: &AnalysisContext) -> Vec<Diagnostic> {
        if let ast::Stmt::FunctionCall(call) = stmt {
            return self.check_call(call);
        }
        Vec::new()
    }
}
impl GameDestroyRule {
    fn check_call(&self, call: &ast::FunctionCall) -> Vec<Diagnostic> {
        if let Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "game" {
                for suffix in call.suffixes() {
                    if let Suffix::Call(Call::MethodCall(method)) = suffix {
                        if method.name().token().to_string() == "Destroy" {
                            return vec![Diagnostic {
                                rule_id: self.id().to_string(),
                                severity: self.severity(),
                                category: self.category().to_string(),
                                message: "game:Destroy() will completely break the game instance".to_string(),
                                span: span_from_node(call),
                                suggestion: Some("Remove this call entirely".to_string()),
                                fixable: false,
                            }];
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct NoSanityCheckRule;
impl Rule for NoSanityCheckRule {
    fn id(&self) -> &'static str { "S008" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "Remote event handler without type checking on arguments" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let has_on_server = ctx.source.contains("OnServerEvent") || ctx.source.contains("OnServerInvoke");
        if !has_on_server {
            return Vec::new();
        }

        let has_type_check = ctx.source.contains("typeof(")
            || ctx.source.contains("type(")
            || ctx.source.contains("tonumber(")
            || ctx.source.contains("tostring(");

        if !has_type_check {
            return vec![Diagnostic {
                rule_id: self.id().to_string(),
                severity: self.severity(),
                category: self.category().to_string(),
                message: "Server event handler without type checking on arguments. Exploiters can send any data type".to_string(),
                span: Some(Span { line: 1, column: 1 }),
                suggestion: Some("Use typeof() to validate argument types before processing".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}

#[derive(Debug)]
pub struct NoRateLimitRule;
impl Rule for NoRateLimitRule {
    fn id(&self) -> &'static str { "S002" }
    fn severity(&self) -> Severity { Severity::Warning }
    fn category(&self) -> &'static str { "Security" }
    fn description(&self) -> &'static str { "Remote event handler without rate limiting" }
    fn tier(&self) -> &'static str { "Advanced" }
    fn finalize(&self, ctx: &AnalysisContext) -> Vec<Diagnostic> {
        let has_on_server = ctx.source.contains("OnServerEvent") || ctx.source.contains("OnServerInvoke");
        if !has_on_server {
            return Vec::new();
        }

        let has_rate_limit = ctx.source.contains("rateLimit")
            || ctx.source.contains("rate_limit")
            || ctx.source.contains("RateLimit")
            || ctx.source.contains("cooldown")
            || ctx.source.contains("Cooldown")
            || ctx.source.contains("throttle")
            || ctx.source.contains("Throttle")
            || ctx.source.contains("lastRequest")
            || ctx.source.contains("last_request")
            || ctx.source.contains("os.clock()")
            || ctx.source.contains("tick()");

        if !has_rate_limit {
            return vec![Diagnostic {
                rule_id: self.id().to_string(),
                severity: self.severity(),
                category: self.category().to_string(),
                message: "Remote event handler without rate limiting. Exploiters can spam events to crash the server".to_string(),
                span: Some(Span { line: 1, column: 1 }),
                suggestion: Some("Track per-player request timestamps and reject rapid-fire calls".to_string()),
                fixable: false,
            }];
        }
        Vec::new()
    }
}
