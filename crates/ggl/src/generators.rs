//! # Graph Generators
//!
//! This module provides built-in graph generators for creating common graph topologies.
//! Generators are invoked using the `generate` statement in GGL programs and can create
//! various types of graph structures with customizable parameters.
//!
//! ## Available Generators
//!
//! * [`generate_complete`] - Complete graphs where every node connects to every other node
//! * [`generate_path`] - Linear chains of connected nodes
//! * [`generate_cycle`] - Circular chains of nodes
//! * [`generate_grid`] - 2D grid structures with optional periodic boundaries
//! * [`generate_star`] - Star topologies with one central hub
//! * [`generate_tree`] - Tree structures with specified branching and depth
//! * [`generate_barabasi_albert`] - Scale-free networks using preferential attachment
//!
//! ## Usage in GGL
//!
//! Generators are invoked using the `generate` statement:
//! ```ggl
//! generate complete {
//!     nodes: 5;
//!     prefix: "vertex";
//! }
//! ```
//!
//! ## Common Parameters
//!
//! Most generators support these parameters:
//! * `nodes` - Number of nodes to generate (required for most generators)
//! * `prefix` - Node name prefix (optional, default: "n")
//! * `directed` - Whether edges should be directed (optional, default: false)
//!
//! ## Examples
//!
//! ```rust
//! use ggl::generators::generate_complete;
//! use ggl::types::MetadataValue;
//! use std::collections::HashMap;
//!
//! let mut params = HashMap::new();
//! params.insert("nodes".to_string(), MetadataValue::Integer(4));
//! params.insert("prefix".to_string(), MetadataValue::String("vertex".to_string()));
//!
//! let graph = generate_complete(&params).unwrap();
//! assert_eq!(graph.node_count(), 4);
//! assert_eq!(graph.edge_count(), 6); // Complete graph: n*(n-1)/2 edges
//! ```

use crate::types::{Edge, Graph, MetadataValue, Node};
use std::collections::HashMap;

/// Function signature for graph generator functions.
///
/// All generators take a parameter map and return either a generated graph or an error message.
pub type GeneratorFn = fn(&HashMap<String, MetadataValue>) -> Result<Graph, String>;

/// Returns the generator function for the given name.
///
/// This function serves as a registry for all available generators, mapping
/// generator names to their implementation functions.
///
/// # Arguments
///
/// * `name` - Name of the generator to retrieve
///
/// # Returns
///
/// `Some(GeneratorFn)` if the generator exists, `None` otherwise.
///
/// # Available Generators
///
/// * `"complete"` - Complete graph generator
/// * `"path"` - Path graph generator
/// * `"cycle"` - Cycle graph generator
/// * `"grid"` - Grid graph generator
/// * `"star"` - Star graph generator
/// * `"tree"` - Tree graph generator
/// * `"barabasi_albert"` - Barabási-Albert scale-free network generator
///
/// # Examples
///
/// ```rust
/// use ggl::generators::get_generator;
///
/// let generator = get_generator("complete");
/// assert!(generator.is_some());
///
/// let unknown = get_generator("unknown");
/// assert!(unknown.is_none());
/// ```
pub fn get_generator(name: &str) -> Option<GeneratorFn> {
    match name {
        "complete" => Some(generate_complete),
        "path" => Some(generate_path),
        "cycle" => Some(generate_cycle),
        "grid" => Some(generate_grid),
        "star" => Some(generate_star),
        "tree" => Some(generate_tree),
        "barabasi_albert" => Some(generate_barabasi_albert),
        _ => None,
    }
}

/// Helper function to extract integer parameters from the parameter map.
///
/// Converts both integer and float values to usize, with negative values clamped to 0.
fn get_param_int(params: &HashMap<String, MetadataValue>, key: &str) -> Result<usize, String> {
    match params.get(key) {
        Some(MetadataValue::Integer(n)) => {
            if *n < 0 {
                Ok(0)
            } else {
                Ok(*n as usize)
            }
        }
        Some(MetadataValue::Float(n)) => {
            if *n < 0.0 {
                Ok(0)
            } else {
                Ok(*n as usize)
            }
        }
        _ => Err(format!("Missing or invalid {} parameter", key)),
    }
}

