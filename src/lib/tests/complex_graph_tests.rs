
#[cfg(test)]
mod complex_graph_patterns_tests {
    use graph_generation_language::GGLEngine;
    use serde_json::Value;


    #[test]
    fn test_grid_like_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
        {
            nodes: range("0..9").map(idx => Node {
                id: `cell_${(idx / 3).floor()}_${idx % 3}`,
                meta: {
                    row: (idx / 3).floor(),
                    col: idx % 3,
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
            satellites: range("0..5").map(i => Node {
                id: `satellite${i}`,
                meta: { type: "satellite" }
            }),
            nodes: [
                Node { id: "hub", meta: { type: "center" } },
                ...satellites
            ],
            edges: range("0..5").map(i => Edge {
                source: "hub",
                target: `satellite${i}`,
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
            nodes: range("0..5").map(i => Node {
                id: `step${i}`,
                meta: { position: i }
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
