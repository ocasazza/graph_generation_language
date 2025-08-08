//! # Graph Generation Language (GGL) v0.3.0
//!
//! GGL is a functional JSON-superset language for creating and manipulating graphs.
//! It supports functional programming constructs, method chaining, and built-in operations.
//!
//! ## Overview
//!
//! GGL v0.3.0 allows you to:
//!
//! * Define graphs using JSON-superset syntax with functional programming features
//! * Use lambda expressions and method chaining for complex transformations
//! * Create tagged Node and Edge objects with metadata
//! * Apply functional operations like map, filter, and pipe
//! * Use built-in functions for common operations (range, combinations, include)
//! * Export only nodes and edges in the final output
//!
//! ## Quick Example
//!
//! ```javascript
//! {
//!   transform: (graph) => ({
//!     ...graph,
//!     edges: graph.edges.map(edge => ({ ...edge, weight: 0.5 }))
//!   }),
//!
//!   nodes: range(0..5).map(i => Node {
//!     id: `user${i}`,
//!     meta: { age: 20 + i * 5, active: true }
//!   }),
//!
//!   edges: [
//!     Edge { source: "user0", target: "user1", meta: { type: "friend" } }
//!   ].pipe(transform, 1)
//! }
//! ```

use std::collections::HashMap;
use std::path::Path;
use serde_json::{Value, Map};

pub mod parser;
pub mod types;

// Re-export for backward compatibility
pub use types::{Graph, Node, Edge};

use crate::parser::{
    ChainItem, Expression, MethodCall, TemplatePart, ArithmeticOp, ComparisonOperator,
    parse_ggl
};

/// Comprehensive error type for GGL operations
#[derive(Debug)]
pub enum GGLError {
    ParseError { line: usize, column: usize, message: String },
    TypeError { expected: String, found: String, context: String },
    RuntimeError { message: String, context: String },
    FileError { path: String, error: String },
    ArgumentError { function: String, expected: usize, found: usize },
}

impl std::fmt::Display for GGLError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GGLError::ParseError { line, column, message } => {
                write!(f, "Parse Error at line {line}, column {column}: {message}")
            }
            GGLError::TypeError { expected, found, context } => {
                write!(f, "Type Error in {context}: expected {expected}, found {found}")
            }
            GGLError::RuntimeError { message, context } => {
                write!(f, "Runtime Error in {context}: {message}")
            }
            GGLError::FileError { path, error } => {
                write!(f, "File Error loading '{path}': {error}")
            }
            GGLError::ArgumentError { function, expected, found } => {
                write!(f, "Argument Error in {function}: expected {expected} arguments, found {found}")
            }
        }
    }
}

impl std::error::Error for GGLError {}

type Result<T> = std::result::Result<T, GGLError>;

/// Execution context for variable and function scoping
#[derive(Debug, Clone)]
pub struct Context {
    variables: HashMap<String, Value>,
    functions: HashMap<String, (Vec<String>, Expression)>, // (params, body)
}

impl Context {
    fn new() -> Self {
        Context {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn with_variable(&self, name: String, value: Value) -> Self {
        let mut new_context = self.clone();
        new_context.variables.insert(name, value);
        new_context
    }

    fn with_function(&self, name: String, params: Vec<String>, body: Expression) -> Self {
        let mut new_context = self.clone();
        new_context.functions.insert(name, (params, body));
        new_context
    }

    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    fn get_function(&self, name: &str) -> Option<&(Vec<String>, Expression)> {
        self.functions.get(name)
    }
}

/// The main GGL engine for parsing and executing GGL programs
pub struct GGLEngine {
    context: Context,
    base_path: std::path::PathBuf,
}

impl Default for GGLEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GGLEngine {
    /// Creates a new GGL engine
    pub fn new() -> Self {
        GGLEngine {
            context: Context::new(),
            base_path: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        }
    }

    /// Sets the base path for relative file inclusions
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }

    /// Parses and executes a GGL program, returning only nodes and edges as JSON
    pub fn generate_from_ggl(&mut self, ggl_code: &str) -> std::result::Result<String, String> {
        match self.evaluate_ggl(ggl_code) {
            Ok(json) => Ok(json),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Evaluates GGL code and returns filtered JSON output
    pub fn evaluate_ggl(&mut self, ggl_code: &str) -> Result<String> {
        let ast = parse_ggl(ggl_code).map_err(|e| GGLError::ParseError {
            line: 1,  // pest errors contain position info, but simplifying for now
            column: 1,
            message: e.to_string(),
        })?;

        let result = self.evaluate_expression(&ast.root, &self.context.clone())?;
        let filtered = self.filter_reserved_keys(result)?;

        serde_json::to_string_pretty(&filtered).map_err(|e| GGLError::RuntimeError {
            message: e.to_string(),
            context: "JSON serialization".to_string(),
        })
    }

    /// Filters result to only include nodes and edges keys
    fn filter_reserved_keys(&self, value: Value) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                let mut filtered = Map::new();

                if let Some(nodes) = obj.get("nodes") {
                    filtered.insert("nodes".to_string(), nodes.clone());
                }

                if let Some(edges) = obj.get("edges") {
                    filtered.insert("edges".to_string(), edges.clone());
                }

                Ok(Value::Object(filtered))
            }
            _ => Err(GGLError::TypeError {
                expected: "object".to_string(),
                found: format!("{value}"),
                context: "root expression".to_string(),
            })
        }
    }

