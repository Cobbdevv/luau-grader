use std::collections::HashMap;
use std::path::{Path, PathBuf};
use full_moon::ast;
use full_moon::visitors::Visitor;

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub file_path: PathBuf,
    pub requires: Vec<String>,
}

#[derive(Debug, Default)]
pub struct DependencyGraph {
    pub nodes: HashMap<PathBuf, DependencyNode>,
}

struct RequireVisitor {
    requires: Vec<String>,
}

impl Visitor for RequireVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if let ast::Prefix::Name(name) = call.prefix() {
            if name.token().to_string() == "require" {
                if let Some(ast::Suffix::Call(ast::Call::AnonymousCall(args))) = call.suffixes().next() {
                    match args {
                        ast::FunctionArgs::Parentheses { arguments, .. } => {
                            if let Some(arg) = arguments.iter().next() {
                                self.requires.push(format!("{arg}").trim().to_string());
                            }
                        }
                        ast::FunctionArgs::String(s) => {
                            self.requires.push(s.token().to_string().trim_matches('"').trim_matches('\'').to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn analyze_workspace(root: &Path) -> Result<DependencyGraph, std::io::Error> {
    let mut graph = DependencyGraph::default();
    let mut to_visit = vec![root.to_path_buf()];

    while let Some(path) = to_visit.pop() {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                to_visit.push(entry?.path());
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "luau" || ext == "lua" {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(ast) = full_moon::parse(&content) {
                        let mut visitor = RequireVisitor { requires: Vec::new() };
                        visitor.visit_ast(&ast);
                        graph.nodes.insert(path.clone(), DependencyNode {
                            file_path: path,
                            requires: visitor.requires,
                        });
                    }
                }
            }
        }
    }

    Ok(graph)
}
