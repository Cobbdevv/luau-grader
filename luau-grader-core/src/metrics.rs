use full_moon::ast;
use full_moon::node::Node;
use full_moon::visitors::Visitor;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ScriptType {
    ServerScript,
    ClientScript,
    ModuleScript,
    SharedModule,
    Plugin,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionMetrics {
    pub name: String,
    pub line: usize,
    pub line_count: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub param_count: usize,
    pub max_nesting: usize,
    pub local_count: usize,
    pub return_count: usize,
    pub has_error_handling: bool,
    pub guard_clause_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileMetrics {
    pub total_lines: usize,
    pub functions: Vec<FunctionMetrics>,
    pub function_count: usize,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub service_count: usize,
    pub services_used: Vec<String>,
    pub global_write_count: usize,
    pub type_annotation_count: usize,
    pub comment_line_count: usize,
    pub naming_quality: f64,
    pub has_strict_mode: bool,
    pub consistency_score: f64,
    pub script_type: ScriptType,
    pub detected_patterns: Vec<String>,
    pub duplicate_function_pairs: Vec<(String, String)>,
    pub short_function_name_count: usize,
    pub code_organization_score: f64,
    pub naming_style_consistency: f64,
}

pub fn collect_metrics(source: &str, ast: &ast::Ast) -> FileMetrics {
    let mut collector = MetricsCollector::new(source);
    collector.visit_ast(ast);
    collector.finalize(source)
}

struct MetricsCollector {
    source: String,
    functions: Vec<FunctionMetrics>,
    current_function: Option<FunctionBuilder>,
    pending_function_name: Option<String>,
    nesting_depth: usize,
    services_used: HashSet<String>,
    global_write_count: usize,
    type_annotation_count: usize,
    all_local_names: Vec<String>,
    deprecated_api_count: usize,
    modern_api_count: usize,
    has_pcall: bool,
    has_datastore: bool,
    _has_remote_events: bool,
    has_fire_server: bool,
    has_fire_client: bool,
    has_debounce_var: bool,
    has_tick_or_clock: bool,
    has_character_added: bool,
    has_connect_calls: usize,
    has_cleanup_pattern: bool,
    ends_with_return: bool,
}

struct FunctionBuilder {
    name: String,
    start_line: usize,
    end_line: usize,
    cyclomatic: usize,
    cognitive: usize,
    cognitive_nesting: usize,
    param_count: usize,
    max_nesting: usize,
    current_nesting: usize,
    local_count: usize,
    return_count: usize,
    has_error_handling: bool,
    guard_clause_count: usize,
    seen_non_guard: bool,
}

impl MetricsCollector {
    fn new(source: &str) -> Self {
        let ends_with_return = source
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim().starts_with("return"))
            .unwrap_or(false);

        Self {
            source: source.to_string(),
            functions: Vec::new(),
            current_function: None,
            pending_function_name: None,
            nesting_depth: 0,
            services_used: HashSet::new(),
            global_write_count: 0,
            type_annotation_count: 0,
            all_local_names: Vec::new(),
            deprecated_api_count: 0,
            modern_api_count: 0,
            has_pcall: false,
            has_datastore: false,
            _has_remote_events: false,
            has_fire_server: false,
            has_fire_client: false,
            has_debounce_var: false,
            has_tick_or_clock: false,
            has_character_added: false,
            has_connect_calls: 0,
            has_cleanup_pattern: false,
            ends_with_return,
        }
    }

    fn finalize(self, source: &str) -> FileMetrics {
        let total_lines = source.lines().count();
        let function_count = self.functions.len();
        let avg_function_length = if function_count > 0 {
            self.functions.iter().map(|f| f.line_count).sum::<usize>() as f64 / function_count as f64
        } else {
            0.0
        };
        let max_function_length = self.functions.iter().map(|f| f.line_count).max().unwrap_or(0);

        let comment_line_count = source.lines().filter(|l| l.trim().starts_with("--")).count();

        let naming_quality = if self.all_local_names.is_empty() {
            5.0
        } else {
            let total_len: usize = self.all_local_names.iter().map(|n| n.len()).sum();
            total_len as f64 / self.all_local_names.len() as f64
        };

        let has_strict_mode = source.lines().next().map(|l| l.trim().starts_with("--!strict")).unwrap_or(false);

        let total_api_calls = self.deprecated_api_count + self.modern_api_count;
        let consistency_score = if total_api_calls == 0 {
            1.0
        } else {
            self.modern_api_count as f64 / total_api_calls as f64
        };

        let script_type = self.detect_script_type();
        let detected_patterns = self.detect_patterns();

        let short_function_name_count = self.functions.iter()
            .filter(|f| {
                let name = &f.name;
                !name.starts_with('<') && name.len() <= 2
            })
            .count();

        let duplicate_function_pairs = self.detect_duplicates(source);
        let code_organization_score = Self::compute_organization_score(source);
        let naming_style_consistency = Self::compute_naming_consistency(&self.all_local_names);

        FileMetrics {
            total_lines,
            functions: self.functions,
            function_count,
            avg_function_length,
            max_function_length,
            service_count: self.services_used.len(),
            services_used: self.services_used.into_iter().collect(),
            global_write_count: self.global_write_count,
            type_annotation_count: self.type_annotation_count,
            comment_line_count,
            naming_quality,
            has_strict_mode,
            consistency_score,
            script_type,
            detected_patterns,
            duplicate_function_pairs,
            short_function_name_count,
            code_organization_score,
            naming_style_consistency,
        }
    }

    fn detect_script_type(&self) -> ScriptType {
        if self.source.contains("plugin") && self.source.contains("toolbar") {
            return ScriptType::Plugin;
        }

        let has_server_services = self.services_used.iter().any(|s| {
            s == "ServerStorage" || s == "ServerScriptService" || s == "DataStoreService"
        });

        let has_client_services = self.services_used.iter().any(|s| {
            s == "UserInputService" || s == "ContextActionService" || s == "StarterGui"
        });

        let has_local_player = self.source.contains("LocalPlayer");

        if self.ends_with_return && !has_server_services && !has_client_services && !has_local_player {
            return ScriptType::SharedModule;
        }

        if self.ends_with_return {
            return ScriptType::ModuleScript;
        }

        if has_server_services || (self.has_fire_client && !self.has_fire_server) {
            return ScriptType::ServerScript;
        }

        if has_client_services || has_local_player || (self.has_fire_server && !self.has_fire_client) {
            return ScriptType::ClientScript;
        }

        ScriptType::Unknown
    }

    fn detect_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();
        if self.has_debounce_var {
            patterns.push("Debounce".to_string());
        }
        if self.has_tick_or_clock {
            patterns.push("Cooldown".to_string());
        }
        if self.has_datastore && self.has_pcall {
            patterns.push("Data Save/Load".to_string());
        }
        if self.has_character_added {
            patterns.push("Character Added Handler".to_string());
        }
        if self.ends_with_return {
            patterns.push("Module Pattern".to_string());
        }
        if self.has_connect_calls >= 3 {
            patterns.push("Observer".to_string());
        }
        if self.has_cleanup_pattern {
            patterns.push("Cleanup/Janitor".to_string());
        }
        patterns
    }
}

