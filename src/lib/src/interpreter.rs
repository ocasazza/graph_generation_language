//! # GGL Interpreter
//!
//! This module is responsible for executing a GGL program, which is represented as a
//! sequence of `GGLStatement`s. It manages the state of the graph being built,
//! including a symbol table for variables.
//!
//! ## Core Components
//!
//! * `Interpreter` - The main struct that holds the graph and symbol table.
//! * `evaluate_expression` - A function to compute the value of expressions.
//! * `execute_statement` - A function to handle each type of GGL statement.
//!
//! ## Execution Flow
//!
//! The interpreter iterates through the statements of a GGL program and executes
//! them one by one. It maintains a `Graph` object and a `HashMap` for the symbol
//! table.
//!
//! - `LetStatement`: Adds a variable to the symbol table.
//! - `ForLoop`: Executes a block of statements multiple times, updating a loop
//!   variable in the symbol table for each iteration.
//! - `NodeDecl` and `EdgeDecl`: Adds nodes and edges to the graph, resolving
//!   any variables or expressions in their definitions.

use crate::generators::get_generator;
use crate::parser::{
    EdgeDeclaration, Expression, GGLStatement, InterpolatedStringPart, NodeDeclaration,
};
// use crate::rules;
use crate::types::{Edge, Graph, MetadataValue, Node};
use std::collections::HashMap;

/// The interpreter for GGL programs.
///
/// It holds the state of the graph being generated and a symbol table for variables.
pub struct Interpreter {
    graph: Graph,
    symbols: HashMap<String, MetadataValue>,
    // rules: HashMap<String, rules::Rule>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new interpreter with an empty graph and symbol table.
    pub fn new() -> Self {
        Interpreter {
            graph: Graph::new(),
            symbols: HashMap::new(),
            // rules: HashMap::new(),
        }
    }

    /// Executes a sequence of GGL statements and returns the resulting graph.
    pub fn run(&mut self, statements: &[GGLStatement]) -> Result<Graph, String> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(self.graph.clone())
    }

    /// Executes a single GGL statement.
    fn execute_statement(&mut self, statement: &GGLStatement) -> Result<(), String> {
        match statement {
            GGLStatement::LetStmt(stmt) => {
                let value = self.evaluate_expression(&stmt.value)?;
                self.symbols.insert(stmt.var_name.clone(), value);
                Ok(())
            }
            GGLStatement::ForLoop(stmt) => {
                let start = self.evaluate_expression(&stmt.start)?.as_int()?;
                let end = self.evaluate_expression(&stmt.end)?.as_int()?;

                for i in start..end {
                    self.symbols
                        .insert(stmt.var_name.clone(), MetadataValue::Integer(i));
                    for body_stmt in &stmt.body {
                        self.execute_statement(body_stmt)?;
                    }
                }
                Ok(())
            }
            GGLStatement::NodeDecl(stmt) => self.execute_node_decl(stmt),
            GGLStatement::EdgeDecl(stmt) => self.execute_edge_decl(stmt),
            GGLStatement::GenerateStmt(stmt) => {
                if let Some(generator) = get_generator(&stmt.name) {
                    let generated =
                        generator(&stmt.params).map_err(|e| format!("Generator error: {e}"))?;

                    // Merge generated graph into current graph
                    for (_, node) in generated.nodes {
                        self.graph.add_node(node);
                    }
                    for (_, edge) in generated.edges {
                        self.graph.add_edge(edge);
                    }
                } else {
                    return Err(format!("Unknown generator: {}", stmt.name));
                }
                Ok(())
            }
            // GGLStatement::RuleDefStmt(stmt) => {
            //     let rule = rules::Rule {
            //         name: stmt.name.clone(),
            //         lhs: stmt.lhs.clone(),
            //         rhs: stmt.rhs.clone(),
            //     };
            //     self.rules.insert(stmt.name.clone(), rule);
            //     Ok(())
            // }
            // GGLStatement::ApplyRuleStmt(stmt) => {
            //     if let Some(rule) = self.rules.get(&stmt.rule_name) {
            //         rule.apply(&mut self.graph, stmt.iterations)
            //             .map_err(|e| format!("Rule application error: {e}"))?;
            //     } else {
            //         return Err(format!("Unknown rule: {}", stmt.rule_name));
            //     }
            //     Ok(())
            // }
            _ => Ok(()),
        }
    }

    // /// Returns the rules defined in the GGL program.
    // pub fn get_rules(&self) -> HashMap<String, rules::Rule> {
    //     self.rules.clone()
    // }

    /// Evaluates an expression to a single `MetadataValue`.
    fn evaluate_expression(&self, expr: &Expression) -> Result<MetadataValue, String> {
        match expr {
            Expression::Value(val) => Ok(val.clone()),
            Expression::Variable(name) => self
                .symbols
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {name}")),
            Expression::BinOp(lhs, op, rhs) => {
                let left = self.evaluate_expression(lhs)?;
                let right = self.evaluate_expression(rhs)?;
                match op {
                    '+' => left.add(&right),
                    '-' => left.sub(&right),
                    '*' => left.mul(&right),
                    '/' => left.div(&right),
                    _ => Err(format!("Unsupported operator: {op}")),
                }
            }
        }
    }

    /// Resolves an interpolated string to a single `String`.
    fn resolve_interpolated_string(
        &self,
        parts: &[InterpolatedStringPart],
    ) -> Result<String, String> {
        let mut result = String::new();
        for part in parts {
            match part {
                InterpolatedStringPart::String(s) => result.push_str(s),
                InterpolatedStringPart::Variable(name) => {
                    let value = self
                        .symbols
                        .get(name)
                        .ok_or_else(|| format!("Undefined variable: {name}"))?;
                    result.push_str(&value.to_string());
                }
                InterpolatedStringPart::Expression(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    result.push_str(&value.to_string());
                }
            }
        }
        Ok(result)
    }

    /// Executes a node declaration statement.
    fn execute_node_decl(&mut self, stmt: &NodeDeclaration) -> Result<(), String> {
        let id = self.resolve_interpolated_string(&stmt.id_parts)?;
        let mut node = Node::new(id);

        if let Some(node_type) = &stmt.node_type {
            node = node.with_type(node_type.clone());
        }

        for (key, expr) in &stmt.attributes {
            let value = self.evaluate_expression(expr)?;
            node = node.with_metadata(key.clone(), value);
        }

        self.graph.add_node(node);
        Ok(())
    }

    /// Executes an edge declaration statement.
    fn execute_edge_decl(&mut self, stmt: &EdgeDeclaration) -> Result<(), String> {
        let source = self.resolve_interpolated_string(&stmt.source_parts)?;
        let target = self.resolve_interpolated_string(&stmt.target_parts)?;

        let id = if let Some(id_parts) = &stmt.id_parts {
            self.resolve_interpolated_string(id_parts)?
        } else {
            format!("e_{source}_{target}")
        };

        let mut edge = Edge::new(id, source, target);

        for (key, expr) in &stmt.attributes {
            let value = self.evaluate_expression(expr)?;
            edge = edge.with_metadata(key.clone(), value);
        }

        self.graph.add_edge(edge);
        Ok(())
    }
}

