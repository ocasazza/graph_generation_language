//! Transformation rule engine for graph manipulation.

use crate::parser::{NodeDeclaration, Pattern};
use crate::types::{Edge, Graph, Node};
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

impl Rule {
    /// Applies the rule to the graph for a specified number of iterations.
    pub fn apply(&self, graph: &mut Graph, iterations: usize) -> Result<(), String> {
        for _ in 0..iterations {
            let matches = self.find_matches(graph)?;
            if matches.is_empty() {
                break; // No more matches found, stop applying.
            }

            // A simple strategy: apply only the first found match per iteration
            // to avoid complex interactions between overlapping matches.
            if let Some(first_match) = matches.into_iter().next() {
                self.apply_transformation(graph, &first_match)?;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Finds all non-overlapping matches of the LHS pattern in the graph.
    fn find_matches(&self, graph: &Graph) -> Result<Vec<Match>, String> {
        let mut all_matches = Vec::new();
        let mut matched_graph_nodes = HashSet::new();

        let graph_node_ids: Vec<_> = graph.nodes.keys().cloned().collect();

        for node_id in &graph_node_ids {
            if matched_graph_nodes.contains(node_id) {
                continue; // Skip nodes that are already part of a match
            }

            if let Some(m) = self.match_pattern_from_node(graph, node_id, &matched_graph_nodes)? {
                for matched_id in m.node_mapping.values() {
                    matched_graph_nodes.insert(matched_id.clone());
                }
                all_matches.push(m);
            }
        }
        Ok(all_matches)
    }

    /// Tries to match the LHS pattern starting from a specific node.
    fn match_pattern_from_node(
        &self,
        graph: &Graph,
        start_node_id: &str,
        globally_matched: &HashSet<String>,
    ) -> Result<Option<Match>, String> {
        let mut p_nodes = self.lhs.nodes.clone();
        if p_nodes.is_empty() {
            return Ok(None);
        }

        let mut node_mapping = HashMap::new();
        let mut edge_mapping = HashMap::new(); // This is used locally within the matching logic

        // Attempt to match the first pattern node to the start_node_id
        let p_start_node = p_nodes.remove(0);
        if !self.node_matches(graph, start_node_id, &p_start_node)? {
            return Ok(None);
        }

        // The ID in the pattern is a placeholder/variable name
        let p_start_node_id = p_start_node.id.to_string(); // In rules, IDs are identifiers
        node_mapping.insert(p_start_node_id, start_node_id.to_string());

        let mut available_graph_nodes: Vec<_> = graph
            .nodes
            .keys()
            .filter(|id| *id != start_node_id && !globally_matched.contains(*id))
            .cloned()
            .collect();

        if self.extend_match(graph, &mut p_nodes, &mut available_graph_nodes, &mut node_mapping, &mut edge_mapping)? {
            Ok(Some(Match { node_mapping }))
        } else {
            Ok(None)
        }
    }

    /// Recursively extends a partial match. (Simplified to iterative for this version)
    fn extend_match(
         &self,
        graph: &Graph,
        p_nodes_to_match: &mut Vec<NodeDeclaration>,
        available_graph_nodes: &mut Vec<String>,
        node_mapping: &mut HashMap<String, String>,
        edge_mapping: &mut HashMap<String, String>,
    ) -> Result<bool, String> {
        // This is a simplified matching algorithm. A full implementation would use backtracking.
        // Match remaining nodes
        for p_node in p_nodes_to_match {
            let p_node_id = p_node.id.to_string();
            let found_node_match = available_graph_nodes.iter().position(|graph_node_id| {
                self.node_matches(graph, graph_node_id, p_node).unwrap_or(false)
            });

            if let Some(index) = found_node_match {
                let graph_node_id = available_graph_nodes.remove(index);
                node_mapping.insert(p_node_id, graph_node_id);
            } else {
                return Ok(false); // Could not find a match for a required pattern node
            }
        }

        // Match edges
        for p_edge in &self.lhs.edges {
            let p_source_id = p_edge.source.to_string();
            let p_target_id = p_edge.target.to_string();

            let g_source_id = node_mapping.get(&p_source_id).ok_or("Invalid LHS pattern")?;
            let g_target_id = node_mapping.get(&p_target_id).ok_or("Invalid LHS pattern")?;

            let found_edge = graph.edges.iter().find(|(g_edge_id, g_edge)| {
                !edge_mapping.values().any(|v| v == *g_edge_id) &&
                g_edge.directed == p_edge.directed &&
                ((g_edge.source == *g_source_id && g_edge.target == *g_target_id) ||
                 (!p_edge.directed && g_edge.source == *g_target_id && g_edge.target == *g_source_id))
            });

            if let Some((g_edge_id, _)) = found_edge {
                edge_mapping.insert(p_edge.id.as_ref().map(|id| id.to_string()).unwrap_or_default(), g_edge_id.clone());
            } else {
                return Ok(false); // Could not find a matching edge
            }
        }

        Ok(true)
    }

    /// Checks if a graph node matches a pattern node's criteria (type, attributes).
    fn node_matches( &self, graph: &Graph, graph_node_id: &str, p_node: &NodeDeclaration) -> Result<bool, String> {
        let g_node = graph.get_node(graph_node_id).ok_or("Internal error: Node disappeared")?;

        // Check type
        if let Some(p_type_expr) = &p_node.node_type {
            if g_node.r#type != p_type_expr.to_string() { // Assumes type is a literal string in pattern
                return Ok(false);
            }
        }
        // Check attributes
        for (p_key, p_val_expr) in &p_node.attributes {
            if g_node.metadata.get(p_key).is_some_and(|g_val| g_val == &p_val_expr.to_string().into()) {
                continue;
            }
            return Ok(false);
        }
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
        for p_node in &self.rhs.nodes {
            let p_node_id = p_node.id.to_string();
            let mut metadata = HashMap::new();
            for (key, val_expr) in &p_node.attributes {
                 // In a full implementation, expressions in RHS would be evaluated.
                 // For now, we assume they are literals.
                metadata.insert(key.clone(), val_expr.to_string().into());
            }

            if let Some(g_node_id) = m.node_mapping.get(&p_node_id) {
                // Update existing node
                if let Some(node) = graph.get_node_mut(g_node_id) {
                    if let Some(p_type_expr) = &p_node.node_type {
                        node.r#type = p_type_expr.to_string();
                    }
                    node.metadata.extend(metadata);
                }
            } else {
                // Create new node
                let node_type = p_node.node_type.as_ref().map(|e| e.to_string()).unwrap_or_default();
                let new_node = Node::new(p_node_id).with_type(node_type).with_metadata_map(metadata);
                graph.add_node(new_node);
            }
        }

        for p_edge in &self.rhs.edges {
             let source = m.node_mapping.get(&p_edge.source.to_string()).unwrap_or(&p_edge.source.to_string()).clone();
             let target = m.node_mapping.get(&p_edge.target.to_string()).unwrap_or(&p_edge.target.to_string()).clone();
             let id = p_edge.id.as_ref().map(|e| e.to_string()).unwrap_or_else(|| format!("new_edge_{source}_{target}"));
             graph.add_edge(Edge::new(id, source, target, p_edge.directed));
        }

        Ok(())
    }
}
