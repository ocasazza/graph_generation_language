use crate::types::MetadataValue;
use pest::Parser as PestParser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar_inline = r#"
// Whitespace and comments
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* | "/*" ~ (!"*/" ~ ANY)* ~ "*/" }

// Identifiers and literals
ident = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
string = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
number = @{ ("+" | "-")? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT*)? }
boolean = @{ "true" | "false" }
variable = @{ ident }

// Interpolated strings for node/edge IDs
interpolated_string = { (string | variable | "{" ~ expression ~ "}")+ }

// Values and expressions
value = { string | number | boolean | variable }
expression = { value ~ (("+" | "-" | "*" | "/") ~ value)* }

// Attributes
attribute = { ident ~ "=" ~ expression }
attribute_list = { (attribute ~ ("," ~ attribute)*)? }
attributes = { "[" ~ attribute_list ~ "]" }

// Node declarations
node_type = { ":" ~ ident }
node_decl = { "node" ~ interpolated_string ~ node_type? ~ attributes? ~ ";" }

// Edge declarations
edge_op = { "->" | "--" }
edge_decl = { "edge" ~ interpolated_string? ~ ":" ~ interpolated_string ~ edge_op ~ interpolated_string ~ attributes? ~ ";" }

// Variable declaration
let_stmt = { "let" ~ ident ~ "=" ~ expression ~ ";" }

// For loop
range = { expression ~ ".." ~ expression }
for_loop = { "for" ~ ident ~ "in" ~ range ~ "{" ~ statement* ~ "}" }

// Generator statements (deprecated but kept for compatibility)
param = { ident ~ ":" ~ value }
param_list = { (param ~ ";")* }
generate_stmt = { "generate" ~ ident ~ "{" ~ param_list ~ "}" }

// Rule application
apply_rule = { "apply" ~ ident ~ number ~ "times" ~ ";" }

// Graph statements
statement = { node_decl | edge_decl | generate_stmt | apply_rule | let_stmt | for_loop }
graph = { "graph" ~ ident? ~ "{" ~ statement* ~ "}" }

// Entry point
program = { SOI ~ graph ~ EOI }
"#]
pub struct GGLParser;