impl Visitor for MetricsCollector {
    fn visit_function_body(&mut self, body: &ast::FunctionBody) {
        let start_line = body.start_position().map(|p| p.line()).unwrap_or(0);
        let end_line = body.end_position().map(|p| p.line()).unwrap_or(0);
        let param_count = body.parameters().into_iter().count();

        let name = if let Some(pending) = self.pending_function_name.take() {
            pending
        } else {
            self.extract_function_name(start_line)
        };

        self.current_function = Some(FunctionBuilder {
            name,
            start_line,
            end_line,
            cyclomatic: 1,
            cognitive: 0,
            cognitive_nesting: 0,
            param_count,
            max_nesting: 0,
            current_nesting: 0,
            local_count: 0,
            return_count: 0,
            has_error_handling: false,
            guard_clause_count: 0,
            seen_non_guard: false,
        });
    }

    fn visit_function_body_end(&mut self, _body: &ast::FunctionBody) {
        if let Some(builder) = self.current_function.take() {
            self.functions.push(FunctionMetrics {
                name: builder.name,
                line: builder.start_line,
                line_count: builder.end_line.saturating_sub(builder.start_line) + 1,
                cyclomatic_complexity: builder.cyclomatic,
                cognitive_complexity: builder.cognitive,
                param_count: builder.param_count,
                max_nesting: builder.max_nesting,
                local_count: builder.local_count,
                return_count: builder.return_count,
                has_error_handling: builder.has_error_handling,
                guard_clause_count: builder.guard_clause_count,
            });
        }
    }

