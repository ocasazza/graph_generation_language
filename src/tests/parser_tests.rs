use graph_generation_language::parser::{parse_ggl, GGLStatement};
use graph_generation_language::types::MetadataValue;

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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 4);

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
            match &statements[i] {
                GGLStatement::NodeDecl(node) => assert_eq!(node.id, *expected_id),
                _ => panic!("Expected NodeDecl at position {}", i),
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 4);

        let expected_labels = [
            "simple string",
            "string with spaces",
            "string_with_underscores",
            "",
        ];
        for (i, expected_label) in expected_labels.iter().enumerate() {
            match &statements[i] {
                GGLStatement::NodeDecl(node) => match node.attributes.get("label") {
                    Some(MetadataValue::String(s)) => assert_eq!(s, expected_label),
                    _ => panic!("Expected string label at position {}", i),
                },
                _ => panic!("Expected NodeDecl at position {}", i),
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 6);

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
            match &statements[i] {
                GGLStatement::NodeDecl(node) => {
                    match (node.attributes.get("weight"), expected_weight) {
                        (Some(MetadataValue::Integer(n)), ExpectedWeight::Integer(expected)) => {
                            assert_eq!(*n, *expected);
                        }
                        (Some(MetadataValue::Float(n)), ExpectedWeight::Float(expected)) => {
                            assert!((n - expected).abs() < f64::EPSILON, "Expected {}, got {}", expected, n);
                        }
                        _ => panic!("Expected correct number type at position {}", i),
                    }
                }
                _ => panic!("Expected NodeDecl at position {}", i),
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => match node.attributes.get("active") {
                Some(MetadataValue::Boolean(true)) => (),
                _ => panic!("Expected true boolean"),
            },
            _ => panic!("Expected NodeDecl"),
        }

        match &statements[1] {
            GGLStatement::NodeDecl(node) => match node.attributes.get("active") {
                Some(MetadataValue::Boolean(false)) => (),
                _ => panic!("Expected false boolean"),
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 3);
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "simple");
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "person");
                assert_eq!(node.node_type, Some("human".to_string()));
            }
            _ => panic!("Expected NodeDecl"),
        }

        match &statements[1] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "building");
                assert_eq!(node.node_type, Some("structure".to_string()));
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "person");
                assert_eq!(node.attributes.len(), 3);

                assert_eq!(
                    node.attributes.get("name"),
                    Some(&MetadataValue::String("Alice".to_string()))
                );
                assert_eq!(
                    node.attributes.get("age"),
                    Some(&MetadataValue::Integer(30))
                );
                assert_eq!(
                    node.attributes.get("active"),
                    Some(&MetadataValue::Boolean(true))
                );
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "alice");
                assert_eq!(node.node_type, Some("person".to_string()));
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::NodeDecl(node) => {
                assert_eq!(node.id, "empty");
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::EdgeDecl(edge) => {
                assert_eq!(edge.source, "source");
                assert_eq!(edge.target, "target");
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::EdgeDecl(edge) => {
                assert_eq!(edge.source, "source");
                assert_eq!(edge.target, "target");
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::EdgeDecl(edge) => {
                assert_eq!(edge.attributes.len(), 2);
                assert_eq!(
                    edge.attributes.get("weight"),
                    Some(&MetadataValue::Float(1.5))
                );
                assert_eq!(
                    edge.attributes.get("label"),
                    Some(&MetadataValue::String("connection".to_string()))
                );
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::EdgeDecl(edge) => {
                assert_eq!(edge.source, "a");
                assert_eq!(edge.target, "b");
                // ID should be auto-generated
                assert!(edge.id.contains("a"));
                assert!(edge.id.contains("b"));
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::GenerateStmt(gen) => {
                assert_eq!(gen.name, "complete");
                assert_eq!(gen.params.len(), 1);
                assert_eq!(gen.params.get("nodes"), Some(&MetadataValue::Integer(5)));
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::GenerateStmt(gen) => {
                assert_eq!(gen.name, "grid");
                assert_eq!(gen.params.len(), 4);
                assert_eq!(gen.params.get("rows"), Some(&MetadataValue::Integer(3)));
                assert_eq!(gen.params.get("cols"), Some(&MetadataValue::Integer(4)));
                assert_eq!(
                    gen.params.get("prefix"),
                    Some(&MetadataValue::String("node".to_string()))
                );
                assert_eq!(
                    gen.params.get("periodic"),
                    Some(&MetadataValue::Boolean(true))
                );
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::GenerateStmt(gen) => {
                assert_eq!(gen.name, "empty");
                assert!(gen.params.is_empty());
            }
            _ => panic!("Expected GenerateStmt"),
        }
    }
}

#[cfg(test)]
mod rule_definition_tests {
    use super::*;

    #[test]
    fn test_simple_rule() {
        let input = r#"
            graph test {
                rule add_leaf {
                    lhs { node N; }
                    rhs {
                        node N;
                        node L;
                        edge: N -> L;
                    }
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::RuleDefStmt(rule) => {
                assert_eq!(rule.name, "add_leaf");
                assert_eq!(rule.lhs.nodes.len(), 1);
                assert_eq!(rule.lhs.edges.len(), 0);
                assert_eq!(rule.rhs.nodes.len(), 2);
                assert_eq!(rule.rhs.edges.len(), 1);

                assert_eq!(rule.lhs.nodes[0].id, "N");
                assert_eq!(rule.rhs.nodes[0].id, "N");
                assert_eq!(rule.rhs.nodes[1].id, "L");
                assert_eq!(rule.rhs.edges[0].source, "N");
                assert_eq!(rule.rhs.edges[0].target, "L");
            }
            _ => panic!("Expected RuleDefStmt"),
        }
    }

    #[test]
    fn test_rule_with_typed_nodes() {
        let input = r#"
            graph test {
                rule transform {
                    lhs {
                        node A :type1;
                        node B :type2;
                        edge: A -> B;
                    }
                    rhs {
                        node A :type1;
                        node B :type3;
                        node C :type2;
                        edge: A -> C;
                        edge: C -> B;
                    }
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::RuleDefStmt(rule) => {
                assert_eq!(rule.name, "transform");
                assert_eq!(rule.lhs.nodes.len(), 2);
                assert_eq!(rule.lhs.edges.len(), 1);
                assert_eq!(rule.rhs.nodes.len(), 3);
                assert_eq!(rule.rhs.edges.len(), 2);

                // Check types
                assert_eq!(rule.lhs.nodes[0].node_type, Some("type1".to_string()));
                assert_eq!(rule.lhs.nodes[1].node_type, Some("type2".to_string()));
                assert_eq!(rule.rhs.nodes[0].node_type, Some("type1".to_string()));
                assert_eq!(rule.rhs.nodes[1].node_type, Some("type3".to_string()));
                assert_eq!(rule.rhs.nodes[2].node_type, Some("type2".to_string()));
            }
            _ => panic!("Expected RuleDefStmt"),
        }
    }

    #[test]
    fn test_rule_with_attributes() {
        let input = r#"
            graph test {
                rule attr_rule {
                    lhs {
                        node N [status="old"];
                    }
                    rhs {
                        node N [status="new", updated=true];
                    }
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::RuleDefStmt(rule) => {
                assert_eq!(rule.lhs.nodes[0].attributes.len(), 1);
                assert_eq!(rule.rhs.nodes[0].attributes.len(), 2);

                assert_eq!(
                    rule.lhs.nodes[0].attributes.get("status"),
                    Some(&MetadataValue::String("old".to_string()))
                );
                assert_eq!(
                    rule.rhs.nodes[0].attributes.get("status"),
                    Some(&MetadataValue::String("new".to_string()))
                );
                assert_eq!(
                    rule.rhs.nodes[0].attributes.get("updated"),
                    Some(&MetadataValue::Boolean(true))
                );
            }
            _ => panic!("Expected RuleDefStmt"),
        }
    }
}

#[cfg(test)]
mod rule_application_tests {
    use super::*;

    #[test]
    fn test_apply_rule() {
        let input = r#"
            graph test {
                apply my_rule 5 times;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::ApplyRuleStmt(apply) => {
                assert_eq!(apply.rule_name, "my_rule");
                assert_eq!(apply.iterations, 5);
            }
            _ => panic!("Expected ApplyRuleStmt"),
        }
    }

    #[test]
    fn test_apply_rule_once() {
        let input = r#"
            graph test {
                apply single_rule 1 times;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::ApplyRuleStmt(apply) => {
                assert_eq!(apply.rule_name, "single_rule");
                assert_eq!(apply.iterations, 1);
            }
            _ => panic!("Expected ApplyRuleStmt"),
        }
    }

    #[test]
    fn test_apply_rule_zero_times() {
        let input = r#"
            graph test {
                apply no_rule 0 times;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            GGLStatement::ApplyRuleStmt(apply) => {
                assert_eq!(apply.rule_name, "no_rule");
                assert_eq!(apply.iterations, 0);
            }
            _ => panic!("Expected ApplyRuleStmt"),
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

                // Rule definition
                rule add_friend {
                    lhs {
                        node A :person;
                        node B :person;
                    }
                    rhs {
                        node A :person;
                        node B :person;
                        edge: A -- B [type="friendship"];
                    }
                }

                // Rule application
                apply add_friend 3 times;
            }
        "#;

        let result = parse_ggl(input);
        assert!(
            result.is_ok(),
            "Failed to parse complex program: {:?}",
            result.err()
        );

        let statements = result.unwrap();
        assert_eq!(statements.len(), 6);

        // Verify statement types in order
        assert!(matches!(statements[0], GGLStatement::NodeDecl(_)));
        assert!(matches!(statements[1], GGLStatement::NodeDecl(_)));
        assert!(matches!(statements[2], GGLStatement::EdgeDecl(_)));
        assert!(matches!(statements[3], GGLStatement::GenerateStmt(_)));
        assert!(matches!(statements[4], GGLStatement::RuleDefStmt(_)));
        assert!(matches!(statements[5], GGLStatement::ApplyRuleStmt(_)));
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

                rule add_metadata {
                    lhs { node N; }
                    rhs { node N [processed=true]; }
                }

                apply add_metadata 10 times;
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 4);
    }

    #[test]
    fn test_empty_graph() {
        let input = r#"
            graph empty {
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 0);
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
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

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
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
            assert!(result.is_err(), "Expected error for input: {}", input);
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
