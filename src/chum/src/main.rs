// main.rs
//
// HOW TO USE THIS FILE:
// 1. Ensure your `Cargo.toml` uses `chumsky = "0.10.1"`.
// 2. This file replaces your old parser code. It is updated to be
//    compatible with the breaking changes in chumsky v0.10.x.

use chumsky::prelude::*;
use std::collections::HashMap;

// =================================================================
//   1. Abstract Syntax Tree (AST) Definition
// =================================================================
// These structs and enums define the structure of your language.
// The parser's only job is to turn source text into these Rust types.

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String), // The identifier inside {..}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    FormattedString(Vec<StringPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeOp {
    Directed,   // ->
    Undirected, // --
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributePair {
    pub key: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeDeclaration {
    pub id: Expression,
    pub label: Option<Expression>,
    pub attributes: Option<Vec<AttributePair>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDeclaration {
    pub label: Option<Expression>,
    pub from: Expression,
    pub op: EdgeOp,
    pub to: Expression,
    pub attributes: Option<Vec<AttributePair>>,
}

// A pattern is a subset of statements allowed inside rules.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternStatement {
    Node(NodeDeclaration),
    Edge(EdgeDeclaration),
}

// A statement is a single command or declaration in the language.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
    },
    ForLoop {
        iterator_name: String,
        from: Expression,
        to: Expression,
        body: Vec<Statement>,
    },
    Node(NodeDeclaration),
    Edge(EdgeDeclaration),
    Generate {
        generator_name: String,
        params: HashMap<String, Expression>,
    },
    Rule {
        name: String,
        lhs: Vec<PatternStatement>,
        rhs: Vec<PatternStatement>,
    },
    Apply {
        rule_name: String,
        times: Expression,
    },
}

// A File is the top-level AST node, representing the entire parsed source.
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub graph_name: String,
    pub statements: Vec<Statement>,
}

// =================================================================
//   2. Chumsky Parser Implementation (v0.10.x)
// =================================================================

