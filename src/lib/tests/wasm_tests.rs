//! WASM-specific tests for the Graph Generation Language library.
//!
//! These tests verify that the WASM bindings work correctly and that
//! the library can be used from JavaScript environments.

#![cfg(target_arch = "wasm32")]

use graph_generation_language::GGLEngine;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_ggl_engine_creation() {
    let engine = GGLEngine::new();
    // If we can create an engine without panicking, the test passes
    assert!(true);
}

#[wasm_bindgen_test]
fn test_simple_graph_generation() {
    let mut engine = GGLEngine::new();
    let ggl_code = r#"
        graph simple {
            node alice;
            node bob;
            edge: alice -- bob;
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains("alice"));
    assert!(json.contains("bob"));
}

#[wasm_bindgen_test]
fn test_graph_with_generators() {
    let mut engine = GGLEngine::new();
    let ggl_code = r#"
        graph generated {
            generate complete {
                nodes: 3;
                prefix: "node";
            }
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
fn test_error_handling() {
    let mut engine = GGLEngine::new();
    let invalid_ggl = "invalid syntax here";

    let result = engine.generate_from_ggl(invalid_ggl);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_panic_hook_setup() {
    // This test ensures that the panic hook is properly set up
    // If it's not, this test might not provide useful error messages
    graph_generation_language::set_panic_hook();

    // Test that we can still create and use an engine after setting up the panic hook
    let mut engine = GGLEngine::new();
    let ggl_code = r#"
        graph test {
            node test_node;
        }
    "#;

    let result = engine.generate_from_ggl(ggl_code);
    assert!(result.is_ok());
}
