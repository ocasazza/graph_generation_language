use crate::parser::{NodeDeclaration, Pattern};
use crate::types::{Edge, Graph, Node};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub lhs: Pattern,
    pub rhs: Pattern,
}

impl Rule {
    pub fn apply(&self, graph: &mut Graph, iterations: usize) -> Result<(), String> {
        for _ in 0..iterations {
            let matches = self.find_matches(graph)?;
            if matches.is_empty() {
                break;
            }

            // For rules that create new elements (nodes or edges) with fixed IDs,
            // apply only one match per iteration to avoid ID conflicts.
            // For rules that only modify existing elements, apply all matches.
            let creates_new_elements = self
                .rhs
                .nodes
                .iter()
                .any(|n| !self.lhs.nodes.iter().any(|ln| ln.id == n.id))
                || self
                    .rhs
                    .edges
                    .iter()
                    .any(|e| !self.lhs.edges.iter().any(|le| le.id == e.id));

            if creates_new_elements {
                // Apply only the first match per iteration to avoid conflicts
                if let Some(m) = matches.into_iter().next() {
                    self.apply_transformation(graph, &m)?;
                }
            } else {
                // Apply all matches for rules that only modify existing elements
                for m in matches {
                    self.apply_transformation(graph, &m)?;
                }
            }
        }

        Ok(())
    }

    fn find_matches(&self, graph: &Graph) -> Result<Vec<Match>, String> {
        let mut matches = Vec::new();
        let mut visited = HashSet::new();

        // Sort node IDs to ensure deterministic iteration order
        let mut node_ids: Vec<_> = graph.nodes.keys().collect();
        node_ids.sort();

        // For each node in the graph, try to match the LHS pattern starting from it
        for node_id in node_ids {
            if visited.contains(node_id) {
                continue;
            }

            if let Some(m) = self.match_pattern_from_node(graph, node_id, &self.lhs)? {
                // Add all matched nodes to visited set to avoid overlapping matches
                for mapped_node in m.node_mapping.values() {
                    visited.insert(mapped_node.clone());
                }
                matches.push(m);
            }
        }

        Ok(matches)
    }

    fn match_pattern_from_node(
        &self,
        graph: &Graph,
        start_node: &str,
        pattern: &Pattern,
    ) -> Result<Option<Match>, String> {
        let mut node_mapping = HashMap::new();
        let mut edge_mapping = HashMap::new();

        // Try to match the first node in the pattern to the start node
        if pattern.nodes.is_empty() {
            return Ok(Some(Match {
                node_mapping,
                edge_mapping,
            }));
        }

        let first_pattern_node = &pattern.nodes[0];
        if !self.node_matches(graph, start_node, first_pattern_node)? {
            return Ok(None);
        }

        node_mapping.insert(first_pattern_node.id.clone(), start_node.to_string());

        // Try to extend the match to the rest of the pattern
        if self.extend_match(graph, pattern, &mut node_mapping, &mut edge_mapping)? {
            Ok(Some(Match {
                node_mapping,
                edge_mapping,
            }))
        } else {
            Ok(None)
        }
    }

    fn extend_match(
        &self,
        graph: &Graph,
        pattern: &Pattern,
        node_mapping: &mut HashMap<String, String>,
        edge_mapping: &mut HashMap<String, String>,
    ) -> Result<bool, String> {
        // Match remaining nodes
        for pattern_node in pattern.nodes.iter().skip(1) {
            let mut found_match = false;

            // Sort node IDs to ensure deterministic iteration order
            let mut available_nodes: Vec<_> = graph
                .nodes
                .keys()
                .filter(|id| !node_mapping.values().any(|v| v == *id))
                .collect();
            available_nodes.sort();

            // Try each unmapped graph node
            for graph_node_id in available_nodes {
                if self.node_matches(graph, graph_node_id, pattern_node)? {
                    node_mapping.insert(pattern_node.id.clone(), graph_node_id.clone());
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                return Ok(false);
            }
        }

        // Match edges - but only if the pattern requires edges
        for pattern_edge in &pattern.edges {
            let mut found_match = false;

            // Get the mapped source and target nodes
            let source = node_mapping
                .get(&pattern_edge.source)
                .ok_or_else(|| "Invalid source node in pattern".to_string())?;
            let target = node_mapping
                .get(&pattern_edge.target)
                .ok_or_else(|| "Invalid target node in pattern".to_string())?;

            // Look for a matching edge in the graph
            for (graph_edge_id, graph_edge) in &graph.edges {
                if edge_mapping.values().any(|v| v == graph_edge_id) {
                    continue;
                }

                // Since graph edges don't store directedness, we need to handle matching differently
                // For undirected pattern edges, allow matching in either direction
                // For directed pattern edges, only match in the specified direction
                let matches = if pattern_edge.directed {
                    // For directed pattern edges, exact match required
                    graph_edge.source == *source && graph_edge.target == *target
                } else {
                    // For undirected pattern edges, either direction works
                    (graph_edge.source == *source && graph_edge.target == *target)
                        || (graph_edge.source == *target && graph_edge.target == *source)
                };

                if matches {
                    edge_mapping.insert(pattern_edge.id.clone(), graph_edge_id.clone());
                    found_match = true;
                    break;
                }
            }

            if !found_match {
                return Ok(false);
            }
        }

        // Special handling for patterns that require specific connectivity constraints
        // For single-node patterns with no edges, we need to check if isolation is required
        // by looking at the rule context (this is a heuristic based on common rule patterns)
        if pattern.edges.is_empty() && pattern.nodes.len() == 1 {
            // If the RHS is empty (deletion rule), then we want isolated nodes
            if self.rhs.nodes.is_empty() {
                let node_id = node_mapping.values().next().unwrap();
                // Check if this node has any edges in the graph
                for edge in graph.edges.values() {
                    if edge.source == *node_id || edge.target == *node_id {
                        return Ok(false); // Node is not isolated
                    }
                }
            }
            // For other single-node patterns (like replacement), don't require isolation
        }

        Ok(true)
    }

    fn node_matches(
        &self,
        graph: &Graph,
        graph_node_id: &str,
        pattern_node: &NodeDeclaration,
    ) -> Result<bool, String> {
        let graph_node = graph
            .get_node(graph_node_id)
            .ok_or_else(|| format!("Node {} not found in graph", graph_node_id))?;

        // Check node type if specified
        if let Some(ref node_type) = pattern_node.node_type {
            if graph_node.r#type != *node_type {
                return Ok(false);
            }
        }

        // Check attributes if specified
        for (key, value) in &pattern_node.attributes {
            match graph_node.metadata.get(key) {
                Some(graph_value) if graph_value == value => continue,
                _ => return Ok(false),
            }
        }

        Ok(true)
    }