    fn visit_if(&mut self, node: &ast::If) {
        self.increment_complexity(1);
        self.enter_nesting();
        if let Some(ref mut func) = self.current_function {
            if !func.seen_non_guard && func.current_nesting == 1 {
                let block = node.block();
                let stmt_count = block.stmts().count();
                let has_early_exit = block.last_stmt().is_some();
                if (stmt_count == 0 || stmt_count == 1) && has_early_exit && node.else_block().is_none() && node.else_if().is_none() {
                    func.guard_clause_count += 1;
                } else {
                    func.seen_non_guard = true;
                }
            } else if func.current_nesting >= 1 {
                func.seen_non_guard = true;
            }
        }
    }

    fn visit_if_end(&mut self, _node: &ast::If) {
        self.leave_nesting();
    }

    fn visit_while(&mut self, _node: &ast::While) {
        self.increment_complexity(1);
        self.enter_nesting();
    }

    fn visit_while_end(&mut self, _node: &ast::While) {
        self.leave_nesting();
    }

    fn visit_repeat(&mut self, _node: &ast::Repeat) {
        self.increment_complexity(1);
        self.enter_nesting();
    }

    fn visit_repeat_end(&mut self, _node: &ast::Repeat) {
        self.leave_nesting();
    }

    fn visit_numeric_for(&mut self, _node: &ast::NumericFor) {
        self.increment_complexity(1);
        self.enter_nesting();
    }

    fn visit_numeric_for_end(&mut self, _node: &ast::NumericFor) {
        self.leave_nesting();
    }

    fn visit_generic_for(&mut self, _node: &ast::GenericFor) {
        self.increment_complexity(1);
        self.enter_nesting();
    }

