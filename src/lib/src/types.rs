//! Core data structures for representing graphs.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub r#type: String,
    pub metadata: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub directed: bool,
    pub metadata: HashMap<String, Value>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: String, node: Node) {
        self.nodes.insert(id, node);
    }

    pub fn add_edge(&mut self, id: String, edge: Edge) {
        self.edges.insert(id, edge);
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn remove_node(&mut self, id: &str) -> Option<Node> {
        self.nodes.remove(id)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generates a unique node ID based on a prefix.
    pub fn generate_unique_node_id(&self, prefix: &str) -> String {
        let mut i = 0;
        loop {
            let id = format!("{prefix}_{i}");
            if !self.nodes.contains_key(&id) {
                return id;
            }
            i += 1;
        }
    }

    /// Generates a unique edge ID based on a prefix.
    pub fn generate_unique_edge_id(&self, prefix: &str) -> String {
        let mut i = 0;
        loop {
            let id = format!("{prefix}_{i}");
            if !self.edges.contains_key(&id) {
                return id;
            }
            i += 1;
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    pub fn new() -> Self {
        Node {
            r#type: "default".to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_type(mut self, node_type: String) -> Self {
        self.r#type = node_type;
        self
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_metadata_map(mut self, metadata_map: HashMap<String, Value>) -> Self {
        self.metadata.extend(metadata_map);
        self
    }
}

impl Edge {
    pub fn new(source: String, target: String, directed: bool) -> Self {
        Edge {
            source,
            target,
            directed,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_metadata_map(mut self, metadata_map: HashMap<String, Value>) -> Self {
        self.metadata.extend(metadata_map);
        self
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
