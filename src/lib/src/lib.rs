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
//! ## More Examples
//!
//! ### Torus
//!
//! This example generates a 2D torus by creating a grid of nodes and connecting them with wrap-around edges.
//!
//! ```ggl
//! graph toroidal_mesh {
//!     let rows = 10;
//!     let cols = 10;
//!
//!     // Create the nodes
//!     for i in 0..rows {
//!         for j in 0..cols {
//!             node n{i}_{j};
//!         }
//!     }
//!
//!     // Create the horizontal edges
//!     for i in 0..rows {
//!         for j in 0..cols {
//!             let next_j = (j + 1) % cols;
//!             edge: n{i}_{j} -> n{i}_{next_j};
//!         }
//!     }
//!
//!     // Create the vertical edges
//!     for i in 0..rows {
//!         for j in 0..cols {
//!             let next_i = (i + 1) % rows;
//!             edge: n{i}_{j} -> n{next_i}_{j};
//!         }
//!     }
//! }
//! ```
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

/// Represents a numeric value that can be either an integer or a float.
/// Supports mixed arithmetic operations with proper type promotion.
#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    Integer(i64),
    Float(f64),
}

impl NumericValue {
    /// Addition with type promotion: int + int = int, int + float = float, float + float = float
    pub fn add(&self, other: &NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a + b),
            (NumericValue::Integer(a), NumericValue::Float(b)) => NumericValue::Float(*a as f64 + b),
            (NumericValue::Float(a), NumericValue::Integer(b)) => NumericValue::Float(a + *b as f64),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a + b),
        }
    }

    /// Subtraction with type promotion
    pub fn subtract(&self, other: &NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a - b),
            (NumericValue::Integer(a), NumericValue::Float(b)) => NumericValue::Float(*a as f64 - b),
            (NumericValue::Float(a), NumericValue::Integer(b)) => NumericValue::Float(a - *b as f64),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a - b),
        }
    }

    /// Multiplication with type promotion
    pub fn multiply(&self, other: &NumericValue) -> NumericValue {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => NumericValue::Integer(a * b),
            (NumericValue::Integer(a), NumericValue::Float(b)) => NumericValue::Float(*a as f64 * b),
            (NumericValue::Float(a), NumericValue::Integer(b)) => NumericValue::Float(a * *b as f64),
            (NumericValue::Float(a), NumericValue::Float(b)) => NumericValue::Float(a * b),
        }
    }

    /// Division always returns float for accuracy
    pub fn divide(&self, other: &NumericValue) -> Result<NumericValue, String> {
        let divisor = match other {
            NumericValue::Integer(b) => *b as f64,
            NumericValue::Float(b) => *b,
        };
        if divisor == 0.0 {
            return Err("Division by zero".to_string());
        }
        let dividend = match self {
            NumericValue::Integer(a) => *a as f64,
            NumericValue::Float(a) => *a,
        };
        Ok(NumericValue::Float(dividend / divisor))
    }

    /// Modulo operation (only valid for integers)
    pub fn modulo(&self, other: &NumericValue) -> Result<NumericValue, String> {
        match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => {
                if *b == 0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(NumericValue::Integer(a % b))
                }
            }
            _ => Err("Modulo operation is only supported for integers".to_string()),
        }
    }

    /// Compare two numeric values with the given operator
    pub fn compare(&self, other: &NumericValue, op: &parser::ComparisonOperator) -> bool {
        use parser::ComparisonOperator;
        let (a, b) = match (self, other) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => (*a as f64, *b as f64),
            (NumericValue::Integer(a), NumericValue::Float(b)) => (*a as f64, *b),
            (NumericValue::Float(a), NumericValue::Integer(b)) => (*a, *b as f64),
            (NumericValue::Float(a), NumericValue::Float(b)) => (*a, *b),
        };

        match op {
            ComparisonOperator::LessThan => a < b,
            ComparisonOperator::GreaterThan => a > b,
            ComparisonOperator::LessEqual => a <= b,
            ComparisonOperator::GreaterEqual => a >= b,
            ComparisonOperator::Equal => (a - b).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (a - b).abs() >= f64::EPSILON,
        }
    }

    /// Convert to JSON Value for serialization
    pub fn to_json_value(&self) -> Value {
        match self {
            NumericValue::Integer(i) => Value::Number(serde_json::Number::from(*i)),
            NumericValue::Float(f) => Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0))),
        }
    }

    /// Get as i64 for compatibility with existing code
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            NumericValue::Integer(i) => Some(*i),
            NumericValue::Float(f) => Some(*f as i64),
        }
    }

    /// Get as f64 for compatibility
    pub fn as_f64(&self) -> f64 {
        match self {
            NumericValue::Integer(i) => *i as f64,
            NumericValue::Float(f) => *f,
        }
    }
}

