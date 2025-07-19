//! # Core Data Types
//!
//! This module defines the fundamental data structures used throughout GGL:
//! nodes, edges, graphs, and metadata values. These types form the foundation
//! for representing and manipulating graph structures.
//!
//! ## Key Types
//!
//! * [`Node`] - Represents a graph vertex with optional type and metadata
//! * [`Edge`] - Represents a graph edge connecting two nodes
//! * [`Graph`] - Container for nodes and edges with manipulation methods
//! * [`MetadataValue`] - Flexible value type for node and edge attributes
//!
//! ## JSON Serialization
//!
//! All types implement Serde serialization for JSON export:
//!
//! ```json
//! {
//!   "nodes": {
//!     "alice": {
//!       "id": "alice",
//!       "type": "person",
//!       "metadata": {
//!         "name": "Alice Johnson",
//!         "age": 30
//!       },
//!       "x": 0.0,
//!       "y": 0.0
//!     }
//!   },
//!   "edges": {
//!     "friendship": {
//!       "id": "friendship",
//!       "source": "alice",
//!       "target": "bob",
//!       "type": "friend",
//!       "metadata": {
//!         "strength": 0.8
//!       }
//!     }
//!   }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Flexible value type for node and edge metadata attributes.
///
/// `MetadataValue` supports the common data types used in graph attributes:
/// strings, integers, floating-point numbers, and booleans. The enum uses
/// Serde's untagged serialization for clean JSON output.
///
/// # Examples
///
/// ```rust
/// use graph_generation_language::types::MetadataValue;
///
/// let name = MetadataValue::String("Alice".to_string());
/// let age = MetadataValue::Integer(30);
/// let score = MetadataValue::Float(98.5);
/// let active = MetadataValue::Boolean(true);
/// ```
///
/// # JSON Representation
///
/// Values serialize directly without type tags:
/// ```json
/// {
///   "name": "Alice",
///   "age": 30,
///   "score": 98.5,
///   "active": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MetadataValue {
    /// String value for text data
    String(String),
    /// 64-bit signed integer for numeric data
    Integer(i64),
    /// 64-bit floating-point number for decimal data
    Float(f64),
    /// Boolean value for true/false flags
    Boolean(bool),
}

/// Represents a graph node (vertex) with optional type and metadata.
///
/// Nodes are the fundamental building blocks of graphs in GGL. Each node has:
/// - A unique identifier
/// - An optional type for categorization
/// - Arbitrary metadata attributes
/// - Position coordinates for visualization
///
/// # Examples
///
/// ```rust
/// use graph_generation_language::types::{Node, MetadataValue};
///
/// // Simple node
/// let node = Node::new("alice".to_string());
///
/// // Node with type and metadata
/// let person = Node::new("bob".to_string())
///     .with_type("person".to_string())
///     .with_metadata("age".to_string(), MetadataValue::Integer(25))
///     .with_metadata("city".to_string(), MetadataValue::String("NYC".to_string()))
///     .with_position(100.0, 200.0);
/// ```
///
/// # GGL Syntax
///
/// Nodes are declared in GGL using the `node` statement:
/// ```ggl
/// node alice;                              // Simple node
/// node bob :person;                        // Typed node
/// node server :machine [cpu=8, ram=16];   // Node with attributes
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for the node
    pub id: String,
    /// Optional type for categorizing nodes (e.g., "person", "server")
    pub r#type: String,
    /// Key-value metadata attributes
    pub metadata: HashMap<String, MetadataValue>,
    /// X coordinate for visualization
    pub x: f64,
    /// Y coordinate for visualization
    pub y: f64,
}

