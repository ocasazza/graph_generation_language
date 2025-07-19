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

// Values
value = { string | number | boolean | ident }

// Attributes
attribute = { ident ~ "=" ~ value }
attribute_list = { (attribute ~ ("," ~ attribute)*)? }
attributes = { "[" ~ attribute_list ~ "]" }

// Node declarations
node_type = { ":" ~ ident }
node_decl = { "node" ~ ident ~ node_type? ~ attributes? ~ ";" }

// Edge declarations
edge_op = { "->" | "--" }
edge_decl = { "edge" ~ ident? ~ ":" ~ ident ~ edge_op ~ ident ~ attributes? ~ ";" }

// Generator statements
param = { ident ~ ":" ~ value }
param_list = { (param ~ ";")* }
generate_stmt = { "generate" ~ ident ~ "{" ~ param_list ~ "}" }

// Rule patterns
node_pattern = { "node" ~ ident ~ node_type? ~ attributes? ~ ";" }
edge_pattern = { ("edge" ~ ident? ~ ":")? ~ ident ~ edge_op ~ ident ~ attributes? ~ ";" }
pattern = { "{" ~ (node_pattern | edge_pattern)* ~ "}" }

// Rule definition
rule_def = { "rule" ~ ident ~ "{" ~ "lhs" ~ pattern ~ "rhs" ~ pattern ~ "}" }

// Rule application
apply_rule = { "apply" ~ ident ~ number ~ "times" ~ ";" }

// Graph statements
statement = { node_decl | edge_decl | generate_stmt | rule_def | apply_rule }
graph = { "graph" ~ ident? ~ "{" ~ statement* ~ "}" }

// Entry point
program = { SOI ~ graph ~ EOI }
"#]
pub struct GGLParser;

#[derive(Debug, Clone)]
pub struct NodeDeclaration {
    pub id: String,
    pub node_type: Option<String>,
    pub attributes: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone)]
pub struct EdgeDeclaration {
    pub id: String,
    pub source: String,
    pub target: String,
    pub directed: bool,
    pub attributes: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone)]
pub struct GenerateStatement {
    pub name: String,
    pub params: HashMap<String, MetadataValue>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub nodes: Vec<NodeDeclaration>,
    pub edges: Vec<EdgeDeclaration>,
}

#[derive(Debug, Clone)]
pub struct RuleDefinition {
    pub name: String,
    pub lhs: Pattern,
    pub rhs: Pattern,
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
    RuleDefStmt(RuleDefinition),
    ApplyRuleStmt(ApplyRuleStatement),
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
                        for stmt_pair in graph_pair.into_inner() {
                            if let Some(stmt) = parse_statement(stmt_pair)? {
                                statements.push(stmt);
                            }
                        }
                    }
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
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
                Rule::rule_def => Ok(Some(GGLStatement::RuleDefStmt(parse_rule_def(inner)?))),
                Rule::apply_rule => Ok(Some(GGLStatement::ApplyRuleStmt(parse_apply_rule(inner)?))),
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn parse_node_decl(pair: pest::iterators::Pair<Rule>) -> Result<NodeDeclaration, String> {
    let mut id = String::new();
    let mut node_type = None;
    let mut attributes = HashMap::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => id = inner_pair.as_str().to_string(),
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
        id,
        node_type,
        attributes,
    })
}

fn parse_edge_decl(pair: pest::iterators::Pair<Rule>) -> Result<EdgeDeclaration, String> {
    let id;
    let source;
    let target;
    let mut directed = false;
    let mut attributes = HashMap::new();
    let mut idents = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => {
                idents.push(inner_pair.as_str().to_string());
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

    // Determine if there's an explicit ID based on the number of identifiers
    match idents.len() {
        2 => {
            // No explicit ID: either "edge: source -> target" or "source -> target"
            source = idents[0].clone();
            target = idents[1].clone();
            id = format!("e{source}_{target}");
        }
        3 => {
            // Explicit ID: edge id: source -> target
            id = idents[0].clone();
            source = idents[1].clone();
            target = idents[2].clone();
        }
        _ => {
            return Err("Invalid edge declaration: expected 2 or 3 identifiers".to_string());
        }
    }

    Ok(EdgeDeclaration {
        id,
        source,
        target,
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

fn parse_rule_def(pair: pest::iterators::Pair<Rule>) -> Result<RuleDefinition, String> {
    let mut name = String::new();
    let mut lhs = Pattern {
        nodes: Vec::new(),
        edges: Vec::new(),
    };
    let mut rhs = Pattern {
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::ident => name = inner_pair.as_str().to_string(),
            Rule::pattern => {
                let pattern = parse_pattern(inner_pair)?;
                if lhs.nodes.is_empty() && lhs.edges.is_empty() {
                    lhs = pattern;
                } else {
                    rhs = pattern;
                }
            }
            _ => (),
        }
    }

    Ok(RuleDefinition { name, lhs, rhs })
}

fn parse_pattern(pair: pest::iterators::Pair<Rule>) -> Result<Pattern, String> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::node_pattern => {
                // Parse node_pattern the same way as node_decl
                nodes.push(parse_node_decl(inner_pair)?);
            }
            Rule::edge_pattern => {
                // Parse edge_pattern the same way as edge_decl
                edges.push(parse_edge_decl(inner_pair)?);
            }
            _ => (),
        }
    }

    Ok(Pattern { nodes, edges })
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
) -> Result<HashMap<String, MetadataValue>, String> {
    let mut attributes = HashMap::new();

    for attr_list in pair.into_inner() {
        if attr_list.as_rule() == Rule::attribute_list {
            for attr in attr_list.into_inner() {
                if attr.as_rule() == Rule::attribute {
                    let mut attr_iter = attr.into_inner();
                    let key = attr_iter.next().unwrap().as_str().to_string();
                    let value = parse_value(attr_iter.next().unwrap())?;
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
        Rule::ident => Ok(MetadataValue::String(value_pair.as_str().to_string())),
        _ => Err(format!("Unexpected value type: {:?}", value_pair.as_rule())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph() {
        let input = r#"
            graph test {
                node A;
                node B;
                edge e1: A -> B [weight=1.0];
            }
        "#;

        let result = parse_ggl(input);
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 3);
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

    #[test]
    fn test_parse_rule_def() {
        let input = r#"
            graph {
                rule add_leaf {
                    lhs { node N :intermediate; }
                    rhs {
                        node N :intermediate;
                        node L :leaf;
                        N -> L;
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
                assert_eq!(rule.rhs.nodes.len(), 2);
                assert_eq!(rule.rhs.edges.len(), 1);
            }
            _ => panic!("Expected RuleDefStmt"),
        }
    }
}
