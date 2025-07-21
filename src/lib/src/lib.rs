//! # Graph Generation Language (GGL)
//!
//! GGL is a domain-specific language for creating and manipulating graphs through a powerful, declarative syntax.
//! It supports dynamic graph construction using variables and loops.
//!
//! ## Overview
//!
//! GGL allows you to:
//!
//! * Define graph structures using intuitive node and edge declarations.
//! * Use variables and loops to generate complex or repetitive patterns programmatically.
//! * Create common graph topologies with built-in generators.
//! * Apply transformation rules to modify graph structure.
//! * Export graphs in standard JSON format for visualization or further processing.
//!
//! ## Quick Example
//!
//! ```ggl
//! graph dynamic_network {
//!     // Declare variables
//!     let prefix: string = "user_";
//!     let count: int = 5;
//!
//!     // Create nodes in a loop
//!     for i in 0..count {
//!         node "{prefix}{i}" :person [id_num=i, status="active"];
//!     }
//!
//!     // Create a chain of relationships
//!     for i in 0..(count - 1) {
//!         edge: "{prefix}{i}" -> "{prefix}{i+1}" [weight=0.5];
//!     }
//!
//!     // Generate a star graph and connect it
//!     generate star {
//!         nodes: 5;
//!         prefix: "service";
//!     }
//!
//!     edge: "user_0" -- "service0";
//! }
//! ```
//!
//! ## Features
//!
//! * **Dynamic Syntax**: Supports variables and for-loops for programmatic graph construction.
//! * **Declarative core**: Intuitive node and edge declarations remain at the core.
//! * **Built-in Generators**: Create common graph structures (complete, path, cycle, grid, star, tree, scale-free).
//! * **Transformation Rules**: Apply pattern-based rules to modify graph structure.
//! * **Rich Attributes**: Support for typed nodes and edges with metadata.
//! * **JSON Output**: Export graphs in standard JSON format.
//!
//! ## Getting Started
//!
//! ### Installation
//!
//! Prerequisites:
//! * Rust 1.70 or later
//! * Cargo (comes with Rust)
//!
//! Building from source:
//! ```bash
//! git clone [https://github.com/ocasazza/graph-generation-language.git](https://github.com/ocasazza/graph-generation-language.git)
//! cd graph-generation-language
//! cargo build --release
//! ```

use std::collections::HashMap;

pub mod generators;
pub mod parser;
pub mod rules;
pub mod types;

use crate::generators::get_generator;
use crate::parser::{
    ApplyStatement, EdgeDeclaration, Expression, ForStatement, GenerateStatement, LetStatement,
    NodeDeclaration, RuleDefinition, Statement,
};
use crate::parser::parse_ggl;
use crate::types::{Edge, Graph, Node};
use serde_json::Value;

/// The main GGL engine for parsing and executing GGL programs.
///
/// `GGLEngine` maintains the state of a graph, transformation rules, and an execution context for variables.
/// It interprets GGL code to build complex graph structures.
pub struct GGLEngine {
    pub graph: Graph,
    rules: HashMap<String, rules::Rule>,
    context: HashMap<String, Value>,
}

impl Default for GGLEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GGLEngine {
    /// Creates a new GGL engine.
    pub fn new() -> Self {
        GGLEngine {
            graph: Graph::new(),
            rules: HashMap::new(),
            context: HashMap::new(),
        }
    }