    /// Evaluates an expression in the given context
    fn evaluate_expression(&self, expr: &Expression, context: &Context) -> Result<Value> {
        match expr {
            Expression::ObjectExpression(pairs) => {
                self.evaluate_object_expression(pairs, context)
            }
            Expression::TaggedObject { tag, fields } => {
                self.evaluate_tagged_object(tag, fields, context)
            }
            Expression::ArrayExpression(elements) => {
                self.evaluate_array_expression(elements, context)
            }
            Expression::ChainExpression { base, chain } => {
                self.evaluate_chain_expression(base, chain, context)
            }
            Expression::BuiltinCall { name, args } => {
                self.evaluate_builtin_call(name, args, context)
            }
            Expression::FunctionDefinition { name, params, body } => {
                self.evaluate_function_definition(name, params, body, context)
            }
            Expression::LambdaExpression { params, body } => {
                self.evaluate_lambda_expression(params, body, context)
            }
            Expression::TemplateLiteral { parts } => {
                self.evaluate_template_literal(parts, context)
            }
            Expression::ArithmeticExpression(op) => {
                self.evaluate_arithmetic_expression(op, context)
            }
            Expression::ComparisonExpression { left, operator, right } => {
                self.evaluate_comparison_expression(left, operator, right, context)
            }
            Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
            Expression::Integer(i) => Ok(Value::Number(serde_json::Number::from(*i))),
            Expression::Float(f) => Ok(Value::Number(serde_json::Number::from_f64(*f).unwrap())),
            Expression::Boolean(b) => Ok(Value::Bool(*b)),
            Expression::Null => Ok(Value::Null),
            Expression::Identifier(name) => {
                if let Some(value) = context.get_variable(name) {
                    Ok(value.clone())
                } else {
                    // For backward compatibility, treat undefined identifiers as string literals
                    // This handles cases where variable names are used as string constants
                    Ok(Value::String(name.clone()))
                }
            }
            Expression::SpreadExpression(expr) => {
                // Evaluate the inner expression and ensure it's an array
                let value = self.evaluate_expression(expr, context)?;
                match value {
                    Value::Array(_) => Ok(value),
                    _ => {
                        // If it's not an array, check if it's an identifier that should resolve to an array
                        if let Expression::Identifier(name) = expr.as_ref() {
                            if let Some(array_value) = context.get_variable(name) {
                                if let Value::Array(_) = array_value {
                                    Ok(array_value.clone())
                                } else {
                                    Err(GGLError::TypeError {
                                        expected: "array".to_string(),
                                        found: format!("{array_value}"),
                                        context: "spread operator".to_string(),
                                    })
                                }
                            } else {
                                Err(GGLError::TypeError {
                                    expected: "array".to_string(),
                                    found: format!("{value}"),
                                    context: "spread operator".to_string(),
                                })
                            }
                        } else {
                            Err(GGLError::TypeError {
                                expected: "array".to_string(),
                                found: format!("{value}"),
                                context: "spread operator".to_string(),
                            })
                        }
                    }
                }
            }
            Expression::BlockExpression { statements, result } => {
                let mut block_context = context.clone();

                // Execute statements and bind variables
                for stmt in statements {
                    match stmt {
                        Expression::VariableDeclaration { name, value } => {
                            let var_value = self.evaluate_expression(value, &block_context)?;
                            block_context = block_context.with_variable(name.clone(), var_value);
                        }
                        _ => {
                            self.evaluate_expression(stmt, &block_context)?;
                        }
                    }
                }

                // Return result expression
                self.evaluate_expression(result, &block_context)
            }
            Expression::VariableDeclaration { name: _, value } => {
                // Variable declarations evaluate their value and bind it in context
                // For now, just evaluate the value and return it
                self.evaluate_expression(value, context)
            }
            Expression::IfExpression { condition, then_block, else_block } => {
                let condition_value = self.evaluate_expression(condition, context)?;

                // Evaluate condition as boolean
                let is_true = match condition_value {
                    Value::Bool(b) => b,
                    Value::Null => false,
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::String(s) => !s.is_empty(),
                    Value::Array(arr) => !arr.is_empty(),
                    Value::Object(obj) => !obj.is_empty(),
                };

                if is_true {
                    self.evaluate_expression(then_block, context)
                } else if let Some(else_block) = else_block {
                    self.evaluate_expression(else_block, context)
                } else {
                    Ok(Value::Null)
                }
            }
            Expression::ReturnStatement(expr) => {
                // Return statements just evaluate their expression and return it
                // In a full implementation, this would have early-return semantics
                self.evaluate_expression(expr, context)
            }
        }
    }

    fn evaluate_object_expression(&self, pairs: &HashMap<String, Expression>, context: &Context) -> Result<Value> {
        let mut object = Map::new();
        let mut new_context = context.clone();

        // First pass: collect all function definitions
        for (key, value_expr) in pairs {
            match value_expr {
                Expression::FunctionDefinition { name, params, body } => {
                    new_context = new_context.with_function(name.clone(), params.clone(), *body.clone());
                }
                Expression::LambdaExpression { params, body } => {
                    new_context = new_context.with_function(key.clone(), params.clone(), *body.clone());
                }
                _ => {} // Skip non-function expressions for now
            }
        }

        // Second pass: evaluate all non-function expressions in dependency order
        let non_function_pairs: Vec<_> = pairs.iter()
            .filter(|(_, value_expr)| !matches!(value_expr, Expression::FunctionDefinition { .. } | Expression::LambdaExpression { .. }))
            .collect();

        // Sort pairs by dependency - expressions without dependencies first
        let mut remaining_pairs = non_function_pairs.clone();
        let mut evaluation_order = Vec::new();

        while !remaining_pairs.is_empty() {
            let mut found_independent = false;

            // Find expressions that don't depend on remaining unevaluated variables
            for i in (0..remaining_pairs.len()).rev() {
                let (_key, value_expr) = remaining_pairs[i];
                let dependencies = get_expression_dependencies(value_expr);

                // Check if all dependencies are already evaluated or are not in the remaining pairs
                let all_deps_satisfied = dependencies.iter().all(|dep| {
                    new_context.get_variable(dep).is_some() ||
                    !remaining_pairs.iter().any(|(k, _)| *k == dep)
                });

                if all_deps_satisfied {
                    let (key, value_expr) = remaining_pairs.remove(i);
                    evaluation_order.push((key, value_expr));
                    found_independent = true;
                }
            }

            // If we couldn't find any independent expressions, break to avoid infinite loop
            if !found_independent {
                // Add remaining in original order
                evaluation_order.extend(remaining_pairs);
                break;
            }
        }

        // Evaluate expressions in dependency order
        for (key, value_expr) in evaluation_order {
            // For chain expressions, pass the updated context so they have access to all variables
            let value = match value_expr {
                Expression::ChainExpression { base, chain } => {
                    self.evaluate_chain_expression_with_context(base, chain, &new_context)
                }
                _ => self.evaluate_expression(value_expr, &new_context)
            }?;
            object.insert((*key).clone(), value.clone());
            new_context = new_context.with_variable((*key).clone(), value);
        }

        Ok(Value::Object(object))
    }

