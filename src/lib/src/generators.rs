//! # Graph Generators
//!
//! This module provides built-in graph generators for creating common graph topologies.
//! Generators are invoked using the `generate` statement in GGL programs.

use crate::types::{Edge, Graph, Node};
use serde_json::Value;
use std::collections::HashMap;

/// Signature for a graph generator function.
pub type GeneratorFn = fn(&HashMap<String, Value>) -> Result<Graph, String>;

/// Retrieves a generator function by name.
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

// --- Helper Functions ---

fn get_param_int(params: &HashMap<String, Value>, key: &str) -> Result<usize, String> {
    params
        .get(key)
        .ok_or_else(|| format!("Missing required parameter: '{key}'"))
        .and_then(|v| {
            v.as_i64()
                .map(|n| if n < 0 { 0 } else { n as usize })
                .ok_or_else(|| format!("Invalid integer for parameter '{key}'"))
        })
}

fn get_param_string(
    params: &HashMap<String, Value>,
    key: &str,
    default: &str,
) -> String {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

fn get_param_bool(params: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    params
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

// --- Generator Implementations ---

/// Generates a complete graph (clique).
/// # Parameters
/// * `nodes` (int, required): Number of nodes.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, generates directed edges. Default: false.
pub fn generate_complete(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();
    let nodes: Vec<_> = (0..n).map(|i| format!("{prefix}{i}")).collect();

    for node_id in &nodes {
        graph.add_node(node_id.clone(), Node::new());
    }

    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            if !directed && i > j {
                continue;
            }
            let source = &nodes[i];
            let target = &nodes[j];
            let edge_id = format!("e_{source}_{target}");
            graph.add_edge(
                edge_id,
                Edge::new(source.clone(), target.clone(), directed),
            );
        }
    }
    Ok(graph)
}

/// Generates a path graph.
/// # Parameters
/// * `nodes` (int, required): Number of nodes in the path.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, edges follow the path order. Default: false.
pub fn generate_path(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    for i in 0..n {
        graph.add_node(format!("{prefix}{i}"), Node::new());
    }
    for i in 0..n - 1 {
        let source = format!("{prefix}{i}");
        let target = format!("{prefix}{}", i + 1);
        let edge_id = format!("e{i}_{}", i + 1);
        graph.add_edge(edge_id, Edge::new(source, target, directed));
    }
    Ok(graph)
}

/// Generates a cycle graph.
/// # Parameters
/// * `nodes` (int, required): Number of nodes in the cycle.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, edges form a directed cycle. Default: false.
pub fn generate_cycle(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    for i in 0..n {
        graph.add_node(format!("{prefix}{i}"), Node::new());
    }
    for i in 0..n {
        let source = format!("{prefix}{i}");
        let target = format!("{prefix}{}", (i + 1) % n);
        let edge_id = format!("e{}_{}", i, (i + 1) % n);
        graph.add_edge(edge_id, Edge::new(source, target, directed));
    }
    Ok(graph)
}

/// Generates a 2D grid graph.
///
/// # Parameters
/// * `rows` (int, required): Number of rows in the grid.
/// * `cols` (int, required): Number of columns in the grid.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `periodic` (bool, optional): If true, wraps edges around (torus). Default: false.
///
/// # Hard Example: Torus
///
/// This example generates a 2D torus by creating a grid of nodes and connecting them with wrap-around edges.
///
/// ```ggl
/// graph toroidal_mesh {
///     let rows = 10;
///     let cols = 10;
///
///     // Create the nodes
///     for i in 0..rows {
///         for j in 0..cols {
///             node n{i}_{j};
///         }
///     }
///
///     // Create the horizontal edges
///     for i in 0..rows {
///         for j in 0..cols {
///             let next_j = (j + 1) % cols;
///             edge: n{i}_{j} -> n{i}_{next_j};
///         }
///     }
///
///     // Create the vertical edges
///     for i in 0..rows {
///         for j in 0..cols {
///             let next_i = (i + 1) % rows;
///             edge: n{i}_{j} -> n{next_i}_{j};
///         }
///     }
/// }
/// ```
pub fn generate_grid(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let rows = get_param_int(params, "rows")?;
    let cols = get_param_int(params, "cols")?;
    let prefix = get_param_string(params, "prefix", "n");
    let periodic = get_param_bool(params, "periodic", false);
    let mut graph = Graph::new();

    for r in 0..rows {
        for c in 0..cols {
            graph.add_node(format!("{prefix}{r}_{c}"), Node::new());
        }
    }

    for r in 0..rows {
        for c in 0..cols {
            let source = format!("{prefix}{r}_{c}");
            // Horizontal connection
            if c < cols - 1 || periodic {
                let target_c = (c + 1) % cols;
                let target = format!("{prefix}{r}_{target_c}");
                let edge_id = format!("eh_{r}_{c}");
                graph.add_edge(edge_id, Edge::new(source.clone(), target, false));
            }
            // Vertical connection
            if r < rows - 1 || periodic {
                let target_r = (r + 1) % rows;
                let target = format!("{prefix}{target_r}_{c}");
                let edge_id = format!("ev_{r}_{c}");
                graph.add_edge(edge_id, Edge::new(source.clone(), target, false));
            }
        }
    }
    Ok(graph)
}

