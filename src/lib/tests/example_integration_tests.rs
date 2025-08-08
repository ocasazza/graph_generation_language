#[cfg(test)]
mod tests {
    use graph_generation_language::GGLEngine;

    #[test]
    fn test_social_network_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/social_network.ggl");

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Social network example failed to parse: {:?}", result.err());

        // Verify we got valid JSON
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Social network example produced invalid JSON");
    }

    #[test]
    fn test_hpc_network_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/hpc_network.ggl");

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "HPC network example failed to parse: {:?}", result.err());

        // Verify we got valid JSON
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "HPC network example produced invalid JSON");
    }

    #[test]
    fn test_chemical_reaction_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/chemical_reaction.ggl");

        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Chemical reaction example failed to parse: {:?}", result.err());

        // Verify we got valid JSON
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Chemical reaction example produced invalid JSON");
    }

    #[test]
    fn test_toroidal_mesh_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/toroidal_mesh.ggl");
        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Toroidal mesh example failed to parse: {:?}", result.err());
        // Verify we got valid JSON
        let json = result.unwrap();
        let graph: serde_json::Value = serde_json::from_str(&json).unwrap();
        // Check nodes
        let nodes = graph.get("nodes").unwrap().as_array().unwrap();
        // Check edges
        let edges = graph.get("edges").unwrap().as_array().unwrap();
        // Should have 36 nodes (6x6 grid)
        assert_eq!(nodes.len(), 36, "Expected 36 nodes in 6x6 mesh");
        // Should have many edges (horizontal + vertical + express links)
        // 6x6 mesh should have at least 72 edges (36 horizontal + 36 vertical + express)
        assert!(edges.len() >= 70, "Toroidal mesh should have many edges, found {}", edges.len());
    }

    #[test]
    fn test_protein_network_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/protein_network.ggl");
        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Protein network example failed to parse: {:?}", result.err());
        // Verify we got valid JSON
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Protein network example produced invalid JSON");
    }

    #[test]
    fn test_quantum_circuit_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/quantum_circuit.ggl");
        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Quantum circuit example failed to parse: {:?}", result.err());
        // Verify we got valid JSON
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Quantum circuit example produced invalid JSON");
    }

    #[test]
    fn test_neural_network_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/neural_network.ggl");
        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Neural network example failed to parse: {:?}", result.err());
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Neural network example produced invalid JSON");
    }

    #[test]
    fn test_crystal_lattice_example() {
        let mut engine = GGLEngine::new();
        let code = include_str!("../../examples/crystal_lattice.ggl");
        let result = engine.generate_from_ggl(code);
        assert!(result.is_ok(), "Crystal lattice example failed to parse: {:?}", result.err());
        let json = result.unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "Crystal lattice example produced invalid JSON");
    }

    #[test]
    fn test_all_examples_parse_successfully() {
        let examples = [
            ("social_network", include_str!("../../examples/social_network.ggl")),
            ("hpc_network", include_str!("../../examples/hpc_network.ggl")),
            ("chemical_reaction", include_str!("../../examples/chemical_reaction.ggl")),
            ("toroidal_mesh", include_str!("../../examples/toroidal_mesh.ggl")),
            ("protein_network", include_str!("../../examples/protein_network.ggl")),
            ("quantum_circuit", include_str!("../../examples/quantum_circuit.ggl")),
            ("neural_network", include_str!("../../examples/neural_network.ggl")),
            ("crystal_lattice", include_str!("../../examples/crystal_lattice.ggl")),
        ];
        for (name, code) in examples.iter() {
            let mut engine = GGLEngine::new();
            let result = engine.generate_from_ggl(code);
            assert!(result.is_ok(), "Example '{}' failed to parse: {:?}", name, result.err());
            let json = result.unwrap();
            assert!(
                serde_json::from_str::<serde_json::Value>(&json).is_ok(),
                "Example '{name}' produced invalid JSON"
            );
            let graph: serde_json::Value = serde_json::from_str(&json).unwrap();
            // Verify the graph has nodes and edges structures
            assert!(graph.get("nodes").is_some(), "Example '{name}' missing nodes");
            assert!(graph.get("edges").is_some(), "Example '{name}' missing edges");
            // Verify nodes array is not empty for most examples
            let nodes = graph.get("nodes").unwrap().as_array().unwrap();
            if *name != "empty_graph" {
                assert!(!nodes.is_empty(), "Example '{name}' has no nodes");
            }
        }
    }
}