    fn evaluate_tagged_object(&self, tag: &str, fields: &HashMap<String, Expression>, context: &Context) -> Result<Value> {
        let mut object = Map::new();

        for (key, value_expr) in fields {
            let value = self.evaluate_expression(value_expr, context)?;
            object.insert(key.clone(), value);
        }

        // Add tag information for Node/Edge objects
        match tag {
            "Node" => {
                // Ensure required 'id' field exists
                if !object.contains_key("id") {
                    return Err(GGLError::RuntimeError {
                        message: "Node must have 'id' field".to_string(),
                        context: "Node object creation".to_string(),
                    });
                }
            }
            "Edge" => {
                // Ensure required 'source' and 'target' fields exist
                if !object.contains_key("source") || !object.contains_key("target") {
                    return Err(GGLError::RuntimeError {
                        message: "Edge must have 'source' and 'target' fields".to_string(),
                        context: "Edge object creation".to_string(),
                    });
                }
            }
            _ => {}
        }

        Ok(Value::Object(object))
    }

    fn evaluate_array_expression(&self, elements: &[Expression], context: &Context) -> Result<Value> {
        let mut array = Vec::new();

        for element in elements {
            match element {
                Expression::SpreadExpression(inner_expr) => {
                    // Handle spread operator by expanding the array
                    let value = self.evaluate_expression(inner_expr, context)?;
                    match value {
                        Value::Array(inner_array) => {
                            // Spread the elements of the inner array
                            array.extend(inner_array);
                        }
                        _ => {
                            return Err(GGLError::TypeError {
                                expected: "array".to_string(),
                                found: format!("{value}"),
                                context: "spread operator".to_string(),
                            });
                        }
                    }
                }
                _ => {
                    let value = self.evaluate_expression(element, context)?;
                    array.push(value);
                }
            }
        }

        Ok(Value::Array(array))
    }

    fn evaluate_chain_expression(&self, base: &Expression, chain: &[ChainItem], context: &Context) -> Result<Value> {
        let mut current = self.evaluate_expression(base, context)?;

        for item in chain {
            current = match item {
                ChainItem::MethodCall { name, args } => {
                    let method_call = MethodCall {
                        name: name.clone(),
                        args: args.clone(),
                    };
                    self.apply_method(current, &method_call, context)?
                }
                ChainItem::PropertyAccess { name } => {
                    self.property_access(current, name, context)?
                }
                ChainItem::BuiltinCall { name, args } => {
                    // For builtin calls in chains, apply them as methods on the current value
                    let method_call = MethodCall {
                        name: name.clone(),
                        args: args.clone(),
                    };
                    self.apply_method(current, &method_call, context)?
                }
            }
        }

        Ok(current)
    }

    fn evaluate_chain_expression_with_context(&self, base: &Expression, chain: &[ChainItem], context: &Context) -> Result<Value> {
        // This version passes the updated context through to all method calls
        let mut current = self.evaluate_expression(base, context)?;

        for item in chain {
            current = match item {
                ChainItem::MethodCall { name, args } => {
                    let method_call = MethodCall {
                        name: name.clone(),
                        args: args.clone(),
                    };
                    self.apply_method_with_context(current, &method_call, context)?
                }
                ChainItem::PropertyAccess { name } => {
                    self.property_access(current, name, context)?
                }
                ChainItem::BuiltinCall { name, args } => {
                    // For builtin calls in chains, apply them as methods on the current value
                    let method_call = MethodCall {
                        name: name.clone(),
                        args: args.clone(),
                    };
                    self.apply_method_with_context(current, &method_call, context)?
                }
            }
        }

        Ok(current)
    }

    fn apply_method(&self, value: Value, method: &MethodCall, context: &Context) -> Result<Value> {
        // Handle method calls
        match method.name.as_str() {
            "map" => self.array_map(value, &method.args, context),
            "filter" => self.array_filter(value, &method.args, context),
            "pipe" => self.array_pipe(value, &method.args, context),
            "concat" => self.array_concat(value, &method.args, context),
            "slice" => self.array_slice(value, &method.args, context),
            "reduce" => self.array_reduce(value, &method.args, context),
            "flat" => self.array_flat(value, &method.args, context),
            "find" => self.array_find(value, &method.args, context),
            "floor" => self.math_floor(value, &method.args, context),
            "sqrt" => self.math_sqrt(value, &method.args, context),
            "pow" => self.math_pow(value, &method.args, context),
            "abs" => self.math_abs(value, &method.args, context),
            _ => Err(GGLError::RuntimeError {
                message: format!("Unknown method: {}", method.name),
                context: "method call".to_string(),
            })
        }
    }

    fn apply_method_with_context(&self, value: Value, method: &MethodCall, context: &Context) -> Result<Value> {
        // Handle method calls with enhanced context passing
        match method.name.as_str() {
            "map" => self.array_map(value, &method.args, context),
            "filter" => self.array_filter(value, &method.args, context),
            "pipe" => self.array_pipe_with_context(value, &method.args, context),
            "concat" => self.array_concat(value, &method.args, context),
            "slice" => self.array_slice(value, &method.args, context),
            "reduce" => self.array_reduce(value, &method.args, context),
            "flat" => self.array_flat(value, &method.args, context),
            "find" => self.array_find(value, &method.args, context),
            "floor" => self.math_floor(value, &method.args, context),
            "sqrt" => self.math_sqrt(value, &method.args, context),
            "pow" => self.math_pow(value, &method.args, context),
            "abs" => self.math_abs(value, &method.args, context),
            _ => Err(GGLError::RuntimeError {
                message: format!("Unknown method: {}", method.name),
                context: "method call".to_string(),
            })
        }
    }

