use full_moon::ast;
use full_moon::visitors::Visitor;

use crate::report::Diagnostic;
use crate::rulesets::Rule;
use super::context::AnalysisContext;

pub struct GraderWalker<'a> {
    rules: &'a [Box<dyn Rule>],
    ctx: &'a mut AnalysisContext,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> GraderWalker<'a> {
    pub fn new(rules: &'a [Box<dyn Rule>], ctx: &'a mut AnalysisContext) -> Self {
        Self {
            rules,
            ctx,
            diagnostics: Vec::new(),
        }
    }

    pub fn walk(mut self, ast: &ast::Ast) -> Vec<Diagnostic> {
        self.visit_ast(ast);
        self.diagnostics
    }
}

impl Visitor for GraderWalker<'_> {
    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        for rule in self.rules.iter() {
            let results = rule.check_stmt(stmt, self.ctx);
            self.diagnostics.extend(results);
        }

        if let ast::Stmt::LocalAssignment(local) = stmt {
            for name in local.names() {
                self.ctx.declare_local(name.token().to_string());
            }
        }
    }

    fn visit_expression(&mut self, expr: &ast::Expression) {
        for rule in self.rules.iter() {
            let results = rule.check_expression(expr, self.ctx);
            self.diagnostics.extend(results);
        }
    }

    fn visit_function_body(&mut self, body: &ast::FunctionBody) {
        self.ctx.push_local_scope();
        for param in body.parameters() {
            if let ast::Parameter::Name(name) = param {
                self.ctx.declare_local(name.token().to_string());
            }
        }

        for rule in self.rules.iter() {
            let results = rule.check_function_body(body, self.ctx);
            self.diagnostics.extend(results);
        }
    }

    fn visit_function_body_end(&mut self, _body: &ast::FunctionBody) {
        self.ctx.pop_local_scope();
    }

    fn visit_block(&mut self, _block: &ast::Block) {
        self.ctx.enter_scope();
    }

    fn visit_block_end(&mut self, _block: &ast::Block) {
        self.ctx.leave_scope();
    }

    fn visit_while(&mut self, _node: &ast::While) {
        self.ctx.enter_loop();
    }

    fn visit_while_end(&mut self, _node: &ast::While) {
        self.ctx.leave_loop();
    }

    fn visit_repeat(&mut self, _node: &ast::Repeat) {
        self.ctx.enter_loop();
    }

    fn visit_repeat_end(&mut self, _node: &ast::Repeat) {
        self.ctx.leave_loop();
    }

    fn visit_numeric_for(&mut self, node: &ast::NumericFor) {
        self.ctx.enter_loop();
        self.ctx.push_local_scope();
        self.ctx.declare_local(node.index_variable().token().to_string());
    }

    fn visit_numeric_for_end(&mut self, _node: &ast::NumericFor) {
        self.ctx.pop_local_scope();
        self.ctx.leave_loop();
    }

    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        self.ctx.enter_loop();
        self.ctx.in_generic_for = true;
        self.ctx.push_local_scope();
        for name in node.names() {
            self.ctx.declare_local(name.token().to_string());
        }
    }

    fn visit_generic_for_end(&mut self, _node: &ast::GenericFor) {
        self.ctx.pop_local_scope();
        self.ctx.in_generic_for = false;
        self.ctx.leave_loop();
    }
}
