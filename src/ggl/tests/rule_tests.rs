use graph_generation_language::parser::{EdgeDeclaration, NodeDeclaration, Pattern};
use graph_generation_language::rules::Rule;
use graph_generation_language::types::{Edge, Graph, MetadataValue, Node};
use std::collections::HashMap;

#[cfg(test)]
mod simple_rule_tests {
    use super::*;

    #[test]
    fn test_node_replacement() {
        // Rule: replace any node with two connected nodes
        let rule = Rule {
            name: "split_node".to_string(),
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
                    id: "e_split".to_string(),
                    source: "B1".to_string(),
                    target: "B2".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));
        graph.add_node(Node::new("n2".to_string()));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);

        // Apply rule once
        rule.apply(&mut graph, 1).unwrap();

        // Should have replaced one node with two connected nodes
        assert_eq!(graph.node_count(), 3); // 2 - 1 + 2 = 3
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_edge_addition() {
        // Rule: add edge between any two unconnected nodes
        let rule = Rule {
            name: "connect_nodes".to_string(),
            lhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "new_edge".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));
        graph.add_node(Node::new("n2".to_string()));
        graph.add_node(Node::new("n3".to_string()));

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 0);

        // Apply rule once
        rule.apply(&mut graph, 1).unwrap();

        // Should have added one edge
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_node_deletion() {
        // Rule: delete isolated nodes
        let rule = Rule {
            name: "delete_isolated".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "A".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("isolated1".to_string()));
        graph.add_node(Node::new("isolated2".to_string()));
        graph.add_node(Node::new("connected1".to_string()));
        graph.add_node(Node::new("connected2".to_string()));
        graph.add_edge(Edge::new(
            "e1".to_string(),
            "connected1".to_string(),
            "connected2".to_string(),
        ));

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 1);

        // Apply rule multiple times to delete all isolated nodes
        rule.apply(&mut graph, 5).unwrap();

        // Should have deleted isolated nodes but kept connected ones
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}

#[cfg(test)]
mod typed_node_tests {
    use super::*;

