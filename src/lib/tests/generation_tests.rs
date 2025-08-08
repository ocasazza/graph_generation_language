use graph_generation_language::GGLEngine;
use serde_json::Value;

#[cfg(test)]
mod range_generation_tests {
    use super::*;

    #[test]
    fn test_simple_range_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("0..5").map(i => Node {
                id: `node${i}`,
                meta: { index: i }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to generate range-based graph: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 5 nodes (0..5 excludes 5)
        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 5);

        // Verify node structure
        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(node["id"], format!("node{i}"));
            assert_eq!(node["meta"]["index"], i);
        }
    }

    #[test]
    fn test_larger_range() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("0..10").map(i => Node {
                id: `n${i}`,
                meta: { value: i * 2 }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 10);

        // Check computed values
        assert_eq!(nodes[0]["meta"]["value"], 0);
        assert_eq!(nodes[5]["meta"]["value"], 10);
        assert_eq!(nodes[9]["meta"]["value"], 18);
    }
}

#[cfg(test)]
mod combinations_generation_tests {
    use super::*;

    #[test]
    fn test_combinations_basic() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: [
                { id: "a", meta: {} },
                { id: "b", meta: {} },
                { id: "c", meta: {} }
            ],
            edges: combinations([
                { id: "a" },
                { id: "b" },
                { id: "c" }
            ], 2).map(([first, second]) => Edge {
                source: first.id,
                target: second.id,
                meta: { type: "connection" }
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to generate combinations: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 3 nodes
        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 3);

        // Should have 3 edges (3 choose 2)
        let edges = graph["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 3);

        // Verify edge structure
        for edge in edges {
            assert!(edge["source"].is_string());
            assert!(edge["target"].is_string());
            assert_eq!(edge["meta"]["type"], "connection");
        }
    }

    #[test]
    fn test_complete_graph_with_combinations() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            vertices: range("0..4").map(i => ({ id: `node${i}` })),
            nodes: vertices.map(v => Node {
                id: v.id,
                meta: { index: v.id.slice(4) }
            }),
            edges: combinations(vertices, 2).map(([a, b]) => Edge {
                source: a.id,
                target: b.id,
                meta: { weight: 1.0 }
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 4 nodes
        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 4);

        // Should have 6 edges (complete graph: 4 choose 2)
        let edges = graph["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 6);
    }

    #[test]
    fn test_larger_combinations() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            items: range("0..6").map(i => ({ id: i })),
            nodes: [],
            edges: combinations(items, 3).map(([first, second, third]) => Edge {
                source: `group_${first.id}`,
                target: `group_${second.id}_${third.id}`,
                meta: { size: 3 }
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 20 edges (6 choose 3 = 20)
        let edges = graph["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 20);
    }
}

#[cfg(test)]
mod functional_operations_tests {
    use super::*;

    #[test]
    fn test_map_operation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("1..4").map(i => Node {
                id: `user${i}`,
                meta: {
                    value: i,
                    squared: i * i,
                    label: `User #${i}`
                }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 3);

        // Check computed values
        assert_eq!(nodes[0]["meta"]["squared"], 1);
        assert_eq!(nodes[1]["meta"]["squared"], 4);
        assert_eq!(nodes[2]["meta"]["squared"], 9);
        assert_eq!(nodes[2]["meta"]["label"], "User #3");
    }

    #[test]
    fn test_filter_operation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            all_nums: range("0..10"),
            even_nums: all_nums.filter(n => n % 2 === 0),
            nodes: even_nums.map(n => Node {
                id: `even${n}`,
                meta: { value: n, type: "even" }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        // Even numbers from 0..10: 0, 2, 4, 6, 8 = 5 nodes
        assert_eq!(nodes.len(), 5);

        // Verify all are even
        for node in nodes {
            let value = node["meta"]["value"].as_u64().unwrap();
            assert_eq!(value % 2, 0);
            assert_eq!(node["meta"]["type"], "even");
        }
    }

    #[test]
    fn test_chained_operations() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("1..6")
                .filter(n => n > 2)
                .map(n => Node {
                    id: `filtered${n}`,
                    meta: {
                        original: n,
                        processed: true
                    }
                }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        // Numbers > 2 from 1..6: 3, 4, 5 = 3 nodes
        assert_eq!(nodes.len(), 3);

        // Check values
        assert_eq!(nodes[0]["meta"]["original"], 3);
        assert_eq!(nodes[1]["meta"]["original"], 4);
        assert_eq!(nodes[2]["meta"]["original"], 5);
    }
}

#[cfg(test)]
mod complex_graph_patterns_tests {
    use super::*;

    #[test]
    fn test_grid_like_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            rows: 3,
            cols: 3,
            positions: range("0..9").map(idx => ({
                row: (idx / 3).floor(),
                col: idx % 3,
                id: `cell_${(idx / 3).floor()}_${idx % 3}`
            })),
            nodes: positions.map(pos => Node {
                id: pos.id,
                meta: {
                    row: pos.row,
                    col: pos.col,
                    type: "grid_cell"
                }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 9); // 3x3 grid

        // Verify grid positioning
        assert_eq!(nodes[0]["id"], "cell_0_0");
        assert_eq!(nodes[4]["id"], "cell_1_1");
        assert_eq!(nodes[8]["id"], "cell_2_2");
    }

    #[test]
    fn test_star_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            center: "hub",
            satellites: range("0..5").map(i => `satellite${i}`),
            nodes: [
                Node { id: center, meta: { type: "center" } },
                ...satellites.map(sat => Node {
                    id: sat,
                    meta: { type: "satellite" }
                })
            ],
            edges: satellites.map(sat => Edge {
                source: center,
                target: sat,
                meta: { type: "spoke" }
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 6); // 1 center + 5 satellites

        let edges = graph["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 5); // 5 spokes

        // Verify all edges connect to center
        for edge in edges {
            assert_eq!(edge["source"], "hub");
        }
    }

    #[test]
    fn test_path_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            path_nodes: range("0..5").map(i => `step${i}`),
            nodes: path_nodes.map(id => Node {
                id: id,
                meta: { position: id.slice(4) }
            }),
            edges: range("0..4").map(i => Edge {
                source: `step${i}`,
                target: `step${i + 1}`,
                meta: { type: "path_segment" }
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 5);

        let edges = graph["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 4); // n-1 edges for path

        // Verify path connectivity
        assert_eq!(edges[0]["source"], "step0");
        assert_eq!(edges[0]["target"], "step1");
        assert_eq!(edges[3]["source"], "step3");
        assert_eq!(edges[3]["target"], "step4");
    }
}

#[cfg(test)]
mod mathematical_operations_tests {
    use super::*;

    #[test]
    fn test_arithmetic_in_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("1..6").map(i => Node {
                id: `calc${i}`,
                meta: {
                    input: i,
                    doubled: i * 2,
                    sum: i + 10,
                    quotient: 20 / i,
                    remainder: i % 3
                }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 5);

        // Check calculations
        let node1 = &nodes[1]; // i=2
        assert_eq!(node1["meta"]["doubled"], 4);
        assert_eq!(node1["meta"]["sum"], 12);
        assert_eq!(node1["meta"]["remainder"], 2);
    }

    #[test]
    fn test_conditional_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("0..10").map(i => Node {
                id: `node${i}`,
                meta: {
                    value: i,
                    category: if (i < 3) { "small" } else {
                        if (i < 7) { "medium" } else { "large" }
                    },
                    even: i % 2 === 0
                }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 10);

        // Check categorization
        assert_eq!(nodes[1]["meta"]["category"], "small");
        assert_eq!(nodes[4]["meta"]["category"], "medium");
        assert_eq!(nodes[8]["meta"]["category"], "large");

        // Check boolean values
        assert_eq!(nodes[2]["meta"]["even"], true);
        assert_eq!(nodes[3]["meta"]["even"], false);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_range() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("invalid").map(i => Node { id: `n${i}`, meta: {} }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_combinations() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: [],
            edges: combinations("not_an_array", 2).map(pair => Edge {
                source: pair[0],
                target: pair[1],
                meta: {}
            })
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }

    #[test]
    fn test_syntax_error() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("0..3").map(i => Node {
                id: `node${i}`,
                meta: { invalid syntax here }
            }),
            edges: []
        }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }
}
