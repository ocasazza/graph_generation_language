//! GGL v0.3.0 Parser - Functional JSON-superset language parser
//! Transforms GGL source code into a functional AST for evaluation

use pest::iterators::Pair;
use pest::Parser as PestParser;
use pest_derive::Parser;
use std::collections::HashMap;
use std::fmt;

/// Type alias for boxed pest error to reduce Result size
type ParseError = Box<pest::error::Error<Rule>>;

#[derive(Parser)]
#[grammar = "ggl.pest"]
pub struct GglParser;

// --- Abstract Syntax Tree (AST) ---

#[derive(Debug, Clone)]
pub struct GraphAST {
    pub root: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Core structures
    ObjectExpression(HashMap<String, Expression>),
    TaggedObject { tag: String, fields: HashMap<String, Expression> },
    ArrayExpression(Vec<Expression>),

    // Functions and lambdas
    FunctionDefinition { name: String, params: Vec<String>, body: Box<Expression> },
    LambdaExpression { params: Vec<String>, body: Box<Expression> },

    // Method chaining
    ChainExpression { base: Box<Expression>, chain: Vec<ChainItem> },

    // Built-ins and templates
    BuiltinCall { name: String, args: Vec<Expression> },
    TemplateLiteral { parts: Vec<TemplatePart> },

    // Arithmetic operations
    ArithmeticExpression(ArithmeticOp),

    // Comparison operations
    ComparisonExpression { left: Box<Expression>, operator: ComparisonOperator, right: Box<Expression> },

    // Literals
    StringLiteral(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Identifier(String),

    // Special expressions
    SpreadExpression(Box<Expression>),
    BlockExpression { statements: Vec<Expression>, result: Box<Expression> },
    VariableDeclaration { name: String, value: Box<Expression> },
    IfExpression { condition: Box<Expression>, then_block: Box<Expression>, else_block: Option<Box<Expression>> },
    ReturnStatement(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum ChainItem {
    MethodCall { name: String, args: Vec<Expression> },
    PropertyAccess { name: String },
    BuiltinCall { name: String, args: Vec<Expression> },
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum TemplatePart {
    Literal(String),
    Variable(Expression),
}

#[derive(Debug, Clone)]
pub enum ArithmeticOp {
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Term(Box<Expression>),
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

/// Implements Display for debugging and error messages
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::StringLiteral(s) => write!(f, "\"{s}\""),
            Expression::Integer(i) => write!(f, "{i}"),
            Expression::Float(n) => write!(f, "{n}"),
            Expression::Boolean(b) => write!(f, "{b}"),
            Expression::Null => write!(f, "null"),
            Expression::Identifier(name) => write!(f, "{name}"),
            Expression::TemplateLiteral { parts } => {
                write!(f, "`")?;
                for part in parts {
                    match part {
                        TemplatePart::Literal(s) => write!(f, "{s}")?,
                        TemplatePart::Variable(expr) => write!(f, "${{{expr}}}")?,
                    }
                }
                write!(f, "`")
            }
            _ => write!(f, "[Expression]"),
        }
    }
}

// --- Parser Implementation ---

/// Parses a GGL source string into a Graph AST
pub fn parse_ggl(source: &str) -> Result<GraphAST, ParseError> {
    let file_pair = GglParser::parse(Rule::file, source)
        .map_err(Box::new)?
        .next()
        .unwrap();

    let expression = file_pair.into_inner()
        .find(|p| p.as_rule() != Rule::EOI)
        .ok_or_else(|| Box::new(pest::error::Error::new_from_pos(
            pest::error::ErrorVariant::CustomError {
                message: "Empty file".to_string()
            },
            pest::Position::from_start("")
        )))?;

    let root = build_expression(expression)?;
    Ok(GraphAST { root })
}

fn build_logical_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::logical_op {
            let _right = build_expression(inner.next().unwrap())?;
            // For now, just return a simple boolean based on the operator
            // In a full implementation, this would create a logical expression
            left = match op_pair.as_str() {
                "&&" => Expression::Boolean(true),  // Simplified - would need proper evaluation
                "||" => Expression::Boolean(false), // Simplified - would need proper evaluation
                _ => Expression::Boolean(false),
            };
        }
    }

