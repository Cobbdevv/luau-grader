use std::collections::HashSet;

#[derive(Debug)]
pub struct AnalysisContext {
    pub scope_depth: usize,
    pub loop_depth: usize,
    pub source: String,
    pub in_generic_for: bool,
    local_scopes: Vec<HashSet<String>>,
}

impl AnalysisContext {
    pub fn new(source: String) -> Self {
        Self {
            scope_depth: 0,
            loop_depth: 0,
            source,
            in_generic_for: false,
            local_scopes: vec![HashSet::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
        self.local_scopes.push(HashSet::new());
    }

    pub fn leave_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_sub(1);
        if self.local_scopes.len() > 1 {
            self.local_scopes.pop();
        }
    }

    pub fn push_local_scope(&mut self) {
        self.local_scopes.push(HashSet::new());
    }

    pub fn pop_local_scope(&mut self) {
        if self.local_scopes.len() > 1 {
            self.local_scopes.pop();
        }
    }

    pub fn declare_local(&mut self, name: String) {
        if let Some(scope) = self.local_scopes.last_mut() {
            scope.insert(name);
        }
    }

    pub fn is_declared_local(&self, name: &str) -> bool {
        self.local_scopes.iter().any(|scope| scope.contains(name))
    }

    pub fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    pub fn leave_loop(&mut self) {
        self.loop_depth = self.loop_depth.saturating_sub(1);
    }

    pub fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }
}