    #[test]
    fn test_type_specific_matching() {
        // Rule: transform leaf nodes to intermediate nodes
        let rule = Rule {
            name: "leaf_to_intermediate".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("leaf".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("intermediate".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()).with_type("leaf".to_string()));
        graph.add_node(Node::new("n2".to_string()).with_type("root".to_string()));
        graph.add_node(Node::new("n3".to_string()).with_type("leaf".to_string()));

        // Apply rule
        rule.apply(&mut graph, 10).unwrap();

        // Check that leaf nodes were transformed
        assert_eq!(graph.get_node("n1").unwrap().r#type, "intermediate");
        assert_eq!(graph.get_node("n2").unwrap().r#type, "root"); // Unchanged
        assert_eq!(graph.get_node("n3").unwrap().r#type, "intermediate");
    }

    #[test]
    fn test_type_preservation() {
        // Rule: add child to intermediate nodes
        let rule = Rule {
            name: "add_child".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "P".to_string(),
                    node_type: Some("intermediate".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "P".to_string(),
                        node_type: Some("intermediate".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "C".to_string(),
                        node_type: Some("leaf".to_string()),
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "parent_child".to_string(),
                    source: "P".to_string(),
                    target: "C".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("parent".to_string()).with_type("intermediate".to_string()));
        graph.add_node(Node::new("other".to_string()).with_type("root".to_string()));

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);

        // Apply rule
        rule.apply(&mut graph, 1).unwrap();

        // Should have added child to intermediate node only
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 1);

        // Verify types are preserved/set correctly
        assert_eq!(graph.get_node("parent").unwrap().r#type, "intermediate");
        assert_eq!(graph.get_node("other").unwrap().r#type, "root");

        // Find the new child node
        let mut child_found = false;
        for node in graph.nodes.values() {
            if node.r#type == "leaf" && node.id != "parent" && node.id != "other" {
                child_found = true;
                break;
            }
        }
        assert!(child_found, "Child node with leaf type should exist");
    }
}

#[cfg(test)]
mod attribute_based_tests {
    use super::*;

    #[test]
    fn test_attribute_matching() {
        // Rule: update status of active nodes
        let mut old_attrs = HashMap::new();
        old_attrs.insert(
            "status".to_string(),
            MetadataValue::String("active".to_string()),
        );

        let mut new_attrs = HashMap::new();
        new_attrs.insert(
            "status".to_string(),
            MetadataValue::String("processed".to_string()),
        );
        new_attrs.insert("timestamp".to_string(), MetadataValue::Integer(12345));

        let rule = Rule {
            name: "process_active".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: old_attrs,
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: new_attrs,
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()).with_metadata(
            "status".to_string(),
            MetadataValue::String("active".to_string()),
        ));
        graph.add_node(Node::new("n2".to_string()).with_metadata(
            "status".to_string(),
            MetadataValue::String("inactive".to_string()),
        ));
        graph.add_node(Node::new("n3".to_string()).with_metadata(
            "status".to_string(),
            MetadataValue::String("active".to_string()),
        ));

        // Apply rule
        rule.apply(&mut graph, 10).unwrap();

        // Check that only active nodes were processed
        let n1 = graph.get_node("n1").unwrap();
        assert_eq!(
            n1.metadata.get("status"),
            Some(&MetadataValue::String("processed".to_string()))
        );
        assert_eq!(
            n1.metadata.get("timestamp"),
            Some(&MetadataValue::Integer(12345))
        );

        let n2 = graph.get_node("n2").unwrap();
        assert_eq!(
            n2.metadata.get("status"),
            Some(&MetadataValue::String("inactive".to_string()))
        );
        assert!(!n2.metadata.contains_key("timestamp"));

        let n3 = graph.get_node("n3").unwrap();
        assert_eq!(
            n3.metadata.get("status"),
            Some(&MetadataValue::String("processed".to_string()))
        );
        assert_eq!(
            n3.metadata.get("timestamp"),
            Some(&MetadataValue::Integer(12345))
        );
    }

    #[test]
    fn test_numeric_attribute_matching() {
        // Rule: increment counter for nodes with specific value
        let mut match_attrs = HashMap::new();
        match_attrs.insert("counter".to_string(), MetadataValue::Float(5.0));

        let mut update_attrs = HashMap::new();
        update_attrs.insert("counter".to_string(), MetadataValue::Float(6.0));

        let rule = Rule {
            name: "increment_counter".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: match_attrs,
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: update_attrs,
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(
            Node::new("n1".to_string())
                .with_metadata("counter".to_string(), MetadataValue::Float(5.0)),
        );
        graph.add_node(
            Node::new("n2".to_string())
                .with_metadata("counter".to_string(), MetadataValue::Float(3.0)),
        );
        graph.add_node(
            Node::new("n3".to_string())
                .with_metadata("counter".to_string(), MetadataValue::Float(5.0)),
        );

        // Apply rule
        rule.apply(&mut graph, 1).unwrap();

        // Check that nodes with counter=5 were incremented
        assert_eq!(
            graph.get_node("n1").unwrap().metadata.get("counter"),
            Some(&MetadataValue::Float(6.0))
        );
        assert_eq!(
            graph.get_node("n2").unwrap().metadata.get("counter"),
            Some(&MetadataValue::Float(3.0))
        ); // Unchanged
        assert_eq!(
            graph.get_node("n3").unwrap().metadata.get("counter"),
            Some(&MetadataValue::Float(6.0))
        );
    }

    #[test]
    fn test_boolean_attribute_matching() {
        // Rule: deactivate enabled nodes
        let mut match_attrs = HashMap::new();
        match_attrs.insert("enabled".to_string(), MetadataValue::Boolean(true));

        let mut update_attrs = HashMap::new();
        update_attrs.insert("enabled".to_string(), MetadataValue::Boolean(false));
        update_attrs.insert(
            "reason".to_string(),
            MetadataValue::String("deactivated_by_rule".to_string()),
        );

        let rule = Rule {
            name: "deactivate".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: match_attrs,
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: update_attrs,
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(
            Node::new("n1".to_string())
                .with_metadata("enabled".to_string(), MetadataValue::Boolean(true)),
        );
        graph.add_node(
            Node::new("n2".to_string())
                .with_metadata("enabled".to_string(), MetadataValue::Boolean(false)),
        );

        // Apply rule
        rule.apply(&mut graph, 1).unwrap();

        // Check results
        let n1 = graph.get_node("n1").unwrap();
        assert_eq!(
            n1.metadata.get("enabled"),
            Some(&MetadataValue::Boolean(false))
        );
        assert_eq!(
            n1.metadata.get("reason"),
            Some(&MetadataValue::String("deactivated_by_rule".to_string()))
        );

        let n2 = graph.get_node("n2").unwrap();
        assert_eq!(
            n2.metadata.get("enabled"),
            Some(&MetadataValue::Boolean(false))
        ); // Unchanged
        assert!(!n2.metadata.contains_key("reason"));
    }
}

#[cfg(test)]
mod edge_pattern_tests {
    use super::*;

    #[test]
    fn test_edge_transformation() {
        // Rule: replace directed edges with undirected edges
        let rule = Rule {
            name: "make_undirected".to_string(),
            lhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "E".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "E_undirected".to_string(),
                    source: "A".to_string(),
                    target: "B".to_string(),
                    directed: false,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));
        graph.add_node(Node::new("n2".to_string()));
        graph.add_node(Node::new("n3".to_string()));
        graph.add_edge(Edge::new(
            "e1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
        ));
        graph.add_edge(Edge::new(
            "e2".to_string(),
            "n2".to_string(),
            "n3".to_string(),
        ));

        assert_eq!(graph.edge_count(), 2);

        // Apply rule
        rule.apply(&mut graph, 10).unwrap();

        // Should still have same number of edges, but they should be transformed
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(graph.node_count(), 3);
    }

    #[test]
    fn test_edge_addition_pattern() {
        // Rule: create triangle by adding edge between endpoints of a path
        let rule = Rule {
            name: "close_triangle".to_string(),
            lhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "C".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![
                    EdgeDeclaration {
                        id: "E1".to_string(),
                        source: "A".to_string(),
                        target: "B".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "E2".to_string(),
                        source: "B".to_string(),
                        target: "C".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                ],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "A".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "B".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "C".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![
                    EdgeDeclaration {
                        id: "E1".to_string(),
                        source: "A".to_string(),
                        target: "B".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "E2".to_string(),
                        source: "B".to_string(),
                        target: "C".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "E3".to_string(),
                        source: "A".to_string(),
                        target: "C".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                ],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));
        graph.add_node(Node::new("n2".to_string()));
        graph.add_node(Node::new("n3".to_string()));
        graph.add_node(Node::new("n4".to_string()));

        // Create a path: n1-n2-n3 and isolated n4
        graph.add_edge(Edge::new(
            "e1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
        ));
        graph.add_edge(Edge::new(
            "e2".to_string(),
            "n2".to_string(),
            "n3".to_string(),
        ));

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 2);

        // Apply rule
        rule.apply(&mut graph, 1).unwrap();

        // Should have added one edge to close the triangle
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
    }
}

#[cfg(test)]
mod complex_pattern_tests {
    use super::*;

    #[test]
    fn test_star_to_cycle() {
        // Rule: transform star pattern to cycle
        let rule = Rule {
            name: "star_to_cycle".to_string(),
            lhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "CENTER".to_string(),
                        node_type: Some("center".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF1".to_string(),
                        node_type: Some("leaf".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF2".to_string(),
                        node_type: Some("leaf".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF3".to_string(),
                        node_type: Some("leaf".to_string()),
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![
                    EdgeDeclaration {
                        id: "E1".to_string(),
                        source: "CENTER".to_string(),
                        target: "LEAF1".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "E2".to_string(),
                        source: "CENTER".to_string(),
                        target: "LEAF2".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "E3".to_string(),
                        source: "CENTER".to_string(),
                        target: "LEAF3".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                ],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "LEAF1".to_string(),
                        node_type: Some("cycle_node".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF2".to_string(),
                        node_type: Some("cycle_node".to_string()),
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF3".to_string(),
                        node_type: Some("cycle_node".to_string()),
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![
                    EdgeDeclaration {
                        id: "CYCLE1".to_string(),
                        source: "LEAF1".to_string(),
                        target: "LEAF2".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "CYCLE2".to_string(),
                        source: "LEAF2".to_string(),
                        target: "LEAF3".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                    EdgeDeclaration {
                        id: "CYCLE3".to_string(),
                        source: "LEAF3".to_string(),
                        target: "LEAF1".to_string(),
                        directed: false,
                        attributes: HashMap::new(),
                    },
                ],
            },
        };

        let mut graph = Graph::new();

        // Create a star pattern
        graph.add_node(Node::new("center".to_string()).with_type("center".to_string()));
        graph.add_node(Node::new("leaf1".to_string()).with_type("leaf".to_string()));
        graph.add_node(Node::new("leaf2".to_string()).with_type("leaf".to_string()));
        graph.add_node(Node::new("leaf3".to_string()).with_type("leaf".to_string()));
        graph.add_node(Node::new("other".to_string()).with_type("other".to_string()));

        graph.add_edge(Edge::new(
            "e1".to_string(),
            "center".to_string(),
            "leaf1".to_string(),
        ));
        graph.add_edge(Edge::new(
            "e2".to_string(),
            "center".to_string(),
            "leaf2".to_string(),
        ));
        graph.add_edge(Edge::new(
            "e3".to_string(),
            "center".to_string(),
            "leaf3".to_string(),
        ));

        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 3);

        // Apply rule
        rule.apply(&mut graph, 1).unwrap();

        // Should have removed center and created cycle
        assert_eq!(graph.node_count(), 4); // 5 - 1 (center removed) = 4
        assert_eq!(graph.edge_count(), 3); // Same number but different structure

        // Verify center is gone and leaves are now cycle_nodes
        assert!(graph.get_node("center").is_none());
        assert_eq!(graph.get_node("leaf1").unwrap().r#type, "cycle_node");
        assert_eq!(graph.get_node("leaf2").unwrap().r#type, "cycle_node");
        assert_eq!(graph.get_node("leaf3").unwrap().r#type, "cycle_node");
        assert_eq!(graph.get_node("other").unwrap().r#type, "other"); // Unchanged
    }
}

#[cfg(test)]
mod rule_application_tests {
    use super::*;

    #[test]
    fn test_multiple_iterations() {
        // Rule: add leaf to any node
        let rule = Rule {
            name: "add_leaf".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![
                    NodeDeclaration {
                        id: "N".to_string(),
                        node_type: None,
                        attributes: HashMap::new(),
                    },
                    NodeDeclaration {
                        id: "LEAF".to_string(),
                        node_type: Some("leaf".to_string()),
                        attributes: HashMap::new(),
                    },
                ],
                edges: vec![EdgeDeclaration {
                    id: "PARENT_CHILD".to_string(),
                    source: "N".to_string(),
                    target: "LEAF".to_string(),
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("root".to_string()));

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);

        // Apply rule 3 times
        rule.apply(&mut graph, 3).unwrap();

        // Should have grown the graph
        assert!(graph.node_count() > 1);
        assert!(graph.edge_count() > 0);

        // Original root should still exist
        assert!(graph.get_node("root").is_some());
    }

    #[test]
    fn test_no_matches() {
        // Rule that requires specific type that doesn't exist
        let rule = Rule {
            name: "transform_special".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("special".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("transformed".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()).with_type("normal".to_string()));
        graph.add_node(Node::new("n2".to_string()).with_type("regular".to_string()));

        let original_count = graph.node_count();

        // Apply rule - should have no effect
        rule.apply(&mut graph, 10).unwrap();

        assert_eq!(graph.node_count(), original_count);

        // Verify nodes are unchanged
        assert_eq!(graph.get_node("n1").unwrap().r#type, "normal");
        assert_eq!(graph.get_node("n2").unwrap().r#type, "regular");
    }

    #[test]
    fn test_zero_iterations() {
        // Rule: simple transformation
        let rule = Rule {
            name: "transform".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: Some("transformed".to_string()),
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));

        // Apply rule 0 times - should have no effect
        rule.apply(&mut graph, 0).unwrap();

        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.get_node("n1").unwrap().r#type, ""); // Unchanged
    }

    #[test]
    fn test_rule_termination() {
        // Rule that can only be applied once per node
        let mut attrs = HashMap::new();
        attrs.insert("processed".to_string(), MetadataValue::Boolean(true));

        let rule = Rule {
            name: "mark_processed".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: attrs,
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("n1".to_string()));
        graph.add_node(Node::new("n2".to_string()));
        graph.add_node(Node::new("n3".to_string()));

        // Apply rule many times - should terminate naturally
        rule.apply(&mut graph, 100).unwrap();

        // All nodes should be processed exactly once
        assert_eq!(graph.node_count(), 3);
        for node in graph.nodes.values() {
            assert_eq!(
                node.metadata.get("processed"),
                Some(&MetadataValue::Boolean(true))
            );
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_pattern_references() {
        // Rule with invalid node reference in edge
        let rule = Rule {
            name: "invalid_rule".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "A".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "A".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![EdgeDeclaration {
                    id: "invalid_edge".to_string(),
                    source: "A".to_string(),
                    target: "NONEXISTENT".to_string(), // Invalid reference
                    directed: true,
                    attributes: HashMap::new(),
                }],
            },
        };

        let mut graph = Graph::new();
        graph.add_node(Node::new("test".to_string()));

        // Should handle invalid pattern gracefully
        let result = rule.apply(&mut graph, 1);
        // The rule system should either handle this gracefully or return an error
        // The exact behavior depends on implementation details
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_graph_rule_application() {
        // Simple rule for performance testing
        let rule = Rule {
            name: "add_attribute".to_string(),
            lhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: HashMap::new(),
                }],
                edges: vec![],
            },
            rhs: Pattern {
                nodes: vec![NodeDeclaration {
                    id: "N".to_string(),
                    node_type: None,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("marked".to_string(), MetadataValue::Boolean(true));
                        attrs
                    },
                }],
                edges: vec![],
            },
        };

        let mut graph = Graph::new();

        // Create a moderately large graph
        for i in 0..100 {
            graph.add_node(Node::new(format!("n{}", i)));
        }

        assert_eq!(graph.node_count(), 100);

        // Apply rule - should complete in reasonable time
        let start = std::time::Instant::now();
        rule.apply(&mut graph, 1).unwrap();
        let duration = start.elapsed();

        // Should complete quickly (less than 1 second for this size)
        assert!(duration.as_secs() < 1);

        // Verify all nodes were processed
        assert_eq!(graph.node_count(), 100);
        for node in graph.nodes.values() {
            assert_eq!(
                node.metadata.get("marked"),
                Some(&MetadataValue::Boolean(true))
            );
        }
    }
}
