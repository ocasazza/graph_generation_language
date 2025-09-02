use graph_generation_language::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expressions() {
        // Test integer literals
        assert_eq!(
            parse_expression("42"),
            Ok(Expr::Literal(Literal::Int(42)))
        );

        // Test float literals
        assert_eq!(
            parse_expression("3.14"),
            Ok(Expr::Literal(Literal::Float(3.14)))
        );

        // Test string literals
        assert_eq!(
            parse_expression("\"hello\""),
            Ok(Expr::Literal(Literal::String("hello".to_string())))
        );

        // Test boolean literals
        assert_eq!(
            parse_expression("true"),
            Ok(Expr::Literal(Literal::Bool(true)))
        );
        assert_eq!(
            parse_expression("false"),
            Ok(Expr::Literal(Literal::Bool(false)))
        );
    }

    #[test]
    fn test_parse_identifiers() {
        assert_eq!(
            parse_expression("myVar"),
            Ok(Expr::Identifier("myVar".to_string()))
        );
    }

    #[test]
    fn test_parse_binary_expressions() {
        // Test basic arithmetic
        assert_eq!(
            parse_expression("1 + 2"),
            Ok(Expr::BinaryOp {
                op: Operator::Add,
                left: Box::new(Expr::Literal(Literal::Int(1))),
                right: Box::new(Expr::Literal(Literal::Int(2)))
            })
        );

        // Test operator precedence: 1 + 2 * 3 => 1 + (2 * 3)
        assert_eq!(
            parse_expression("1 + 2 * 3"),
            Ok(Expr::BinaryOp {
                op: Operator::Add,
                left: Box::new(Expr::Literal(Literal::Int(1))),
                right: Box::new(Expr::BinaryOp {
                    op: Operator::Multiply,
                    left: Box::new(Expr::Literal(Literal::Int(2))),
                    right: Box::new(Expr::Literal(Literal::Int(3)))
                })
            })
        );
    }

    #[test]
    fn test_parse_unary_expressions() {
        assert_eq!(
            parse_expression("-42"),
            Ok(Expr::UnaryOp {
                op: Operator::UnaryMinus,
                expr: Box::new(Expr::Literal(Literal::Int(42)))
            })
        );

        assert_eq!(
            parse_expression("!false"),
            Ok(Expr::UnaryOp {
                op: Operator::Not,
                expr: Box::new(Expr::Literal(Literal::Bool(false)))
            })
        );
    }

    #[test]
    fn test_parse_parenthesized_expressions() {
        // Test that parentheses override precedence: (1 + 2) * 3
        assert_eq!(
            parse_expression("(1 + 2) * 3"),
            Ok(Expr::BinaryOp {
                op: Operator::Multiply,
                left: Box::new(Expr::BinaryOp {
                    op: Operator::Add,
                    left: Box::new(Expr::Literal(Literal::Int(1))),
                    right: Box::new(Expr::Literal(Literal::Int(2)))
                }),
                right: Box::new(Expr::Literal(Literal::Int(3)))
            })
        );
    }

    #[test]
    fn test_parse_assignment_statement() {
        assert_eq!(
            parse_statement("x : 42;"),
            Ok(Stmt::Assignment("x".to_string(), Expr::Literal(Literal::Int(42))))
        );
    }

    #[test]
    fn test_parse_expression_statement() {
        assert_eq!(
            parse_statement("1 + 2;"),
            Ok(Stmt::Expr(Expr::BinaryOp {
                op: Operator::Add,
                left: Box::new(Expr::Literal(Literal::Int(1))),
                right: Box::new(Expr::Literal(Literal::Int(2)))
            }))
        );
    }

    #[test]
    fn test_parse_simple_program() {
        let code = "x : 42; y : x + 1;";
        let result = parse_program(code);
        assert!(result.is_ok(), "Failed to parse simple program: {:?}", result);

        let program = result.unwrap();
        assert_eq!(program.statements.len(), 2);

        // Check first statement
        match &program.statements[0] {
            Stmt::Assignment(var, expr) => {
                assert_eq!(var, "x");
                assert_eq!(*expr, Expr::Literal(Literal::Int(42)));
            }
            _ => panic!("Expected assignment statement")
        }
    }

    #[test]
    fn test_parse_function_definition() {
        let code = "add : x y -> x + y;";
        let result = parse_statement(code);
        assert!(result.is_ok(), "Failed to parse function definition: {:?}", result);

        match result.unwrap() {
            Stmt::Function(name, params, return_type, body, with_locals, with_rec_fns) => {
                assert_eq!(name, "add");
                assert_eq!(params, vec![("x".to_string(), None), ("y".to_string(), None)]);
                assert_eq!(return_type, None);
                assert_eq!(with_locals, None);
                assert_eq!(with_rec_fns, None);
                // Body should be x + y
                match body {
                    Expr::BinaryOp { op: Operator::Add, left, right } => {
                        assert_eq!(*left.as_ref(), Expr::Identifier("x".to_string()));
                        assert_eq!(*right.as_ref(), Expr::Identifier("y".to_string()));
                    }
                    _ => panic!("Expected binary addition expression")
                }
            }
            _ => panic!("Expected function statement")
        }
    }

    #[test]
    fn test_parse_list_literal() {
        assert_eq!(
            parse_expression("[1, 2, 3]"),
            Ok(Expr::Literal(Literal::List(vec![
                Expr::Literal(Literal::Int(1)),
                Expr::Literal(Literal::Int(2)),
                Expr::Literal(Literal::Int(3))
            ])))
        );
    }

    #[test]
    fn test_parse_table_literal() {
        assert_eq!(
            parse_expression("{a: 1, b: \"two\"}"),
            Ok(Expr::Literal(Literal::Table(vec![
                ("a".to_string(), Expr::Literal(Literal::Int(1))),
                ("b".to_string(), Expr::Literal(Literal::String("two".to_string())))
            ])))
        );
    }

    #[test]
    fn test_parse_complex_program() {
        let code = r#"
            add : x y -> x + y;
            x : 42;
            result : x + 1;
        "#;
        let result = parse_program(code);
        assert!(result.is_ok(), "Failed to parse complex program: {:?}", result);

        let program = result.unwrap();
        assert_eq!(program.statements.len(), 3);
    }

    #[test]
    fn test_parse_graph_declaration() {
        let code = r#"
            graph {
                nodes : [{id: "a", value: 1}],
                edges : [{from: "a", to: "b"}]
            };
        "#;
        let result = parse_statement(code);
        assert!(result.is_ok(), "Failed to parse graph declaration: {:?}", result);

        match result.unwrap() {
            Stmt::GraphDeclaration { nodes, edges } => {
                assert_eq!(nodes.len(), 1);
                assert_eq!(edges.len(), 1);
            }
            _ => panic!("Expected graph declaration statement")
        }
    }

    #[test]
    fn test_parse_errors() {
        // Test that invalid syntax produces errors
        assert!(parse_expression("1 +").is_err());
        assert!(parse_expression("(1 + 2").is_err());
        assert!(parse_statement("x :").is_err());
        assert!(parse_program("invalid syntax here").is_err());
    }
}