    Ok(left)
}

fn build_comparison_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::comparison_operator {
            let right = build_expression(inner.next().unwrap())?;
            let operator = match op_pair.as_str() {
                "!==" | "!=" => ComparisonOperator::NotEqual,
                "===" | "==" => ComparisonOperator::Equal,
                "<" => ComparisonOperator::LessThan,
                ">" => ComparisonOperator::GreaterThan,
                "<=" => ComparisonOperator::LessEqual,
                ">=" => ComparisonOperator::GreaterEqual,
                _ => return Err(Box::new(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: format!("Unknown comparison operator: {}", op_pair.as_str()),
                    },
                    op_pair.as_span(),
                ))),
            };
            left = Expression::ComparisonExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

#[allow(dead_code)]
fn build_comparison_term(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_expression(inner)
}

fn build_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next().unwrap();
            build_expression(inner)
        }
        Rule::object_expression => build_object_expression(pair),
        Rule::tagged_object => build_tagged_object(pair),
        Rule::array_expression => build_array_expression(pair),
        Rule::chain_expression => build_chain_expression(pair),
        Rule::builtin_call => build_builtin_call(pair),
        Rule::function_definition => build_function_definition(pair),
        Rule::lambda_expression => build_lambda_expression(pair),
        Rule::template_literal => build_template_literal(pair),
        Rule::comparison_expression => build_comparison_expression(pair),
        Rule::range_expr => build_range_expression(pair),
        Rule::logical_expression => build_logical_expression(pair),
        Rule::additive_expression => build_additive_expression(pair),
        Rule::multiplicative_expression => build_multiplicative_expression(pair),
        Rule::postfix_expression => build_postfix_expression(pair),
        Rule::primary_expression => {
            let inner = pair.into_inner().next().unwrap();
            build_expression(inner)
        },
        Rule::if_expression => build_if_expression(pair),
        Rule::block_expression => build_block_expression(pair),
        Rule::spread_expression => build_spread_expression(pair),
        Rule::variable_declaration => build_variable_declaration(pair),
        Rule::return_statement => build_return_statement(pair),
        Rule::lambda_body => {
            let inner = pair.into_inner().next().unwrap();
            build_expression(inner)
        },
        Rule::literal => build_literal(pair),
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string())),
        Rule::string_literal => build_string_literal(pair),
        Rule::integer => Ok(Expression::Integer(pair.as_str().parse().unwrap())),
        Rule::float => Ok(Expression::Float(pair.as_str().parse().unwrap())),
        Rule::boolean => Ok(Expression::Boolean(pair.as_str().parse().unwrap())),
        Rule::null => Ok(Expression::Null),
        _ => {
            eprintln!("Unexpected rule in build_expression: {:?}", pair.as_rule());
            Err(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Unexpected expression rule: {:?}", pair.as_rule()),
                },
                pair.as_span(),
            )))
        }
    }
}