#[derive(Debug, Clone)]
pub enum Expression {
    Value(MetadataValue),
    Variable(String),
    BinOp(Box<Expression>, char, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum InterpolatedStringPart {
    String(String),
    Variable(String),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct NodeDeclaration {
    pub id_parts: Vec<InterpolatedStringPart>,
    pub node_type: Option<String>,
    pub attributes: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
pub struct EdgeDeclaration {
    pub id_parts: Option<Vec<InterpolatedStringPart>>,
    pub source_parts: Vec<InterpolatedStringPart>,
    pub target_parts: Vec<InterpolatedStringPart>,
    pub directed: bool,
    pub attributes: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
pub struct GenerateStatement {
    pub name: String,
    pub params: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub var_name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub var_name: String,
    pub start: Expression,
    pub end: Expression,
    pub body: Vec<GGLStatement>,
}

#[derive(Debug, Clone)]
pub struct ApplyRuleStatement {
    pub rule_name: String,
    pub iterations: usize,
}

#[derive(Debug, Clone)]
pub enum GGLStatement {
    NodeDecl(NodeDeclaration),
    EdgeDecl(EdgeDeclaration),
    GenerateStmt(GenerateStatement),
    ApplyRuleStmt(ApplyRuleStatement),
    LetStmt(LetStatement),
    ForLoop(ForLoop),
}

pub fn parse_ggl(input: &str) -> Result<Vec<GGLStatement>, String> {
    let pairs = <GGLParser as PestParser<Rule>>::parse(Rule::program, input)
        .map_err(|e| format!("Parse error: {e}"))?;

    let mut statements = Vec::new();

    // There should be exactly one program rule that contains one graph rule
    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                // Find the graph rule within the program
                for graph_pair in pair.into_inner() {
                    if graph_pair.as_rule() == Rule::graph {
                        // Process all statements within the graph
                        statements = parse_statements(graph_pair.into_inner())?;
                    }
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }

    Ok(statements)
}

fn parse_statements(
    pairs: pest::iterators::Pairs<Rule>,
) -> Result<Vec<GGLStatement>, String> {
    let mut statements = Vec::new();
    for pair in pairs {
        if let Some(stmt) = parse_statement(pair)? {
            statements.push(stmt);
        }
    }
    Ok(statements)
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Option<GGLStatement>, String> {
    match pair.as_rule() {
        Rule::statement => {
            // Get the actual statement type from within the statement rule
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::node_decl => Ok(Some(GGLStatement::NodeDecl(parse_node_decl(inner)?))),
                Rule::edge_decl => Ok(Some(GGLStatement::EdgeDecl(parse_edge_decl(inner)?))),
                Rule::generate_stmt => Ok(Some(GGLStatement::GenerateStmt(parse_generate_stmt(
                    inner,
                )?))),
                Rule::apply_rule => Ok(Some(GGLStatement::ApplyRuleStmt(parse_apply_rule(inner)?))),
                Rule::let_stmt => Ok(Some(GGLStatement::LetStmt(parse_let_stmt(inner)?))),
                Rule::for_loop => Ok(Some(GGLStatement::ForLoop(parse_for_loop(inner)?))),
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn parse_node_decl(pair: pest::iterators::Pair<Rule>) -> Result<NodeDeclaration, String> {
    let mut id_parts = Vec::new();
    let mut node_type = None;
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::interpolated_string => id_parts = parse_interpolated_string(inner_pair)?,
            Rule::node_type => {
                node_type = Some(inner_pair.into_inner().next().unwrap().as_str().to_string());
            }
            Rule::attributes => {
                attributes = parse_attributes(inner_pair)?;
            }
            _ => (),
        }
    }

    Ok(NodeDeclaration {
        id_parts,
        node_type,
        attributes,
    })
}

fn parse_edge_decl(pair: pest::iterators::Pair<Rule>) -> Result<EdgeDeclaration, String> {
    let mut id_parts = None;
    let source_parts;
    let target_parts;
    let mut directed = false;
    let mut attributes = HashMap::new();
    let mut string_parts = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::interpolated_string => {
                string_parts.push(parse_interpolated_string(inner_pair)?);
            }
            Rule::edge_op => {
                directed = inner_pair.as_str() == "->";
            }
            Rule::attributes => {
                attributes = parse_attributes(inner_pair)?;
            }
            _ => (),
        }
    }

    // Determine if there's an explicit ID based on the number of interpolated strings
    match string_parts.len() {
        2 => {
            // No explicit ID
            source_parts = string_parts.remove(0);
            target_parts = string_parts.remove(0);
        }
        3 => {
            // Explicit ID
            id_parts = Some(string_parts.remove(0));
            source_parts = string_parts.remove(0);
            target_parts = string_parts.remove(0);
        }
        _ => {
            return Err("Invalid edge declaration: expected 2 or 3 interpolated strings".to_string());
        }
    }

    Ok(EdgeDeclaration {
        id_parts,
        source_parts,
        target_parts,
        directed,
        attributes,
    })
}

fn parse_generate_stmt(pair: pest::iterators::Pair<Rule>) -> Result<GenerateStatement, String> {
    let mut name = String::new();
    let mut params = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => name = inner_pair.as_str().to_string(),
            Rule::param_list => {
                for param_pair in inner_pair.into_inner() {
                    if param_pair.as_rule() == Rule::param {
                        let mut param_iter = param_pair.into_inner();
                        let param_name = param_iter.next().unwrap().as_str().to_string();
                        let param_value = parse_value(param_iter.next().unwrap())?;
                        params.insert(param_name, param_value);
                    }
                }
            }
            _ => (),
        }
    }

    Ok(GenerateStatement { name, params })
}


fn parse_apply_rule(pair: pest::iterators::Pair<Rule>) -> Result<ApplyRuleStatement, String> {
    let mut rule_name = String::new();
    let mut iterations = 0;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => rule_name = inner_pair.as_str().to_string(),
            Rule::number => {
                iterations = inner_pair
                    .as_str()
                    .parse::<usize>()
                    .map_err(|e| format!("Invalid iteration count: {e}"))?;
            }
            _ => (),
        }
    }

    Ok(ApplyRuleStatement {
        rule_name,
        iterations,
    })
}

fn parse_attributes(
    pair: pest::iterators::Pair<Rule>,
) -> Result<HashMap<String, Expression>, String> {
    let mut attributes = HashMap::new();

    for attr_list in pair.into_inner() {
        if attr_list.as_rule() == Rule::attribute_list {
            for attr in attr_list.into_inner() {
                if attr.as_rule() == Rule::attribute {
                    let mut attr_iter = attr.into_inner();
                    let key = attr_iter.next().unwrap().as_str().to_string();
                    let value = parse_expression(attr_iter.next().unwrap())?;
                    attributes.insert(key, value);
                }
            }
        }
    }

    Ok(attributes)
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<MetadataValue, String> {
    let value_pair = pair.clone().into_inner().next().unwrap_or(pair);

    match value_pair.as_rule() {
        Rule::string => Ok(MetadataValue::String(
            value_pair.as_str().trim_matches('"').to_string(),
        )),
        Rule::number => {
            let num_str = value_pair.as_str();
            // Try to parse as integer first, then as float
            if num_str.contains('.') {
                num_str
                    .parse::<f64>()
                    .map(MetadataValue::Float)
                    .map_err(|e| format!("Invalid float: {e}"))
            } else {
                num_str
                    .parse::<i64>()
                    .map(MetadataValue::Integer)
                    .map_err(|e| format!("Invalid integer: {e}"))
            }
        }
        Rule::boolean => Ok(MetadataValue::Boolean(value_pair.as_str() == "true")),
        Rule::variable => Ok(MetadataValue::String(value_pair.as_str().to_string())),
        _ => Err(format!("Unexpected value type: {:?}", value_pair.as_rule())),
    }
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    let lhs = match first.as_rule() {
        Rule::value => {
            let val = parse_value(first)?;
            Expression::Value(val)
        }
        Rule::variable => Expression::Variable(first.as_str().to_string()),
        _ => return Err(format!("Unexpected expression part: {:?}", first.as_rule())),
    };

    if let Some(op) = inner.next() {
        let rhs = parse_expression(inner.next().unwrap())?;
        Ok(Expression::BinOp(
            Box::new(lhs),
            op.as_str().chars().next().unwrap(),
            Box::new(rhs),
        ))
    } else {
        Ok(lhs)
    }
}

fn parse_interpolated_string(
    pair: pest::iterators::Pair<Rule>,
) -> Result<Vec<InterpolatedStringPart>, String> {
    let mut parts = Vec::new();
    for part_pair in pair.into_inner() {
        match part_pair.as_rule() {
            Rule::string => parts.push(InterpolatedStringPart::String(
                part_pair.as_str().trim_matches('"').to_string(),
            )),
            Rule::variable => {
                parts.push(InterpolatedStringPart::Variable(part_pair.as_str().to_string()))
            }
            Rule::expression => parts.push(InterpolatedStringPart::Expression(parse_expression(
                part_pair,
            )?)),
            _ => (),
        }
    }
    Ok(parts)
}

fn parse_let_stmt(pair: pest::iterators::Pair<Rule>) -> Result<LetStatement, String> {
    let mut inner = pair.into_inner();
    let var_name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;
    Ok(LetStatement { var_name, value })
}

fn parse_range(pair: pest::iterators::Pair<Rule>) -> Result<(Expression, Expression), String> {
    let mut inner = pair.into_inner();
    let start = parse_expression(inner.next().unwrap())?;
    let end = parse_expression(inner.next().unwrap())?;
    Ok((start, end))
}

fn parse_for_loop(pair: pest::iterators::Pair<Rule>) -> Result<ForLoop, String> {
    let mut inner = pair.into_inner();
    let var_name = inner.next().unwrap().as_str().to_string();
    let (start, end) = parse_range(inner.next().unwrap())?;
    let body = parse_statements(inner)?;
    Ok(ForLoop {
        var_name,
        start,
        end,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph_with_variables() {
        let input = r#"
            graph test {
                let size = 10;
                for i in 0..size {
                    node n{i};
                }
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 2);

        match &statements[0] {
            GGLStatement::LetStmt(let_stmt) => {
                assert_eq!(let_stmt.var_name, "size");
            }
            _ => panic!("Expected LetStmt"),
        }

        match &statements[1] {
            GGLStatement::ForLoop(for_loop) => {
                assert_eq!(for_loop.var_name, "i");
                assert_eq!(for_loop.body.len(), 1);
            }
            _ => panic!("Expected ForLoop"),
        }
    }

    #[test]
    fn test_parse_generate_stmt() {
        let input = r#"
            graph {
                generate complete {
                    nodes: 5;
                    prefix: "n";
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
                assert_eq!(gen.params.len(), 2);
            }
            _ => panic!("Expected GenerateStmt"),
        }
    }

}
