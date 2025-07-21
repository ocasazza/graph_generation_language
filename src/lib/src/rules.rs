//! Transformation rule engine for graph manipulation.

use crate::parser::{Expression, NodeDeclaration, Pattern};
use crate::types::{Edge, Graph, Node};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub lhs: Pattern,
    pub rhs: Pattern,
}

#[derive(Debug)]
struct Match {
    node_mapping: HashMap<String, String>, // Pattern node ID -> Graph node ID
}

fn expression_to_value(expr: &Expression) -> Result<Value, String> {
    match expr {
        Expression::StringLiteral(s) => Ok(Value::String(s.clone())),
        Expression::Integer(i) => Ok(Value::Number((*i).into())),
        Expression::Float(f) => Ok(Value::Number(
            serde_json::Number::from_f64(*f).ok_or_else(|| "Invalid float value".to_string())?,
        )),
        Expression::Boolean(b) => Ok(Value::Bool(*b)),
        Expression::Identifier(s) => Ok(Value::String(s.clone())), // Treat identifiers in RHS as strings
        Expression::FormattedString(_) => {
            Err("Formatted strings are not supported in rule RHS attributes".to_string())
        }
    }
}

impl Rule {
    /// Applies the rule to the graph for a specified number of iterations.
    pub fn apply(&self, graph: &mut Graph, iterations: usize) -> Result<(), String> {
        for _ in 0..iterations {
            let matches = self.find_matches(graph)?;

            if matches.is_empty() {
                break; // No more matches found, stop applying.
            }

            for m in matches {
                self.apply_transformation(graph, &m)?;
            }
        }
        Ok(())
    }

    /// Finds all non-overlapping matches of the LHS pattern in the graph.
    fn find_matches(&self, graph: &Graph) -> Result<Vec<Match>, String> {
        let mut all_matches = Vec::new();
        let mut used_graph_nodes = HashSet::new();

        let p_nodes = &self.lhs.nodes;
        if p_nodes.is_empty() {
            return Ok(all_matches);
        }

        let mut potential_matches = Vec::new();
        self.find_potential_matches_recursive(
            graph,
            p_nodes,
            &mut HashMap::new(),
            &mut used_graph_nodes,
            &mut potential_matches,
            0,
        )?;

        // Filter for valid matches that satisfy edge constraints
        for potential_match in potential_matches {
            if self.is_valid_match(graph, &potential_match)? {
                // Add to results and mark nodes as used
                let mut is_overlapping = false;
                for node_id in potential_match.values() {
                    if used_graph_nodes.contains(node_id) {
                        is_overlapping = true;
                        break;
                    }
                }

                if !is_overlapping {
                    for node_id in potential_match.values() {
                        used_graph_nodes.insert(node_id.clone());
                    }
                    all_matches.push(Match {
                        node_mapping: potential_match,
                    });
                }
            }
        }

        Ok(all_matches)
    }

    /// Recursively finds all possible node mappings (potential matches) using backtracking.
    fn find_potential_matches_recursive(
        &self,
        graph: &Graph,
        p_nodes: &[NodeDeclaration],
        current_mapping: &mut HashMap<String, String>,
        used_graph_nodes: &mut HashSet<String>,
        results: &mut Vec<HashMap<String, String>>,
        p_node_index: usize,
    ) -> Result<(), String> {
        if p_node_index == p_nodes.len() {
            results.push(current_mapping.clone());
            return Ok(());
        }

        let p_node = &p_nodes[p_node_index];
        let p_node_id = p_node.id.to_string();

        for g_node_id in graph.nodes.keys() {
            if !used_graph_nodes.contains(g_node_id) && !current_mapping.values().any(|v| v == g_node_id)
                && self.node_matches(graph, g_node_id, p_node)? {
                    current_mapping.insert(p_node_id.clone(), g_node_id.clone());

                    self.find_potential_matches_recursive(
                        graph,
                        p_nodes,
                        current_mapping,
                        used_graph_nodes,
                        results,
                        p_node_index + 1,
                    )?;

                    current_mapping.remove(&p_node_id); // Backtrack
                }
        }
        Ok(())
    }