fn build_object_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut object = HashMap::new();

    for object_item in pair.into_inner() {
        match object_item.as_rule() {
            Rule::object_item => {
                // object_item contains either spread_expression or object_pair
                let inner_item = object_item.into_inner().next().unwrap();

                match inner_item.as_rule() {
                    Rule::object_pair => {
                        let mut pair_inner = inner_item.into_inner();
                        let key_pair = pair_inner.next().unwrap();
                        let value_pair = pair_inner.next().unwrap();

                        let key = match key_pair.as_rule() {
                            Rule::string_literal => {
                                let content = key_pair.as_str();
                                content[1..content.len()-1].to_string() // Remove quotes
                            }
                            Rule::identifier => key_pair.as_str().to_string(),
                            _ => return Err(Box::new(pest::error::Error::new_from_span(
                                pest::error::ErrorVariant::CustomError {
                                    message: "Invalid object key".to_string(),
                                },
                                key_pair.as_span(),
                            ))),
                        };

                        let value = build_expression(value_pair)?;
                        object.insert(key, value);
                    }
                    Rule::spread_expression => {
                        // For now, just ignore spread expressions in object parsing
                        // A full implementation would merge the spread object properties
                        continue;
                    }
                    _ => {}
                }
            }
            Rule::object_pair => {
                let mut inner = object_item.into_inner();
                let key_pair = inner.next().unwrap();
                let value_pair = inner.next().unwrap();

                let key = match key_pair.as_rule() {
                    Rule::string_literal => {
                        let content = key_pair.as_str();
                        content[1..content.len()-1].to_string() // Remove quotes
                    }
                    Rule::identifier => key_pair.as_str().to_string(),
                    _ => return Err(Box::new(pest::error::Error::new_from_span(
                        pest::error::ErrorVariant::CustomError {
                            message: "Invalid object key".to_string(),
                        },
                        key_pair.as_span(),
                    ))),
                };

                let value = build_expression(value_pair)?;
                object.insert(key, value);
            }
            Rule::spread_expression => {
                // For now, just ignore spread expressions in object parsing
                // A full implementation would merge the spread object properties
                continue;
            }
            _ => {}
        }
    }

    Ok(Expression::ObjectExpression(object))
}

fn build_tagged_object(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let span = pair.as_span(); // Capture span before moving pair
    let mut inner = pair.into_inner();
    let tag = inner.next().unwrap().as_str().to_string();
    let mut fields = HashMap::new();

    for object_pair in inner {
        if object_pair.as_rule() == Rule::object_pair {
            let mut pair_inner = object_pair.into_inner();
            let key_pair = pair_inner.next().unwrap();
            let value_pair = pair_inner.next().unwrap();

            let key = match key_pair.as_rule() {
                Rule::string_literal => {
                    let content = key_pair.as_str();
                    content[1..content.len()-1].to_string()
                }
                Rule::identifier => key_pair.as_str().to_string(),
                _ => return Err(Box::new(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: "Invalid field key in tagged object".to_string(),
                    },
                    key_pair.as_span(),
                ))),
            };

            let value = build_expression(value_pair)?;
            fields.insert(key, value);
        }
    }

    // Validate required fields for Node and Edge
    match tag.as_str() {
        "Node" => {
            if !fields.contains_key("id") {
                return Err(Box::new(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: "Node object must have 'id' field".to_string(),
                    },
                    span,
                )));
            }
        }
        "Edge" => {
            if !fields.contains_key("source") || !fields.contains_key("target") {
                return Err(Box::new(pest::error::Error::new_from_span(
                    pest::error::ErrorVariant::CustomError {
                        message: "Edge object must have 'source' and 'target' fields".to_string(),
                    },
                    span,
                )));
            }
        }
        _ => {}
    }

    Ok(Expression::TaggedObject { tag, fields })
}

fn build_array_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let elements = pair.into_inner()
        .map(build_expression)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Expression::ArrayExpression(elements))
}

fn build_chain_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let base = build_expression(inner.next().unwrap())?;

    let mut chain = Vec::new();
    for segment in inner {
        let item = segment.into_inner().next().unwrap();
        match item.as_rule() {
            Rule::method_call => {
                let mut method_inner = item.into_inner();
                let name = method_inner.next().unwrap().as_str().to_string();
                let args = method_inner
                    .map(build_expression)
                    .collect::<Result<Vec<_>, _>>()?;
                chain.push(ChainItem::MethodCall { name, args });
            }
            Rule::builtin_call => {
                let mut builtin_inner = item.into_inner();
                let name = builtin_inner.next().unwrap().as_str().to_string();
                let args = if let Some(args_pair) = builtin_inner.next() {
                    args_pair.into_inner()
                        .map(build_expression)
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    Vec::new()
                };
                chain.push(ChainItem::BuiltinCall { name, args });
            }
            Rule::identifier => {
                let name = item.as_str().to_string();
                chain.push(ChainItem::PropertyAccess { name });
            }
            _ => unreachable!("Unexpected rule in chain_segment: {:?}", item.as_rule()),
        }
    }

    // Since we now require at least one chain segment, we always have a chain
    Ok(Expression::ChainExpression {
        base: Box::new(base),
        chain,
    })
}

