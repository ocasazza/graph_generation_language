
#[cfg(test)]
mod mathematical_operations_tests {
    use graph_generation_language::GGLEngine;
    use serde_json::Value;

    

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
