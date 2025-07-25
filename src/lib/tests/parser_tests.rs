use graph_generation_language::parser::{parse_ggl, Expression, Statement};

#[cfg(test)]
mod lexical_tests {
    use super::*;

    #[test]
    fn test_identifiers() {
        let input = r#"
            graph test {
                node simple_id;
                node CamelCase;
                node with_123_numbers;
                node _underscore_start;
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse valid identifiers: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 4);

        // Verify all are node declarations with correct IDs
        for (i, expected_id) in [
            "simple_id",
            "CamelCase",
            "with_123_numbers",
            "_underscore_start",
        ]
        .iter()
        .enumerate()
        {
            match &ast.statements[i] {
                Statement::Node(node) => {
                    match &node.id {
                        Expression::Identifier(s) => assert_eq!(s, *expected_id),
                        _ => panic!("Expected identifier"),
                    }
                }
                _ => panic!("Expected NodeDecl at position {i}"),
            }
        }
    }

    #[test]
    fn test_strings() {
        let input = r#"
            graph test {
                node n1 [label="simple string"];
                node n2 [label="string with spaces"];
                node n3 [label="string_with_underscores"];
                node n4 [label=""];
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse strings: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 4);

        let expected_labels = [
            "simple string",
            "string with spaces",
            "string_with_underscores",
            "",
        ];
        for (i, expected_label) in expected_labels.iter().enumerate() {
            match &ast.statements[i] {
                Statement::Node(node) => {
                    // Find the label attribute in the Vec<(String, Expression)>
                    let label_attr = node.attributes.iter().find(|(key, _)| key == "label");
                    match label_attr {
                        Some((_, Expression::StringLiteral(s))) => {
                            assert_eq!(s, expected_label)
                        }
                        _ => panic!("Expected string label at position {i}"),
                    }
                }
                _ => panic!("Expected NodeDecl at position {i}"),
            }
        }
    }

    #[test]
    fn test_numbers() {
        let input = r#"
            graph test {
                node n1 [weight=42];
                node n2 [weight=-17];
                node n3 [weight=3.75];
                node n4 [weight=-2.5];
                node n5 [weight=0];
                node n6 [weight=0.0];
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse numbers: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 6);

        #[derive(Debug)]
        enum ExpectedWeight {
            Integer(i64),
            Float(f64),
        }

        let expected_weights = [
            ExpectedWeight::Integer(42),
            ExpectedWeight::Integer(-17),
            ExpectedWeight::Float(3.75),
            ExpectedWeight::Float(-2.5),
            ExpectedWeight::Integer(0),
            ExpectedWeight::Float(0.0),
        ];

        for (i, expected_weight) in expected_weights.iter().enumerate() {
            match &ast.statements[i] {
                Statement::Node(node) => {
                    let weight_attr = node.attributes.iter().find(|(key, _)| key == "weight");
                    match (weight_attr, expected_weight) {
                        (
                            Some((_, Expression::Integer(n))),
                            ExpectedWeight::Integer(expected),
                        ) => {
                            assert_eq!(*n, *expected);
                        }
                        (
                            Some((_, Expression::Float(n))),
                            ExpectedWeight::Float(expected),
                        ) => {
                            assert!((n - expected).abs() < f64::EPSILON, "Expected {expected}, got {n}");
                        }
                        _ => panic!("Expected correct number type at position {i}"),
                    }
                }
                _ => panic!("Expected NodeDecl at position {i}"),
            }
        }
    }

    #[test]
    fn test_booleans() {
        let input = r#"
            graph test {
                node n1 [active=true];
                node n2 [active=false];
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse booleans: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 2);

        match &ast.statements[0] {
            Statement::Node(node) => {
                let active_attr = node.attributes.iter().find(|(key, _)| key == "active");
                match active_attr {
                    Some((_, Expression::Boolean(true))) => (),
                    _ => panic!("Expected true boolean"),
                }
            },
            _ => panic!("Expected NodeDecl"),
        }

        match &ast.statements[1] {
            Statement::Node(node) => {
                let active_attr = node.attributes.iter().find(|(key, _)| key == "active");
                match active_attr {
                    Some((_, Expression::Boolean(false))) => (),
                    _ => panic!("Expected false boolean"),
                }
            },
            _ => panic!("Expected NodeDecl"),
        }
    }

    #[test]
    fn test_comments() {
        let input = r#"
            // This is a line comment
            graph test {
                node n1; // End of line comment
                /* Block comment */
                node n2;
                /*
                 * Multi-line
                 * block comment
                 */
                node n3;
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse with comments: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 3);
    }
}

#[cfg(test)]
mod node_declaration_tests {
    use super::*;