/// Helper function to extract string parameters with default values.
fn get_param_string(params: &HashMap<String, MetadataValue>, key: &str, default: &str) -> String {
    match params.get(key) {
        Some(MetadataValue::String(s)) => s.clone(),
        _ => default.to_string(),
    }
}

/// Helper function to extract boolean parameters with default values.
fn get_param_bool(params: &HashMap<String, MetadataValue>, key: &str, default: bool) -> bool {
    match params.get(key) {
        Some(MetadataValue::Boolean(b)) => *b,
        _ => default,
    }
}

/// Generates a complete graph where every node is connected to every other node.
///
/// A complete graph is a graph where every pair of distinct nodes is connected by a unique edge.
/// This creates the maximum possible number of edges for a given number of nodes.
///
/// # Parameters
///
/// * `nodes` (required) - Number of nodes to generate
/// * `prefix` (optional) - Node name prefix (default: "n")
/// * `directed` (optional) - Whether edges should be directed (default: false)
///
/// # Properties
///
/// * **Nodes**: n
/// * **Edges**: n(n-1)/2 for undirected, n(n-1) for directed
/// * **Connectivity**: Every node connected to every other node
///
/// # Examples
///
/// ```rust
/// use ggl::generators::generate_complete;
/// use ggl::types::MetadataValue;
/// use std::collections::HashMap;
///
/// let mut params = HashMap::new();
/// params.insert("nodes".to_string(), MetadataValue::Integer(4));
/// params.insert("prefix".to_string(), MetadataValue::String("vertex".to_string()));
///
/// let graph = generate_complete(&params).unwrap();
/// assert_eq!(graph.node_count(), 4);
/// assert_eq!(graph.edge_count(), 6); // 4*(4-1)/2 = 6 edges
/// ```
///
/// # GGL Usage
///
/// ```ggl
/// generate complete {
///     nodes: 5;
///     prefix: "vertex";
///     directed: false;
/// }
/// ```
///
/// # Use Cases
///
/// * Fully connected networks
/// * Cliques in social networks
/// * Reference topologies for comparison
/// * Dense communication networks
pub fn generate_complete(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let _directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();

    // Add nodes
    for i in 0..n {
        let node_id = format!("{}{}", prefix, i);
        graph.add_node(Node::new(node_id));
    }

    // Add edges (all-to-all)
    for i in 0..n {
        for j in (if _directed { 0 } else { i + 1 })..n {
            if i != j {
                let source = format!("{}{}", prefix, i);
                let target = format!("{}{}", prefix, j);
                let edge_id = format!("e{}_{}", i, j);
                graph.add_edge(Edge::new(edge_id, source, target));
            }
        }
    }

    Ok(graph)
}

/// Generates a path graph (linear chain of connected nodes).
///
/// A path graph consists of nodes arranged in a line, where each node is connected
/// to its immediate neighbors, forming a linear chain.
///
/// # Parameters
///
/// * `nodes` (required) - Number of nodes to generate
/// * `prefix` (optional) - Node name prefix (default: "n")
/// * `directed` (optional) - Whether edges should be directed (default: false)
///
/// # Properties
///
/// * **Nodes**: n
/// * **Edges**: n-1
/// * **Connectivity**: Linear chain
///
/// # GGL Usage
///
/// ```ggl
/// generate path {
///     nodes: 6;
///     prefix: "step";
///     directed: true;
/// }
/// ```
///
/// # Use Cases
///
/// * Sequences and pipelines
/// * Linear processes
/// * Chain of command structures
/// * Sequential workflows
pub fn generate_path(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let _directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();

    // Add nodes
    for i in 0..n {
        let node_id = format!("{}{}", prefix, i);
        graph.add_node(Node::new(node_id));
    }

    // Add edges
    for i in 0..n - 1 {
        let source = format!("{}{}", prefix, i);
        let target = format!("{}{}", prefix, i + 1);
        let edge_id = format!("e{}_{}", i, i + 1);
        graph.add_edge(Edge::new(edge_id, source, target));
    }

    Ok(graph)
}

