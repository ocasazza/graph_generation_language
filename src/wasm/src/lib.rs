//! WebAssembly bindings for the Graph Generation Language (GGL)
//!
//! This crate provides WebAssembly bindings for the GGL library, allowing
//! GGL to be used in web browsers and other JavaScript environments.

use graph_generation_language::GGLEngine;
use wasm_bindgen::prelude::*;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization, and then
// we will get better error messages if our code ever panics.
//
// For more details see
// https://github.com/rustwasm/console_error_panic_hook#readme
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(start)]
pub fn run() {
    // Set up panic hook for better error reporting
    set_panic_hook();
    web_sys::console::log_1(&"🚀 Graph Generation Language WASM module loaded!".into());
}

/// WebAssembly wrapper for the GGL engine.
///
/// This provides a JavaScript-compatible interface to the core GGL functionality.
///
/// # Examples
///
/// ```javascript
/// import init, { GGLEngine } from './pkg/ggl_wasm.js';
///
/// async function main() {
///     await init();
///
///     const engine = new GGLEngine();
///     const gglCode = `{
///       nodes: ["a", "b", "c"],
///       edges: combinations(["a", "b", "c"], 2).map(([a, b]) => Edge {
///         source: a,
///         target: b,
///         meta: {}
///       })
///     }`;
///
///     try {
///         const result = engine.generate_from_ggl(gglCode);
///         console.log("Graph:", JSON.parse(result));
///     } catch (error) {
///         console.error("Error:", error);
///     }
/// }
///
/// main();
/// ```
#[wasm_bindgen]
pub struct WASMGGLEngine {
    inner: GGLEngine,
}

 impl Default for WASMGGLEngine {
    fn default() -> Self {
       Self::new()
    }
}

#[wasm_bindgen]
impl WASMGGLEngine {
    /// Creates a new GGL engine.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const engine = new WASMGGLEngine();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> WASMGGLEngine {
        set_panic_hook();
        WASMGGLEngine {
            inner: GGLEngine::new(),
        }
    }

    /// Parses and executes a GGL program, returning the resulting graph as JSON.
    ///
    /// # Arguments
    ///
    /// * `ggl_code` - A string containing the GGL program to execute
    ///
    /// # Returns
    ///
    /// Returns the JSON representation of the generated graph.
    /// Throws a JavaScript error if processing fails.
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const engine = new GGLEngine();
    /// const gglCode = `
    ///     graph simple {
    ///         node a;
    ///         node b;
    ///         edge: a -- b;
    ///     }
    /// `;
    ///
    /// try {
    ///     const result = engine.generate_from_ggl(gglCode);
    ///     console.log("Graph:", JSON.parse(result));
    /// } catch (error) {
    ///     console.error("Error:", error);
    /// }
    /// ```
    #[wasm_bindgen]
    pub fn generate_from_ggl(&mut self, ggl_code: &str) -> Result<String, JsValue> {
        self.inner
            .generate_from_ggl(ggl_code)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Sets the base path for relative file inclusions in GGL programs.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path to use for resolving relative file paths
    ///
    /// # Examples
    ///
    /// ```javascript
    /// const engine = new GGLEngine();
    /// engine.set_base_path("/path/to/ggl/files");
    /// ```
    #[wasm_bindgen]
    pub fn set_base_path(&mut self, path: &str) {
        self.inner = std::mem::take(&mut self.inner).with_base_path(path);
    }
}

/// Utility function to parse GGL code and return the result as JSON.
///
/// This is a convenience function that creates a new engine, processes the code,
/// and returns the result in one call.
///
/// # Arguments
///
/// * `ggl_code` - A string containing the GGL program to execute
///
/// # Returns
///
/// Returns the JSON representation of the generated graph.
/// Throws a JavaScript error if processing fails.
///
/// # Examples
///
/// ```javascript
/// import { parse_ggl } from './pkg/ggl_wasm.js';
///
/// const gglCode = `
///     graph simple {
///         node a;
///         node b;
///         edge: a -- b;
///     }
/// `;
///
/// try {
///     const result = parse_ggl(gglCode);
///     console.log("Graph:", JSON.parse(result));
/// } catch (error) {
///     console.error("Error:", error);
/// }
/// ```
#[wasm_bindgen]
pub fn parse_ggl(ggl_code: &str) -> Result<String, JsValue> {
    let mut engine = GGLEngine::new();
    engine
        .generate_from_ggl(ggl_code)
        .map_err(|e| JsValue::from_str(&e))
}
