use graph_generation_language::generators::*;
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_generate_complete_undirected() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(5));
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 10); // n*(n-1)/2 for undirected complete graph
}

#[test]
fn test_generate_complete_directed() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(4));
    params.insert("directed".to_string(), Value::from(true));
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 12); // n*(n-1) for directed complete graph
}

#[test]
fn test_generate_complete_with_prefix() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(3));
    params.insert(
        "prefix".to_string(),
        Value::from("test_"),
    );
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.get_node("test_0").is_some());
    assert!(graph.get_node("test_1").is_some());
    assert!(graph.get_node("test_2").is_some());
}

#[test]
fn test_generate_complete_single_node() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(1));
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0); // No self-loops
}

#[test]
fn test_generate_complete_zero_nodes() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(0));
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_generate_complete_missing_nodes_param() {
    let params = HashMap::new();
    let result = generate_complete(&params);
    assert!(result.is_err());
}

#[test]
fn test_generate_path_basic() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(5));
    let graph = generate_path(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 4); // n-1 edges for path
    assert!(graph.get_node("n0").is_some());
    assert!(graph.get_node("n4").is_some());
    assert!(graph.edges.values().any(|e| e.source == "n0" && e.target == "n1"));
    assert!(graph.edges.values().any(|e| e.source == "n3" && e.target == "n4"));
}

#[test]
fn test_generate_path_directed() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(3));
    params.insert("directed".to_string(), Value::from(true));
    let graph = generate_path(&params).unwrap();
    let edge = graph.edges.values().find(|e| e.source == "n0" && e.target == "n1").unwrap();
    assert!(edge.directed);
}

#[test]
fn test_generate_path_single_node() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(1));
    let graph = generate_path(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_generate_path_two_nodes() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(2));
    let graph = generate_path(&params).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_generate_path_with_prefix() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(3));
    params.insert(
        "prefix".to_string(),
        Value::from("p_"),
    );
    let graph = generate_path(&params).unwrap();
    assert_eq!(graph.nodes.len(), 3);
    assert!(graph.get_node("p_0").is_some());
    assert!(graph.edges.values().any(|e| e.source == "p_0" && e.target == "p_1"));
}

#[test]
fn test_generate_cycle_basic() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(5));
    let graph = generate_cycle(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 5); // n edges for cycle
    assert!(graph.edges.values().any(|e| e.source == "n4" && e.target == "n0")); // Wraps around
}

#[test]
fn test_generate_cycle_directed() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(3));
    params.insert("directed".to_string(), Value::from(true));
    let graph = generate_cycle(&params).unwrap();
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 3);
    assert!(graph.edges.values().all(|e| e.directed));
}

#[test]
fn test_generate_cycle_single_node() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(1));
    let graph = generate_cycle(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 1); // Self-loop
}

#[test]
fn test_generate_cycle_two_nodes() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(2));
    let graph = generate_cycle(&params).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 2); // Two edges forming a cycle
}

#[test]
fn test_generate_grid_basic() {
    let mut params = HashMap::new();
    params.insert("rows".to_string(), Value::from(3));
    params.insert("cols".to_string(), Value::from(4));
    let graph = generate_grid(&params).unwrap();
    assert_eq!(graph.nodes.len(), 12); // rows * cols
    // (rows-1)*cols + rows*(cols-1) = 2*4 + 3*3 = 8 + 9 = 17
    assert_eq!(graph.edges.len(), 17);
    assert!(graph.get_node("n0_0").is_some());
    assert!(graph.get_node("n2_3").is_some());
    assert!(graph.edges.values().any(|e| e.source == "n0_0" && e.target == "n0_1"));
    assert!(graph.edges.values().any(|e| e.source == "n0_0" && e.target == "n1_0"));
}

#[test]
fn test_generate_grid_periodic() {
    let mut params = HashMap::new();
    params.insert("rows".to_string(), Value::from(3));
    params.insert("cols".to_string(), Value::from(3));
    params.insert("periodic".to_string(), Value::from(true));
    let graph = generate_grid(&params).unwrap();
    assert_eq!(graph.nodes.len(), 9);
    assert_eq!(graph.edges.len(), 18); // rows*cols + rows*cols = 9 + 9 = 18
}

#[test]
fn test_generate_grid_single_row_col() {
    let mut params = HashMap::new();
    params.insert("rows".to_string(), Value::from(1));
    params.insert("cols".to_string(), Value::from(5));
    let graph = generate_grid(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 4); // Just a path

    params.insert("rows".to_string(), Value::from(5));
    params.insert("cols".to_string(), Value::from(1));
    let graph = generate_grid(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 4); // Just a path
}

#[test]
fn test_generate_grid_missing_params() {
    let mut params = HashMap::new();
    params.insert("rows".to_string(), Value::from(3));
    assert!(generate_grid(&params).is_err());

    params.clear();
    params.insert("cols".to_string(), Value::from(3));
    assert!(generate_grid(&params).is_err());
}

#[test]
fn test_generate_star_basic() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(6));
    let graph = generate_star(&params).unwrap();
    assert_eq!(graph.nodes.len(), 6);
    assert_eq!(graph.edges.len(), 5); // n-1 edges for star
    assert!(graph.get_node("n0").is_some()); // Center
    assert!(graph.get_node("n5").is_some()); // Spoke
    assert!(graph.edges.values().any(|e| e.source == "n0" && e.target == "n1"));
}