    /// Parses and executes a GGL program, returning the resulting graph as JSON.
    pub fn generate_from_ggl(&mut self, ggl_code: &str) -> Result<String, String> {
        let ast = parse_ggl(ggl_code).map_err(|e| format!("Parse error: {e}"))?;

        // Reset state for a new run
        self.graph = Graph::new();
        self.rules.clear();
        self.context.clear();

        self.execute_statements(&ast.statements)?;

        // Serialize final graph to JSON
        serde_json::to_string_pretty(&self.graph).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Executes a sequence of GGL statements within the current context.
    fn execute_statements(&mut self, statements: &[Statement]) -> Result<(), String> {
        for stmt in statements {
            self.execute_statement(stmt)?;
        }
        Ok(())
    }

    /// Executes a single GGL statement.
    fn execute_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let(stmt) => self.handle_let(stmt),
            Statement::For(stmt) => self.handle_for(stmt),
            Statement::Node(stmt) => self.handle_node(stmt),
            Statement::Edge(stmt) => self.handle_edge(stmt),
            Statement::Generate(stmt) => self.handle_generate(stmt),
            Statement::RuleDef(stmt) => self.handle_rule_def(stmt),
            Statement::Apply(stmt) => self.handle_apply(stmt),
        }
    }

    // --- Statement Handlers ---

    fn handle_let(&mut self, stmt: &LetStatement) -> Result<(), String> {
        let value = self.evaluate_expression(&stmt.value)?;
        self.context.insert(stmt.name.clone(), value);
        Ok(())
    }

    fn handle_for(&mut self, stmt: &ForStatement) -> Result<(), String> {
        let start = self.evaluate_expression(&stmt.start)?.as_i64().ok_or("For loop start must be an integer")? as isize;
        let end = self.evaluate_expression(&stmt.end)?.as_i64().ok_or("For loop end must be an integer")? as isize;

        for i in start..end {
            self.context
                .insert(stmt.variable.clone(), Value::Number(serde_json::Number::from(i as i64)));
            self.execute_statements(&stmt.body)?;
        }
        // Remove loop variable from context after loop finishes
        self.context.remove(&stmt.variable);
        Ok(())
    }

    fn handle_node(&mut self, stmt: &NodeDeclaration) -> Result<(), String> {
        let id = self.evaluate_expression(&stmt.id)?.to_string().replace('"', "");
        let node_type = match &stmt.node_type {
            Some(expr) => self.evaluate_expression(expr)?.to_string().replace('"', ""),
            None => String::new(),
        };
        let mut metadata = HashMap::new();
        for (key, expr) in &stmt.attributes {
            metadata.insert(key.clone(), self.evaluate_expression(expr)?);
        }

        self.graph
            .add_node(id, Node::new().with_type(node_type).with_metadata_map(metadata));
        Ok(())
    }

    fn handle_edge(&mut self, stmt: &EdgeDeclaration) -> Result<(), String> {
        let id = match &stmt.id {
            Some(expr) => self.evaluate_expression(expr)?.to_string().replace('"', ""),
            None => self.graph.generate_unique_edge_id("edge"),
        };
        let source = self.evaluate_expression(&stmt.source)?.to_string().replace('"', "");
        let target = self.evaluate_expression(&stmt.target)?.to_string().replace('"', "");
        let mut metadata = HashMap::new();
        for (key, expr) in &stmt.attributes {
            metadata.insert(key.clone(), self.evaluate_expression(expr)?);
        }

        self.graph.add_edge(
            id,
            Edge::new(source, target, stmt.directed).with_metadata_map(metadata),
        );
        Ok(())
    }

    fn handle_generate(&mut self, stmt: &GenerateStatement) -> Result<(), String> {
        let generator_name = &stmt.name;
        if let Some(generator) = get_generator(generator_name) {
            let mut params = HashMap::new();
            for (key, expr) in &stmt.params {
                params.insert(key.clone(), self.evaluate_expression(expr)?);
            }
            let generated_graph =
                generator(&params).map_err(|e| format!("Generator '{generator_name}' error: {e}"))?;

            // Merge generated graph into the current graph
            for (id, node) in generated_graph.nodes {
                self.graph.add_node(id, node);
            }
            for (id, edge) in generated_graph.edges {
                self.graph.add_edge(id, edge);
            }
        } else {
            return Err(format!("Unknown generator: {generator_name}"));
        }
        Ok(())
    }

    fn handle_rule_def(&mut self, stmt: &RuleDefinition) -> Result<(), String> {
        let rule = rules::Rule {
            name: stmt.name.clone(),
            lhs: stmt.lhs.clone(),
            rhs: stmt.rhs.clone(),
        };
        self.rules.insert(stmt.name.clone(), rule);
        Ok(())
    }

    fn handle_apply(&mut self, stmt: &ApplyStatement) -> Result<(), String> {
        let iterations = self.evaluate_expression(&stmt.iterations)?.as_i64().ok_or("Apply iterations must be an integer")? as usize;
        if let Some(rule) = self.rules.get(&stmt.rule_name).cloned() {
            rule.apply(&mut self.graph, iterations)
                .map_err(|e| format!("Rule '{}' application error: {e}", stmt.rule_name))?;
        } else {
            return Err(format!("Unknown rule: {}", stmt.rule_name));
        }
        Ok(())
    }

    /// Evaluates an expression by resolving variables or interpreting literals.
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
            Expression::Integer(i) => Ok(Value::Number(serde_json::Number::from(*i))),
            Expression::Float(f) => Ok(Value::Number(serde_json::Number::from_f64(*f).unwrap())),
            Expression::Boolean(b) => Ok(Value::Bool(*b)),
            Expression::Identifier(name) => {
                // First try to resolve as a variable, if not found treat as string literal
                Ok(self.context
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| Value::String(name.clone())))
            }
            Expression::FormattedString(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        parser::StringPart::Literal(s) => result.push_str(s),
                        parser::StringPart::Variable(var) => {
                            let value = self.context.get(var).ok_or(format!("Undefined variable: '{var}'"))?;
                            result.push_str(&value.to_string().replace('"', ""));
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }
}
