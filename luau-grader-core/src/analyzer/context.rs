#[derive(Debug)]
pub struct AnalysisContext {
    pub scope_depth: usize,
    pub loop_depth: usize,
    pub source: String,
}

impl AnalysisContext {
    pub fn new(source: String) -> Self {
        Self {
            scope_depth: 0,
            loop_depth: 0,
            source,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn leave_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_sub(1);
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