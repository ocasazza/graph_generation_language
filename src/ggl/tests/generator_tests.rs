use graph_generation_language::generators::*;
use graph_generation_language::types::MetadataValue;
use std::collections::HashMap;

#[cfg(test)]
mod complete_graph_tests {
    use super::*;

    #[test]
    fn test_complete_graph_basic() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 10); // n*(n-1)/2 for undirected complete graph

        // Verify all nodes exist
        for i in 0..5 {
            let node_id = format!("n{}", i);
            assert!(
                graph.get_node(&node_id).is_some(),
                "Node {} should exist",
                node_id
            );
        }
    }

    #[test]
    fn test_complete_graph_directed() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(4));
        params.insert("directed".to_string(), MetadataValue::Boolean(true));

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 12); // n*(n-1) for directed complete graph
    }

    #[test]
    fn test_complete_graph_custom_prefix() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("vertex".to_string()),
        );

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 3);

        // Verify custom prefix is used
        for i in 0..3 {
            let node_id = format!("vertex{}", i);
            assert!(
                graph.get_node(&node_id).is_some(),
                "Node {} should exist",
                node_id
            );
        }
    }

    #[test]
    fn test_complete_graph_single_node() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(1));

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0); // No self-loops
    }

    #[test]
    fn test_complete_graph_zero_nodes() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(0));

        let graph = generate_complete(&params).unwrap();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_complete_graph_missing_nodes_param() {
        let params = HashMap::new();
        let result = generate_complete(&params);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod path_graph_tests {
    use super::*;

    #[test]
    fn test_path_graph_basic() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_path(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4); // n-1 edges for path

        // Verify path structure: each node (except endpoints) has degree 2
        let mut degrees = HashMap::new();
        for edge in graph.edges.values() {
            *degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let mut degree_counts = HashMap::new();
        for (_, degree) in degrees {
            *degree_counts.entry(degree).or_insert(0) += 1;
        }

        assert_eq!(degree_counts.get(&1), Some(&2)); // 2 endpoints with degree 1
        assert_eq!(degree_counts.get(&2), Some(&3)); // 3 middle nodes with degree 2
    }

    #[test]
    fn test_path_graph_single_node() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(1));

        let graph = generate_path(&params).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_path_graph_two_nodes() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(2));

        let graph = generate_path(&params).unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_path_graph_custom_prefix() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("step".to_string()),
        );

        let graph = generate_path(&params).unwrap();
        assert_eq!(graph.node_count(), 3);

        // Verify custom prefix
        assert!(graph.get_node("step0").is_some());
        assert!(graph.get_node("step1").is_some());
        assert!(graph.get_node("step2").is_some());
    }
}

#[cfg(test)]
mod cycle_graph_tests {
    use super::*;

    #[test]
    fn test_cycle_graph_basic() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let graph = generate_cycle(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 5); // n edges for cycle

        // Verify cycle structure: all nodes have degree 2
        let mut degrees = HashMap::new();
        for edge in graph.edges.values() {
            *degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        for (_, degree) in degrees {
            assert_eq!(degree, 2, "All nodes in cycle should have degree 2");
        }
    }

    #[test]
    fn test_cycle_graph_triangle() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));

        let graph = generate_cycle(&params).unwrap();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);
    }

    #[test]
    fn test_cycle_graph_single_node() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(1));

        let graph = generate_cycle(&params).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 1); // Self-loop
    }

    #[test]
    fn test_cycle_graph_two_nodes() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(2));

        let graph = generate_cycle(&params).unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 2); // Two edges forming a cycle
    }
}

#[cfg(test)]
mod grid_graph_tests {
    use super::*;

