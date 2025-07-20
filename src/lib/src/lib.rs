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
//! git clone https://github.com/ocasazza/graph-generation-language.git
//! cd graph-generation-language
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


pub mod generators;
pub mod interpreter;
pub mod parser;
pub mod rules;
pub mod types;

use crate::interpreter::Interpreter;
use crate::parser::parse_ggl;
use crate::types::Graph;

/// The main GGL engine for parsing and executing GGL programs.
///
/// `GGLEngine` maintains the state of a graph and associated transformation rules,
/// allowing you to build complex graph structures through GGL programs.
///
/// # Examples
///
/// ```rust
/// use graph_generation_language::GGLEngine;
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
    // rules: HashMap<String, rules::Rule>,
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
    /// use graph_generation_language::GGLEngine;
    ///
    /// let engine = GGLEngine::new();
    /// ```
    pub fn new() -> Self {
        GGLEngine {
            graph: Graph::new(),
            // rules: HashMap::new(),
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
    /// use graph_generation_language::GGLEngine;
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
        let statements = parse_ggl(ggl_code).map_err(|e| format!("Parse error: {e}"))?;

        // Create a new interpreter and run the statements
        let mut interpreter = Interpreter::new();
        let graph = interpreter.run(&statements)?;

        // Update the engine's state
        self.graph = graph;
        // self.rules = interpreter.get_rules();

        // Serialize final graph to JSON
        serde_json::to_string(&self.graph).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Returns a reference to the current graph.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns a mutable reference to the current graph.
    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.graph
    }

    // /// Returns a reference to the current rules.
    // pub fn rules(&self) -> &HashMap<String, rules::Rule> {
    //     &self.rules
    // }
}
