use crate::workspace::DependencyGraph;
use crate::workspace_rules::{WorkspaceDiagnostic, WorkspaceRule};
use crate::report::Severity;
use std::collections::HashSet;

pub struct DeadCodeRule;

impl WorkspaceRule for DeadCodeRule {
    fn id(&self) -> &'static str { "W002" }
    fn description(&self) -> &'static str { "Detects modules that are never required" }
    
    fn analyze(&self, graph: &DependencyGraph) -> Vec<WorkspaceDiagnostic> {
        let mut diagnostics = Vec::new();
        
        let mut required_files = HashSet::new();
        
        for dep_node in graph.nodes.values() {
            for req in &dep_node.requires {
                let req_lower = req.to_lowercase();
                
                let target_name = if req_lower.contains('.') {
                    req_lower.split('.').next_back().unwrap_or(&req_lower).trim_matches('"').trim_matches('\'').to_string()
                } else if req_lower.contains('/') {
                    req_lower.split('/').next_back().unwrap_or(&req_lower).trim_matches('"').trim_matches('\'').to_string()
                } else {
                    req_lower.trim_matches('"').trim_matches('\'').to_string()
                };

                for path in graph.nodes.keys() {
                    if let Some(file_name) = path.file_stem() {
                        if file_name.to_string_lossy().to_lowercase() == target_name {
                            required_files.insert(path.clone());
                        }
                    }
                }
            }
        }
        
        for path in graph.nodes.keys() {
            if !required_files.contains(path) {
                if !is_entry_point(path) {
                    diagnostics.push(WorkspaceDiagnostic {
                        rule_id: self.id().to_string(),
                        severity: Severity::Warning,
                        message: "Module is never required. Consider removing this dead code.".to_string(),
                        file_path: Some(path.clone()),
                    });
                }
            }
        }
        
        diagnostics
    }
}

fn is_entry_point(path: &std::path::Path) -> bool {
    if let Some(name) = path.file_name() {
        let s = name.to_string_lossy().to_lowercase();
        if s.contains("server") || s.contains("client") || s.contains("main") || s.contains("init") {
            return true;
        }
    }
    false
}