    #[test]
    fn test_simple_node() {
        let input = r#"
            graph test {
                node simple;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "simple"),
                    _ => panic!("Expected identifier"),
                }
                assert!(node.node_type.is_none());
                assert!(node.attributes.is_empty());
            }
            _ => panic!("Expected NodeDecl"),
        }
    }

    #[test]
    fn test_typed_node() {
        let input = r#"
            graph test {
                node person :human;
                node building :structure;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 2);

        match &ast.statements[0] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "person"),
                    _ => panic!("Expected identifier"),
                }
                match &node.node_type {
                    Some(Expression::Identifier(s)) => assert_eq!(s, "human"),
                    _ => panic!("Expected node type"),
                }
            }
            _ => panic!("Expected NodeDecl"),
        }

        match &ast.statements[1] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "building"),
                    _ => panic!("Expected identifier"),
                }
                match &node.node_type {
                    Some(Expression::Identifier(s)) => assert_eq!(s, "structure"),
                    _ => panic!("Expected node type"),
                }
            }
            _ => panic!("Expected NodeDecl"),
        }
    }

    #[test]
    fn test_node_with_attributes() {
        let input = r#"
            graph test {
                node person [name="Alice", age=30, active=true];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "person"),
                    _ => panic!("Expected identifier"),
                }
                assert_eq!(node.attributes.len(), 3);

                let name_attr = node.attributes.iter().find(|(key, _)| key == "name");
                match name_attr {
                    Some((_, Expression::StringLiteral(s))) => {
                        assert_eq!(s, "Alice")
                    }
                    _ => panic!("Expected name attribute"),
                }

                let age_attr = node.attributes.iter().find(|(key, _)| key == "age");
                match age_attr {
                    Some((_, Expression::Integer(n))) => assert_eq!(*n, 30),
                    _ => panic!("Expected age attribute"),
                }

                let active_attr = node.attributes.iter().find(|(key, _)| key == "active");
                match active_attr {
                    Some((_, Expression::Boolean(b))) => assert!(*b),
                    _ => panic!("Expected active attribute"),
                }
            }
            _ => panic!("Expected NodeDecl"),
        }
    }

    #[test]
    fn test_node_with_type_and_attributes() {
        let input = r#"
            graph test {
                node alice :person [name="Alice", age=30];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "alice"),
                    _ => panic!("Expected identifier"),
                }
                match &node.node_type {
                    Some(Expression::Identifier(s)) => assert_eq!(s, "person"),
                    _ => panic!("Expected node type"),
                }
                assert_eq!(node.attributes.len(), 2);
            }
            _ => panic!("Expected NodeDecl"),
        }
    }

    #[test]
    fn test_empty_attributes() {
        let input = r#"
            graph test {
                node empty [];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Node(node) => {
                match &node.id {
                    Expression::Identifier(s) => assert_eq!(s, "empty"),
                    _ => panic!("Expected identifier"),
                }
                assert!(node.attributes.is_empty());
            }
            _ => panic!("Expected NodeDecl"),
        }
    }
}

#[cfg(test)]
mod edge_declaration_tests {
    use super::*;

    #[test]
    fn test_directed_edge() {
        let input = r#"
            graph test {
                edge e1: source -> target;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Edge(edge) => {
                assert!(edge.directed);
                assert!(edge.attributes.is_empty());
            }
            _ => panic!("Expected EdgeDecl"),
        }
    }

    #[test]
    fn test_undirected_edge() {
        let input = r#"
            graph test {
                edge e1: source -- target;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Edge(edge) => {
                assert!(!edge.directed);
            }
            _ => panic!("Expected EdgeDecl"),
        }
    }

    #[test]
    fn test_edge_with_attributes() {
        let input = r#"
            graph test {
                edge e1: a -> b [weight=1.5, label="connection"];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Edge(edge) => {
                assert_eq!(edge.attributes.len(), 2);
                let weight_attr = edge.attributes.iter().find(|(key, _)| key == "weight");
                match weight_attr {
                    Some((_, Expression::Float(n))) => {
                        assert!((n - 1.5).abs() < f64::EPSILON)
                    }
                    _ => panic!("Expected weight attribute"),
                }
                let label_attr = edge.attributes.iter().find(|(key, _)| key == "label");
                match label_attr {
                    Some((_, Expression::StringLiteral(s))) => {
                        assert_eq!(s, "connection")
                    }
                    _ => panic!("Expected label attribute"),
                }
            }
            _ => panic!("Expected EdgeDecl"),
        }
    }

    #[test]
    fn test_edge_without_explicit_id() {
        let input = r#"
            graph test {
                edge: a -> b;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Edge(edge) => {
                assert!(edge.id.is_none());
            }
            _ => panic!("Expected EdgeDecl"),
        }
    }
}