fn build_builtin_call(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let args = if let Some(args_pair) = inner.next() {
        args_pair.into_inner()
            .map(build_expression)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    Ok(Expression::BuiltinCall { name, args })
}

fn build_range_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let start = build_range_term(inner.next().unwrap())?;
    let end = build_range_term(inner.next().unwrap())?;

    // Convert range expression to a builtin call for now
    Ok(Expression::BuiltinCall {
        name: "range".to_string(),
        args: vec![start, end],
    })
}

fn build_range_term(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_expression(inner)
}

fn build_function_definition(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let span = pair.as_span();
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let lambda_pair = inner.next().unwrap();

    if let Expression::LambdaExpression { params, body } = build_lambda_expression(lambda_pair)? {
        Ok(Expression::FunctionDefinition {
            name,
            params,
            body,
        })
    } else {
        Err(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Invalid function definition".to_string(),
            },
            span,
        )))
    }
}

fn build_lambda_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let span = pair.as_span();
    let mut inner = pair.into_inner();

    let mut params = Vec::new();

    // The grammar has two patterns:
    // 1. ("(" ~ lambda_param_list ~ ")" ~ "=>" ~ lambda_body)
    // 2. (lambda_param ~ "=>" ~ lambda_body)

    let first_item = inner.next().unwrap();

    let body_pair = match first_item.as_rule() {
        Rule::lambda_param_list => {
            // Pattern 1: parentheses around parameter list
            for param_pair in first_item.into_inner() {
                params.push(extract_param_name(param_pair)?);
            }
            inner.next() // lambda_body
        }
        Rule::lambda_param => {
            // Pattern 2: single parameter without parentheses
            params.push(extract_param_name(first_item)?);
            inner.next() // lambda_body
        }
        Rule::lambda_body | Rule::expression | Rule::block_expression => {
            // This means we have no parameters (empty lambda_param_list)
            Some(first_item)
        }
        _ => {
            return Err(Box::new(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: format!("Unexpected lambda element: {:?}", first_item.as_rule()),
                },
                span,
            )));
        }
    };

    let body = if let Some(body_pair) = body_pair {
        Box::new(build_expression(body_pair)?)
    } else {
        return Err(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Lambda expression missing body".to_string(),
            },
            span,
        )));
    };

    Ok(Expression::LambdaExpression { params, body })
}

fn extract_param_name(param_pair: Pair<Rule>) -> Result<String, ParseError> {
    let inner = param_pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::identifier => Ok(inner.as_str().to_string()),
        Rule::array_destructure => {
            // Preserve the destructuring syntax for proper handling in evaluation
            let destructure_params: Vec<String> = inner.into_inner()
                .map(|p| p.as_str().to_string())
                .collect();
            Ok(format!("[{}]", destructure_params.join(", "))) // Keep proper format
        }
        _ => Err(Box::new(pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: "Invalid parameter type".to_string(),
            },
            inner.as_span(),
        ))),
    }
}

fn build_template_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut parts = Vec::new();

    for part in pair.into_inner() {
        match part.as_rule() {
            Rule::template_part => {
                parts.push(TemplatePart::Literal(part.as_str().to_string()));
            }
            Rule::template_var => {
                let expr_pair = part.into_inner().next().unwrap();
                let expr = build_expression(expr_pair)?;
                parts.push(TemplatePart::Variable(expr));
            }
            _ => {}
        }
    }

    Ok(Expression::TemplateLiteral { parts })
}

