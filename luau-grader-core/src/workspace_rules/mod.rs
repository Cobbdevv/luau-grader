use crate::workspace::DependencyGraph;
use crate::report::Severity;

use serde::Serialize;

pub mod cyclic;
pub mod dead_code;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceDiagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub file_path: Option<std::path::PathBuf>,
}

pub trait WorkspaceRule {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn analyze(&self, graph: &DependencyGraph) -> Vec<WorkspaceDiagnostic>;
}

pub fn all_workspace_rules() -> Vec<Box<dyn WorkspaceRule>> {
    vec![
        Box::new(cyclic::CyclicDependencyRule),
        Box::new(dead_code::DeadCodeRule),
    ]
}
