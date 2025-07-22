use graph_generation_language::GGLEngine;
use serde_json::Value;

#[test]
fn test_mixed_numeric_operations() {
    let mut engine = GGLEngine::new();

    let ggl_code = r#"
        graph mixed_numeric_test {
            // Test float literals
            let stress_level = 0.7;
            if stress_level > 0.5 {
                node high_stress [level=stress_level, critical=true];
            }

            // Test mixed comparisons: int compared to float
            let count = 10;
            if count > 5.5 {
                node enough_nodes [count=count, sufficient=true];
            }

            // Test float values in attributes
            node float_node [value=3.14, ratio=1.5];

            // Test mixed arithmetic in loops (simple addition)
            for i in 0..3 {
                node "node_{i}" [index=i, base_value=0.5];
            }
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok(), "Failed to process mixed numeric operations: {:?}", result.err());

    let json_str = result.unwrap();
    let graph: Value = serde_json::from_str(&json_str).unwrap();

    let nodes = graph["nodes"].as_object().unwrap();

    // Verify nodes were created (indicating conditions worked)
    assert!(nodes.contains_key("high_stress"));
    assert!(nodes.contains_key("enough_nodes"));
    assert!(nodes.contains_key("ratio_node"));
    assert!(nodes.contains_key("weighted_0"));
    assert!(nodes.contains_key("weighted_1"));
    assert!(nodes.contains_key("weighted_2"));

    // Verify float values are preserved
    assert_eq!(nodes["high_stress"]["metadata"]["level"], 0.7);
    assert_eq!(nodes["high_stress"]["metadata"]["critical"], true);

    // Verify division result is float
    assert_eq!(nodes["ratio_node"]["metadata"]["ratio"], 1.5);

    // Verify mixed arithmetic in loop
    assert_eq!(nodes["weighted_0"]["metadata"]["weight"], 0.5);
    assert_eq!(nodes["weighted_1"]["metadata"]["weight"], 0.6);
    assert_eq!(nodes["weighted_2"]["metadata"]["weight"], 0.7);

    println!("✅ Mixed numeric operations test passed!");
    println!("Generated graph with {} nodes", nodes.len());
    for (name, node) in nodes {
        println!("  {}: {:?}", name, node["metadata"]);
    }
}
