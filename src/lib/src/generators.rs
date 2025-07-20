//! # Graph Generators
//!
//! This module provides built-in graph generators for creating common graph topologies.
//! Generators are invoked using the `generate` statement in GGL programs.

use crate::types::{Edge, Graph, MetadataValue, Node};
use std::collections::HashMap;

/// Signature for a graph generator function.
pub type GeneratorFn = fn(&HashMap<String, MetadataValue>) -> Result<Graph, String>;

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

fn get_param_int(params: &HashMap<String, MetadataValue>, key: &str) -> Result<usize, String> {
    params
        .get(key)
        .ok_or_else(|| format!("Missing required parameter: '{key}'"))
        .and_then(|v| v.as_int().map(|n| n as usize))
}

fn get_param_string(
    params: &HashMap<String, MetadataValue>,
    key: &str,
    default: &str,
) -> String {
    params
        .get(key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| default.to_string())
}

fn get_param_bool(params: &HashMap<String, MetadataValue>, key: &str, default: bool) -> bool {
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
pub fn generate_complete(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);

    let mut graph = Graph::new();
    let nodes: Vec<_> = (0..n).map(|i| format!("{prefix}{i}")).collect();

    for node_id in &nodes {
        graph.add_node(Node::new(node_id.clone()));
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
            graph.add_edge(Edge::new(
                format!("e_{source}_{target}"),
                source.clone(),
                target.clone(),
                directed,
            ));
        }
    }
    Ok(graph)
}

/// Generates a path graph.
/// # Parameters
/// * `nodes` (int, required): Number of nodes in the path.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, edges follow the path order. Default: false.
pub fn generate_path(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    for i in 0..n {
        graph.add_node(Node::new(format!("{prefix}{i}")));
    }
    for i in 0..n - 1 {
        let source = format!("{prefix}{i}");
        let target = format!("{prefix}{}", i + 1);
        graph.add_edge(Edge::new(
            format!("e{i}_{}", i + 1),
            source,
            target,
            directed,
        ));
    }
    Ok(graph)
}

/// Generates a cycle graph.
/// # Parameters
/// * `nodes` (int, required): Number of nodes in the cycle.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `directed` (bool, optional): If true, edges form a directed cycle. Default: false.
pub fn generate_cycle(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    for i in 0..n {
        graph.add_node(Node::new(format!("{prefix}{i}")));
    }
    for i in 0..n {
        let source = format!("{prefix}{i}");
        let target = format!("{prefix}{}", (i + 1) % n);
        graph.add_edge(Edge::new(
            format!("e{}_{}", i, (i + 1) % n),
            source,
            target,
            directed,
        ));
    }
    Ok(graph)
}

/// Generates a 2D grid graph.
/// # Parameters
/// * `rows` (int, required): Number of rows in the grid.
/// * `cols` (int, required): Number of columns in the grid.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
/// * `periodic` (bool, optional): If true, wraps edges around (torus). Default: false.
pub fn generate_grid(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let rows = get_param_int(params, "rows")?;
    let cols = get_param_int(params, "cols")?;
    let prefix = get_param_string(params, "prefix", "n");
    let periodic = get_param_bool(params, "periodic", false);
    let mut graph = Graph::new();

    for r in 0..rows {
        for c in 0..cols {
            graph.add_node(Node::new(format!("{prefix}{r}_{c}")));
        }
    }

    for r in 0..rows {
        for c in 0..cols {
            let source = format!("{prefix}{r}_{c}");
            // Horizontal connection
            if c < cols - 1 || periodic {
                let target_c = (c + 1) % cols;
                let target = format!("{prefix}{r}_{target_c}");
                graph.add_edge(Edge::new(
                    format!("eh_{r}_{c}"),
                    source.clone(),
                    target,
                    false,
                ));
            }
            // Vertical connection
            if r < rows - 1 || periodic {
                let target_r = (r + 1) % rows;
                let target = format!("{prefix}{target_r}_{c}");
                graph.add_edge(Edge::new(
                    format!("ev_{r}_{c}"),
                    source.clone(),
                    target,
                    false,
                ));
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
pub fn generate_star(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let n = get_param_int(params, "nodes")?;
    let prefix = get_param_string(params, "prefix", "n");
    let directed = get_param_bool(params, "directed", false);
    let mut graph = Graph::new();

    if n == 0 {
        return Ok(graph);
    }

    let center_id = format!("{prefix}_center");
    graph.add_node(Node::new(center_id.clone()));

    for i in 0..n - 1 {
        let spoke_id = format!("{prefix}{i}");
        graph.add_node(Node::new(spoke_id.clone()));
        let (source, target) = if directed {
            (center_id.clone(), spoke_id)
        } else {
            (spoke_id, center_id.clone())
        };
        graph.add_edge(Edge::new(format!("e_center_{i}"), source, target, directed));
    }
    Ok(graph)
}

/// Generates a balanced tree.
/// # Parameters
/// * `branching` (int, required): The branching factor of the tree.
/// * `depth` (int, required): The depth of the tree.
/// * `prefix` (string, optional): Prefix for node IDs. Default: "n".
pub fn generate_tree(params: &HashMap<String, MetadataValue>) -> Result<Graph, String> {
    let branching = get_param_int(params, "branching")?;
    let depth = get_param_int(params, "depth")?;
    let prefix = get_param_string(params, "prefix", "n");
    let mut graph = Graph::new();

    if depth == 0 {
        return Ok(graph);
    }

    graph.add_node(Node::new(format!("{prefix}0")));
    let mut parent_queue = vec![0];
    let mut id_counter = 1;

    for _d in 0..depth - 1 {
        let mut next_level_parents = Vec::new();
        for parent_id_val in parent_queue.drain(..) {
            for _b in 0..branching {
                let parent_id = format!("{prefix}{parent_id_val}");
                let child_id = format!("{prefix}{id_counter}");
                graph.add_node(Node::new(child_id.clone()));
                graph.add_edge(Edge::new(
                    format!("e{parent_id_val}_{id_counter}"),
                    parent_id,
                    child_id,
                    true,
                ));
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
    params: &HashMap<String, MetadataValue>,
) -> Result<Graph, String> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

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
    let mut rng = thread_rng();
    let m0 = m + 1; // Initial number of nodes

    // 1. Start with an initial connected graph of m0 nodes
    for i in 0..m0 {
        graph.add_node(Node::new(format!("{prefix}{i}")));
    }
    for i in 0..m0 {
        for j in i + 1..m0 {
            graph.add_edge(Edge::new(
                format!("e{i}_{j}"),
                format!("{prefix}{i}"),
                format!("{prefix}{j}"),
                false,
            ));
        }
    }

    let mut degrees: Vec<String> = graph
        .edges
        .values()
        .flat_map(|e| vec![e.source.clone(), e.target.clone()])
        .collect();

    // 2. Add remaining n - m0 nodes
    for i in m0..n {
        let new_node_id = format!("{prefix}{i}");
        graph.add_node(Node::new(new_node_id.clone()));

        let targets: Vec<String> = degrees
            .choose_multiple(&mut rng, m)
            .cloned()
            .collect();

        for target_id in targets {
            graph.add_edge(Edge::new(
                format!("e{i}_{}", target_id.strip_prefix(&prefix).unwrap_or("?")),
                new_node_id.clone(),
                target_id.clone(),
                false,
            ));
            degrees.push(new_node_id.clone());
            degrees.push(target_id);
        }
    }

    Ok(graph)
}
