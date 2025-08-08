#[cfg(test)]
mod tests {
    use graph_generation_language::GGLEngine;
    use serde_json::Value;

    #[test]
    fn test_simple_node_edge_creation() {
        // Covers: test_simple.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: [
            Node { id: "user1", meta: { age: 25 } },
            Node { id: "user2", meta: { age: 30 } }
          ],
          edges: [
            Edge { source: "user1", target: "user2", meta: { type: "friend" } }
          ]
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Simple node/edge creation failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify structure
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
        assert_eq!(graph["edges"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_combinations_with_pipe() {
        // Covers: test_combinations.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: ["a", "b", "c"],
          edges: combinations(["a", "b", "c"], 2).map(([a, b]) => Edge {
            source: a,
            target: b,
            meta: {}
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Combinations with pipe failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify structure has all expected fields
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_range_with_map_and_pipe() {
        // Covers: test_with_range.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          friend_recommendation: (graph) => ({
            ...graph,
            edges: graph.edges.concat(
              combinations(graph.nodes, 2)
            )
          }),

          nodes: range("0..3").map(i => ({ id: `user${i}` })),

          edges: [
            { source: "user0", target: "user1" }
          ].pipe(friend_recommendation, 1)
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Range with map and pipe failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify nodes were generated from range
        assert!(graph["nodes"].is_array());
        let nodes = graph["nodes"].as_array().unwrap();
        assert!(nodes.len() >= 3); // Should have at least 3 nodes from range(0..3)

        // Verify edges exist
        assert!(graph["edges"].is_array());
    }

    #[test]
    fn test_lambda_destructuring_advanced() {
        // Covers: test_destructuring.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: [
            { id: "a", meta: { x: 1 } },
            { id: "b", meta: { x: 2 } }
          ],

          edges: combinations([
            { id: "a", meta: { x: 1 } },
            { id: "b", meta: { x: 2 } }
          ], 2).map(([a, b]) => {
            return { source: a.id, target: b.id };
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Lambda destructuring failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify nodes and edges
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_pipe_operations() {
        // Covers: test_pipe.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: ["a", "b", "c"],
          edges: range("0..2").map(i => Edge {
            source: "a",
            target: "b",
            meta: { index: i }
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Pipe operations failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify valid JSON structure
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
    }

    #[test]
    fn test_context_variables() {
        // Covers: test_context.ggl, test_debug_context.ggl, test_lambda_context.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..3").map(i => Node {
            id: `node${i}`,
            meta: { value: 10 + i }
          }),
          edges: []
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Context variables failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify valid JSON structure
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
    }

    #[test]
    fn test_node_combinations() {
        // Covers: test_node_combinations.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..4").map(i => Node { id: `n${i}`, meta: {} }),
          edges: combinations(range("0..4"), 2).map(([a, b]) => Edge {
            source: `n${a}`,
            target: `n${b}`,
            meta: { weight: 1.0 }
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Node combinations failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify nodes and edges were generated
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        let nodes = graph["nodes"].as_array().unwrap();
        let edges = graph["edges"].as_array().unwrap();
        assert!(nodes.len() >= 4);
        assert!(!edges.is_empty());
    }

    #[test]
    fn test_with_edges_patterns() {
        // Covers: test_with_edges.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: [
            Node { id: "a", meta: { type: "start" } },
            Node { id: "b", meta: { type: "middle" } },
            Node { id: "c", meta: { type: "end" } }
          ],
          edges: [
            Edge { source: "a", target: "b", meta: { weight: 0.8 } },
            Edge { source: "b", target: "c", meta: { weight: 0.6 } }
          ]
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "With edges patterns failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify structure
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 3);
        assert_eq!(graph["edges"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_graph_structure_validation() {
        // Covers: test_graph_structure.ggl, test_exact_structure.ggl, test_matching_structure.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..5").map(i => Node {
            id: `vertex${i}`,
            meta: {
              degree: 0,
              visited: false,
              distance: i * 2
            }
          }),
          edges: combinations(range("0..5"), 2)
            .slice(0, 7)
            .map(([a, b]) => Edge {
              source: `vertex${a}`,
              target: `vertex${b}`,
              meta: {
                weight: (a + b) * 0.1,
                bidirectional: true
              }
            })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Graph structure validation failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify structure and metadata
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());

        let nodes = graph["nodes"].as_array().unwrap();
        let edges = graph["edges"].as_array().unwrap();

        assert_eq!(nodes.len(), 5);
        assert!(edges.len() <= 7); // Should be sliced to max 7

        // Verify node structure
        for node in nodes {
            assert!(node["id"].is_string());
            assert!(node["meta"].is_object());
            assert!(node["meta"]["degree"].is_number());
            assert!(node["meta"]["visited"].is_boolean());
            assert!(node["meta"]["distance"].is_number());
        }
    }

    #[test]
    fn test_iterations_and_loops() {
        // Covers: test_iterations.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..10").map(i => Node {
            id: `n${i}`,
            meta: { level: i % 3 }
          }),
          edges: range("0..9").map(i => Edge {
            source: `n${i}`,
            target: `n${i + 1}`,
            meta: { step: i }
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Iterations and loops failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify iterative generation
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());

        let nodes = graph["nodes"].as_array().unwrap();
        let edges = graph["edges"].as_array().unwrap();

        assert_eq!(nodes.len(), 10);
        assert_eq!(edges.len(), 9);
    }

    #[test]
    fn test_chain_issue_resolution() {
        // Covers: test_chain_issue.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..5")
            .map(x => x * 2)
            .filter(x => x > 4)
            .slice(0, 3)
            .map(i => Node { id: `n${i}`, meta: {} }),
          edges: []
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Chain issue resolution failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify chaining works correctly
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
    }

    #[test]
    fn test_debug_functionality() {
        // Covers: test_debug.ggl, test_combinations_debug.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: range("0..3").map(i => Node {
            id: `debug${i}`,
            meta: {
              debug: true,
              index: i
            }
          }),
          edges: combinations(range("0..3"), 2).map(([a, b]) => Edge {
            source: `debug${a}`,
            target: `debug${b}`,
            meta: { debugEdge: true }
          })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Debug functionality failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify valid JSON structure
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
    }

    #[test]
    fn test_simple_destructuring_patterns() {
        // Covers: test_simple_destructuring.ggl
        let mut engine = GGLEngine::new();
        let code = r#"
        {
          nodes: [[1, 2], [3, 4], [5, 6]].map(([first, second]) => Node {
            id: `n${first}${second}`,
            meta: {
              sum: first + second,
              product: first * second
            }
          }),
          edges: []
        }
        "#;

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Simple destructuring patterns failed: {:?}", result.err());

        let json = result.unwrap();
        let graph: Value = serde_json::from_str(&json).unwrap();

        // Verify destructuring worked
        assert!(graph["nodes"].is_array());
        assert!(graph["edges"].is_array());
        assert_eq!(graph["nodes"].as_array().unwrap().len(), 3);
    }
}
