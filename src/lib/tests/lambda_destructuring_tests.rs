#[cfg(test)]
mod tests {
    use graph_generation_language::GGLEngine;

    #[test]
    fn test_lambda_destructuring() {
        let mut engine = GGLEngine::new();
        let code = r#"
        {
            nodes: [],
            edges: combinations([
                { id: "a", value: 1 },
                { id: "b", value: 2 }
            ], 2).map(([first, second]) => {
                return {
                    source: first.id,
                    target: second.id
                };
            })
        }
        "#;

        let result = engine.generate_from_ggl(code);
        println!("Destructuring test result: {result:?}");
        assert!(result.is_ok(), "Lambda destructuring failed: {:?}", result.err());
    }

    #[test]
    fn test_simple_lambda_destructuring() {
        let mut engine = GGLEngine::new();
        let code = r#"
        {
            nodes: [],
            edges: [[{ id: "test" }]].map(([item]) => item.id)
        }
        "#;

        let result = engine.generate_from_ggl(code);
        println!("Simple destructuring test result: {result:?}");
        assert!(result.is_ok(), "Simple destructuring failed: {:?}", result.err());
    }
}