    fn visit_generic_for_end(&mut self, _node: &ast::GenericFor) {
        self.leave_nesting();
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        if let Some(ref mut func) = self.current_function {
            func.local_count += 1;
        }
        for name in node.names() {
            let var_name = name.token().to_string();
            self.all_local_names.push(var_name.clone());
            if var_name == "debounce" || var_name == "isDebounce" || var_name == "cooldown" {
                self.has_debounce_var = true;
            }
        }
        let names: Vec<String> = node.names().iter().map(|n| n.token().to_string()).collect();
        for expr in node.expressions() {
            if matches!(expr, ast::Expression::Function(_)) && !names.is_empty() {
                self.pending_function_name = Some(names[0].clone());
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        if let ast::Stmt::FunctionDeclaration(decl) = stmt {
            let func_name = decl.name();
            let mut parts = Vec::new();
            parts.push(func_name.names().iter().map(|n| n.token().to_string()).collect::<Vec<_>>().join("."));
            if let Some(method) = func_name.method_name() {
                let last = parts.pop().unwrap_or_default();
                parts.push(format!("{}:{}", last, method.token()));
            }
            let name = parts.join(".");
            if !name.is_empty() {
                self.pending_function_name = Some(name);
            }
        }
        if let Some(ref mut func) = self.current_function {
            if !matches!(stmt, ast::Stmt::If(_)) && func.current_nesting == 0 {
                let is_local = matches!(stmt, ast::Stmt::LocalAssignment(_));
                if !is_local {
                    func.seen_non_guard = true;
                }
            }
        }
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if let ast::Prefix::Name(name) = call.prefix() {
            let fn_name = name.token().to_string();
            if fn_name == "pcall" || fn_name == "xpcall" {
                self.has_pcall = true;
                if let Some(ref mut func) = self.current_function {
                    func.has_error_handling = true;
                }
            }
            if fn_name == "tick" {
                self.has_tick_or_clock = true;
            }
            if fn_name == "os" {
                for suffix in call.suffixes() {
                    if let ast::Suffix::Index(ast::Index::Dot { name: m, .. }) = suffix
                        && m.token().to_string() == "clock" {
                            self.has_tick_or_clock = true;
                    }
                }
            }
            if fn_name == "game" {
                for suffix in call.suffixes() {
                    if let ast::Suffix::Call(ast::Call::MethodCall(method)) = suffix
                        && method.name().token().to_string() == "GetService" {
                            if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                                if let Some(ast::Expression::String(s)) = arguments.iter().next() {
                                    let service = s.token().to_string()
                                        .trim_matches('"')
                                        .trim_matches('\'')
                                        .to_string();
                                    if service == "DataStoreService" {
                                        self.has_datastore = true;
                                    }
                                    self.services_used.insert(service);
                                }
                            }
                    }
                }
            }
        }

        for suffix in call.suffixes() {
            if let ast::Suffix::Call(ast::Call::MethodCall(method)) = suffix {
                let method_name = method.name().token().to_string();
                match method_name.as_str() {
                    "Connect" | "Once" => self.has_connect_calls += 1,
                    "FireServer" => self.has_fire_server = true,
                    "FireClient" | "FireAllClients" => self.has_fire_client = true,
                    "Destroy" | "Disconnect" => self.has_cleanup_pattern = true,
                    "CharacterAdded" => self.has_character_added = true,
                    _ => {}
                }
            }
        }
    }

    fn visit_return(&mut self, _node: &ast::Return) {
        if let Some(ref mut func) = self.current_function {
            func.return_count += 1;
        }
    }

    fn visit_assignment(&mut self, node: &ast::Assignment) {
        for var in node.variables() {
            if let ast::Var::Name(_) = var {
                self.global_write_count += 1;
            }
        }
    }
}

impl MetricsCollector {
    fn increment_complexity(&mut self, base: usize) {
        if let Some(ref mut func) = self.current_function {
            func.cyclomatic += 1;
            func.cognitive += base + func.cognitive_nesting;
        }
    }

    fn enter_nesting(&mut self) {
        self.nesting_depth += 1;
        if let Some(ref mut func) = self.current_function {
            func.current_nesting += 1;
            func.cognitive_nesting += 1;
            if func.current_nesting > func.max_nesting {
                func.max_nesting = func.current_nesting;
            }
        }
    }

    fn leave_nesting(&mut self) {
        self.nesting_depth = self.nesting_depth.saturating_sub(1);
        if let Some(ref mut func) = self.current_function {
            func.current_nesting = func.current_nesting.saturating_sub(1);
            func.cognitive_nesting = func.cognitive_nesting.saturating_sub(1);
        }
    }

