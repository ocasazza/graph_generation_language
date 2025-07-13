//! # Graph Generation Language (GGL)
//!
//! GGL is a domain-specific language for creating and manipulating graphs through declarative syntax.
//!
//! ## Overview
//!
//! GGL allows you to:
//!
//! * Define graph structures using intuitive node and edge declarations
//! * Generate common graph topologies with built-in generators
//! * Apply transformation rules to modify graph structure
//! * Export graphs in standard JSON format
//!
//! ## Quick Example
//!
//! ```ggl
//! graph social_network {
//!     // Define nodes with types and attributes
//!     node alice :person [name="Alice", age=30];
//!     node bob :person [name="Bob", age=25];
//!
//!     // Create relationships
//!     edge friendship: alice -- bob [strength=0.8];
//!
//!     // Generate additional structure
//!     generate complete {
//!         nodes: 5;
//!         prefix: "user";
//!     }
//!
//!     // Apply transformation rules
//!     rule add_metadata {
//!         lhs { node N :person; }
//!         rhs { node N :person [active=true]; }
//!     }
//!
//!     apply add_metadata 10 times;
//! }
//! ```
//!
//! ## Features
//!
//! * **Declarative Syntax**: Define graphs using intuitive node and edge declarations
//! * **Built-in Generators**: Create common graph structures (complete, path, cycle, grid, star, tree, scale-free)
//! * **Transformation Rules**: Apply pattern-based rules to modify graph structure
//! * **Rich Attributes**: Support for typed nodes and edges with metadata
//! * **JSON Output**: Export graphs in standard JSON format
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
//! git clone https://github.com/ocasazza/graph_generation_language.git
//! cd graph_generation_language
//! cargo build --release
//! ```
//!
//! ### Your First Graph
//!
//! Create a simple graph:
//! ```ggl
//! graph hello_world {
//!     node alice;
//!     node bob;
//!     edge friendship: alice -- bob;
//! }
//! ```
//!
//! ## Modules
//!
//! * [`types`] - Core data structures for nodes, edges, and graphs
//! * [`parser`] - GGL language parser and AST definitions
//! * [`generators`] - Built-in graph generators for common topologies
//! * [`rules`] - Transformation rule engine for graph manipulation

use std::collections::HashMap;

pub mod generators;
pub mod parser;
pub mod rules;
pub mod types;

use crate::generators::get_generator;
use crate::parser::{parse_ggl, GGLStatement};
use crate::types::{Edge, Graph, Node};

/// The main GGL engine for parsing and executing GGL programs.
///
/// `GGLEngine` maintains the state of a graph and associated transformation rules,
/// allowing you to build complex graph structures through GGL programs.
///
/// # Examples
///
/// ```rust
/// use ggl_lib::GGLEngine;
///
/// let mut engine = GGLEngine::new();
/// let ggl_code = r#"
///     graph example {
///         node alice :person;
///         node bob :person;
///         edge: alice -- bob;
///     }
/// "#;
///
/// let result = engine.generate_from_ggl(ggl_code).unwrap();
/// println!("Generated graph: {}", result);
/// ```
pub struct GGLEngine {
    graph: Graph,
    rules: HashMap<String, rules::Rule>,
}

impl Default for GGLEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GGLEngine {
    /// Creates a new GGL engine with an empty graph and no rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ggl_lib::GGLEngine;
    ///
    /// let engine = GGLEngine::new();
    /// ```
    pub fn new() -> Self {
        GGLEngine {
            graph: Graph::new(),
            rules: HashMap::new(),
        }
    }

    /// Parses and executes a GGL program, returning the resulting graph as JSON.
    ///
    /// # Arguments
    ///
    /// * `ggl_code` - A string containing the GGL program to execute
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    /// - `Ok(String)` - JSON representation of the generated graph
    /// - `Err(String)` - Error message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ggl_lib::GGLEngine;
    ///
    /// let mut engine = GGLEngine::new();
    /// let ggl_code = r#"
    ///     graph simple {
    ///         node a;
    ///         node b;
    ///         edge: a -- b;
    ///     }
    /// "#;
    ///
    /// match engine.generate_from_ggl(ggl_code) {
    ///     Ok(json) => println!("Generated graph: {}", json),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn generate_from_ggl(&mut self, ggl_code: &str) -> Result<String, String> {
        // Parse GGL code
        let statements = parse_ggl(ggl_code).map_err(|e| format!("Parse error: {}", e))?;

        // Reset graph state
        self.graph = Graph::new();
        self.rules.clear();

        // Process statements
        for stmt in statements {
            match stmt {
                GGLStatement::NodeDecl(node) => {
                    self.graph.add_node(
                        Node::new(node.id.clone())
                            .with_type(node.node_type.unwrap_or_default())
                            .with_metadata_map(node.attributes),
                    );
                }
                GGLStatement::EdgeDecl(edge) => {
                    self.graph.add_edge(
                        Edge::new(edge.id, edge.source, edge.target)
                            .with_metadata_map(edge.attributes),
                    );
                }
                GGLStatement::GenerateStmt(gen) => {
                    if let Some(generator) = get_generator(&gen.name) {
                        let generated = generator(&gen.params)
                            .map_err(|e| format!("Generator error: {}", e))?;

                        // Merge generated graph into current graph
                        for (_, node) in generated.nodes {
                            self.graph.add_node(node);
                        }
                        for (_, edge) in generated.edges {
                            self.graph.add_edge(edge);
                        }
                    } else {
                        return Err(format!("Unknown generator: {}", gen.name));
                    }
                }
                GGLStatement::RuleDefStmt(rule_def) => {
                    let rule = rules::Rule {
                        name: rule_def.name.clone(),
                        lhs: rule_def.lhs,
                        rhs: rule_def.rhs,
                    };
                    self.rules.insert(rule_def.name, rule);
                }
                GGLStatement::ApplyRuleStmt(apply) => {
                    if let Some(rule) = self.rules.get(&apply.rule_name) {
                        rule.apply(&mut self.graph, apply.iterations)
                            .map_err(|e| format!("Rule application error: {}", e))?;
                    } else {
                        return Err(format!("Unknown rule: {}", apply.rule_name));
                    }
                }
            }
        }

        // Serialize final graph to JSON
        serde_json::to_string(&self.graph).map_err(|e| format!("Serialization error: {}", e))
    }

    /// Returns a reference to the current graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns a mutable reference to the current graph.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    /// Returns a reference to the current rules.
    pub fn rules(&self) -> &HashMap<String, rules::Rule> {
        &self.rules
    }
}