    fn apply_transformation(&self, graph: &mut Graph, m: &Match) -> Result<(), String> {
        // Handle nodes from RHS pattern
        for node in &self.rhs.nodes {
            let node_id = if let Some(mapped_id) = m.node_mapping.get(&node.id) {
                // This is a preserved/updated node from LHS
                mapped_id.clone()
            } else {
                // This is a new node
                node.id.clone()
            };

            if let Some(mapped_id) = m.node_mapping.get(&node.id) {
                // This node exists in LHS, update it
                let existing_node = graph.get_node(mapped_id).unwrap().clone();
                let mut updated_node = Node::new(mapped_id.clone())
                    .with_type(existing_node.r#type.clone())
                    .with_metadata_map(existing_node.metadata.clone())
                    .with_position(existing_node.x, existing_node.y);

                // Update type if specified in RHS
                if let Some(ref node_type) = node.node_type {
                    updated_node = updated_node.with_type(node_type.clone());
                }

                // Add/update metadata from RHS
                for (key, value) in &node.attributes {
                    updated_node = updated_node.with_metadata(key.clone(), value.clone());
                }
                graph.add_node(updated_node); // This will replace the existing node
            } else {
                // This is a new node, create it
                let mut new_node = Node::new(node_id.clone());
                if let Some(ref node_type) = node.node_type {
                    new_node = new_node.with_type(node_type.clone());
                }
                for (key, value) in &node.attributes {
                    new_node = new_node.with_metadata(key.clone(), value.clone());
                }
                graph.add_node(new_node);
            }
        }

        // Remove nodes that are in LHS but not in RHS
        for (pattern_id, graph_id) in &m.node_mapping {
            if !self.rhs.nodes.iter().any(|n| &n.id == pattern_id) {
                graph.remove_node(graph_id);
            }
        }

        // Remove edges that are in LHS but not in RHS
        for (pattern_id, graph_id) in &m.edge_mapping {
            if !self.rhs.edges.iter().any(|e| &e.id == pattern_id) {
                graph.remove_edge(graph_id);
            }
        }

        // Create new edges from RHS pattern
        for edge in &self.rhs.edges {
            let source = if let Some(s) = m.node_mapping.get(&edge.source) {
                s.clone()
            } else {
                edge.source.clone()
            };

            let target = if let Some(t) = m.node_mapping.get(&edge.target) {
                t.clone()
            } else {
                edge.target.clone()
            };

            // Generate a unique edge ID
            let edge_id = if let Some(mapped_id) = m.edge_mapping.get(&edge.id) {
                // This edge exists in LHS, keep its ID
                mapped_id.clone()
            } else {
                // This is a new edge, generate a unique ID
                let base_id = if edge.id.is_empty() {
                    format!("{}_{}", source, target)
                } else {
                    edge.id.clone()
                };

                let mut counter = 0;
                let mut unique_id = base_id.clone();
                while graph.edges.contains_key(&unique_id) {
                    counter += 1;
                    unique_id = format!("{}_{}", base_id, counter);
                }
                unique_id
            };

            let mut new_edge = Edge::new(edge_id, source, target);
            for (key, value) in &edge.attributes {
                new_edge = new_edge.with_metadata(key.clone(), value.clone());
            }
            graph.add_edge(new_edge);
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Match {
    node_mapping: HashMap<String, String>, // Pattern node ID -> Graph node ID
    #[allow(dead_code)]
    edge_mapping: HashMap<String, String>, // Pattern edge ID -> Graph edge ID
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{EdgeDeclaration, NodeDeclaration, Pattern};
    use crate::types::MetadataValue;

    #[test]
    fn test_simple_rule() {
        // Create a rule that replaces a single node with two connected nodes
        let rule = Rule {
            name: "split".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "A".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "B1".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B2".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "e".to_string(),
                    source: "B1".to_string(),
                    target: "B2".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        // Create a test graph
        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));

        // Apply the rule
        rule.apply(&mut graph, 1).unwrap();

        // Check the result
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_typed_node_rule() {
        // Create a rule that matches nodes by type
        let mut type_attrs = HashMap::new();
        type_attrs.insert("type".to_string(), MetadataValue::String("A".to_string()));

        let rule = Rule {
            name: "type_match".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("A".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("B".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
        };

        // Create a test graph
        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()).with_type("A".to_string()));
        graph.add_node(Node::new("n2".to_string()).with_type("C".to_string()));

        // Apply the rule
        rule.apply(&mut graph, 1).unwrap();

        // Check that only the type A node was transformed
        assert!(graph.get_node("n1").unwrap().r#type == "B");
        assert!(graph.get_node("n2").unwrap().r#type == "C");
    }
}