    fn extract_function_name(&self, start_line: usize) -> String {
        if start_line == 0 {
            return "<anonymous>".to_string();
        }
        let line = self.source.lines().nth(start_line - 1).unwrap_or("");
        let trimmed = line.trim();

        if let Some(pos) = trimmed.find("function ") {
            let after = &trimmed[pos + 9..];
            if let Some(paren) = after.find('(') {
                let name = after[..paren].trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }

        if let Some(pos) = trimmed.find('=') {
            let before = trimmed[..pos].trim();
            let name = before.trim_start_matches("local").trim();
            if !name.is_empty() && name.len() < 60 {
                return name.to_string();
            }
        }

        format!("<anonymous@{start_line}>")
    }

    fn detect_duplicates(&self, source: &str) -> Vec<(String, String)> {
        let lines: Vec<&str> = source.lines().collect();
        let mut pairs = Vec::new();
        for i in 0..self.functions.len() {
            for j in (i + 1)..self.functions.len() {
                let a = &self.functions[i];
                let b = &self.functions[j];
                if a.line_count < 5 || b.line_count < 5 {
                    continue;
                }
                let a_start = a.line.saturating_sub(1);
                let a_end = (a_start + a.line_count).min(lines.len());
                let b_start = b.line.saturating_sub(1);
                let b_end = (b_start + b.line_count).min(lines.len());
                let a_body: Vec<String> = lines[a_start..a_end].iter()
                    .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
                    .collect();
                let b_body: Vec<String> = lines[b_start..b_end].iter()
                    .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
                    .collect();
                let max_len = a_body.len().max(b_body.len());
                if max_len == 0 { continue; }
                let matching = a_body.iter().zip(b_body.iter())
                    .filter(|(al, bl)| al == bl)
                    .count();
                let similarity = matching as f64 / max_len as f64;
                if similarity >= 0.70 {
                    pairs.push((a.name.clone(), b.name.clone()));
                }
            }
        }
        pairs
    }

    fn compute_organization_score(source: &str) -> f64 {
        let mut last_service_line: usize = 0;
        let mut last_require_line: usize = 0;
        let mut first_function_line: usize = usize::MAX;
        let mut first_connect_line: usize = usize::MAX;
        let mut has_services = false;
        let mut has_requires = false;
        let mut has_functions = false;
        let mut has_connects = false;

        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") || trimmed.is_empty() { continue; }

            if trimmed.contains("GetService(") {
                last_service_line = i;
                has_services = true;
            }
            if trimmed.contains("require(") && (trimmed.starts_with("local ") || trimmed.starts_with("local\t")) {
                last_require_line = i;
                has_requires = true;
            }
            if (trimmed.starts_with("function ") || trimmed.starts_with("local function "))
                && i < first_function_line
            {
                first_function_line = i;
                has_functions = true;
            }
            if (trimmed.contains(":Connect(") || trimmed.contains(":Once(")) && i < first_connect_line {
                first_connect_line = i;
                has_connects = true;
            }
        }

        let mut checks = 0;
        let mut passed = 0;

        if has_services && has_functions {
            checks += 1;
            if last_service_line < first_function_line { passed += 1; }
        }
        if has_requires && has_functions {
            checks += 1;
            if last_require_line < first_function_line { passed += 1; }
        }
        if has_functions && has_connects {
            checks += 1;
            if first_function_line < first_connect_line { passed += 1; }
        }
        if has_services && has_requires {
            checks += 1;
            if last_service_line <= last_require_line + 5 { passed += 1; }
        }

        if checks == 0 { return 1.0; }
        passed as f64 / checks as f64
    }

    fn compute_naming_consistency(names: &[String]) -> f64 {
        let filtered: Vec<&String> = names.iter()
            .filter(|n| n.len() > 1 && *n != "_")
            .collect();

        if filtered.len() < 3 { return 1.0; }

        let mut camel = 0usize;
        let mut snake = 0usize;
        let mut pascal = 0usize;
        let mut screaming = 0usize;

        for name in &filtered {
            let bytes = name.as_bytes();
            let has_underscore = name.contains('_');
            let first_upper = bytes[0].is_ascii_uppercase();
            let all_upper = name.chars().all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit());
            let has_mixed_case = name.chars().any(|c| c.is_ascii_uppercase()) && name.chars().any(|c| c.is_ascii_lowercase());

            if all_upper && has_underscore {
                screaming += 1;
            } else if first_upper && has_mixed_case && !has_underscore {
                pascal += 1;
            } else if !first_upper && has_underscore && !has_mixed_case {
                snake += 1;
            } else if !first_upper && !has_underscore {
                camel += 1;
            }
        }

        let dominant = camel.max(snake).max(pascal).max(screaming);
        let classified = camel + snake + pascal + screaming;
        if classified == 0 { return 1.0; }
        dominant as f64 / classified as f64
    }
}
