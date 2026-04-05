use crate::workspace::DependencyGraph;
use crate::workspace_rules::{WorkspaceDiagnostic, WorkspaceRule};
use crate::report::Severity;
use std::collections::{HashSet};
use std::path::PathBuf;

pub struct CyclicDependencyRule;

impl WorkspaceRule for CyclicDependencyRule {
    fn id(&self) -> &'static str { "W001" }
    fn description(&self) -> &'static str { "Detects cyclic dependencies between modules" }
    
    fn analyze(&self, graph: &DependencyGraph) -> Vec<WorkspaceDiagnostic> {
        let mut diagnostics = Vec::new();
        
        let mut visited = HashSet::new();
        let mut path_stack = Vec::new();
        let mut in_stack = HashSet::new();
        
        for root_node in graph.nodes.keys() {
            if !visited.contains(root_node) {
                self.dfs(root_node, graph, &mut visited, &mut path_stack, &mut in_stack, &mut diagnostics);
            }
        }
        
        diagnostics
    }
}

impl CyclicDependencyRule {
    fn dfs(
        &self,
        node: &PathBuf,
        graph: &DependencyGraph,
        visited: &mut HashSet<PathBuf>,
        path_stack: &mut Vec<PathBuf>,
        in_stack: &mut HashSet<PathBuf>,
        diagnostics: &mut Vec<WorkspaceDiagnostic>
    ) {
        visited.insert(node.clone());
        path_stack.push(node.clone());
        in_stack.insert(node.clone());
        
        if let Some(dep_node) = graph.nodes.get(node) {
            for req in &dep_node.requires {
                if let Some(target) = self.resolve_require(req, graph) {
                    if in_stack.contains(&target) {
                        let cycle_str = path_stack.iter()
                            .skip_while(|p| **p != target)
                            .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
                            .collect::<Vec<_>>()
                            .join(" -> ");
                            
                        let full_cycle = format!("{} -> {}", cycle_str, target.file_name().unwrap_or_default().to_string_lossy());
                        
                        diagnostics.push(WorkspaceDiagnostic {
                            rule_id: self.id().to_string(),
                            severity: Severity::Error,
                            message: format!("Cyclic dependency detected: {}", full_cycle),
                            file_path: Some(node.clone()),
                        });
                    } else if !visited.contains(&target) {
                        self.dfs(&target, graph, visited, path_stack, in_stack, diagnostics);
                    }
                }
            }
        }
        
        path_stack.pop();
        in_stack.remove(node);
    }
    
    fn resolve_require(&self, req: &str, graph: &DependencyGraph) -> Option<PathBuf> {
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
                    return Some(path.clone());
                }
            }
        }
        None
    }
}