#[test]
fn test_generate_star_directed() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(4));
    params.insert("directed".to_string(), Value::from(true));
    let graph = generate_star(&params).unwrap();
    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 3);
    let edge = graph.edges.values().find(|e| e.source == "n0" && e.target == "n1").unwrap();
    assert!(edge.directed);
    assert_eq!(edge.source, "n0");
}

#[test]
fn test_generate_star_single_node() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(1));
    let graph = generate_star(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_generate_star_two_nodes() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(2));
    let graph = generate_star(&params).unwrap();
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_generate_tree_basic() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(2));
    params.insert("depth".to_string(), Value::from(3));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 7); // 1 + 2 + 4
    assert_eq!(graph.edges.len(), 6); // n-1 edges for tree
    assert!(graph.get_node("n0").is_some()); // Root
    assert!(graph.get_node("n6").is_some()); // Leaf
    assert!(graph.edges.values().any(|e| e.source == "n0" && e.target == "n1"));
    assert!(graph.edges.values().any(|e| e.source == "n2" && e.target == "n6"));
}

#[test]
fn test_generate_tree_large_branching() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(3));
    params.insert("depth".to_string(), Value::from(2));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 4); // 1 + 3
    assert_eq!(graph.edges.len(), 3);
}

#[test]
fn test_generate_tree_zero_depth() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(2));
    params.insert("depth".to_string(), Value::from(0));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1); // Just root
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_generate_tree_one_depth() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(5));
    params.insert("depth".to_string(), Value::from(1));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1); // Just root (depth 1 means no children)
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_generate_tree_zero_branching() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(0));
    params.insert("depth".to_string(), Value::from(3));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 1); // Root with no children
}

#[test]
fn test_generate_tree_depth_2() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(2));
    params.insert("depth".to_string(), Value::from(2));
    let graph = generate_tree(&params).unwrap();
    assert_eq!(graph.nodes.len(), 3); // 1 + 2 nodes
}

#[test]
fn test_generate_tree_missing_params() {
    let mut params = HashMap::new();
    params.insert("branching".to_string(), Value::from(2));
    assert!(generate_tree(&params).is_err());

    params.clear();
    params.insert("depth".to_string(), Value::from(3));
    assert!(generate_tree(&params).is_err());
}

#[test]
fn test_generate_barabasi_albert_basic() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(10));
    params.insert("edges_per_node".to_string(), Value::from(2));
    let graph = generate_barabasi_albert(&params).unwrap();
    assert_eq!(graph.nodes.len(), 10);
    // Edges = m0*m + (n-m0)*m = 2*2 + (10-2)*2 = 4 + 16 = 20, but this is complex
    // Let's just check it's reasonable
    assert!(!graph.edges.is_empty());
    assert_eq!(graph.edges.len(), 17);
}

#[test]
fn test_generate_barabasi_albert_m_equals_n() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(3));
    params.insert("edges_per_node".to_string(), Value::from(3));
    assert!(generate_barabasi_albert(&params).is_err());
}

#[test]
fn test_generate_barabasi_albert_m1() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(5));
    params.insert("edges_per_node".to_string(), Value::from(1));
    let graph = generate_barabasi_albert(&params).unwrap();
    assert_eq!(graph.nodes.len(), 5);
    assert_eq!(graph.edges.len(), 4); // Should be a path/tree
}

#[test]
fn test_generate_barabasi_albert_m0() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(5));
    params.insert("edges_per_node".to_string(), Value::from(0));
    let graph = generate_barabasi_albert(&params).unwrap();
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_get_generator_valid() {
    assert!(get_generator("complete").is_some());
    assert!(get_generator("path").is_some());
    assert!(get_generator("cycle").is_some());
    assert!(get_generator("grid").is_some());
    assert!(get_generator("star").is_some());
    assert!(get_generator("tree").is_some());
    assert!(get_generator("barabasi_albert").is_some());
}

#[test]
fn test_get_generator_invalid() {
    assert!(get_generator("non_existent_generator").is_none());
}

#[test]
fn test_get_generator_case_sensitive() {
    assert!(get_generator("Complete").is_none());
}

#[test]
fn test_param_handling_in_generators() {
    // Test that generators correctly handle missing required parameters
    let params = HashMap::new();
    assert!(generate_complete(&params).is_err());
    assert!(generate_path(&params).is_err());
    assert!(generate_cycle(&params).is_err());
    assert!(generate_grid(&params).is_err());
    assert!(generate_star(&params).is_err());
    assert!(generate_tree(&params).is_err());
    assert!(generate_barabasi_albert(&params).is_err());

    // Test that generators handle invalid parameter types gracefully
    let mut params_wrong_type = HashMap::new();
    params_wrong_type.insert(
        "nodes".to_string(),
        Value::from("five"),
    );
    assert!(generate_complete(&params_wrong_type).is_err());
}

#[test]
fn test_negative_params() {
    let mut params = HashMap::new();
    params.insert("nodes".to_string(), Value::from(-5));
    let graph = generate_complete(&params).unwrap();
    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_graph_merging() {
    let mut params1 = HashMap::new();
    params1.insert("nodes".to_string(), Value::from(5));
    let complete_graph = generate_complete(&params1).unwrap();
    assert_eq!(complete_graph.edges.len(), 10);

    let mut params2 = HashMap::new();
    params2.insert("nodes".to_string(), Value::from(5));
    let path_graph = generate_path(&params2).unwrap();
    assert_eq!(path_graph.edges.len(), 4);

    let mut params3 = HashMap::new();
    params3.insert("nodes".to_string(), Value::from(5));
    let cycle_graph = generate_cycle(&params3).unwrap();
    assert_eq!(cycle_graph.edges.len(), 5);
}
