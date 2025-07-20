//! # Core Data Types
//!
//! This module defines the fundamental data structures used throughout GGL.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl MetadataValue {
    pub fn as_int(&self) -> Result<i64, String> {
        match self {
            MetadataValue::Integer(i) => Ok(*i),
            _ => Err(format!("Expected an integer, but found {self}")),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MetadataValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for MetadataValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataValue::String(s) => write!(f, "{s}"),
            MetadataValue::Integer(i) => write!(f, "{i}"),
            MetadataValue::Float(n) => write!(f, "{n}"),
            MetadataValue::Boolean(b) => write!(f, "{b}"),
        }
    }
}

impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        MetadataValue::String(s)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub r#type: String,
    pub metadata: HashMap<String, MetadataValue>,
    pub x: f64,
    pub y: f64,
}

impl Node {
    pub fn new(id: String) -> Self {
        Node {
            id,
            r#type: String::new(),
            metadata: HashMap::new(),
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn with_type(mut self, node_type: String) -> Self {
        self.r#type = node_type;
        self
    }

    pub fn with_metadata(mut self, key: String, value: MetadataValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_metadata_map(mut self, metadata: HashMap<String, MetadataValue>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    /// Indicates if the edge is directed (source -> target).
    pub directed: bool,
    pub r#type: String,
    pub metadata: HashMap<String, MetadataValue>,
}

impl Edge {
    pub fn new(id: String, source: String, target: String, directed: bool) -> Self {
        Edge {
            id,
            source,
            target,
            directed,
            r#type: String::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_type(mut self, edge_type: String) -> Self {
        self.r#type = edge_type;
        self
    }

    pub fn with_metadata(mut self, key: String, value: MetadataValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_metadata_map(mut self, metadata: HashMap<String, MetadataValue>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.insert(edge.id.clone(), edge);
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.edges
            .retain(|_, edge| edge.source != id && edge.target != id);
    }

    pub fn remove_edge(&mut self, id: &str) {
        self.edges.remove(id);
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn get_edge(&self, id: &str) -> Option<&Edge> {
        self.edges.get(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