    fn array_map(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "map".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        if let Value::Array(array) = value {
            let lambda = &args[0];
            let mut result = Vec::new();

            for item in array {
                let mapped = self.apply_lambda(lambda, &[item], context)?;
                result.push(mapped);
            }

            Ok(Value::Array(result))
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "map method".to_string(),
            })
        }
    }

    fn array_filter(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "filter".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        if let Value::Array(array) = value {
            let lambda = &args[0];
            let mut result = Vec::new();

            for item in array {
                let keep = self.apply_lambda(lambda, &[item.clone()], context)?;
                if let Value::Bool(true) = keep {
                    result.push(item);
                }
            }

            Ok(Value::Array(result))
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "filter method".to_string(),
            })
        }
    }

    fn array_pipe(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 2 {
            return Err(GGLError::ArgumentError {
                function: "pipe".to_string(),
                expected: 2,
                found: args.len(),
            });
        }

        let transform_fn = &args[0];
        let iterations = self.evaluate_expression(&args[1], context)?;

        let iter_count = if let Value::Number(n) = iterations {
            n.as_u64().unwrap_or(0) as usize
        } else {
            return Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{iterations}"),
                context: "pipe iterations".to_string(),
            });
        };

        // Create a graph object with both nodes and edges
        let mut graph_obj = Map::new();

        // If we have nodes in the context, add them to the graph
        if let Some(nodes) = context.get_variable("nodes") {
            graph_obj.insert("nodes".to_string(), nodes.clone());
        }

        // Add the current value (edges) to the graph
        graph_obj.insert("edges".to_string(), value);

        let mut current = Value::Object(graph_obj);

        for _ in 0..iter_count {
            current = self.apply_lambda(transform_fn, &[current], context)?;
        }

        // Extract just the edges from the resulting graph object
        if let Value::Object(ref obj) = current {
            if let Some(edges) = obj.get("edges") {
                Ok(edges.clone())
            } else {
                Ok(Value::Array(vec![]))
            }
        } else {
            Ok(current)
        }
    }

    fn array_pipe_with_context(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        // This version ensures the lambda functions have access to the updated context
        if args.len() != 2 {
            return Err(GGLError::ArgumentError {
                function: "pipe".to_string(),
                expected: 2,
                found: args.len(),
            });
        }

        let transform_fn = &args[0];
        let iterations = self.evaluate_expression(&args[1], context)?;

        let iter_count = if let Value::Number(n) = iterations {
            n.as_u64().unwrap_or(0) as usize
        } else {
            return Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{iterations}"),
                context: "pipe iterations".to_string(),
            });
        };

        // Create a graph object with both nodes and edges using the enhanced context
        let mut graph_obj = Map::new();

        // If we have nodes in the context, add them to the graph
        if let Some(nodes) = context.get_variable("nodes") {
            graph_obj.insert("nodes".to_string(), nodes.clone());
        }

        // Add the current value (edges) to the graph
        graph_obj.insert("edges".to_string(), value);

        let mut current = Value::Object(graph_obj);
        let mut updated_context = context.clone();

        for _ in 0..iter_count {
            // Update the context with the current graph state before each iteration
            if let Value::Object(ref current_obj) = current {
                if let Some(nodes) = current_obj.get("nodes") {
                    updated_context = updated_context.with_variable("nodes".to_string(), nodes.clone());
                }
                if let Some(edges) = current_obj.get("edges") {
                    updated_context = updated_context.with_variable("edges".to_string(), edges.clone());
                }
                // Also add the entire graph object as a variable for property access
                updated_context = updated_context.with_variable("graph".to_string(), current.clone());
            }

            // Pass the updated context to the lambda
            current = self.apply_lambda(transform_fn, &[current], &updated_context)?;
        }

        // Extract just the edges from the resulting graph object
        if let Value::Object(ref obj) = current {
            if let Some(edges) = obj.get("edges") {
                Ok(edges.clone())
            } else {
                Ok(Value::Array(vec![]))
            }
        } else {
            Ok(current)
        }
    }

    fn array_concat(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "concat".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        if let Value::Array(mut array1) = value {
            let other = self.evaluate_expression(&args[0], context)?;
            if let Value::Array(array2) = other {
                array1.extend(array2);
                Ok(Value::Array(array1))
            } else {
                Err(GGLError::TypeError {
                    expected: "array".to_string(),
                    found: format!("{other}"),
                    context: "concat argument".to_string(),
                })
            }
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "concat method".to_string(),
            })
        }
    }

    fn array_slice(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.is_empty() || args.len() > 2 {
            return Err(GGLError::ArgumentError {
                function: "slice".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        match value {
            Value::Array(array) => {
                let start = self.evaluate_expression(&args[0], context)?;
                let start_idx = if let Value::Number(n) = start {
                    n.as_u64().unwrap_or(0) as usize
                } else {
                    return Err(GGLError::TypeError {
                        expected: "number".to_string(),
                        found: format!("{start}"),
                        context: "slice start".to_string(),
                    });
                };

                let end_idx = if args.len() == 2 {
                    let end = self.evaluate_expression(&args[1], context)?;
                    if let Value::Number(n) = end {
                        n.as_u64().unwrap_or(array.len() as u64) as usize
                    } else {
                        return Err(GGLError::TypeError {
                            expected: "number".to_string(),
                            found: format!("{end}"),
                            context: "slice end".to_string(),
                        });
                    }
                } else {
                    array.len() // Default to end of array
                };

                let sliced = array.get(start_idx..end_idx.min(array.len()))
                    .unwrap_or(&[])
                    .to_vec();

                Ok(Value::Array(sliced))
            }
            Value::String(string) => {
                // Handle string slicing (like JavaScript)
                let start = self.evaluate_expression(&args[0], context)?;
                let start_idx = if let Value::Number(n) = start {
                    n.as_u64().unwrap_or(0) as usize
                } else {
                    return Err(GGLError::TypeError {
                        expected: "number".to_string(),
                        found: format!("{start}"),
                        context: "slice start".to_string(),
                    });
                };

                let end_idx = if args.len() == 2 {
                    let end = self.evaluate_expression(&args[1], context)?;
                    if let Value::Number(n) = end {
                        n.as_u64().unwrap_or(string.len() as u64) as usize
                    } else {
                        return Err(GGLError::TypeError {
                            expected: "number".to_string(),
                            found: format!("{end}"),
                            context: "slice end".to_string(),
                        });
                    }
                } else {
                    string.len() // Default to end of string
                };

                let sliced = string.chars()
                    .skip(start_idx)
                    .take(end_idx.saturating_sub(start_idx))
                    .collect::<String>();

                Ok(Value::String(sliced))
            }
            _ => Err(GGLError::TypeError {
                expected: "array or string".to_string(),
                found: format!("{value}"),
                context: "slice method".to_string(),
            })
        }
    }

    fn array_reduce(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 2 {
            return Err(GGLError::ArgumentError {
                function: "reduce".to_string(),
                expected: 2,
                found: args.len(),
            });
        }

        if let Value::Array(array) = value {
            let lambda = &args[0];
            let mut accumulator = self.evaluate_expression(&args[1], context)?;

            for item in array {
                accumulator = self.apply_lambda(lambda, &[accumulator, item], context)?;
            }

            Ok(accumulator)
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "reduce method".to_string(),
            })
        }
    }

    fn array_flat(&self, value: Value, args: &[Expression], _context: &Context) -> Result<Value> {
        if !args.is_empty() {
            return Err(GGLError::ArgumentError {
                function: "flat".to_string(),
                expected: 0,
                found: args.len(),
            });
        }

        if let Value::Array(array) = value {
            let mut result = Vec::new();
            for item in array {
                if let Value::Array(inner_array) = item {
                    result.extend(inner_array);
                } else {
                    result.push(item);
                }
            }
            Ok(Value::Array(result))
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "flat method".to_string(),
            })
        }
    }

    fn array_find(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "find".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        if let Value::Array(array) = value {
            let lambda = &args[0];

            for item in array {
                let matches = self.apply_lambda(lambda, &[item.clone()], context)?;
                if let Value::Bool(true) = matches {
                    return Ok(item);
                }
            }

            Ok(Value::Null)
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{value}"),
                context: "find method".to_string(),
            })
        }
    }

    fn math_floor(&self, value: Value, args: &[Expression], _context: &Context) -> Result<Value> {
        if !args.is_empty() {
            return Err(GGLError::ArgumentError {
                function: "floor".to_string(),
                expected: 0,
                found: args.len(),
            });
        }

        if let Value::Number(n) = value {
            let float_val = n.as_f64().unwrap_or(0.0);
            let floored = float_val.floor() as i64;
            Ok(Value::Number(serde_json::Number::from(floored)))
        } else {
            Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{value}"),
                context: "floor method".to_string(),
            })
        }
    }

    fn math_sqrt(&self, value: Value, args: &[Expression], _context: &Context) -> Result<Value> {
        if !args.is_empty() {
            return Err(GGLError::ArgumentError {
                function: "sqrt".to_string(),
                expected: 0,
                found: args.len(),
            });
        }

        if let Value::Number(n) = value {
            let float_val = n.as_f64().unwrap_or(0.0);
            if float_val < 0.0 {
                return Err(GGLError::RuntimeError {
                    message: "Cannot take square root of negative number".to_string(),
                    context: "sqrt method".to_string(),
                });
            }
            let result = float_val.sqrt();
            Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
        } else {
            Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{value}"),
                context: "sqrt method".to_string(),
            })
        }
    }

    fn math_pow(&self, value: Value, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "pow".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        if let Value::Number(base) = value {
            let exponent = self.evaluate_expression(&args[0], context)?;
            if let Value::Number(exp) = exponent {
                let base_val = base.as_f64().unwrap_or(0.0);
                let exp_val = exp.as_f64().unwrap_or(0.0);
                let result = base_val.powf(exp_val);
                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            } else {
                Err(GGLError::TypeError {
                    expected: "number".to_string(),
                    found: format!("{exponent}"),
                    context: "pow exponent".to_string(),
                })
            }
        } else {
            Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{value}"),
                context: "pow method".to_string(),
            })
        }
    }

    fn math_abs(&self, value: Value, args: &[Expression], _context: &Context) -> Result<Value> {
        if !args.is_empty() {
            return Err(GGLError::ArgumentError {
                function: "abs".to_string(),
                expected: 0,
                found: args.len(),
            });
        }

        if let Value::Number(n) = value {
            let float_val = n.as_f64().unwrap_or(0.0);
            let result = float_val.abs();
            if result.fract() == 0.0 && result >= i64::MIN as f64 && result <= i64::MAX as f64 {
                Ok(Value::Number(serde_json::Number::from(result as i64)))
            } else {
                Ok(Value::Number(serde_json::Number::from_f64(result).unwrap()))
            }
        } else {
            Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{value}"),
                context: "abs method".to_string(),
            })
        }
    }

    #[allow(dead_code)]
    fn evaluate_property_access_chain(&self, base: &str, properties: &[String], context: &Context) -> Result<Value> {
        // Start with the base variable
        let mut current = if let Some(value) = context.get_variable(base) {
            value.clone()
        } else {
            return Err(GGLError::RuntimeError {
                message: format!("Undefined variable: {base}"),
                context: "property access chain".to_string(),
            });
        };

        // Chain through properties
        for property in properties {
            current = self.property_access(current, property, context)?;
        }

        Ok(current)
    }

    fn property_access(&self, value: Value, property: &str, context: &Context) -> Result<Value> {
        match value {
            Value::Object(obj) => {
                if let Some(prop_value) = obj.get(property) {
                    Ok(prop_value.clone())
                } else {
                    // If property not found in object, check context for common graph properties
                    match property {
                        "nodes" => {
                            if let Some(nodes) = context.get_variable("nodes") {
                                Ok(nodes.clone())
                            } else {
                                Ok(Value::Array(vec![]))
                            }
                        }
                        "edges" => {
                            if let Some(edges) = context.get_variable("edges") {
                                Ok(edges.clone())
                            } else {
                                Ok(Value::Array(vec![]))
                            }
                        }
                        _ => Ok(Value::Null)
                    }
                }
            }
            Value::Array(arr) => {
                match property {
                    "length" => Ok(Value::Number(serde_json::Number::from(arr.len()))),
                    "edges" => Ok(Value::Array(arr)), // Return the array itself when accessing .edges on an edges array
                    "nodes" => {
                        // When accessing .nodes on an edges array, look in context for nodes
                        if let Some(nodes) = context.get_variable("nodes") {
                            Ok(nodes.clone())
                        } else {
                            Ok(Value::Array(vec![])) // Return empty array if nodes not found
                        }
                    }
                    _ => Ok(Value::Null)
                }
            }
            _ => Err(GGLError::TypeError {
                expected: "object or array".to_string(),
                found: format!("{value}"),
                context: format!("property access .{property}"),
            })
        }
    }

    fn apply_lambda(&self, lambda_expr: &Expression, args: &[Value], context: &Context) -> Result<Value> {
        match lambda_expr {
            Expression::LambdaExpression { params, body } => {
                if args.len() != params.len() {
                    return Err(GGLError::ArgumentError {
                        function: "lambda".to_string(),
                        expected: params.len(),
                        found: args.len(),
                    });
                }

                let mut lambda_context = context.clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    // Handle destructuring assignment for array parameters like [a, b]
                    if param.starts_with('[') && param.ends_with(']') {
                        // Parse destructuring pattern like "[a, b]"
                        let inner = &param[1..param.len()-1]; // Remove [ and ]
                        let var_names: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

                        if let Value::Array(arr) = arg {
                            for (i, var_name) in var_names.iter().enumerate() {
                                if i < arr.len() {
                                    lambda_context = lambda_context.with_variable(var_name.to_string(), arr[i].clone());
                                } else {
                                    lambda_context = lambda_context.with_variable(var_name.to_string(), Value::Null);
                                }
                            }
                        } else {
                            return Err(GGLError::TypeError {
                                expected: "array for destructuring".to_string(),
                                found: format!("{arg}"),
                                context: "lambda destructuring".to_string(),
                            });
                        }
                    } else {
                        lambda_context = lambda_context.with_variable(param.clone(), arg.clone());
                    }
                }

                self.evaluate_expression(body, &lambda_context)
            }
            Expression::Identifier(name) => {
                // Look up function by name
                if let Some((params, body)) = context.get_function(name) {
                    if args.len() != params.len() {
                        return Err(GGLError::ArgumentError {
                            function: name.clone(),
                            expected: params.len(),
                            found: args.len(),
                        });
                    }

                    let mut func_context = context.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        func_context = func_context.with_variable(param.clone(), arg.clone());
                    }

                    self.evaluate_expression(body, &func_context)
                } else {
                    Err(GGLError::RuntimeError {
                        message: format!("Unknown function: {name}"),
                        context: "function call".to_string(),
                    })
                }
            }
            _ => Err(GGLError::TypeError {
                expected: "lambda or function".to_string(),
                found: format!("{lambda_expr:?}"),
                context: "function application".to_string(),
            })
        }
    }

    fn evaluate_builtin_call(&self, name: &str, args: &[Expression], context: &Context) -> Result<Value> {
        match name {
            "range" => self.builtin_range(args, context),
            "combinations" => self.builtin_combinations(args, context),
            "include" => self.builtin_include(args, context),
            _ => Err(GGLError::RuntimeError {
                message: format!("Unknown built-in function: {name}"),
                context: "built-in call".to_string(),
            })
        }
    }

    fn builtin_range(&self, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "range".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        // Parse range expression (start..end)
        let range_arg = self.evaluate_expression(&args[0], context)?;

        // For now, expect a string like "0..10" - in a real implementation,
        // this would be handled by the grammar as a range_expr
        if let Value::String(range_str) = range_arg {
            if let Some(dot_pos) = range_str.find("..") {
                let start_str = &range_str[..dot_pos];
                let end_str = &range_str[dot_pos + 2..];

                let start: i64 = start_str.parse().map_err(|_| GGLError::TypeError {
                    expected: "integer".to_string(),
                    found: start_str.to_string(),
                    context: "range start".to_string(),
                })?;

                let end: i64 = end_str.parse().map_err(|_| GGLError::TypeError {
                    expected: "integer".to_string(),
                    found: end_str.to_string(),
                    context: "range end".to_string(),
                })?;

                let range: Vec<Value> = (start..end)
                    .map(|i| Value::Number(serde_json::Number::from(i)))
                    .collect();

                Ok(Value::Array(range))
            } else {
                Err(GGLError::RuntimeError {
                    message: "Invalid range format, expected 'start..end'".to_string(),
                    context: "range parsing".to_string(),
                })
            }
        } else {
            Err(GGLError::TypeError {
                expected: "string (range format)".to_string(),
                found: format!("{range_arg}"),
                context: "range function".to_string(),
            })
        }
    }

    fn builtin_combinations(&self, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 2 {
            return Err(GGLError::ArgumentError {
                function: "combinations".to_string(),
                expected: 2,
                found: args.len(),
            });
        }

        let array = self.evaluate_expression(&args[0], context)?;
        let r = self.evaluate_expression(&args[1], context)?;

        if let Value::Array(items) = array {
            if let Value::Number(r_num) = r {
                let r_val = r_num.as_u64().unwrap_or(0) as usize;

                if r_val > items.len() {
                    return Ok(Value::Array(vec![]));
                }

                let combinations = generate_combinations(&items, r_val);
                Ok(Value::Array(combinations))
            } else {
                Err(GGLError::TypeError {
                    expected: "number".to_string(),
                    found: format!("{r}"),
                    context: "combinations r".to_string(),
                })
            }
        } else {
            Err(GGLError::TypeError {
                expected: "array".to_string(),
                found: format!("{array}"),
                context: "combinations array".to_string(),
            })
        }
    }

    fn builtin_include(&self, args: &[Expression], context: &Context) -> Result<Value> {
        if args.len() != 1 {
            return Err(GGLError::ArgumentError {
                function: "include".to_string(),
                expected: 1,
                found: args.len(),
            });
        }

        let path_value = self.evaluate_expression(&args[0], context)?;

        if let Value::String(path_str) = path_value {
            let file_path = self.base_path.join(&path_str);

            let content = std::fs::read_to_string(&file_path).map_err(|e| GGLError::FileError {
                path: path_str.clone(),
                error: e.to_string(),
            })?;

            // Parse and evaluate the included file
            let ast = parse_ggl(&content).map_err(|e| GGLError::ParseError {
                line: 1,
                column: 1,
                message: format!("In included file '{path_str}': {e}"),
            })?;

            self.evaluate_expression(&ast.root, context)
        } else {
            Err(GGLError::TypeError {
                expected: "string".to_string(),
                found: format!("{path_value}"),
                context: "include path".to_string(),
            })
        }
    }

    fn evaluate_function_definition(&self, _name: &str, _params: &[String], _body: &Expression, _context: &Context) -> Result<Value> {
        // Function definitions don't produce values, they modify context
        // This should be handled at the object level
        Ok(Value::Null)
    }

    fn evaluate_lambda_expression(&self, params: &[String], _body: &Expression, _context: &Context) -> Result<Value> {
        // Lambda expressions are function values - for now return a placeholder
        // In a full implementation, these would be first-class values
        Ok(Value::String(format!("lambda({params:?})")))
    }

    fn evaluate_template_literal(&self, parts: &[TemplatePart], context: &Context) -> Result<Value> {
        let mut result = String::new();

        for part in parts {
            match part {
                TemplatePart::Literal(s) => result.push_str(s),
                TemplatePart::Variable(expr) => {
                    let value = self.evaluate_expression(expr, context)?;
                    let str_value = match value {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => format!("{value}"),
                    };
                    result.push_str(&str_value);
                }
            }
        }

        Ok(Value::String(result))
    }

    fn evaluate_comparison_expression(&self, left: &Expression, operator: &ComparisonOperator, right: &Expression, context: &Context) -> Result<Value> {
        let left_val = self.evaluate_expression(left, context)?;
        let right_val = self.evaluate_expression(right, context)?;

        let result = match (left_val, right_val) {
            (Value::Number(l), Value::Number(r)) => {
                let l_f64 = l.as_f64().unwrap_or(0.0);
                let r_f64 = r.as_f64().unwrap_or(0.0);
                match operator {
                    ComparisonOperator::LessThan => l_f64 < r_f64,
                    ComparisonOperator::GreaterThan => l_f64 > r_f64,
                    ComparisonOperator::LessEqual => l_f64 <= r_f64,
                    ComparisonOperator::GreaterEqual => l_f64 >= r_f64,
                    ComparisonOperator::Equal => (l_f64 - r_f64).abs() < f64::EPSILON,
                    ComparisonOperator::NotEqual => (l_f64 - r_f64).abs() >= f64::EPSILON,
                }
            }
            (Value::String(l), Value::String(r)) => {
                match operator {
                    ComparisonOperator::Equal => l == r,
                    ComparisonOperator::NotEqual => l != r,
                    ComparisonOperator::LessThan => l < r,
                    ComparisonOperator::GreaterThan => l > r,
                    ComparisonOperator::LessEqual => l <= r,
                    ComparisonOperator::GreaterEqual => l >= r,
                }
            }
            (Value::Bool(l), Value::Bool(r)) => {
                match operator {
                    ComparisonOperator::Equal => l == r,
                    ComparisonOperator::NotEqual => l != r,
                    _ => return Err(GGLError::TypeError {
                        expected: "boolean equality comparison only".to_string(),
                        found: format!("boolean {operator:?} comparison"),
                        context: "comparison".to_string(),
                    }),
                }
            }
            (Value::Null, Value::Number(r)) => {
                // Null compared to number: treat null as 0 (JavaScript-like semantics)
                let r_f64 = r.as_f64().unwrap_or(0.0);
                match operator {
                    ComparisonOperator::LessThan => 0.0 < r_f64,
                    ComparisonOperator::GreaterThan => 0.0 > r_f64,
                    ComparisonOperator::LessEqual => 0.0 <= r_f64,
                    ComparisonOperator::GreaterEqual => 0.0 >= r_f64,
                    ComparisonOperator::Equal => false, // null is never equal to a number
                    ComparisonOperator::NotEqual => true, // null is always not equal to a number
                }
            }
            (Value::Number(l), Value::Null) => {
                // Number compared to null: treat null as 0
                let l_f64 = l.as_f64().unwrap_or(0.0);
                match operator {
                    ComparisonOperator::LessThan => l_f64 < 0.0,
                    ComparisonOperator::GreaterThan => l_f64 > 0.0,
                    ComparisonOperator::LessEqual => l_f64 <= 0.0,
                    ComparisonOperator::GreaterEqual => l_f64 >= 0.0,
                    ComparisonOperator::Equal => false, // number is never equal to null
                    ComparisonOperator::NotEqual => true, // number is always not equal to null
                }
            }
            (Value::Null, Value::Null) => {
                // Null compared to null
                match operator {
                    ComparisonOperator::Equal => true,
                    ComparisonOperator::NotEqual => false,
                    ComparisonOperator::LessThan => false,
                    ComparisonOperator::GreaterThan => false,
                    ComparisonOperator::LessEqual => true,
                    ComparisonOperator::GreaterEqual => true,
                }
            }
            (l, r) => {
                // For other non-matching types, only equality comparisons make sense
                match operator {
                    ComparisonOperator::Equal => false, // Different types are never equal
                    ComparisonOperator::NotEqual => true, // Different types are always not equal
                    _ => return Err(GGLError::TypeError {
                        expected: "comparable types".to_string(),
                        found: format!("{l:?} and {r:?}"),
                        context: "comparison".to_string(),
                    }),
                }
            }
        };

        Ok(Value::Bool(result))
    }

    fn evaluate_arithmetic_expression(&self, op: &ArithmeticOp, context: &Context) -> Result<Value> {
        match op {
            ArithmeticOp::Add(left, right) => {
                let l = self.evaluate_expression(left, context)?;
                let r = self.evaluate_expression(right, context)?;
                self.add_values(l, r)
            }
            ArithmeticOp::Subtract(left, right) => {
                let l = self.evaluate_expression(left, context)?;
                let r = self.evaluate_expression(right, context)?;
                self.subtract_values(l, r)
            }
            ArithmeticOp::Multiply(left, right) => {
                let l = self.evaluate_expression(left, context)?;
                let r = self.evaluate_expression(right, context)?;
                self.multiply_values(l, r)
            }
            ArithmeticOp::Divide(left, right) => {
                let l = self.evaluate_expression(left, context)?;
                let r = self.evaluate_expression(right, context)?;
                self.divide_values(l, r)
            }
            ArithmeticOp::Modulo(left, right) => {
                let l = self.evaluate_expression(left, context)?;
                let r = self.evaluate_expression(right, context)?;
                self.modulo_values(l, r)
            }
            ArithmeticOp::Term(expr) => {
                self.evaluate_expression(expr, context)
            }
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a_int), Some(b_int)) = (a.as_i64(), b.as_i64()) {
                    Ok(Value::Number(serde_json::Number::from(a_int + b_int)))
                } else {
                    let a_float = a.as_f64().unwrap_or(0.0);
                    let b_float = b.as_f64().unwrap_or(0.0);
                    Ok(Value::Number(serde_json::Number::from_f64(a_float + b_float).unwrap()))
                }
            }
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.iter().cloned());
                Ok(Value::Array(result))
            }
            _ => Err(GGLError::TypeError {
                expected: "number + number, string + string, or array + array".to_string(),
                found: format!("{left} + {right}"),
                context: "addition".to_string(),
            }),
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Number(l), Value::Number(r)) => {
                if let (Some(l_int), Some(r_int)) = (l.as_i64(), r.as_i64()) {
                    Ok(Value::Number(serde_json::Number::from(l_int - r_int)))
                } else {
                    let l_float = l.as_f64().unwrap_or(0.0);
                    let r_float = r.as_f64().unwrap_or(0.0);
                    Ok(Value::Number(serde_json::Number::from_f64(l_float - r_float).unwrap()))
                }
            }
            _ => Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{left} - {right}"),
                context: "subtraction".to_string(),
            })
        }
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Number(l), Value::Number(r)) => {
                if let (Some(l_int), Some(r_int)) = (l.as_i64(), r.as_i64()) {
                    Ok(Value::Number(serde_json::Number::from(l_int * r_int)))
                } else {
                    let l_float = l.as_f64().unwrap_or(0.0);
                    let r_float = r.as_f64().unwrap_or(0.0);
                    Ok(Value::Number(serde_json::Number::from_f64(l_float * r_float).unwrap()))
                }
            }
            _ => Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{left} * {right}"),
                context: "multiplication".to_string(),
            })
        }
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Number(l), Value::Number(r)) => {
                let l_float = l.as_f64().unwrap_or(0.0);
                let r_float = r.as_f64().unwrap_or(0.0);

                if r_float == 0.0 {
                    return Err(GGLError::RuntimeError {
                        message: "Division by zero".to_string(),
                        context: "division".to_string(),
                    });
                }

                Ok(Value::Number(serde_json::Number::from_f64(l_float / r_float).unwrap()))
            }
            _ => Err(GGLError::TypeError {
                expected: "number".to_string(),
                found: format!("{left} / {right}"),
                context: "division".to_string(),
            })
        }
    }

    fn modulo_values(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Number(l), Value::Number(r)) => {
                if let (Some(l_int), Some(r_int)) = (l.as_i64(), r.as_i64()) {
                    if r_int == 0 {
                        return Err(GGLError::RuntimeError {
                            message: "Modulo by zero".to_string(),
                            context: "modulo".to_string(),
                        });
                    }
                    Ok(Value::Number(serde_json::Number::from(l_int % r_int)))
                } else {
                    Err(GGLError::TypeError {
                        expected: "integer".to_string(),
                        found: format!("{l} % {r}"),
                        context: "modulo".to_string(),
                    })
                }
            }
            _ => Err(GGLError::TypeError {
                expected: "integer".to_string(),
                found: format!("{left} % {right}"),
                context: "modulo".to_string(),
            })
        }
    }
}

