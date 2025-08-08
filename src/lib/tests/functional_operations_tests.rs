
#[cfg(test)]
mod functional_operations_tests {
    use graph_generation_language::GGLEngine;
    use serde_json::Value;



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