#[cfg(test)]
mod generator_statement_tests {
    use super::*;

    #[test]
    fn test_simple_generator() {
        let input = r#"
            graph test {
                generate complete {
                    nodes: 5;
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Generate(gen) => {
                assert_eq!(gen.name, "complete");
                assert_eq!(gen.params.len(), 1);
                let nodes_param = gen.params.iter().find(|(key, _)| key == "nodes");
                match nodes_param {
                    Some((_, Expression::Integer(5))) => (),
                    _ => panic!("Expected nodes parameter"),
                }
            }
            _ => panic!("Expected GenerateStmt"),
        }
    }

    #[test]
    fn test_generator_with_multiple_params() {
        let input = r#"
            graph test {
                generate grid {
                    rows: 3;
                    cols: 4;
                    prefix: "node";
                    periodic: true;
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Generate(generator) => {
                assert_eq!(generator.name, "grid");
                assert_eq!(generator.params.len(), 4);

                let rows_param = generator.params.iter().find(|(key, _)| key == "rows");
                match rows_param {
                    Some((_, Expression::Integer(3))) => (),
                    _ => panic!("Expected rows parameter"),
                }

                let cols_param = generator.params.iter().find(|(key, _)| key == "cols");
                match cols_param {
                    Some((_, Expression::Integer(4))) => (),
                    _ => panic!("Expected cols parameter"),
                }

                let prefix_param = generator.params.iter().find(|(key, _)| key == "prefix");
                match prefix_param {
                    Some((_, Expression::StringLiteral(s))) => assert_eq!(s, "node"),
                    _ => panic!("Expected prefix parameter"),
                }

                let periodic_param = generator.params.iter().find(|(key, _)| key == "periodic");
                match periodic_param {
                    Some((_, Expression::Boolean(true))) => (),
                    _ => panic!("Expected periodic parameter"),
                }
            }
            _ => panic!("Expected GenerateStmt"),
        }
    }

    #[test]
    fn test_generator_with_no_params() {
        let input = r#"
            graph test {
                generate empty {
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::Generate(gen) => {
                assert_eq!(gen.name, "empty");
                assert!(gen.params.is_empty());
            }
            _ => panic!("Expected GenerateStmt"),
        }
    }
}

#[cfg(test)]
mod complex_program_tests {
    use super::*;

    #[test]
    fn test_mixed_statements() {
        let input = r#"
            graph social_network {
                // Manual nodes
                node alice :person [name="Alice", age=30];
                node bob :person [name="Bob", age=25];

                // Manual edge
                edge friendship: alice -- bob [strength=0.8];

                // Generated structure
                generate complete {
                    nodes: 5;
                    prefix: "user";
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex program: {:?}",
            result.err()
        );

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 4);

        // Verify statement types in order
        assert!(matches!(ast.statements[0], Statement::Node(_)));
        assert!(matches!(ast.statements[1], Statement::Node(_)));
        assert!(matches!(ast.statements[2], Statement::Edge(_)));
        assert!(matches!(ast.statements[3], Statement::Generate(_)));
    }

    #[test]
    fn test_nested_graph_structure() {
        let input = r#"
            graph hierarchical {
                node root :directory [name="root"];

                generate tree {
                    depth: 3;
                    branching: 2;
                    prefix: "node";
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_empty_graph() {
        let input = r#"
            graph empty {
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 0);
    }

    #[test]
    fn test_graph_with_name() {
        let input = r#"
            graph my_graph_name {
                node test;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_graph_without_name() {
        let input = r#"
            graph {
                node test;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_syntax() {
        let invalid_inputs = vec![
            "invalid syntax",
            "graph { node }",              // Missing node ID
            "graph { edge: -> }",          // Missing source/target
            "graph { generate }",          // Missing generator name
            "graph { rule }",              // Missing rule name
            "graph { apply }",             // Missing rule name
            "graph { node n [invalid=] }", // Missing attribute value
            "graph { node n [=value] }",   // Missing attribute key
        ];

        for input in invalid_inputs {
            let result = parse_ggl(input);
            assert!(result.is_err(), "Expected error for input: {input}");
        }
    }

    #[test]
    fn test_missing_semicolons() {
        let input = r#"
            graph test {
                node a
                node b
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected error for missing semicolons");
    }

    #[test]
    fn test_invalid_numbers() {
        let input = r#"
            graph test {
                node n [value=12.34.56];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected error for invalid number");
    }

    #[test]
    fn test_unclosed_strings() {
        let input = r#"
            graph test {
                node n [label="unclosed string];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected error for unclosed string");
    }

    #[test]
    fn test_unclosed_comments() {
        let input = r#"
            graph test {
                node n; /* unclosed comment
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_err(), "Expected error for unclosed comment");
    }
}