/// Gets all variable dependencies from an expression
fn get_expression_dependencies(expr: &Expression) -> Vec<String> {
    let mut deps = Vec::new();
    collect_dependencies(expr, &mut deps);
    deps
}

fn collect_dependencies(expr: &Expression, deps: &mut Vec<String>) {
    match expr {
        Expression::Identifier(name) => {
            if !deps.contains(name) {
                deps.push(name.clone());
            }
        }
        Expression::ChainExpression { base, chain } => {
            collect_dependencies(base, deps);
            // Also collect dependencies from chain method arguments
            for item in chain {
                match item {
                    ChainItem::MethodCall { args, .. } | ChainItem::BuiltinCall { args, .. } => {
                        for arg in args {
                            collect_dependencies(arg, deps);
                        }
                    }
                    ChainItem::PropertyAccess { .. } => {
                        // Property access doesn't introduce dependencies
                    }
                }
            }
        }
        Expression::ArrayExpression(elements) => {
            for elem in elements {
                collect_dependencies(elem, deps);
            }
        }
        Expression::ObjectExpression(pairs) => {
            for value_expr in pairs.values() {
                collect_dependencies(value_expr, deps);
            }
        }
        Expression::TaggedObject { fields, .. } => {
            for value_expr in fields.values() {
                collect_dependencies(value_expr, deps);
            }
        }
        Expression::BuiltinCall { args, .. } => {
            for arg in args {
                collect_dependencies(arg, deps);
            }
        }
        Expression::TemplateLiteral { parts } => {
            for part in parts {
                if let TemplatePart::Variable(expr) = part {
                    collect_dependencies(expr, deps);
                }
            }
        }
        Expression::SpreadExpression(inner_expr) => {
            collect_dependencies(inner_expr, deps);
        }
        Expression::ArithmeticExpression(op) => {
            match op {
                ArithmeticOp::Add(left, right) |
                ArithmeticOp::Subtract(left, right) |
                ArithmeticOp::Multiply(left, right) |
                ArithmeticOp::Divide(left, right) |
                ArithmeticOp::Modulo(left, right) => {
                    collect_dependencies(left, deps);
                    collect_dependencies(right, deps);
                }
                ArithmeticOp::Term(expr) => {
                    collect_dependencies(expr, deps);
                }
            }
        }
        Expression::ComparisonExpression { left, right, .. } => {
            collect_dependencies(left, deps);
            collect_dependencies(right, deps);
        }
        _ => {}
    }
}