/// Generates a star graph.
/// # Parameters
/// * `nodes` (int, required): Total number of nodes (1 center + N-1 spokes).
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, edges point from center to spokes. Default: false.
pub fn generate_star(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    let center_id = format!("{prefix}0");
    graph.add_node(center_id.clone(), Node::new());

    for i in 1..n {
        let spoke_id = format!("{prefix}{i}");
        graph.add_node(spoke_id.clone(), Node::new());
        let (source, target) = (center_id.clone(), spoke_id);
        let edge_id = format!("e_center_{i}");
        graph.add_edge(edge_id, Edge::new(source, target, directed));
    }
    Ok(graph)
}

/// Generates a balanced tree.
/// # Parameters
/// * `branching` (int, required): The branching factor of the tree.
/// * `depth` (int, required): The depth of the tree.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
pub fn generate_tree(params: &HashMap<String, Value>) -> Result<Graph, String> {
    let branching = get_param_int(params, "branching")?;
    let depth = get_param_int(params, "depth")?;
    let prefix = get_param_string(params, "prefix", "n");
    let mut graph = Graph::new();

    // Always create at least the root node
    graph.add_node(format!("{prefix}0"), Node::new());

    if depth <= 1 {
        return Ok(graph);
    }

    let mut parent_queue = vec![0];
    let mut id_counter = 1;

    for _d in 0..depth - 1 {
        let mut next_level_parents = Vec::new();
        for parent_id_val in parent_queue.drain(..) {
            for _b in 0..branching {
                let parent_id = format!("{prefix}{parent_id_val}");
                let child_id = format!("{prefix}{id_counter}");
                graph.add_node(child_id.clone(), Node::new());
                let edge_id = format!("e{parent_id_val}_{id_counter}");
                graph.add_edge(edge_id, Edge::new(parent_id, child_id, true));
                next_level_parents.push(id_counter);
                id_counter += 1;
            }
        }
        parent_queue = next_level_parents;
    }
    Ok(graph)
}

/// Generates a scale-free graph using the Barabási-Albert model.
/// # Parameters
/// * `nodes` (int, required): The final number of nodes in the graph.
/// * `edges_per_node` (int, required): Number of edges to attach from a new node to existing nodes.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
pub fn generate_barabasi_albert(
    params: &HashMap<String, Value>,
) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let m = get_param_int(params, "edges_per_node")?;
    let prefix = get_param_string(params, "prefix", "n");

    if m == 0 || n == 0 {
        return Ok(Graph::new());
    }
    if m >= n {
        return Err("Parameter 'edges_per_node' must be less than 'nodes'".to_string());
    }

    let mut graph = Graph::new();

    // Start with m nodes and create a complete graph among them
    for i in 0..m {
        graph.add_node(format!("{prefix}{i}"), Node::new());
    }

    // Create complete graph among initial m nodes
    for i in 0..m {
        for j in i + 1..m {
            let edge_id = format!("e{i}_{j}");
            graph.add_edge(
                edge_id,
                Edge::new(format!("{prefix}{i}"), format!("{prefix}{j}"), false),
            );
        }
    }

    // Initialize degree list for preferential attachment
    let mut degrees: Vec<String> = graph
        .edges
        .values()
        .flat_map(|e| vec![e.source.clone(), e.target.clone()])
        .collect();

    // Add remaining n - m nodes
    for i in m..n {
        let new_node_id = format!("{prefix}{i}");
        graph.add_node(new_node_id.clone(), Node::new());

        // Select m unique targets based on preferential attachment
        let mut selected_targets = std::collections::HashSet::new();
        let mut attempts = 0;

        while selected_targets.len() < m && attempts < 100 {
            if !degrees.is_empty() {
                let idx = fastrand::usize(..degrees.len());
                selected_targets.insert(degrees[idx].clone());
            }
            attempts += 1;
        }

        // If we couldn't get enough unique targets, fill with any available nodes
        if selected_targets.len() < m {
            for j in 0..i {
                let node_id = format!("{prefix}{j}");
                selected_targets.insert(node_id);
                if selected_targets.len() >= m {
                    break;
                }
            }
        }

        // Create edges to selected targets
        for target_id in selected_targets.iter().take(m) {
            let edge_id = format!("e{i}_{}", target_id.strip_prefix(&prefix).unwrap_or("?"));
            graph.add_edge(
                edge_id,
                Edge::new(new_node_id.clone(), target_id.clone(), false),
            );
            // Add both endpoints to degree list for future preferential attachment
            degrees.push(new_node_id.clone());
            degrees.push(target_id.clone());
        }
    }

    Ok(graph)
}