    #[test]
    fn test_grid_graph_basic() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(3));
        params.insert("cols".to_string(), MetadataValue::Integer(4));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 12); // rows * cols

        // For a 3x4 grid: (rows-1)*cols + rows*(cols-1) = 2*4 + 3*3 = 17 edges
        assert_eq!(graph.edge_count(), 17);

        // Verify specific nodes exist
        assert!(graph.get_node("n0_0").is_some());
        assert!(graph.get_node("n2_3").is_some());
    }

    #[test]
    fn test_grid_graph_square() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(3));
        params.insert("cols".to_string(), MetadataValue::Integer(3));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 9);
        assert_eq!(graph.edge_count(), 12); // 2*3 + 3*2 = 12
    }

    #[test]
    fn test_grid_graph_periodic() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(3));
        params.insert("cols".to_string(), MetadataValue::Integer(3));
        params.insert("periodic".to_string(), MetadataValue::Boolean(true));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 9);
        // Regular edges + periodic edges: 12 + 3 + 3 = 18
        assert_eq!(graph.edge_count(), 18);
    }

    #[test]
    fn test_grid_graph_single_row() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(1));
        params.insert("cols".to_string(), MetadataValue::Integer(5));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4); // Just a path
    }

    #[test]
    fn test_grid_graph_single_col() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(5));
        params.insert("cols".to_string(), MetadataValue::Integer(1));

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4); // Just a path
    }

    #[test]
    fn test_grid_graph_custom_prefix() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(2));
        params.insert("cols".to_string(), MetadataValue::Integer(2));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("cell".to_string()),
        );

        let graph = generate_grid(&params).unwrap();
        assert_eq!(graph.node_count(), 4);

        // Verify custom prefix
        assert!(graph.get_node("cell0_0").is_some());
        assert!(graph.get_node("cell1_1").is_some());
    }

    #[test]
    fn test_grid_graph_missing_params() {
        let mut params = HashMap::new();
        params.insert("rows".to_string(), MetadataValue::Integer(3));
        // Missing cols parameter

        let result = generate_grid(&params);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod star_graph_tests {
    use super::*;

    #[test]
    fn test_star_graph_basic() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(6));

        let graph = generate_star(&params).unwrap();
        assert_eq!(graph.node_count(), 6);
        assert_eq!(graph.edge_count(), 5); // n-1 edges for star

        // Verify star structure: center has degree n-1, leaves have degree 1
        let mut degrees = HashMap::new();
        for edge in graph.edges.values() {
            *degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let mut degree_counts = HashMap::new();
        for (_, degree) in degrees {
            *degree_counts.entry(degree).or_insert(0) += 1;
        }

        assert_eq!(degree_counts.get(&5), Some(&1)); // 1 center with degree 5
        assert_eq!(degree_counts.get(&1), Some(&5)); // 5 leaves with degree 1
    }

    #[test]
    fn test_star_graph_directed() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(4));
        params.insert("directed".to_string(), MetadataValue::Boolean(true));

        let graph = generate_star(&params).unwrap();
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);

        // In directed star, center should be source of all edges
        let center_id = "n0";
        let mut center_out_degree = 0;
        for edge in graph.edges.values() {
            if edge.source == center_id {
                center_out_degree += 1;
            }
        }
        assert_eq!(center_out_degree, 3);
    }

    #[test]
    fn test_star_graph_single_node() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(1));

        let graph = generate_star(&params).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_star_graph_two_nodes() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(2));

        let graph = generate_star(&params).unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}

#[cfg(test)]
mod tree_graph_tests {
    use super::*;

