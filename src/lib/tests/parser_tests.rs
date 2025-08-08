use graph_generation_language::parser::{parse_ggl, Expression};
use graph_generation_language::GGLEngine;
use serde_json::Value;

#[cfg(test)]
mod basic_parsing_tests {
    use super::*;

    #[test]
    fn test_simple_object_parsing() {
        let input = r#"
        {
            nodes: [],
            edges: []
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok(), "Failed to parse simple object: {:?}", result.err());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert_eq!(pairs.len(), 2);
                assert!(pairs.contains_key("nodes"));
                assert!(pairs.contains_key("edges"));
            }
            _ => panic!("Expected ObjectExpression at root"),
        }
    }

    #[test]
    fn test_array_parsing() {
        let input = r#"
        {
            numbers: [1, 2, 3, 4, 5],
            strings: ["hello", "world"],
            mixed: [1, "test", true, null]
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert_eq!(pairs.len(), 3);

                // Test that arrays are properly parsed
                assert!(matches!(pairs.get("numbers"), Some(Expression::ArrayExpression(_))));
                assert!(matches!(pairs.get("strings"), Some(Expression::ArrayExpression(_))));
                assert!(matches!(pairs.get("mixed"), Some(Expression::ArrayExpression(_))));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_literal_values() {
        let input = r#"
        {
            integer: 42,
            float: 3.14159,
            string: "hello world",
            boolean_true: true,
            boolean_false: false,
            null_value: null
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert!(matches!(pairs.get("integer"), Some(Expression::Integer(42))));
                assert!(matches!(pairs.get("float"), Some(Expression::Float(_))));
                assert!(matches!(pairs.get("string"), Some(Expression::StringLiteral(_))));
                assert!(matches!(pairs.get("boolean_true"), Some(Expression::Boolean(true))));
                assert!(matches!(pairs.get("boolean_false"), Some(Expression::Boolean(false))));
                assert!(matches!(pairs.get("null_value"), Some(Expression::Null)));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_nested_objects() {
        let input = r#"
        {
            outer: {
                inner: {
                    deep: "value"
                },
                sibling: 123
            }
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert!(matches!(pairs.get("outer"), Some(Expression::ObjectExpression(_))));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod tagged_object_tests {
    use super::*;

    #[test]
    fn test_node_parsing() {
        let input = r#"
        {
            nodes: [
                Node { id: "alice", meta: { name: "Alice", age: 30 } },
                Node { id: "bob", meta: { type: "person" } }
            ],
            edges: []
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok(), "Failed to parse Node objects: {:?}", result.err());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("nodes") {
                    Some(Expression::ArrayExpression(elements)) => {
                        assert_eq!(elements.len(), 2);

                        // Check first Node
                        match &elements[0] {
                            Expression::TaggedObject { tag, fields } => {
                                assert_eq!(tag, "Node");
                                assert!(fields.contains_key("id"));
                                assert!(fields.contains_key("meta"));
                            }
                            _ => panic!("Expected TaggedObject for Node"),
                        }
                    }
                    _ => panic!("Expected ArrayExpression for nodes"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_edge_parsing() {
        let input = r#"
        {
            nodes: [],
            edges: [
                Edge { source: "a", target: "b", meta: { weight: 1.0 } },
                Edge { source: "b", target: "c", meta: { type: "connection" } }
            ]
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("edges") {
                    Some(Expression::ArrayExpression(elements)) => {
                        assert_eq!(elements.len(), 2);

                        // Check first Edge
                        match &elements[0] {
                            Expression::TaggedObject { tag, fields } => {
                                assert_eq!(tag, "Edge");
                                assert!(fields.contains_key("source"));
                                assert!(fields.contains_key("target"));
                                assert!(fields.contains_key("meta"));
                            }
                            _ => panic!("Expected TaggedObject for Edge"),
                        }
                    }
                    _ => panic!("Expected ArrayExpression for edges"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_invalid_node_missing_id() {
        let input = r#"
        {
            nodes: [Node { meta: { name: "Alice" } }],
            edges: []
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected parse error for Node without id");
    }

    #[test]
    fn test_invalid_edge_missing_source_target() {
        let input = r#"
        {
            nodes: [],
            edges: [Edge { meta: { weight: 1.0 } }]
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected parse error for Edge without source/target");
    }
}

#[cfg(test)]
mod functional_expression_tests {
    use super::*;

    #[test]
    fn test_builtin_calls() {
        let input = r#"
        {
            numbers: range("0..5"),
            pairs: combinations(["a", "b", "c"], 2)
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // Check range call
                match pairs.get("numbers") {
                    Some(Expression::BuiltinCall { name, args }) => {
                        assert_eq!(name, "range");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected BuiltinCall for range"),
                }

                // Check combinations call
                match pairs.get("pairs") {
                    Some(Expression::BuiltinCall { name, args }) => {
                        assert_eq!(name, "combinations");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected BuiltinCall for combinations"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_method_chaining() {
        let input = r#"
        {
            processed: range("0..5").map(x => x * 2).filter(x => x > 4)
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("processed") {
                    Some(Expression::ChainExpression { base, chain }) => {
                        // Base should be range call
                        assert!(matches!(**base, Expression::BuiltinCall { .. }));

                        // Should have two method calls in chain
                        assert_eq!(chain.len(), 2);
                    }
                    _ => panic!("Expected ChainExpression for method chaining"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_lambda_expressions() {
        let input = r#"
        {
            doubled: range("0..3").map(x => x * 2),
            filtered: [1, 2, 3, 4, 5].filter(n => n % 2 === 0)
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // Both should be chain expressions with lambda arguments
                assert!(matches!(pairs.get("doubled"), Some(Expression::ChainExpression { .. })));
                assert!(matches!(pairs.get("filtered"), Some(Expression::ChainExpression { .. })));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_template_literals() {
        let input = r#"
        {
            message: `Hello, world!`,
            dynamic: `Node ${5} has value ${10 * 2}`
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // Check simple template
                match pairs.get("message") {
                    Some(Expression::TemplateLiteral { parts }) => {
                        assert_eq!(parts.len(), 1);
                    }
                    _ => panic!("Expected TemplateLiteral"),
                }

                // Check dynamic template
                match pairs.get("dynamic") {
                    Some(Expression::TemplateLiteral { parts }) => {
                        assert!(parts.len() > 1); // Should have mixed parts
                    }
                    _ => panic!("Expected TemplateLiteral"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod arithmetic_tests {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        let input = r#"
        {
            addition: 5 + 3,
            subtraction: 10 - 4,
            multiplication: 6 * 7,
            division: 15 / 3,
            modulo: 17 % 5
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // All should be arithmetic expressions
                assert!(matches!(pairs.get("addition"), Some(Expression::ArithmeticExpression(_))));
                assert!(matches!(pairs.get("subtraction"), Some(Expression::ArithmeticExpression(_))));
                assert!(matches!(pairs.get("multiplication"), Some(Expression::ArithmeticExpression(_))));
                assert!(matches!(pairs.get("division"), Some(Expression::ArithmeticExpression(_))));
                assert!(matches!(pairs.get("modulo"), Some(Expression::ArithmeticExpression(_))));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_operator_precedence() {
        let input = r#"
        {
            complex: 2 + 3 * 4,
            parentheses: (2 + 3) * 4
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        // Verify that precedence is handled (detailed verification would require deeper AST inspection)
        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert!(matches!(pairs.get("complex"), Some(Expression::ArithmeticExpression(_))));
                assert!(matches!(pairs.get("parentheses"), Some(Expression::ArithmeticExpression(_))));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod conditional_tests {
    use super::*;

    #[test]
    fn test_if_expressions() {
        let input = r#"
        {
            simple_if: if (true) { "yes" } else { "no" },
            nested_if: if (5 > 3) {
                if (2 < 4) { "both true" } else { "first true" }
            } else {
                "first false"
            }
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert!(matches!(pairs.get("simple_if"), Some(Expression::IfExpression { .. })));
                assert!(matches!(pairs.get("nested_if"), Some(Expression::IfExpression { .. })));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_if_without_else() {
        let input = r#"
        {
            conditional: if (true) { "value" }
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("conditional") {
                    Some(Expression::IfExpression { condition: _, then_block: _, else_block }) => {
                        assert!(else_block.is_none());
                    }
                    _ => panic!("Expected IfExpression"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod block_expression_tests {
    use super::*;

    #[test]
    fn test_block_with_statements() {
        let input = r#"
        {
            result: (() => {
                let x = 5;
                let y = 10;
                return x + y;
            })
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("result") {
                    Some(Expression::LambdaExpression { params, body }) => {
                        // Should be lambda with empty params and block body
                        assert!(params.is_empty());
                        match **body {
                            Expression::BlockExpression { ref statements, .. } => {
                                assert!(statements.len() >= 2);
                            }
                            _ => panic!("Expected BlockExpression as lambda body"),
                        }
                    }
                    _ => panic!("Expected LambdaExpression"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_variable_declarations() {
        let input = r#"
        {
            computed: if (true) {
                let base = 10;
                let doubled = base * 2;
                return doubled + 5;
            }
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        // Verify variable declarations are parsed
        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                match pairs.get("computed") {
                    Some(Expression::IfExpression { condition: _, then_block, else_block: _ }) => {
                        // Check the then_block is a BlockExpression with variable declarations
                        match **then_block {
                            Expression::BlockExpression { ref statements, .. } => {
                                let has_var_decl = statements.iter().any(|stmt| {
                                    matches!(stmt, Expression::VariableDeclaration { .. })
                                });
                                assert!(has_var_decl, "Expected variable declarations in block");
                            }
                            _ => panic!("Expected BlockExpression in if then_block"),
                        }
                    }
                    _ => panic!("Expected IfExpression"),
                }
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod spread_expression_tests {
    use super::*;

    #[test]
    fn test_object_spread() {
        let input = r#"
        {
            base: { a: 1, b: 2 },
            extended: { ...base, c: 3 }
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // Verify spread expressions are parsed
                assert!(pairs.contains_key("base"));
                assert!(pairs.contains_key("extended"));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }

    #[test]
    fn test_array_spread() {
        let input = r#"
        {
            base: [1, 2, 3],
            extended: [...base, 4, 5, 6]
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                assert!(pairs.contains_key("base"));
                assert!(pairs.contains_key("extended"));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_syntax_errors() {
        let invalid_inputs = vec![
            "invalid syntax",
            "{ nodes: [Node { }] }", // Missing Node fields
            "{ invalid: function() }", // Invalid function syntax
            "{ broken: 1 + }", // Incomplete arithmetic
            "{ unclosed: [ }", // Unclosed array
            "{ bad_template: `unclosed template }", // Unclosed template
        ];

        for input in invalid_inputs {
            let result = parse_ggl(input);
            assert!(result.is_err(), "Expected parse error for: {input}");
        }
    }

    #[test]
    fn test_incomplete_expressions() {
        let input = r#"
        {
            incomplete: range("0..5").map(
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_identifiers() {
        let input = r#"
        {
            "quoted-key": "valid"
        }
        "#;

        let result = parse_ggl(input);
        // Should handle quoted keys correctly
        if result.is_ok() {
            let ast = result.unwrap();
            match ast.root {
                Expression::ObjectExpression(pairs) => {
                    // Should have the quoted key
                    assert!(pairs.contains_key("quoted-key"));
                }
                _ => panic!("Expected ObjectExpression"),
            }
        }
    }
}

#[cfg(test)]
mod integration_parsing_tests {
    use super::*;

    #[test]
    fn test_complete_graph_program() {
        let input = r#"
        {
            // Function definition
            add_edges: (graph) => ({
                ...graph,
                edges: graph.edges.concat([
                    Edge { source: "new1", target: "new2", meta: { added: true } }
                ])
            }),

            // Node generation
            nodes: range("0..5").map(i => Node {
                id: `node${i}`,
                meta: {
                    index: i,
                    even: i % 2 === 0,
                    label: `Node #${i}`
                }
            }),

            // Edge generation with transformations
            edges: combinations(range("0..5"), 2)
                .slice(0, 6)
                .map(([a, b]) => Edge {
                    source: `node${a}`,
                    target: `node${b}`,
                    meta: { weight: (a + b) * 0.1 }
                })
                .pipe(add_edges, 1)
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok(), "Failed to parse complete program: {:?}", result.err());

        // Also test that it can execute
        let mut engine = GGLEngine::new();
        let execution_result = engine.generate_from_ggl(input);
        assert!(execution_result.is_ok(), "Failed to execute parsed program: {:?}", execution_result.err());

        if let Ok(json_str) = execution_result {
            let graph: Value = serde_json::from_str(&json_str).unwrap();
            assert!(graph["nodes"].is_array());
            assert!(graph["edges"].is_array());
        }
    }

    #[test]
    fn test_realistic_social_network() {
        let input = r#"
        {
            friend_recommendation: (graph) => ({
                ...graph,
                edges: graph.edges.concat(
                    combinations(graph.nodes, 2)
                        .filter(([a, b]) => a.id !== b.id)
                        .slice(0, 3)
                        .map(([a, b]) => Edge {
                            source: a.id,
                            target: b.id,
                            meta: { type: "suggested", strength: 0.5 }
                        })
                )
            }),

            nodes: range("0..4").map(i => Node {
                id: `user${i}`,
                meta: {
                    age: 20 + i * 5,
                    active: true,
                    name: `User ${i}`
                }
            }),

            edges: [
                Edge { source: "user0", target: "user1", meta: { type: "friend" } },
                Edge { source: "user1", target: "user2", meta: { type: "friend" } }
            ].pipe(friend_recommendation, 1)
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        // Test execution
        let mut engine = GGLEngine::new();
        let execution_result = engine.generate_from_ggl(input);
        assert!(execution_result.is_ok());
    }

    #[test]
    fn test_mathematical_computations() {
        let input = r#"
        {
            nodes: range("1..6").map(n => Node {
                id: `calc${n}`,
                meta: {
                    input: n,
                    squared: n * n,
                    factorial: range("1..10").slice(0, n).reduce((acc, i) => acc * i, 1),
                    fibonacci: if (n <= 2) { 1 } else {
                        // Simplified fibonacci for parsing test
                        n + (n - 1)
                    }
                }
            }),
            edges: []
        }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match ast.root {
            Expression::ObjectExpression(pairs) => {
                // Verify complex nested structure is parsed
                assert!(pairs.contains_key("nodes"));
                assert!(pairs.contains_key("edges"));
            }
            _ => panic!("Expected ObjectExpression"),
        }
    }
}