impl Node {
    /// Creates a new node with the given identifier.
    ///
    /// The node starts with no type, empty metadata, and position (0, 0).
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the node
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Node;
    ///
    /// let node = Node::new("alice".to_string());
    /// assert_eq!(node.id, "alice");
    /// assert_eq!(node.r#type, "");
    /// assert!(node.metadata.is_empty());
    /// ```
    pub fn new(id: String) -> Self {
        Node {
            id,
            r#type: String::new(),
            metadata: HashMap::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    /// Sets the node type using the builder pattern.
    ///
    /// Node types are used for categorization and pattern matching in rules.
    ///
    /// # Arguments
    ///
    /// * `node_type` - Type identifier (e.g., "person", "server", "location")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Node;
    ///
    /// let node = Node::new("alice".to_string())
    ///     .with_type("person".to_string());
    /// assert_eq!(node.r#type, "person");
    /// ```
    pub fn with_type(mut self, node_type: String) -> Self {
        self.r#type = node_type;
        self
    }

    /// Adds a single metadata attribute using the builder pattern.
    ///
    /// Metadata provides additional information about nodes that can be used
    /// in rules, generators, and visualization.
    ///
    /// # Arguments
    ///
    /// * `key` - Attribute name
    /// * `value` - Attribute value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Node, MetadataValue};
    ///
    /// let node = Node::new("alice".to_string())
    ///     .with_metadata("age".to_string(), MetadataValue::Integer(30))
    ///     .with_metadata("active".to_string(), MetadataValue::Boolean(true));
    /// ```
    pub fn with_metadata(mut self, key: String, value: MetadataValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Adds multiple metadata attributes using the builder pattern.
    ///
    /// This method extends the existing metadata with the provided map,
    /// overwriting any existing keys.
    ///
    /// # Arguments
    ///
    /// * `metadata` - HashMap of attribute key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Node, MetadataValue};
    /// use std::collections::HashMap;
    ///
    /// let mut attrs = HashMap::new();
    /// attrs.insert("name".to_string(), MetadataValue::String("Alice".to_string()));
    /// attrs.insert("age".to_string(), MetadataValue::Integer(30));
    ///
    /// let node = Node::new("alice".to_string())
    ///     .with_metadata_map(attrs);
    /// ```
    pub fn with_metadata_map(mut self, metadata: HashMap<String, MetadataValue>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Sets the node position using the builder pattern.
    ///
    /// Position coordinates are used for graph visualization and layout algorithms.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Node;
    ///
    /// let node = Node::new("alice".to_string())
    ///     .with_position(100.0, 200.0);
    /// assert_eq!(node.x, 100.0);
    /// assert_eq!(node.y, 200.0);
    /// ```
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

/// Represents a graph edge connecting two nodes.
///
/// Edges define relationships between nodes in the graph. Each edge has:
/// - A unique identifier
/// - Source and target node references
/// - An optional type for categorization
/// - Arbitrary metadata attributes
///
/// # Examples
///
/// ```rust
/// use graph_generation_language::types::{Edge, MetadataValue};
///
/// // Simple edge
/// let edge = Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string());
///
/// // Edge with type and metadata
/// let friendship = Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string())
///     .with_type("friend".to_string())
///     .with_metadata("strength".to_string(), MetadataValue::Float(0.8))
///     .with_metadata("since".to_string(), MetadataValue::String("2020".to_string()));
/// ```
///
/// # GGL Syntax
///
/// Edges are declared in GGL using the `edge` statement:
/// ```ggl
/// edge friendship: alice -- bob;                    // Named undirected edge
/// edge: employee -> manager;                        // Anonymous directed edge
/// edge connection: server1 -- server2 [weight=0.8]; // Edge with attributes
/// ```
///
/// # Edge Operators
///
/// * `->` : Directed edge (source points to target)
/// * `--` : Undirected edge (bidirectional connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique identifier for the edge
    pub id: String,
    /// ID of the source node
    pub source: String,
    /// ID of the target node
    pub target: String,
    /// Optional type for categorizing edges (e.g., "friend", "connection")
    pub r#type: String,
    /// Key-value metadata attributes
    pub metadata: HashMap<String, MetadataValue>,
}

impl Edge {
    /// Creates a new edge connecting two nodes.
    ///
    /// The edge starts with no type and empty metadata.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the edge
    /// * `source` - ID of the source node
    /// * `target` - ID of the target node
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Edge;
    ///
    /// let edge = Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string());
    /// assert_eq!(edge.id, "e1");
    /// assert_eq!(edge.source, "alice");
    /// assert_eq!(edge.target, "bob");
    /// assert_eq!(edge.r#type, "");
    /// assert!(edge.metadata.is_empty());
    /// ```
    pub fn new(id: String, source: String, target: String) -> Self {
        Edge {
            id,
            source,
            target,
            r#type: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Sets the edge type using the builder pattern.
    ///
    /// Edge types are used for categorization and pattern matching in rules.
    ///
    /// # Arguments
    ///
    /// * `edge_type` - Type identifier (e.g., "friend", "connection", "dependency")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Edge;
    ///
    /// let edge = Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string())
    ///     .with_type("friendship".to_string());
    /// assert_eq!(edge.r#type, "friendship");
    /// ```
    pub fn with_type(mut self, edge_type: String) -> Self {
        self.r#type = edge_type;
        self
    }

    /// Adds a single metadata attribute using the builder pattern.
    ///
    /// Metadata provides additional information about edges that can be used
    /// in rules, analysis, and visualization.
    ///
    /// # Arguments
    ///
    /// * `key` - Attribute name
    /// * `value` - Attribute value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Edge, MetadataValue};
    ///
    /// let edge = Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string())
    ///     .with_metadata("weight".to_string(), MetadataValue::Float(0.8))
    ///     .with_metadata("bidirectional".to_string(), MetadataValue::Boolean(true));
    /// ```
    pub fn with_metadata(mut self, key: String, value: MetadataValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Adds multiple metadata attributes using the builder pattern.
    ///
    /// This method extends the existing metadata with the provided map,
    /// overwriting any existing keys.
    ///
    /// # Arguments
    ///
    /// * `metadata` - HashMap of attribute key-value pairs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Edge, MetadataValue};
    /// use std::collections::HashMap;
    ///
    /// let mut attrs = HashMap::new();
    /// attrs.insert("weight".to_string(), MetadataValue::Float(0.8));
    /// attrs.insert("created".to_string(), MetadataValue::String("2024".to_string()));
    ///
    /// let edge = Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string())
    ///     .with_metadata_map(attrs);
    /// ```
    pub fn with_metadata_map(mut self, metadata: HashMap<String, MetadataValue>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}

/// Container for nodes and edges representing a complete graph structure.
///
/// `Graph` is the main data structure that holds all nodes and edges in a GGL program.
/// It provides methods for adding, removing, and querying graph elements, and maintains
/// referential integrity by automatically removing edges when their connected nodes are deleted.
///
/// # Examples
///
/// ```rust
/// use graph_generation_language::types::{Graph, Node, Edge, MetadataValue};
///
/// let mut graph = Graph::new();
///
/// // Add nodes
/// let alice = Node::new("alice".to_string())
///     .with_type("person".to_string())
///     .with_metadata("age".to_string(), MetadataValue::Integer(30));
/// let bob = Node::new("bob".to_string())
///     .with_type("person".to_string());
///
/// graph.add_node(alice);
/// graph.add_node(bob);
///
/// // Add edge
/// let friendship = Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string())
///     .with_type("friend".to_string())
///     .with_metadata("strength".to_string(), MetadataValue::Float(0.8));
///
/// graph.add_edge(friendship);
///
/// assert_eq!(graph.node_count(), 2);
/// assert_eq!(graph.edge_count(), 1);
/// ```
///
/// # JSON Serialization
///
/// Graphs serialize to JSON with separate `nodes` and `edges` objects:
/// ```json
/// {
///   "nodes": {
///     "alice": { "id": "alice", "type": "person", ... },
///     "bob": { "id": "bob", "type": "person", ... }
///   },
///   "edges": {
///     "friendship": { "id": "friendship", "source": "alice", "target": "bob", ... }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Map of node ID to Node objects
    pub nodes: HashMap<String, Node>,
    /// Map of edge ID to Edge objects
    pub edges: HashMap<String, Edge>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Creates a new empty graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::Graph;
    ///
    /// let graph = Graph::new();
    /// assert_eq!(graph.node_count(), 0);
    /// assert_eq!(graph.edge_count(), 0);
    /// ```
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Adds a node to the graph.
    ///
    /// If a node with the same ID already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node};
    ///
    /// let mut graph = Graph::new();
    /// let node = Node::new("alice".to_string());
    /// graph.add_node(node);
    ///
    /// assert_eq!(graph.node_count(), 1);
    /// assert!(graph.get_node("alice").is_some());
    /// ```
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Adds an edge to the graph.
    ///
    /// If an edge with the same ID already exists, it will be replaced.
    /// Note: This method does not validate that the source and target nodes exist.
    ///
    /// # Arguments
    ///
    /// * `edge` - The edge to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node, Edge};
    ///
    /// let mut graph = Graph::new();
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    ///
    /// let edge = Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string());
    /// graph.add_edge(edge);
    ///
    /// assert_eq!(graph.edge_count(), 1);
    /// assert!(graph.get_edge("friendship").is_some());
    /// ```
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id.clone(), edge);
    }

    /// Removes a node and all connected edges from the graph.
    ///
    /// This method maintains referential integrity by automatically removing
    /// any edges that reference the deleted node as either source or target.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the node to remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node, Edge};
    ///
    /// let mut graph = Graph::new();
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    /// graph.add_edge(Edge::new("e1".to_string(), "alice".to_string(), "bob".to_string()));
    ///
    /// assert_eq!(graph.node_count(), 2);
    /// assert_eq!(graph.edge_count(), 1);
    ///
    /// graph.remove_node("alice");
    ///
    /// assert_eq!(graph.node_count(), 1);
    /// assert_eq!(graph.edge_count(), 0); // Edge was automatically removed
    /// ```
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        // Remove any edges connected to this node
        self.edges
            .retain(|_, edge| edge.source != id && edge.target != id);
    }

    /// Removes an edge from the graph.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the edge to remove
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node, Edge};
    ///
    /// let mut graph = Graph::new();
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    /// graph.add_edge(Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string()));
    ///
    /// assert_eq!(graph.edge_count(), 1);
    /// graph.remove_edge("friendship");
    /// assert_eq!(graph.edge_count(), 0);
    /// ```
    pub fn remove_edge(&mut self, id: &str) {
        self.edges.remove(id);
    }

    /// Gets a reference to a node by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the node to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&Node)` if the node exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node};
    ///
    /// let mut graph = Graph::new();
    /// graph.add_node(Node::new("alice".to_string()));
    ///
    /// assert!(graph.get_node("alice").is_some());
    /// assert!(graph.get_node("bob").is_none());
    /// ```
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Gets a reference to an edge by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the edge to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&Edge)` if the edge exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node, Edge};
    ///
    /// let mut graph = Graph::new();
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    /// graph.add_edge(Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string()));
    ///
    /// assert!(graph.get_edge("friendship").is_some());
    /// assert!(graph.get_edge("rivalry").is_none());
    /// ```
    pub fn get_edge(&self, id: &str) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Returns the number of nodes in the graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node};
    ///
    /// let mut graph = Graph::new();
    /// assert_eq!(graph.node_count(), 0);
    ///
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    /// assert_eq!(graph.node_count(), 2);
    /// ```
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges in the graph.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use graph_generation_language::types::{Graph, Node, Edge};
    ///
    /// let mut graph = Graph::new();
    /// assert_eq!(graph.edge_count(), 0);
    ///
    /// graph.add_node(Node::new("alice".to_string()));
    /// graph.add_node(Node::new("bob".to_string()));
    /// graph.add_edge(Edge::new("friendship".to_string(), "alice".to_string(), "bob".to_string()));
    /// assert_eq!(graph.edge_count(), 1);
    /// ```
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_builder() {
        let node = Node::new("test".to_string())
            .with_type("person".to_string())
            .with_metadata("age".to_string(), MetadataValue::Integer(30))
            .with_position(10.0, 20.0);

        assert_eq!(node.id, "test");
        assert_eq!(node.r#type, "person");
        assert_eq!(node.x, 10.0);
        assert_eq!(node.y, 20.0);
        assert_eq!(node.metadata.len(), 1);
        assert!(matches!(
            node.metadata.get("age"),
            Some(MetadataValue::Integer(30))
        ));
    }

    #[test]
    fn test_edge_builder() {
        let edge = Edge::new("e1".to_string(), "n1".to_string(), "n2".to_string())
            .with_type("friend".to_string())
            .with_metadata("weight".to_string(), MetadataValue::Float(1.0));

        assert_eq!(edge.id, "e1");
        assert_eq!(edge.source, "n1");
        assert_eq!(edge.target, "n2");
        assert_eq!(edge.r#type, "friend");
        assert_eq!(edge.metadata.len(), 1);
        assert!(matches!(
            edge.metadata.get("weight"),
            Some(MetadataValue::Float(1.0))
        ));
    }

    #[test]
    fn test_graph_operations() {
        let mut graph = Graph::new();

        // Add nodes
        let node1 = Node::new("n1".to_string());
        let node2 = Node::new("n2".to_string());
        graph.add_node(node1);
        graph.add_node(node2);

        // Add edge
        let edge = Edge::new("e1".to_string(), "n1".to_string(), "n2".to_string());
        graph.add_edge(edge);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        // Remove node and verify connected edge is removed
        graph.remove_node("n1");
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }
}