fn build_additive_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::add_op {
            let right = build_expression(inner.next().unwrap())?;
            left = match op_pair.as_str() {
                "+" => Expression::ArithmeticExpression(ArithmeticOp::Add(
                    Box::new(left),
                    Box::new(right)
                )),
                "-" => Expression::ArithmeticExpression(ArithmeticOp::Subtract(
                    Box::new(left),
                    Box::new(right)
                )),
                _ => unreachable!(),
            };
        }
    }

    Ok(left)
}

fn build_multiplicative_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_expression(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::mul_op {
            let right = build_expression(inner.next().unwrap())?;
            left = match op_pair.as_str() {
                "*" => Expression::ArithmeticExpression(ArithmeticOp::Multiply(
                    Box::new(left),
                    Box::new(right)
                )),
                "/" => Expression::ArithmeticExpression(ArithmeticOp::Divide(
                    Box::new(left),
                    Box::new(right)
                )),
                "%" => Expression::ArithmeticExpression(ArithmeticOp::Modulo(
                    Box::new(left),
                    Box::new(right)
                )),
                _ => unreachable!(),
            };
        }
    }

    Ok(left)
}

fn build_postfix_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let base = build_expression(inner.next().unwrap())?;

    let mut chain = Vec::new();
    for item in inner {
        match item.as_rule() {
            Rule::chain_segment => {
                let segment_inner = item.into_inner().next().unwrap();
                match segment_inner.as_rule() {
                    Rule::method_call => {
                        let mut method_inner = segment_inner.into_inner();
                        let name = method_inner.next().unwrap().as_str().to_string();
                        let args = method_inner
                            .map(build_expression)
                            .collect::<Result<Vec<_>, _>>()?;
                        chain.push(ChainItem::MethodCall { name, args });
                    }
                    Rule::builtin_call => {
                        let mut builtin_inner = segment_inner.into_inner();
                        let name = builtin_inner.next().unwrap().as_str().to_string();
                        let args = if let Some(args_pair) = builtin_inner.next() {
                            args_pair.into_inner()
                                .map(build_expression)
                                .collect::<Result<Vec<_>, _>>()?
                        } else {
                            Vec::new()
                        };
                        chain.push(ChainItem::BuiltinCall { name, args });
                    }
                    Rule::identifier => {
                        let name = segment_inner.as_str().to_string();
                        chain.push(ChainItem::PropertyAccess { name });
                    }
                    _ => unreachable!("Unexpected rule in chain_segment: {:?}", segment_inner.as_rule()),
                }
            }
            Rule::range_op => {
                // Handle range operations like a..b
                let range_end = build_expression(item.into_inner().next().unwrap())?;
                return Ok(Expression::BuiltinCall {
                    name: "range".to_string(),
                    args: vec![base, range_end],
                });
            }
            _ => unreachable!("Unexpected rule in postfix_expression: {:?}", item.as_rule()),
        }
    }

    if chain.is_empty() {
        Ok(base)
    } else {
        Ok(Expression::ChainExpression {
            base: Box::new(base),
            chain,
        })
    }
}

#[allow(dead_code)]
fn build_arithmetic_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_term(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::add_op {
            let right = build_term(inner.next().unwrap())?;
            left = match op_pair.as_str() {
                "+" => ArithmeticOp::Add(
                    Box::new(Expression::ArithmeticExpression(left)),
                    Box::new(Expression::ArithmeticExpression(right))
                ),
                "-" => ArithmeticOp::Subtract(
                    Box::new(Expression::ArithmeticExpression(left)),
                    Box::new(Expression::ArithmeticExpression(right))
                ),
                _ => unreachable!(),
            };
        }
    }

    Ok(Expression::ArithmeticExpression(left))
}