/// Generates a cycle graph (circular chain of nodes).
///
/// A cycle graph forms a closed loop where each node is connected to exactly two neighbors,
/// and the last node connects back to the first, creating a circular structure.
///
/// # Parameters
///
/// * `nodes` (required) - Number of nodes to generate
/// * `prefix` (optional) - Node name prefix (default: "n")
///
/// # Properties
///
/// * **Nodes**: n
/// * **Edges**: n
/// * **Connectivity**: Circular chain
///
/// # GGL Usage
///
/// ```ggl
/// generate cycle {
///     nodes: 5;
///     prefix: "vertex";
/// }
/// ```
///
/// # Use Cases
///
/// * Ring topologies
/// * Circular processes
/// * Closed loops
/// * Round-robin systems
pub fn generate_cycle(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let _directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();

    // Add nodes
    for i in 0..n {
        let node_id = format!("{}{}", prefix, i);
        graph.add_node(Node::new(node_id));
    }

    // Add edges
    for i in 0..n {
        let source = format!("{}{}", prefix, i);
        let target = format!("{}{}", prefix, (i + 1) % n);
        let edge_id = format!("e{}_{}", i, (i + 1) % n);
        graph.add_edge(Edge::new(edge_id, source, target));
    }

    Ok(graph)
}

pub fn generate_grid(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let rows = get_param_int(params, "rows")?;
    let cols = get_param_int(params, "cols")?;
    let prefix = get_param_string(params, "prefix", "n");
    let periodic = get_param_bool(params, "periodic", false);

    let mut graph = Graph::new();

    // Add nodes
    for i in 0..rows {
        for j in 0..cols {
            let node_id = format!("{}{}_{}", prefix, i, j);
            graph.add_node(Node::new(node_id));
        }
    }

    // Add horizontal edges
    for i in 0..rows {
        for j in 0..cols - 1 {
            let source = format!("{}{}_{}", prefix, i, j);
            let target = format!("{}{}_{}", prefix, i, j + 1);
            let edge_id = format!("eh{}_{}", i, j);
            graph.add_edge(Edge::new(edge_id, source, target));
        }
        // Add periodic horizontal edges if requested
        if periodic {
            let source = format!("{}{}_{}", prefix, i, cols - 1);
            let target = format!("{}{}_{}", prefix, i, 0);
            let edge_id = format!("ehp_{}", i);
            graph.add_edge(Edge::new(edge_id, source, target));
        }
    }

    // Add vertical edges
    for j in 0..cols {
        // Regular vertical edges
        for i in 0..rows - 1 {
            let source = format!("{}{}_{}", prefix, i, j);
            let target = format!("{}{}_{}", prefix, i + 1, j);
            let edge_id = format!("ev{}_{}", i, j);
            graph.add_edge(Edge::new(edge_id, source, target));
        }
        // Add periodic vertical edges if requested
        if periodic {
            let source = format!("{}{}_{}", prefix, rows - 1, j);
            let target = format!("{}{}_{}", prefix, 0, j);
            let edge_id = format!("evp_{}", j);
            graph.add_edge(Edge::new(edge_id, source, target));
        }
    }

    Ok(graph)
}

pub fn generate_star(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();

    // Add center node
    let center = format!("{}0", prefix);
    graph.add_node(Node::new(center.clone()));

    // Add leaf nodes and connect to center
    for i in 1..n {
        let node_id = format!("{}{}", prefix, i);
        graph.add_node(Node::new(node_id.clone()));

        let edge_id = format!("e0_{}", i);
        if directed {
            graph.add_edge(Edge::new(edge_id, center.clone(), node_id));
        } else {
            graph.add_edge(Edge::new(edge_id, node_id, center.clone()));
        }
    }

    Ok(graph)
}