/// Creates a parser for the entire graph language.
/// The signature now correctly specifies `&str` as the input type.
pub fn file_parser() -> impl Parser<&str, File, Error = Simple<char>> {
    let mut statement = recursive(|statement| {
        let ident = text::ident().padded().map(|s: &str| s.to_string());

        let int = text::int(10)
            .try_map(|s: &str, span| {
                s.parse::<i64>()
                    .map_err(|_| Simple::custom(span, "Invalid integer"))
            })
            .map(Literal::Integer);

        let float = text::int(10)
            .then_ignore(just('.'))
            .then(text::digits(10))
            .slice()
            .try_map(|s: &str, span| {
                s.parse::<f64>()
                    .map_err(|_| Simple::custom(span, "Invalid float"))
            })
            .map(Literal::Float);

        let boolean = choice((
            text::keyword("true").to(Literal::Boolean(true)),
            text::keyword("false").to(Literal::Boolean(false)),
        ));

        let string = just('"')
            .ignore_then(filter(|c| *c != '"').repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(Literal::String);

        let literal = float.or(int).or(boolean).or(string).padded();

        let var_in_string = just('{')
            .ignore_then(ident.clone())
            .then_ignore(just('}'))
            .map(StringPart::Variable);
        let string_part = filter(|c: &char| !matches!(*c, '"' | '{'))
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(StringPart::Literal);
        let formatted_string = just('"')
            .ignore_then(string_part.or(var_in_string).repeated().collect())
            .then_ignore(just('"'))
            .map(Expression::FormattedString)
            .padded();

        let expression = choice((
            literal.map(Expression::Literal),
            formatted_string,
            ident.clone().map(Expression::Identifier),
        ))
        .padded();

        let attribute_pair = ident
            .clone()
            .then_ignore(just('=').padded())
            .then(expression.clone())
            .map(|(key, value)| AttributePair { key, value });

        let attributes = attribute_pair
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect::<Vec<_>>()
            .delimited_by(just('[').padded(), just(']').padded());

        let let_decl = text::keyword("let")
            .ignore_then(ident.clone())
            .then_ignore(just('=').padded())
            .then(expression.clone())
            .then_ignore(just(';').padded())
            .map(|(name, value)| Statement::Let { name, value });

        let for_loop = text::keyword("for")
            .ignore_then(ident.clone())
            .then_ignore(text::keyword("in").padded())
            .then(expression.clone())
            .then_ignore(just("..").padded())
            .then(expression.clone())
            .then(
                statement
                    .clone()
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map(|(((iterator_name, from), to), body)| Statement::ForLoop {
                iterator_name,
                from,
                to,
                body,
            });

        let node_decl_inner = text::keyword("node")
            .ignore_then(expression.clone())
            .then(just(':').padded().ignore_then(expression.clone()).or_not())
            .then(attributes.clone().or_not())
            .then_ignore(just(';').padded())
            .map(|((id, label), attributes)| NodeDeclaration {
                id,
                label,
                attributes,
            });

        let edge_decl_inner = text::keyword("edge")
            .ignore_then(expression.clone().then_ignore(just(':').padded()).or_not())
            .then(expression.clone())
            .then(
                choice((
                    just("->").to(EdgeOp::Directed),
                    just("--").to(EdgeOp::Undirected),
                ))
                .padded(),
            )
            .then(expression.clone())
            .then(attributes.or_not())
            .then_ignore(just(';').padded())
            .map(|((((label, from), op), to), attributes)| EdgeDeclaration {
                label,
                from,
                op,
                to,
                attributes,
            });

        let generator_param = ident
            .clone()
            .then_ignore(just(':').padded())
            .then(expression.clone())
            .then_ignore(just(';').padded());

        let generate_stmt = text::keyword("generate")
            .ignore_then(ident.clone())
            .then(
                generator_param
                    .repeated()
                    .collect::<HashMap<String, Expression>>()
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map(|(generator_name, params)| Statement::Generate {
                generator_name,
                params,
            });

        let pattern_statement = node_decl_inner
            .clone()
            .map(PatternStatement::Node)
            .or(edge_decl_inner.clone().map(PatternStatement::Edge));

        let rule_def = text::keyword("rule")
            .ignore_then(ident.clone())
            .then(
                text::keyword("lhs")
                    .ignore_then(
                        pattern_statement
                            .clone()
                            .repeated()
                            .collect()
                            .delimited_by(just('{').padded(), just('}').padded()),
                    )
                    .then(
                        text::keyword("rhs").ignore_then(
                            pattern_statement
                                .repeated()
                                .collect()
                                .delimited_by(just('{').padded(), just('}').padded()),
                        ),
                    )
                    .delimited_by(just('{').padded(), just('}').padded()),
            )
            .map(|(name, (lhs, rhs))| Statement::Rule { name, lhs, rhs });

        let apply_stmt = text::keyword("apply")
            .ignore_then(ident)
            .then_ignore(text::keyword("times").padded())
            .then(expression)
            .then_ignore(just(';').padded())
            .map(|(rule_name, times)| Statement::Apply { rule_name, times });

        choice((
            let_decl,
            for_loop,
            node_decl_inner.map(Statement::Node),
            edge_decl_inner.map(Statement::Edge),
            generate_stmt,
            rule_def,
            apply_stmt,
        ))
    });

    let file = text::keyword("graph")
        .ignore_then(text::ident().padded().map(|s: &str| s.to_string()))
        .then(
            statement
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just('{').padded(), just('}').padded()),
        )
        .map(|(graph_name, statements)| File {
            graph_name,
            statements,
        });

    file.padded_by(comment().repeated()).then_ignore(end())
}

fn comment() -> impl Parser<&str, (), Error = Simple<char>> {
    just("//").then(take_until(just('\n'))).padded().ignored()
}

// =================================================================
//   3. Main Function (Example Usage)
// =================================================================
fn main() {
    let src = r#"
        // This is a graph definition file
        graph my_awesome_graph {
            let scale = 10;
            let is_directed = true;

            for i in 0..scale {
                node "node_{i}" [
                    color = "blue",
                    size = i
                ];
            }

            node "start_node";
            edge "start_node" -> "node_0"; // This is a directed edge
        }
    "#;

    println!("Attempting to parse source code...");
    let parser = file_parser();

    // In chumsky v0.10+, you parse a `&str` directly.
    match parser.parse(src).into_result() {
        Ok(ast) => {
            println!("\nSuccessfully parsed into AST!");
            println!("{:#?}", ast);
        }
        Err(errors) => {
            println!("\nFailed to parse with {} errors:", errors.len());
            for e in errors {
                // For rich error reporting, you can use the `ariadne` crate
                // along with the error spans provided by chumsky.
                println!("- {:?}", e);
            }
        }
    }
}