    #[test]
    fn test_tree_graph_basic() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(2));
        params.insert("depth".to_string(), MetadataValue::Integer(3));

        let graph = generate_tree(&params).unwrap();

        // For binary tree of depth 3: 1 + 2 + 4 = 7 nodes
        assert_eq!(graph.node_count(), 7);
        assert_eq!(graph.edge_count(), 6); // n-1 edges for tree

        // Verify root exists
        assert!(graph.get_node("n0").is_some());
    }

    #[test]
    fn test_tree_graph_ternary() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(3));
        params.insert("depth".to_string(), MetadataValue::Integer(2));

        let graph = generate_tree(&params).unwrap();

        // For ternary tree of depth 2: 1 + 3 = 4 nodes
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
    }

    #[test]
    fn test_tree_graph_depth_zero() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(2));
        params.insert("depth".to_string(), MetadataValue::Integer(0));

        let graph = generate_tree(&params).unwrap();
        assert_eq!(graph.node_count(), 1); // Just root
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_tree_graph_depth_one() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(4));
        params.insert("depth".to_string(), MetadataValue::Integer(1));

        let graph = generate_tree(&params).unwrap();
        assert_eq!(graph.node_count(), 1); // Just root (depth 1 means no children)
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_tree_graph_custom_prefix() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(2));
        params.insert("depth".to_string(), MetadataValue::Integer(2));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("tree".to_string()),
        );

        let graph = generate_tree(&params).unwrap();
        assert_eq!(graph.node_count(), 3); // 1 + 2 nodes

        // Verify custom prefix
        assert!(graph.get_node("tree0").is_some());
        assert!(graph.get_node("tree1").is_some());
        assert!(graph.get_node("tree2").is_some());
    }

    #[test]
    fn test_tree_graph_missing_params() {
        let mut params = HashMap::new();
        params.insert("branching".to_string(), MetadataValue::Integer(2));
        // Missing depth parameter

        let result = generate_tree(&params);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod barabasi_albert_tests {
    use super::*;

    #[test]
    fn test_barabasi_albert_basic() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(10));
        params.insert("edges_per_node".to_string(), MetadataValue::Integer(2));

        let graph = generate_barabasi_albert(&params).unwrap();
        assert_eq!(graph.node_count(), 10);

        // Should have initial complete graph (m=2) plus additional edges
        // Initial: 2 choose 2 = 1 edge, then 8 nodes each adding 2 edges = 17 total
        assert_eq!(graph.edge_count(), 17);
    }

    #[test]
    fn test_barabasi_albert_minimal() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert("edges_per_node".to_string(), MetadataValue::Integer(1));

        let graph = generate_barabasi_albert(&params).unwrap();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2); // Initial 0 edges + 2 nodes adding 1 each
    }

    #[test]
    fn test_barabasi_albert_equal_m_and_n() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert("edges_per_node".to_string(), MetadataValue::Integer(3));

        let result = generate_barabasi_albert(&params);
        assert!(result.is_err()); // Should fail when m >= n
    }

    #[test]
    fn test_barabasi_albert_custom_prefix() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));
        params.insert("edges_per_node".to_string(), MetadataValue::Integer(2));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("ba".to_string()),
        );

        let graph = generate_barabasi_albert(&params).unwrap();
        assert_eq!(graph.node_count(), 5);

        // Verify custom prefix
        assert!(graph.get_node("ba0").is_some());
        assert!(graph.get_node("ba4").is_some());
    }

    #[test]
    fn test_barabasi_albert_missing_params() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));
        // Missing edges_per_node parameter

        let result = generate_barabasi_albert(&params);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod generator_registry_tests {
    use super::*;

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
        assert!(get_generator("nonexistent").is_none());
        assert!(get_generator("").is_none());
        assert!(get_generator("COMPLETE").is_none()); // Case sensitive
    }

    #[test]
    fn test_all_generators_callable() {
        let generators = [
            "complete",
            "path",
            "cycle",
            "grid",
            "star",
            "tree",
            "barabasi_albert",
        ];

        for gen_name in generators {
            let generator = get_generator(gen_name).unwrap();

            // Test with minimal valid parameters
            let mut params = HashMap::new();
            match gen_name {
                "complete" | "path" | "cycle" | "star" => {
                    params.insert("nodes".to_string(), MetadataValue::Integer(3));
                }
                "grid" => {
                    params.insert("rows".to_string(), MetadataValue::Integer(2));
                    params.insert("cols".to_string(), MetadataValue::Integer(2));
                }
                "tree" => {
                    params.insert("branching".to_string(), MetadataValue::Integer(2));
                    params.insert("depth".to_string(), MetadataValue::Integer(2));
                }
                "barabasi_albert" => {
                    params.insert("nodes".to_string(), MetadataValue::Integer(5));
                    params.insert("edges_per_node".to_string(), MetadataValue::Integer(2));
                }
                _ => unreachable!(),
            }

            let result = generator(&params);
            assert!(
                result.is_ok(),
                "Generator {} failed with minimal params: {:?}",
                gen_name,
                result.err()
            );
        }
    }
}

