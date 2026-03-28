use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub span: Option<Span>,
    pub suggestion: Option<String>,
    #[serde(default)]
    pub fixable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub file: String,
    pub tier: String,
    pub diagnostics: Vec<Diagnostic>,
    pub passed: bool,
}

impl Report {
    pub fn new(file: String, tier: String) -> Self {
        Self { file, tier, diagnostics: Vec::new(), passed: true }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        if diagnostic.severity == Severity::Error { self.passed = false; }
        self.diagnostics.push(diagnostic);
    }

    pub fn merge(&mut self, diagnostics: Vec<Diagnostic>) {
        for d in diagnostics { self.push(d); }
    }

    pub fn apply_severity_overrides(&mut self, overrides: &std::collections::HashMap<String, String>) {
        for diag in &mut self.diagnostics {
            if let Some(sev_str) = overrides.get(&diag.rule_id) {
                match sev_str.as_str() {
                    "Error" => diag.severity = Severity::Error,
                    "Warning" => diag.severity = Severity::Warning,
                    "Info" => diag.severity = Severity::Info,
                    _ => {}
                }
            }
        }
        self.passed = !self.diagnostics.iter().any(|d| d.severity == Severity::Error);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}