pub fn generate_tree(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let branching = get_param_int(params, "branching")?;
    let depth = get_param_int(params, "depth")?;
    let prefix = get_param_string(params, "prefix", "n");

    let mut graph = Graph::new();

    // Add root node
    let root = format!("{}0", prefix);
    graph.add_node(Node::new(root.clone()));

    // Generate tree recursively
    generate_tree_recursive(&mut graph, &root, 0, depth, branching, prefix);

    Ok(graph)
}

fn generate_tree_recursive(
    graph: &mut Graph,
    parent: &str,
    current_depth: usize,
    max_depth: usize,
    branching: usize,
    prefix: String,
) {
    if max_depth == 0 || current_depth >= max_depth - 1 {
        return;
    }

    let parent_index = parent.trim_start_matches(&prefix).parse::<usize>().unwrap();

    for i in 0..branching {
        let child_index = parent_index * branching + i + 1;
        let child_id = format!("{}{}", prefix, child_index);

        graph.add_node(Node::new(child_id.clone()));
        graph.add_edge(Edge::new(
            format!("e{}_{}", parent_index, child_index),
            parent.to_string(),
            child_id.clone(),
        ));

        generate_tree_recursive(
            graph,
            &child_id,
            current_depth + 1,
            max_depth,
            branching,
            prefix.clone(),
        );
    }
}

pub fn generate_barabasi_albert(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    use rand::Rng;

    let n = get_param_int(params, "nodes")?;
    let m = get_param_int(params, "edges_per_node")?;
    let prefix = get_param_string(params, "prefix", "n");

    if m >= n {
        return Err("edges_per_node must be less than nodes".to_string());
    }

    if m == 0 {
        return Err("edges_per_node must be greater than 0".to_string());
    }

    let mut graph = Graph::new();
    let mut rng = rand::thread_rng();

    // Add initial complete graph with m+1 nodes (to ensure we have enough nodes)
    let initial_nodes = std::cmp::max(m + 1, 2);
    for i in 0..initial_nodes {
        let node_id = format!("{}{}", prefix, i);
        graph.add_node(Node::new(node_id));
    }

    // Create initial edges to ensure connectivity
    for i in 0..initial_nodes {
        for j in i + 1..initial_nodes {
            let source = format!("{}{}", prefix, i);
            let target = format!("{}{}", prefix, j);
            let edge_id = format!("e{}_{}", i, j);
            graph.add_edge(Edge::new(edge_id, source, target));
        }
    }

    // Add remaining nodes using preferential attachment
    for i in initial_nodes..n {
        let new_node = format!("{}{}", prefix, i);
        graph.add_node(Node::new(new_node.clone()));

        // Calculate node degrees for preferential attachment
        let mut candidates = Vec::new();
        for j in 0..i {
            let node_id = format!("{}{}", prefix, j);
            let degree = graph
                .edges
                .values()
                .filter(|e| e.source == node_id || e.target == node_id)
                .count();
            // Add each node multiple times based on its degree (preferential attachment)
            for _ in 0..std::cmp::max(1, degree) {
                candidates.push(node_id.clone());
            }
        }

        // Select m distinct nodes
        let mut selected = std::collections::HashSet::new();
        let mut attempts = 0;
        while selected.len() < m && attempts < 1000 {
            if !candidates.is_empty() {
                let idx = rng.gen_range(0..candidates.len());
                selected.insert(candidates[idx].clone());
            }
            attempts += 1;
        }

        // Add edges to selected nodes
        for target in selected {
            let target_idx = target
                .trim_start_matches(&prefix)
                .parse::<usize>()
                .unwrap_or(0);
            let edge_id = format!("e{}_{}", i, target_idx);
            graph.add_edge(Edge::new(edge_id, new_node.clone(), target));
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_graph() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 10); // n*(n-1)/2 edges for undirected
    }

    #[test]
    fn test_path_graph() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_path(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4); // n-1 edges
    }

    #[test]
    fn test_cycle_graph() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_cycle(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 5); // n edges
    }

    #[test]
    fn test_grid_graph() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(3));
        params.insert("cols".to_string(), MetadataValue::Integer(4));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 12); // rows * cols
        assert_eq!(graph.edge_count(), 17); // (rows-1)*cols + rows*(cols-1)
    }
}