use crate::generators::get_generator;
use crate::parser::{
    ApplyStatement, EdgeDeclaration, Expression, ForStatement, GenerateStatement, IfStatement, LetStatement,
    NodeDeclaration, RuleDefinition, Statement, ConditionalExpression, ArithmeticExpression, AttributeMatch,
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
            Statement::If(stmt) => self.handle_if(stmt),
            Statement::Node(stmt) => self.handle_node(stmt),
            Statement::Edge(stmt) => self.handle_edge(stmt),
            Statement::Generate(stmt) => self.handle_generate(stmt),
            Statement::RuleDef(stmt) => self.handle_rule_def(stmt),
            Statement::Apply(stmt) => self.handle_apply(stmt),
        }
    }

    // --- Statement Handlers ---

    fn handle_let(&mut self, stmt: &LetStatement) -> Result<(), String> {
        let value = match &stmt.value {
            // Handle the special case where the parser returned a placeholder for arithmetic
            Expression::Integer(0) => {
                // This might be a placeholder for a complex arithmetic expression
                // For now, we'll try to evaluate common arithmetic patterns
                // This is a workaround until we have proper arithmetic expression evaluation
                self.evaluate_expression(&stmt.value)?
            },
            _ => self.evaluate_expression(&stmt.value)?
        };
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

    fn handle_if(&mut self, stmt: &IfStatement) -> Result<(), String> {
        let condition_result = self.evaluate_conditional_expression(&stmt.condition)?;
        if condition_result {
            self.execute_statements(&stmt.body)?;
        }
        Ok(())
    }

    fn handle_node(&mut self, stmt: &NodeDeclaration) -> Result<(), String> {
        let id = self.evaluate_expression(&stmt.id)?.to_string().replace('"', "");
        let node_type = match &stmt.node_type {
            Some(expr) => self.evaluate_expression(expr)?.to_string().replace('"', ""),
            None => String::new(),
        };
        let mut metadata = HashMap::new();
        for (key, attr_match) in &stmt.attributes {
            let value = match attr_match {
                AttributeMatch::Exact(expr) => self.evaluate_expression(expr)?,
                AttributeMatch::Condition(_, _) => {
                    return Err("Conditional attributes are not supported in node declarations".to_string());
                }
            };
            metadata.insert(key.clone(), value);
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
        for (key, attr_match) in &stmt.attributes {
            let value = match attr_match {
                AttributeMatch::Exact(expr) => self.evaluate_expression(expr)?,
                AttributeMatch::Condition(_, _) => {
                    return Err("Conditional attributes are not supported in edge declarations".to_string());
                }
            };
            metadata.insert(key.clone(), value);
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

    /// Returns a reference to the current graph.
    pub fn get_graph(&self) -> &Graph {
        &self.graph
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
                            // Try to parse and evaluate as arithmetic expression first
                            if let Ok(arith_value) = self.evaluate_arithmetic_string(var) {
                                let display_value = match arith_value {
                                    NumericValue::Integer(i) => i.to_string(),
                                    NumericValue::Float(f) => f.to_string(),
                                };
                                result.push_str(&display_value);
                            } else {
                                // Fall back to variable lookup
                                let value = self.context.get(var).ok_or(format!("Undefined variable: '{var}'"))?;
                                result.push_str(&value.to_string().replace('"', ""));
                            }
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }

    /// Evaluates a conditional expression and returns a boolean result
    fn evaluate_conditional_expression(&self, condition: &ConditionalExpression) -> Result<bool, String> {
        let left_val = self.evaluate_arithmetic_expression(&condition.left)?;
        let right_val = self.evaluate_arithmetic_expression(&condition.right)?;

        Ok(left_val.compare(&right_val, &condition.operator))
    }

    /// Evaluates an arithmetic expression string and returns a numeric value
    fn evaluate_arithmetic_string(&self, expr_str: &str) -> Result<NumericValue, String> {
        // Simple arithmetic expression evaluator for string interpolation
        // This handles basic cases like "j+1", "i-1", etc.

        // Try addition first
        if let Some(pos) = expr_str.find('+') {
            let left = &expr_str[..pos].trim();
            let right = &expr_str[pos+1..].trim();
            let left_val = self.evaluate_simple_term(left)?;
            let right_val = self.evaluate_simple_term(right)?;
            return Ok(left_val.add(&right_val));
        }

        // Try subtraction
        if let Some(pos) = expr_str.find('-') {
            // Make sure it's not a negative number
            if pos > 0 {
                let left = &expr_str[..pos].trim();
                let right = &expr_str[pos+1..].trim();
                let left_val = self.evaluate_simple_term(left)?;
                let right_val = self.evaluate_simple_term(right)?;
                return Ok(left_val.subtract(&right_val));
            }
        }

        // Try multiplication
        if let Some(pos) = expr_str.find('*') {
            let left = &expr_str[..pos].trim();
            let right = &expr_str[pos+1..].trim();
            let left_val = self.evaluate_simple_term(left)?;
            let right_val = self.evaluate_simple_term(right)?;
            return Ok(left_val.multiply(&right_val));
        }

        // Try division
        if let Some(pos) = expr_str.find('/') {
            let left = &expr_str[..pos].trim();
            let right = &expr_str[pos+1..].trim();
            let left_val = self.evaluate_simple_term(left)?;
            let right_val = self.evaluate_simple_term(right)?;
            return left_val.divide(&right_val);
        }

        // No operator found, treat as simple term
        self.evaluate_simple_term(expr_str.trim())
    }

    /// Evaluates a simple term (variable or literal)
    fn evaluate_simple_term(&self, term: &str) -> Result<NumericValue, String> {
        // Try parsing as integer literal first
        if let Ok(val) = term.parse::<i64>() {
            return Ok(NumericValue::Integer(val));
        }

        // Try parsing as float literal
        if let Ok(val) = term.parse::<f64>() {
            return Ok(NumericValue::Float(val));
        }

        // Try resolving as variable
        if let Some(Value::Number(n)) = self.context.get(term) {
            if let Some(i) = n.as_i64() {
                return Ok(NumericValue::Integer(i));
            } else if let Some(f) = n.as_f64() {
                return Ok(NumericValue::Float(f));
            }
        }

        Err(format!("Cannot evaluate term: {term}"))
    }

    /// Evaluates an arithmetic expression and returns a numeric value
    fn evaluate_arithmetic_expression(&self, arith: &ArithmeticExpression) -> Result<NumericValue, String> {
        match arith {
            ArithmeticExpression::Term(expr) => {
                let val = self.evaluate_expression(expr)?;
                match val {
                    Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Ok(NumericValue::Integer(i))
                        } else if let Some(f) = n.as_f64() {
                            Ok(NumericValue::Float(f))
                        } else {
                            Err(format!("Invalid numeric value: {n}"))
                        }
                    }
                    _ => Err(format!("Expected numeric value, got: {val}"))
                }
            }
            ArithmeticExpression::Add(left, right) => {
                let left_val = self.evaluate_arithmetic_expression(left)?;
                let right_val = self.evaluate_arithmetic_expression(right)?;
                Ok(left_val.add(&right_val))
            }
            ArithmeticExpression::Subtract(left, right) => {
                let left_val = self.evaluate_arithmetic_expression(left)?;
                let right_val = self.evaluate_arithmetic_expression(right)?;
                Ok(left_val.subtract(&right_val))
            }
            ArithmeticExpression::Multiply(left, right) => {
                let left_val = self.evaluate_arithmetic_expression(left)?;
                let right_val = self.evaluate_arithmetic_expression(right)?;
                Ok(left_val.multiply(&right_val))
            }
            ArithmeticExpression::Divide(left, right) => {
                let left_val = self.evaluate_arithmetic_expression(left)?;
                let right_val = self.evaluate_arithmetic_expression(right)?;
                left_val.divide(&right_val)
            }
            ArithmeticExpression::Modulo(left, right) => {
                let left_val = self.evaluate_arithmetic_expression(left)?;
                let right_val = self.evaluate_arithmetic_expression(right)?;
                left_val.modulo(&right_val)
            }
        }
    }
}