/// Checks if an expression might have dependencies on other variables in the same object
#[allow(dead_code)]
fn expression_might_have_dependencies(expr: &Expression) -> bool {
    match expr {
        Expression::ChainExpression { .. } => true,
        Expression::Identifier(_) => true,
        Expression::ArrayExpression(elements) => {
            elements.iter().any(expression_might_have_dependencies)
        }
        Expression::ObjectExpression(pairs) => {
            pairs.values().any(expression_might_have_dependencies)
        }
        Expression::BuiltinCall { args, .. } => {
            args.iter().any(expression_might_have_dependencies)
        }
        Expression::TemplateLiteral { parts } => {
            parts.iter().any(|part| match part {
                TemplatePart::Variable(expr) => expression_might_have_dependencies(expr),
                _ => false,
            })
        }
        _ => false,
    }
}

/// Generates all combinations of r elements from the given array
fn generate_combinations(items: &[Value], r: usize) -> Vec<Value> {
    if r == 0 {
        return vec![Value::Array(vec![])];
    }
    if r > items.len() {
        return vec![];
    }

    let mut result = Vec::new();
    generate_combinations_recursive(items, r, 0, &mut Vec::new(), &mut result);
    result
}

fn generate_combinations_recursive(
    items: &[Value],
    r: usize,
    start: usize,
    current: &mut Vec<Value>,
    result: &mut Vec<Value>
) {
    if current.len() == r {
        result.push(Value::Array(current.clone()));
        return;
    }

    for i in start..items.len() {
        current.push(items[i].clone());
        generate_combinations_recursive(items, r, i + 1, current, result);
        current.pop();
    }
}

#[cfg(test)]
mod lambda_destructuring_tests {
    use super::*;

    #[test]
    fn test_lambda_destructuring() {
        let mut engine = GGLEngine::new();
        let code = r#"
        {
            nodes: [],
            edges: combinations([
                { id: "a", value: 1 },
                { id: "b", value: 2 }
            ], 2).map(([first, second]) => {
                return {
                    source: first.id,
                    target: second.id
                };
            })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Lambda destructuring failed: {:?}", result.err());
    }

    #[test]
    fn test_toroidal_mesh_simple() {
        let mut engine = GGLEngine::new();
        let code = r#"
        {
            nodes: range("0..4").map(i => Node {
                id: `n${i}`,
                meta: { index: i }
            }),

            edges: range("0..3").map(i => Edge {
                source: `n${i}`,
                target: `n${i + 1}`,
                meta: { type: "simple" }
            })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Simple toroidal test failed: {:?}", result.err());
        if let Ok(json_str) = result {
            let graph: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            let edges = graph.get("edges").unwrap().as_array().unwrap();
            assert!(edges.len() > 0, "Should have some edges");
        }
    }
}