impl MetadataValue {
    fn as_int(&self) -> Result<i64, String> {
        match self {
            MetadataValue::Integer(i) => Ok(*i),
            _ => Err("Expected integer value".to_string()),
        }
    }

    fn add(&self, other: &MetadataValue) -> Result<MetadataValue, String> {
        match (self, other) {
            (MetadataValue::Integer(a), MetadataValue::Integer(b)) => Ok(MetadataValue::Integer(a + b)),
            (MetadataValue::Float(a), MetadataValue::Float(b)) => Ok(MetadataValue::Float(a + b)),
            _ => Err("Cannot add these types".to_string()),
        }
    }

    fn sub(&self, other: &MetadataValue) -> Result<MetadataValue, String> {
        match (self, other) {
            (MetadataValue::Integer(a), MetadataValue::Integer(b)) => Ok(MetadataValue::Integer(a - b)),
            (MetadataValue::Float(a), MetadataValue::Float(b)) => Ok(MetadataValue::Float(a - b)),
            _ => Err("Cannot subtract these types".to_string()),
        }
    }

    fn mul(&self, other: &MetadataValue) -> Result<MetadataValue, String> {
        match (self, other) {
            (MetadataValue::Integer(a), MetadataValue::Integer(b)) => Ok(MetadataValue::Integer(a * b)),
            (MetadataValue::Float(a), MetadataValue::Float(b)) => Ok(MetadataValue::Float(a * b)),
            _ => Err("Cannot multiply these types".to_string()),
        }
    }

    fn div(&self, other: &MetadataValue) -> Result<MetadataValue, String> {
        match (self, other) {
            (MetadataValue::Integer(a), MetadataValue::Integer(b)) => {
                if *b != 0 {
                    Ok(MetadataValue::Integer(a / b))
                } else {
                    Err("Division by zero".to_string())
                }
            }
            (MetadataValue::Float(a), MetadataValue::Float(b)) => {
                if *b != 0.0 {
                    Ok(MetadataValue::Float(a / b))
                } else {
                    Err("Division by zero".to_string())
                }
            }
            _ => Err("Cannot divide these types".to_string()),
        }
    }
}

impl ToString for MetadataValue {
    fn to_string(&self) -> String {
        match self {
            MetadataValue::String(s) => s.clone(),
            MetadataValue::Integer(i) => i.to_string(),
            MetadataValue::Float(f) => f.to_string(),
            MetadataValue::Boolean(b) => b.to_string(),
        }
    }
}
