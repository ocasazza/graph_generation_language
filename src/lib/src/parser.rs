//! GGL language parser and Abstract Syntax Tree (AST) definitions.
//! This module uses the `pest` library to parse GGL source code into a structured AST.

use pest::iterators::Pair;
use pest::Parser as PestParser;
use pest_derive::Parser;
use std::fmt;

/// Type alias for boxed pest error to reduce Result size
type ParseError = Box<pest::error::Error<Rule>>;

#[derive(Parser)]
#[grammar = "ggl.pest"]
pub struct GglParser;

// --- Abstract Syntax Tree (AST) ---

#[derive(Debug)]
pub struct GraphAST {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStatement),
    For(ForStatement),
    If(IfStatement),
    Node(NodeDeclaration),
    Edge(EdgeDeclaration),
    Generate(GenerateStatement),
    RuleDef(RuleDefinition),
    Apply(ApplyStatement),
}

/// Represents a `let` statement for variable assignment.
///
/// # Easy Example
///
/// ```ggl
/// let count = 10;
/// ```
#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub value: Expression,
}

/// Represents a `for` loop for iterative graph construction.
///
/// # Medium Example
///
/// ```ggl
/// let node_count = 5;
/// for i in 0..node_count {
///     node "node_{i}";
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ForStatement {
    pub variable: String,
    pub start: Expression,
    pub end: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: ConditionalExpression,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ConditionalExpression {
    pub left: ArithmeticExpression,
    pub operator: ComparisonOperator,
    pub right: ArithmeticExpression,
}

#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone)]
pub enum ArithmeticExpression {
    Term(Expression),
    Add(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
    Subtract(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
    Multiply(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
    Divide(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
    Modulo(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
}

#[derive(Debug, Clone)]
pub struct NodeDeclaration {
    pub id: Expression,
    pub node_type: Option<Expression>,
    pub attributes: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub struct EdgeDeclaration {
    pub id: Option<Expression>,
    pub source: Expression,
    pub target: Expression,
    pub directed: bool,
    pub attributes: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub struct GenerateStatement {
    pub name: String,
    pub params: Vec<(String, Expression)>,
}

#[derive(Debug, Clone)]
pub struct RuleDefinition {
    pub name: String,
    pub lhs: Pattern,
    pub rhs: Pattern,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub nodes: Vec<NodeDeclaration>,
    pub edges: Vec<EdgeDeclaration>,
}

#[derive(Debug, Clone)]
pub struct ApplyStatement {
    pub rule_name: String,
    pub iterations: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    StringLiteral(String),
    FormattedString(Vec<StringPart>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Identifier(String),
}

/// Implements the Display trait to allow Expressions to be converted to strings.
/// This is crucial for resolving identifiers and literals in rules and statements.
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::StringLiteral(s) => write!(f, "{s}"),
            Expression::Integer(i) => write!(f, "{i}"),
            Expression::Float(n) => write!(f, "{n}"),
            Expression::Boolean(b) => write!(f, "{b}"),
            Expression::Identifier(name) => write!(f, "{name}"),
            Expression::FormattedString(parts) => {
                // This formatting is for pattern matching in rules, where variables
                // are not yet resolved.
                for part in parts {
                    match part {
                        StringPart::Literal(s) => write!(f, "{s}")?,
                        StringPart::Variable(v) => write!(f, "{{{v}}}")?,
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
}

// --- Parser Implementation ---

/// Parses a GGL source string into a Graph AST.
pub fn parse_ggl(source: &str) -> Result<GraphAST, ParseError> {
    let file_pair = GglParser::parse(Rule::file, source).map_err(Box::new)?.next().unwrap();
    build_ast_from_file(file_pair)
}

fn build_ast_from_file(pair: Pair<Rule>) -> Result<GraphAST, ParseError> {
    let mut inner = pair.into_inner();

    // Check if the first item is an identifier (graph name) or a statement
    let first = inner.next().unwrap();
    let (name, statements) = if first.as_rule() == Rule::identifier {
        // Graph has a name
        let name = first.as_str().to_string();
        let statements = inner
            .filter(|p| p.as_rule() != Rule::EOI)
            .map(build_statement)
            .collect::<Result<_, _>>()?;
        (name, statements)
    } else if first.as_rule() == Rule::EOI {
        // Empty graph with no name
        let name = "unnamed".to_string();
        (name, vec![])
    } else {
        // Graph has no name, first item is a statement
        let name = "unnamed".to_string();
        let mut statements = vec![build_statement(first)?];
        statements.extend(
            inner
                .filter(|p| p.as_rule() != Rule::EOI)
                .map(build_statement)
                .collect::<Result<Vec<_>, _>>()?
        );
        (name, statements)
    };

    Ok(GraphAST { name, statements })
}

fn build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        Rule::let_declaration => build_let_statement(pair).map(Statement::Let),
        Rule::for_loop => build_for_loop(pair).map(Statement::For),
        Rule::if_statement => build_if_statement(pair).map(Statement::If),
        Rule::node_declaration => build_node_declaration(pair).map(Statement::Node),
        Rule::edge_declaration => build_edge_declaration(pair).map(Statement::Edge),
        Rule::generate_statement => build_generate_statement(pair).map(Statement::Generate),
        Rule::rule_definition => build_rule_definition(pair).map(Statement::RuleDef),
        Rule::apply_statement => build_apply_statement(pair).map(Statement::Apply),
        _ => unreachable!("Unexpected statement rule: {:?}", pair.as_rule()),
    }
}

fn build_let_statement(pair: Pair<Rule>) -> Result<LetStatement, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = build_expression(inner.next().unwrap())?;
    Ok(LetStatement { name, value })
}

fn build_for_loop(pair: Pair<Rule>) -> Result<ForStatement, ParseError> {
    let mut inner = pair.into_inner();
    let variable = inner.next().unwrap().as_str().to_string();
    let start = build_expression(inner.next().unwrap())?;
    let end = build_expression(inner.next().unwrap())?;
    let body = inner.map(build_statement).collect::<Result<_, _>>()?;
    Ok(ForStatement {
        variable,
        start,
        end,
        body,
    })
}

fn build_if_statement(pair: Pair<Rule>) -> Result<IfStatement, ParseError> {
    let mut inner = pair.into_inner();
    let condition = build_conditional_expression(inner.next().unwrap())?;
    let body = inner.map(build_statement).collect::<Result<_, _>>()?;
    Ok(IfStatement { condition, body })
}

fn build_conditional_expression(pair: Pair<Rule>) -> Result<ConditionalExpression, ParseError> {
    let mut inner = pair.into_inner();
    let left = build_arithmetic_expression(inner.next().unwrap())?;
    let operator = build_comparison_operator(inner.next().unwrap());
    let right = build_arithmetic_expression(inner.next().unwrap())?;
    Ok(ConditionalExpression { left, operator, right })
}

fn build_comparison_operator(pair: Pair<Rule>) -> ComparisonOperator {
    match pair.as_str() {
        "<" => ComparisonOperator::LessThan,
        ">" => ComparisonOperator::GreaterThan,
        "<=" => ComparisonOperator::LessEqual,
        ">=" => ComparisonOperator::GreaterEqual,
        "==" => ComparisonOperator::Equal,
        "!=" => ComparisonOperator::NotEqual,
        _ => unreachable!("Unknown comparison operator: {}", pair.as_str()),
    }
}

fn build_arithmetic_expression(pair: Pair<Rule>) -> Result<ArithmeticExpression, ParseError> {
    let mut inner = pair.into_inner().peekable();
    let mut left = build_term(inner.next().unwrap())?;

    while let Some(op_pair) = inner.peek() {
        if op_pair.as_rule() == Rule::add_op {
            let op = inner.next().unwrap();
            let right = build_term(inner.next().unwrap())?;
            left = match op.as_str() {
                "+" => ArithmeticExpression::Add(Box::new(left), Box::new(right)),
                "-" => ArithmeticExpression::Subtract(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok(left)
}

fn build_term(pair: Pair<Rule>) -> Result<ArithmeticExpression, ParseError> {
    let mut inner = pair.into_inner().peekable();
    let mut left = build_factor(inner.next().unwrap())?;

    while let Some(op_pair) = inner.peek() {
        if op_pair.as_rule() == Rule::mul_op {
            let op = inner.next().unwrap();
            let right = build_factor(inner.next().unwrap())?;
            left = match op.as_str() {
                "*" => ArithmeticExpression::Multiply(Box::new(left), Box::new(right)),
                "/" => ArithmeticExpression::Divide(Box::new(left), Box::new(right)),
                "%" => ArithmeticExpression::Modulo(Box::new(left), Box::new(right)),
                _ => unreachable!(),
            };
        } else {
            break;
        }
    }

    Ok(left)
}

fn build_factor(pair: Pair<Rule>) -> Result<ArithmeticExpression, ParseError> {
    match pair.as_rule() {
        Rule::factor => {
            let inner = pair.into_inner().next().unwrap();
            build_factor(inner)
        },
        Rule::arithmetic_expression => build_arithmetic_expression(pair),
        Rule::literal | Rule::identifier => {
            let expr = build_expression(pair)?;
            Ok(ArithmeticExpression::Term(expr))
        },
        _ => {
            let expr = build_expression(pair)?;
            Ok(ArithmeticExpression::Term(expr))
        }
    }
}

fn build_node_declaration(pair: Pair<Rule>) -> Result<NodeDeclaration, ParseError> {
    let mut inner = pair.into_inner();
    let id = build_expression(inner.next().unwrap())?;
    let next = inner.next();
    let (node_type, attributes) = match next {
        Some(pair) if pair.as_rule() == Rule::expression => {
            let type_expr = build_expression(pair)?;
            let attrs = inner.next().map(build_attributes).transpose()?.unwrap_or_default();
            (Some(type_expr), attrs)
        }
        Some(pair) if pair.as_rule() == Rule::attributes => (None, build_attributes(pair)?),
        _ => (None, vec![]),
    };
    Ok(NodeDeclaration { id, node_type, attributes })
}

fn build_edge_declaration(pair: Pair<Rule>) -> Result<EdgeDeclaration, ParseError> {
    let span = pair.as_span(); // Capture span before moving pair
    let mut inner_pairs: Vec<_> = pair.into_inner().collect();

    let operator_pos = inner_pairs.iter().position(|p| p.as_rule() == Rule::edge_operator)
        .ok_or_else(|| Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError { message: "Edge operator not found".to_string() },
            span,
        )))?;

    let attributes = if inner_pairs.last().is_some_and(|p| p.as_rule() == Rule::attributes) {
        build_attributes(inner_pairs.pop().unwrap())?
    } else {
        Vec::new()
    };

    let directed = inner_pairs[operator_pos].as_str() == "->";

    let id: Option<Expression>;
    let source: Expression;
    let target: Expression;

    // Check if there's an edge_id
    let edge_id_pos = inner_pairs.iter().position(|p| p.as_rule() == Rule::edge_id);

    if let Some(edge_id_pos) = edge_id_pos {
        // There's an edge_id, check if it has an expression or is just ":"
        let edge_id_pair = &inner_pairs[edge_id_pos];
        let edge_id_inner: Vec<_> = edge_id_pair.clone().into_inner().collect();

        if edge_id_inner.len() == 1 { // expression (the ":" is part of the rule but not captured)
            id = Some(build_expression(edge_id_inner[0].clone())?);
        } else { // just ":" (empty inner)
            id = None;
        }

        // Source and target are after the edge_id
        source = build_expression(inner_pairs[edge_id_pos + 1].clone())?;
        target = build_expression(inner_pairs[edge_id_pos + 3].clone())?; // Skip operator
    } else {
        // No edge_id, so it's: source operator target
        id = None;
        source = build_expression(inner_pairs[0].clone())?;
        target = build_expression(inner_pairs[2].clone())?; // Skip operator
    }

    Ok(EdgeDeclaration { id, source, target, directed, attributes })
}


fn build_generate_statement(pair: Pair<Rule>) -> Result<GenerateStatement, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let params = inner
        .map(|p| -> Result<(String, Expression), ParseError> {
            let mut kv = p.into_inner();
            let key = kv.next().unwrap().as_str().to_string();
            let value = build_expression(kv.next().unwrap())?;
            Ok((key, value))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GenerateStatement { name, params })
}

fn build_rule_definition(pair: Pair<Rule>) -> Result<RuleDefinition, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let lhs_pair = inner.next().unwrap();
    let rhs_pair = inner.next().unwrap();
    let lhs = build_pattern(lhs_pair)?;
    let rhs = build_pattern(rhs_pair)?;
    Ok(RuleDefinition { name, lhs, rhs })
}

fn build_pattern(pair: Pair<Rule>) -> Result<Pattern, ParseError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for pattern_stmt_pair in pair.into_inner() {
        if let Some(stmt_pair) = pattern_stmt_pair.into_inner().next() {
            match stmt_pair.as_rule() {
                Rule::node_declaration => nodes.push(build_node_declaration(stmt_pair)?),
                Rule::edge_declaration => edges.push(build_edge_declaration(stmt_pair)?),
                _ => continue,
            }
        }
    }
    Ok(Pattern { nodes, edges })
}

fn build_apply_statement(pair: Pair<Rule>) -> Result<ApplyStatement, ParseError> {
    let mut inner = pair.into_inner();
    let rule_name = inner.next().unwrap().as_str().to_string();
    let iterations = build_expression(inner.next().unwrap())?;
    Ok(ApplyStatement { rule_name, iterations })
}

fn build_attributes(pair: Pair<Rule>) -> Result<Vec<(String, Expression)>, ParseError> {
    pair.into_inner()
        .map(|p| -> Result<(String, Expression), ParseError> {
            let mut kv = p.into_inner();
            let key = kv.next().unwrap().as_str().to_string();
            let value = build_expression(kv.next().unwrap())?;
            Ok((key, value))
        })
        .collect()
}

fn build_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next().unwrap();
            build_expression(inner)
        },
        Rule::arithmetic_expression => {
            // For now, convert arithmetic expressions to simple expressions
            // This is a simplified approach - in a full implementation you might want to evaluate them
            let _arith = build_arithmetic_expression(pair)?;
            // For compatibility, let's just return the first term if it's simple
            match _arith {
                ArithmeticExpression::Term(expr) => Ok(expr),
                _ => {
                    // For now, return a placeholder - this should be evaluated properly
                    Ok(Expression::Integer(0))
                }
            }
        },
        Rule::literal => build_literal(pair),
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string())),
        Rule::formatted_string => {
            let parts = pair.into_inner().map(|p| match p.as_rule() {
                Rule::string_part => StringPart::Literal(p.as_str().to_string()),
                Rule::var_in_string => {
                    let arith_expr = p.into_inner().next().unwrap();
                    // For now, we'll serialize the arithmetic expression as a string
                    // In a full implementation, you might want to evaluate it
                    StringPart::Variable(arith_expr.as_str().to_string())
                },
                _ => unreachable!(),
            }).collect();
            Ok(Expression::FormattedString(parts))
        },
        Rule::string => {
            // Direct string literal
            let content = pair.as_str();
            let trimmed = &content[1..content.len()-1]; // Remove quotes
            Ok(Expression::StringLiteral(trimmed.to_string()))
        },
        Rule::integer => Ok(Expression::Integer(pair.as_str().parse().unwrap())),
        Rule::float => Ok(Expression::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => Ok(Expression::Boolean(pair.as_str().parse().unwrap())),
        _ => unreachable!("Unexpected expression rule: {:?}", pair.as_rule()),
    }
}

fn build_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string => {
            // Extract the content between quotes
            let content = inner.as_str();
            let trimmed = &content[1..content.len()-1]; // Remove quotes
            Ok(Expression::StringLiteral(trimmed.to_string()))
        },
        Rule::integer => Ok(Expression::Integer(inner.as_str().parse().unwrap())),
        Rule::float => Ok(Expression::Float(inner.as_str().parse().unwrap())),
        Rule::boolean => Ok(Expression::Boolean(inner.as_str().parse().unwrap())),
        _ => unreachable!("Unexpected literal rule: {:?}", inner.as_rule()),
    }
}
