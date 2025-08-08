//! WASM-specific tests for the Graph Generation Language library.
//!
//! These tests verify that the WASM bindings work correctly and that
//! the library can be used from JavaScript environments.

#![cfg(target_arch = "wasm32")]

use ggl_wasm::{WASMGGLEngine, parse_ggl};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_engine_creation() {
    let _engine = WASMGGLEngine::new();
    // If we can create an engine without panicking, the test passes
    assert!(true);
}

#[wasm_bindgen_test]
fn test_simple_json_generation() {
    let mut engine = WASMGGLEngine::new();
    let ggl_code = r#"
        {
            nodes: [
                Node { id: "alice", meta: { type: "person" } },
                Node { id: "bob", meta: { type: "person" } }
            ],
            edges: [
                Edge { source: "alice", target: "bob", meta: { type: "friend" } }
            ]
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("alice"));
    assert!(json.contains("bob"));
    assert!(json.contains("friend"));
}

#[wasm_bindgen_test]
fn test_range_generation() {
    let mut engine = WASMGGLEngine::new();
    let ggl_code = r#"
        {
            nodes: range("0..3").map(i => Node {
                id: `node${i}`,
                meta: { index: i }
            }),
            edges: []
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("node0"));
    assert!(json.contains("node1"));
    assert!(json.contains("node2"));
}

#[wasm_bindgen_test]
fn test_combinations_generation() {
    let mut engine = WASMGGLEngine::new();
    let ggl_code = r#"
        {
            nodes: ["a", "b", "c"],
            edges: combinations(["a", "b", "c"], 2).map(([source, target]) => Edge {
                source: source,
                target: target,
                meta: { type: "connection" }
            })
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("connection"));
    // Should have 3 combinations: (a,b), (a,c), (b,c)
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["edges"].as_array().unwrap().len(), 3);
}

#[wasm_bindgen_test]
fn test_error_handling() {
    let mut engine = WASMGGLEngine::new();
    let invalid_ggl = "{ invalid syntax here }";

    let result = engine.generate_from_ggl(invalid_ggl);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_parse_ggl_utility() {
    let ggl_code = r#"
        {
            nodes: [Node { id: "test", meta: {} }],
            edges: []
        }
    "#;

    let result = parse_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("test"));
}

#[wasm_bindgen_test]
fn test_complex_graph_operations() {
    let mut engine = WASMGGLEngine::new();
    let ggl_code = r#"
        {
            vertices: range("0..4").map(i => ({ id: `v${i}`, value: i * 2 })),
            nodes: vertices.map(v => Node {
                id: v.id,
                meta: {
                    value: v.value,
                    type: if (v.value > 4) { "high" } else { "low" }
                }
            }),
            edges: combinations(vertices, 2)
                .slice(0, 3)
                .map(([a, b]) => Edge {
                    source: a.id,
                    target: b.id,
                    meta: { weight: (a.value + b.value) / 2 }
                })
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Should have 4 nodes
    assert_eq!(parsed["nodes"].as_array().unwrap().len(), 4);
    // Should have 3 edges (sliced)
    assert_eq!(parsed["edges"].as_array().unwrap().len(), 3);
}