    /// Checks if a potential node mapping also satisfies the edge constraints of the pattern.
    fn is_valid_match(&self, graph: &Graph, node_mapping: &HashMap<String, String>) -> Result<bool, String> {
        for p_edge in &self.lhs.edges {
            let p_source_id = p_edge.source.to_string();
            let p_target_id = p_edge.target.to_string();

            let g_source_id = node_mapping.get(&p_source_id).ok_or("Invalid LHS pattern")?;
            let g_target_id = node_mapping.get(&p_target_id).ok_or("Invalid LHS pattern")?;

            let edge_exists = graph.edges.values().any(|g_edge| {
                (g_edge.source == *g_source_id && g_edge.target == *g_target_id) ||
                (!p_edge.directed && g_edge.source == *g_target_id && g_edge.target == *g_source_id)
            });

            if !edge_exists {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Checks if a graph node matches a pattern node's criteria (type, attributes).
    fn node_matches( &self, graph: &Graph, graph_node_id: &str, p_node: &NodeDeclaration) -> Result<bool, String> {
        let g_node = graph.get_node(graph_node_id).ok_or("Internal error: Node disappeared")?;

        println!("Checking if node '{}' (type: '{}') matches pattern node '{}' (type: {:?})",
                 graph_node_id, g_node.r#type, p_node.id, p_node.node_type);

        // Check type
        if let Some(p_type_expr) = &p_node.node_type {
            let p_type_str = p_type_expr.to_string();
            if g_node.r#type != p_type_str {
                return Ok(false);
            }
        }
        // Check attributes
        for (p_key, p_val_expr) in &p_node.attributes {
            if let Some(g_val) = g_node.metadata.get(p_key) {
                if g_val == &expression_to_value(p_val_expr)? {
                    continue;
                }
            }
            return Ok(false);
        }
        println!("  Match successful!");
        Ok(true)
    }


    /// Applies the RHS transformation based on a match.
    fn apply_transformation(&self, graph: &mut Graph, m: &Match) -> Result<(), String> {
        // --- Deletion Phase ---
        let lhs_nodes: HashSet<_> = self.lhs.nodes.iter().map(|n| n.id.to_string()).collect();
        let rhs_nodes: HashSet<_> = self.rhs.nodes.iter().map(|n| n.id.to_string()).collect();
        let nodes_to_delete = lhs_nodes.difference(&rhs_nodes);
        for p_node_id in nodes_to_delete {
            if let Some(g_node_id) = m.node_mapping.get(p_node_id) {
                graph.remove_node(g_node_id);
            }
        }

        // --- Creation/Update Phase ---
        let mut rhs_node_mapping = m.node_mapping.clone();

        for p_node in &self.rhs.nodes {
            let p_node_id = p_node.id.to_string();
            let mut metadata = HashMap::new();
            for (key, val_expr) in &p_node.attributes {
                metadata.insert(key.clone(), expression_to_value(val_expr)?);
            }

            if let Some(g_node_id) = m.node_mapping.get(&p_node_id) {
                // Update existing node matched in LHS
                if let Some(node) = graph.get_node_mut(g_node_id) {
                    if let Some(p_type_expr) = &p_node.node_type {
                        let new_type = p_type_expr.to_string();
                        if !new_type.is_empty() {
                            node.r#type = new_type;
                        }
                    }
                    node.metadata.extend(metadata);
                }
            } else {
                // This is a new node declared only in the RHS.
                // Treat its ID as a literal ID in the graph.
                let new_g_node_id = p_node_id.clone();
                if graph.get_node(&new_g_node_id).is_none() {
                    let node_type = p_node.node_type.as_ref().map(|e| e.to_string()).unwrap_or_default();
                    let new_node = Node::new().with_type(node_type).with_metadata_map(metadata);
                    graph.add_node(new_g_node_id.clone(), new_node);
                }
                // Add this new/referenced node to a temporary mapping for edge creation.
                rhs_node_mapping.insert(p_node_id, new_g_node_id);
            }
        }

        for p_edge in &self.rhs.edges {
            let source_p_id = p_edge.source.to_string();
            let target_p_id = p_edge.target.to_string();

            let source_g_id = rhs_node_mapping.get(&source_p_id).ok_or(format!("RHS source node '{source_p_id}' not found in mapping"))?.clone();
            let target_g_id = rhs_node_mapping.get(&target_p_id).ok_or(format!("RHS target node '{target_p_id}' not found in mapping"))?.clone();

            // Check if an equivalent edge already exists
            let edge_exists = graph.edges.values().any(|g_edge| {
                (g_edge.source == source_g_id && g_edge.target == target_g_id && g_edge.directed == p_edge.directed) ||
                (!p_edge.directed && g_edge.source == target_g_id && g_edge.target == source_g_id)
            });

            if !edge_exists {
                let id = graph.generate_unique_edge_id("new_edge");
                graph.add_edge(id, Edge::new(source_g_id, target_g_id, p_edge.directed));
            }
        }

        Ok(())
    }
}