#[allow(dead_code)]
fn build_term(pair: Pair<Rule>) -> Result<ArithmeticOp, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = build_factor(inner.next().unwrap())?;

    while let Some(op_pair) = inner.next() {
        if op_pair.as_rule() == Rule::mul_op {
            let right = build_factor(inner.next().unwrap())?;
            left = match op_pair.as_str() {
                "*" => ArithmeticOp::Multiply(
                    Box::new(Expression::ArithmeticExpression(left)),
                    Box::new(Expression::ArithmeticExpression(right))
                ),
                "/" => ArithmeticOp::Divide(
                    Box::new(Expression::ArithmeticExpression(left)),
                    Box::new(Expression::ArithmeticExpression(right))
                ),
                "%" => ArithmeticOp::Modulo(
                    Box::new(Expression::ArithmeticExpression(left)),
                    Box::new(Expression::ArithmeticExpression(right))
                ),
                _ => unreachable!(),
            };
        }
    }

    Ok(left)
}

#[allow(dead_code)]
fn build_factor(pair: Pair<Rule>) -> Result<ArithmeticOp, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::additive_expression => {
            let expr = build_additive_expression(inner)?;
            if let Expression::ArithmeticExpression(op) = expr {
                Ok(op)
            } else {
                Ok(ArithmeticOp::Term(Box::new(expr)))
            }
        }
        Rule::literal => {
            let expr = build_expression(inner)?;
            Ok(ArithmeticOp::Term(Box::new(expr)))
        }
        Rule::identifier => {
            let expr = build_expression(inner)?;
            Ok(ArithmeticOp::Term(Box::new(expr)))
        }
        _ => {
            unreachable!("Unexpected rule in factor: {:?}", inner.as_rule())
        }
    }
}

fn build_block_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut statements = Vec::new();
    let mut last_expression = None;

    // Process all items in the block
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::statement => {
                let expr_pair = item.into_inner().next().unwrap();
                statements.push(build_expression(expr_pair)?);
            }
            Rule::expression => {
                // If we see a standalone expression, it becomes the result
                last_expression = Some(Box::new(build_expression(item)?));
            }
            _ => {}
        }
    }

    // If we have a final expression, use that as result
    // Otherwise, use the last statement as result, or Null if empty
    let result = last_expression.unwrap_or_else(|| {
        if let Some(last_stmt) = statements.pop() {
            Box::new(last_stmt)
        } else {
            Box::new(Expression::Null)
        }
    });

    Ok(Expression::BlockExpression { statements, result })
}

fn build_spread_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let expr = build_expression(inner)?;
    Ok(Expression::SpreadExpression(Box::new(expr)))
}

fn build_variable_declaration(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let value = build_expression(inner.next().unwrap())?;

    Ok(Expression::VariableDeclaration {
        name,
        value: Box::new(value),
    })
}

fn build_if_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let condition = build_expression(inner.next().unwrap())?;
    let then_block = build_expression(inner.next().unwrap())?;
    let else_block = if let Some(else_pair) = inner.next() {
        Some(Box::new(build_expression(else_pair)?))
    } else {
        None
    };

    Ok(Expression::IfExpression {
        condition: Box::new(condition),
        then_block: Box::new(then_block),
        else_block,
    })
}

fn build_return_statement(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let expr = build_expression(inner)?;
    Ok(Expression::ReturnStatement(Box::new(expr)))
}

fn build_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_expression(inner)
}

fn build_string_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let content = pair.as_str();
    let mut result = String::new();
    let mut chars = content[1..content.len()-1].chars(); // Remove quotes

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                match escaped {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    'u' => {
                        // Unicode escape sequence
                        let mut unicode_digits = String::new();
                        for _ in 0..4 {
                            if let Some(digit) = chars.next() {
                                unicode_digits.push(digit);
                            }
                        }
                        if let Ok(code_point) = u32::from_str_radix(&unicode_digits, 16) {
                            if let Some(unicode_char) = char::from_u32(code_point) {
                                result.push(unicode_char);
                            }
                        }
                    }
                    _ => {
                        result.push('\\');
                        result.push(escaped);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }

    Ok(Expression::StringLiteral(result))
}