#[cfg(test)]
mod parameter_validation_tests {
    use super::*;

    #[test]
    fn test_parameter_types() {
        // Test that generators handle different parameter types correctly
        let mut params = HashMap::new();
        params.insert(
            "nodes".to_string(),
            MetadataValue::String("not_a_number".to_string()),
        );

        let result = generate_complete(&params);
        assert!(result.is_err(), "Should reject non-numeric nodes parameter");
    }

    #[test]
    fn test_boolean_parameters() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert(
            "directed".to_string(),
            MetadataValue::String("not_a_bool".to_string()),
        );

        // Should use default value for invalid boolean
        let result = generate_complete(&params);
        assert!(result.is_ok(), "Should handle invalid boolean gracefully");
    }

    #[test]
    fn test_string_parameters() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert("prefix".to_string(), MetadataValue::Integer(123));

        // Should convert number to string or use default
        let result = generate_complete(&params);
        assert!(result.is_ok(), "Should handle invalid string gracefully");
    }

    #[test]
    fn test_negative_numbers() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(-5));

        // Negative numbers should be handled (likely converted to 0)
        let result = generate_complete(&params);
        assert!(result.is_ok(), "Should handle negative numbers");

        let graph = result.unwrap();
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_fractional_numbers() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Float(3.7));

        let result = generate_complete(&params);
        assert!(result.is_ok(), "Should handle fractional numbers");

        let graph = result.unwrap();
        assert_eq!(graph.node_count(), 3); // Should truncate to integer
    }
}

#[cfg(test)]
mod graph_properties_tests {
    use super::*;

    #[test]
    fn test_graph_connectivity() {
        // Test that generated graphs have expected connectivity properties
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(5));

        let complete_graph = generate_complete(&params).unwrap();
        let path_graph = generate_path(&params).unwrap();
        let cycle_graph = generate_cycle(&params).unwrap();

        // Complete graph should be fully connected
        assert_eq!(complete_graph.edge_count(), 10);

        // Path graph should be minimally connected
        assert_eq!(path_graph.edge_count(), 4);

        // Cycle graph should be connected with one extra edge
        assert_eq!(cycle_graph.edge_count(), 5);
    }

    #[test]
    fn test_node_naming_consistency() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));
        params.insert(
            "prefix".to_string(),
            MetadataValue::String("test".to_string()),
        );

        let graph = generate_complete(&params).unwrap();

        // All nodes should follow the naming pattern
        assert!(graph.get_node("test0").is_some());
        assert!(graph.get_node("test1").is_some());
        assert!(graph.get_node("test2").is_some());
        assert!(graph.get_node("test3").is_none());
    }

    #[test]
    fn test_edge_properties() {
        let mut params = HashMap::new();
        params.insert("nodes".to_string(), MetadataValue::Integer(3));

        let graph = generate_complete(&params).unwrap();

        // All edges should have valid source and target nodes
        for edge in graph.edges.values() {
            assert!(
                graph.get_node(&edge.source).is_some(),
                "Edge source {} should exist",
                edge.source
            );
            assert!(
                graph.get_node(&edge.target).is_some(),
                "Edge target {} should exist",
                edge.target
            );
            assert_ne!(edge.source, edge.target, "No self-loops in complete graph");
        }
    }
}
