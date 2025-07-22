use graph_generation_language::GGLEngine;
use serde_json::Value;

#[cfg(test)]
mod basic_integration_tests {
    use super::*;

    #[test]
    fn test_simple_node_declaration() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node alice :person [name="Alice", age=30];
                node bob :person [name="Bob", age=25];
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(
            result.is_ok(),
            "Failed to process simple node declarations: {:?}",
            result.err()
        );

        let json_str = result.unwrap();
        println!("Generated JSON: {json_str}");
        let graph: Value = serde_json::from_str(&json_str).unwrap();
        println!(
            "Alice age value: {:?}",
            graph["nodes"]["alice"]["metadata"]["age"]
        );

        // Verify nodes exist
        let nodes = &graph["nodes"];
        assert!(nodes.is_object());
        assert!(nodes["alice"].is_object());
        assert!(nodes["bob"].is_object());

        // Verify node properties
        assert_eq!(nodes["alice"]["type"], "person");
        assert_eq!(nodes["alice"]["metadata"]["name"], "Alice");
        assert_eq!(nodes["alice"]["metadata"]["age"], 30);

        assert_eq!(nodes["bob"]["type"], "person");
        assert_eq!(nodes["bob"]["metadata"]["name"], "Bob");
        assert_eq!(nodes["bob"]["metadata"]["age"], 25);
    }

    #[test]
    fn test_simple_edge_declaration() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node alice;
                node bob;
                edge friendship: alice -- bob [strength=0.8];
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        if result.is_err() {
            println!("Error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());

        let json_str = result.unwrap();
        println!("Generated JSON: {json_str}");
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Verify nodes and edges exist
        assert!(graph["nodes"]["alice"].is_object());
        assert!(graph["nodes"]["bob"].is_object());
        assert!(graph["edges"]["friendship"].is_object());

        // Verify edge properties
        let edge = &graph["edges"]["friendship"];
        assert_eq!(edge["source"], "alice");
        assert_eq!(edge["target"], "bob");
        assert_eq!(edge["metadata"]["strength"], 0.8);
    }

    #[test]
    fn test_directed_vs_undirected_edges() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a;
                node b;
                node c;
                node d;
                edge directed: a -> b;
                edge undirected: c -- d;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        if result.is_err() {
            println!("Error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Both edges should exist
        assert!(graph["edges"]["directed"].is_object());
        assert!(graph["edges"]["undirected"].is_object());
    }

    #[test]
    fn test_empty_graph() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph empty {
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        if result.is_err() {
            println!("Error: {:?}", result.as_ref().err());
        }
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have empty nodes and edges objects
        assert!(graph["nodes"].is_object());
        assert!(graph["edges"].is_object());
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 0);
        assert_eq!(graph["edges"].as_object().unwrap().len(), 0);
    }
}

#[cfg(test)]
mod generator_integration_tests {
    use super::*;

    #[test]
    fn test_complete_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate complete {
                    nodes: 4;
                    prefix: "node";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 4 nodes
        let nodes = graph["nodes"].as_object().unwrap();
        assert_eq!(nodes.len(), 4);

        // Should have 6 edges (complete graph: n*(n-1)/2)
        let edges = graph["edges"].as_object().unwrap();
        assert_eq!(edges.len(), 6);

        // Verify node names
        assert!(nodes.contains_key("node0"));
        assert!(nodes.contains_key("node1"));
        assert!(nodes.contains_key("node2"));
        assert!(nodes.contains_key("node3"));
    }

    #[test]
    fn test_path_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate path {
                    nodes: 5;
                    prefix: "step";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 5 nodes and 4 edges
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 5);
        assert_eq!(graph["edges"].as_object().unwrap().len(), 4);

        // Verify node names with custom prefix
        let nodes = graph["nodes"].as_object().unwrap();
        assert!(nodes.contains_key("step0"));
        assert!(nodes.contains_key("step4"));
    }

    #[test]
    fn test_grid_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate grid {
                    rows: 3;
                    cols: 3;
                    prefix: "cell";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 9 nodes (3x3 grid)
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 9);

        // Should have 12 edges ((rows-1)*cols + rows*(cols-1))
        assert_eq!(graph["edges"].as_object().unwrap().len(), 12);

        // Verify grid node naming
        let nodes = graph["nodes"].as_object().unwrap();
        assert!(nodes.contains_key("cell0_0"));
        assert!(nodes.contains_key("cell2_2"));
    }

    #[test]
    fn test_star_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate star {
                    nodes: 6;
                    prefix: "vertex";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 6 nodes and 5 edges
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 6);
        assert_eq!(graph["edges"].as_object().unwrap().len(), 5);
    }

    #[test]
    fn test_tree_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate tree {
                    branching: 2;
                    depth: 3;
                    prefix: "node";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Binary tree of depth 3: 1 + 2 + 4 = 7 nodes
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 7);

        // Tree has n-1 edges
        assert_eq!(graph["edges"].as_object().unwrap().len(), 6);
    }

    #[test]
    fn test_barabasi_albert_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate barabasi_albert {
                    nodes: 10;
                    edges_per_node: 2;
                    prefix: "ba";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have 10 nodes
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 10);

        // Should have specific number of edges based on BA model
        let edge_count = graph["edges"].as_object().unwrap().len();
        assert!(edge_count > 0);
        assert!(edge_count <= 45); // Maximum for 10 nodes
    }

    #[test]
    fn test_invalid_generator() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate nonexistent {
                    nodes: 5;
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());

        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Unknown generator"));
    }

    #[test]
    fn test_generator_missing_params() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate complete {
                    // Missing nodes parameter
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod mixed_content_tests {
    use super::*;

    #[test]
    fn test_manual_and_generated_content() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph social_network {
                // Manual nodes
                node alice :person [name="Alice", age=30];
                node bob :person [name="Bob", age=25];

                // Manual edge
                edge friendship: alice -- bob [strength=0.9];

                // Generated content
                generate complete {
                    nodes: 3;
                    prefix: "user";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have manual nodes + generated nodes
        assert!(nodes.len() >= 5); // 2 manual + 3 generated

        // Should have manual edge + generated edges
        assert!(edges.len() >= 4); // 1 manual + 3 generated (complete graph)

        // Verify manual content exists
        assert!(nodes.contains_key("alice"));
        assert!(nodes.contains_key("bob"));
        assert!(edges.contains_key("friendship"));

        // Verify generated content exists
        assert!(nodes.contains_key("user0"));
        assert!(nodes.contains_key("user1"));
        assert!(nodes.contains_key("user2"));
    }

    #[test]
    fn test_multiple_generators() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph multi_gen {
                generate path {
                    nodes: 3;
                    prefix: "path";
                }

                generate star {
                    nodes: 4;
                    prefix: "star";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Should have nodes from both generators
        assert_eq!(nodes.len(), 7); // 3 from path + 4 from star

        // Verify path nodes
        assert!(nodes.contains_key("path0"));
        assert!(nodes.contains_key("path1"));
        assert!(nodes.contains_key("path2"));

        // Verify star nodes
        assert!(nodes.contains_key("star0"));
        assert!(nodes.contains_key("star1"));
        assert!(nodes.contains_key("star2"));
        assert!(nodes.contains_key("star3"));
    }

    #[test]
    fn test_complex_attributes() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph complex {
                node person [
                    name="John Doe",
                    age=35,
                    active=true,
                    score=98.5,
                    status="verified"
                ];

                node company [
                    name="Tech Corp",
                    employees=1000,
                    public=false,
                    revenue=50000000.0
                ];

                edge employment: person -> company [
                    role="Software Engineer",
                    salary=120000,
                    remote=true,
                    start_date="2023-01-15"
                ];
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Verify complex node attributes
        let person = &graph["nodes"]["person"]["metadata"];
        assert_eq!(person["name"], "John Doe");
        assert_eq!(person["age"], 35);
        assert_eq!(person["active"], true);
        assert_eq!(person["score"], 98.5);
        assert_eq!(person["status"], "verified");

        let company = &graph["nodes"]["company"]["metadata"];
        assert_eq!(company["name"], "Tech Corp");
        assert_eq!(company["employees"], 1000);
        assert_eq!(company["public"], false);
        assert_eq!(company["revenue"], 50000000.0);

        // Verify complex edge attributes
        let employment = &graph["edges"]["employment"]["metadata"];
        assert_eq!(employment["role"], "Software Engineer");
        assert_eq!(employment["salary"], 120000);
        assert_eq!(employment["remote"], true);
        assert_eq!(employment["start_date"], "2023-01-15");
    }
}

#[cfg(test)]
mod rule_integration_tests {
    use super::*;

    #[test]
    fn test_simple_rule_application() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a :leaf;
                node b :leaf;
                node c :root;

                rule promote_leaf {
                    lhs { node N :leaf; }
                    rhs { node N :intermediate; }
                }

                apply promote_leaf 5 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Leaf nodes should be promoted to intermediate
        assert_eq!(nodes["a"]["type"], "intermediate");
        assert_eq!(nodes["b"]["type"], "intermediate");
        assert_eq!(nodes["c"]["type"], "root"); // Unchanged
    }

    #[test]
    fn test_rule_with_edge_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a;
                node b;
                node c;
                edge e1: a -> b;
                edge e2: b -> c;

                rule close_triangle {
                    lhs {
                        node A;
                        node B;
                        node C;
                        edge: A -> B;
                        edge: B -> C;
                    }
                    rhs {
                        node A;
                        node B;
                        node C;
                        edge: A -> B;
                        edge: B -> C;
                        edge: A -> C;
                    }
                }

                apply close_triangle 1 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Should have added one edge to close the triangle
        let edges = graph["edges"].as_object().unwrap();
        assert_eq!(edges.len(), 3); // Original 2 + 1 new
    }

    #[test]
    fn test_generator_with_rules() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate path {
                    nodes: 4;
                    prefix: "node";
                }

                rule add_metadata {
                    lhs { node N; }
                    rhs { node N [processed=true]; }
                }

                apply add_metadata 10 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // All generated nodes should have been processed by the rule
        for (_, node) in nodes {
            assert_eq!(node["metadata"]["processed"], true);
        }
    }

    #[test]
    fn test_multiple_rule_applications() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node root;

                rule add_child {
                    lhs { node P; }
                    rhs {
                        node P;
                        node C :child;
                        edge: P -> C;
                    }
                }

                apply add_child 3 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have grown the graph
        assert!(nodes.len() > 1);
        assert!(!edges.is_empty());

        // Original root should still exist
        assert!(nodes.contains_key("root"));
    }

    #[test]
    fn test_rule_with_no_matches() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a :normal;
                node b :regular;

                rule transform_special {
                    lhs { node N :special; }
                    rhs { node N :transformed; }
                }

                apply transform_special 5 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Nodes should be unchanged since rule doesn't match
        assert_eq!(nodes["a"]["type"], "normal");
        assert_eq!(nodes["b"]["type"], "regular");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_syntax_errors() {
        let mut engine = GGLEngine::new();

        let invalid_programs = vec![
            "invalid syntax",
            "graph { node }",                       // Missing node ID
            "graph { edge: -> }",                   // Missing source/target
            "graph { node n [invalid=] }",          // Missing attribute value
            "graph { apply nonexistent 5 times; }", // Rule doesn't exist
        ];

        for program in invalid_programs {
            let result = engine.generate_from_ggl(program);
            assert!(result.is_err(), "Expected error for program: {program}");
        }
    }

    #[test]
    fn test_missing_semicolons() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a
                node b
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_attribute_values() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node n [value=12.34.56];
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_rule_application() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                node a;
                apply unknown_rule 1 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());

        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Unknown rule"));
    }

    #[test]
    fn test_generator_parameter_errors() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                generate barabasi_albert {
                    nodes: 3;
                    edges_per_node: 5;  // Invalid: m >= n
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod real_world_scenarios {
    use super::*;

    #[test]
    fn test_social_network_scenario() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph social_network {
                // Core users
                node alice :person [name="Alice", age=30, location="NYC"];
                node bob :person [name="Bob", age=25, location="SF"];
                node charlie :person [name="Charlie", age=35, location="LA"];

                // Friendships
                edge f1: alice -- bob [type="friendship", strength=0.8];
                edge f2: bob -- charlie [type="friendship", strength=0.6];

                // Generate additional users
                generate complete {
                    nodes: 5;
                    prefix: "user";
                }

                // Add metadata to generated users
                rule add_user_metadata {
                    lhs { node U; }
                    rhs { node U [active=true, joined="2024"]; }
                }

                apply add_user_metadata 10 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have core users + generated users
        assert!(nodes.len() >= 8);

        // Should have friendships + generated edges
        assert!(edges.len() >= 12);

        // Verify core users exist with proper attributes
        assert_eq!(nodes["alice"]["metadata"]["name"], "Alice");
        assert_eq!(nodes["alice"]["metadata"]["location"], "NYC");

        // Verify generated users have metadata from rule
        assert_eq!(nodes["user0"]["metadata"]["active"], true);
        assert_eq!(nodes["user0"]["metadata"]["joined"], "2024");
    }

    #[test]
    fn test_hierarchical_organization() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph organization {
                // Root
                node ceo :executive [title="CEO", level=1];

                // Generate tree structure
                generate tree {
                    branching: 3;
                    depth: 3;
                    prefix: "emp";
                }

                // Assign roles based on tree structure
                rule assign_manager_role {
                    lhs {
                        node M;
                        node S;
                        edge: M -> S;
                    }
                    rhs {
                        node M :manager;
                        node S :employee;
                        edge: M -> S [type="reports_to"];
                    }
                }

                apply assign_manager_role 20 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have hierarchical structure
        assert!(nodes.len() > 5);
        assert!(!edges.is_empty());

        // CEO should still exist
        assert!(nodes.contains_key("ceo"));
        assert_eq!(nodes["ceo"]["type"], "executive");
    }

    #[test]
    fn test_infrastructure_network() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph infrastructure {
                // Core infrastructure
                node datacenter :facility [location="primary", capacity=1000];
                node backup_dc :facility [location="secondary", capacity=500];

                // Generate server grid
                generate grid {
                    rows: 3;
                    cols: 4;
                    prefix: "server";
                }

                // Connect servers to datacenters
                rule connect_to_datacenter {
                    lhs { node S :default; }
                    rhs {
                        node S :server [status="active"];
                        node datacenter :facility;
                        edge: S -> datacenter [type="hosted_in"];
                    }
                }

                apply connect_to_datacenter 1 times;
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have infrastructure + servers
        assert!(nodes.len() >= 14); // 2 datacenters + 12 servers

        // Should have grid connections + datacenter connections
        assert!(edges.len() > 12);

        // Verify infrastructure nodes
        assert!(nodes.contains_key("datacenter"));
        assert!(nodes.contains_key("backup_dc"));
        assert_eq!(nodes["datacenter"]["type"], "facility");
    }
}

#[cfg(test)]
mod conditional_statement_integration_tests {
    use super::*;

    #[test]
    fn test_basic_conditional_execution() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                let size = 3;
                for i in 0..size {
                    if i < 2 {
                        node "conditional_node_{i}";
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute conditional statements: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Should only create 2 nodes (i=0, i=1), not 3
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains_key("conditional_node_0"));
        assert!(nodes.contains_key("conditional_node_1"));
        assert!(!nodes.contains_key("conditional_node_2"));
    }

    #[test]
    fn test_conditional_edge_creation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph mesh {
                let rows = 3;
                let cols = 3;

                // Create nodes
                for i in 0..rows {
                    for j in 0..cols {
                        node "n{i}_{j}";
                    }
                }

                // Create horizontal edges with conditionals
                for i in 0..rows {
                    for j in 0..cols {
                        if j + 1 < cols {
                            edge: "n{i}_{j}" -> "n{i}_{j+1}" [direction="horizontal"];
                        }
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute conditional edge creation: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have 9 nodes (3x3 grid)
        assert_eq!(nodes.len(), 9);

        // Should have 6 horizontal edges (3 rows * 2 edges per row)
        assert_eq!(edges.len(), 6);

        // Verify specific nodes exist
        assert!(nodes.contains_key("n0_0"));
        assert!(nodes.contains_key("n2_2"));

        // Verify edge attributes
        for (_, edge) in edges {
            assert_eq!(edge["metadata"]["direction"], "horizontal");
        }
    }

    #[test]
    fn test_arithmetic_in_conditions() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                let threshold = 5;
                for i in 0..10 {
                    if i * 2 < threshold {
                        node "below_threshold_{i}";
                    }
                    if i + 3 > threshold {
                        node "above_threshold_{i}";
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute arithmetic in conditions: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // i*2 < 5: i=0,1,2 (2*0=0<5, 2*1=2<5, 2*2=4<5)
        assert!(nodes.contains_key("below_threshold_0"));
        assert!(nodes.contains_key("below_threshold_1"));
        assert!(nodes.contains_key("below_threshold_2"));
        assert!(!nodes.contains_key("below_threshold_3")); // 2*3=6 >= 5

        // i+3 > 5: i=3,4,5,6,7,8,9 (3+3=6>5, etc.)
        assert!(nodes.contains_key("above_threshold_3"));
        assert!(nodes.contains_key("above_threshold_9"));
        assert!(!nodes.contains_key("above_threshold_2")); // 2+3=5 <= 5
    }

    #[test]
    fn test_all_comparison_operators() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                let value = 5;
                for i in 0..10 {
                    if i < value {
                        node "less_than_{i}";
                    }
                    if i > value {
                        node "greater_than_{i}";
                    }
                    if i <= value {
                        node "less_equal_{i}";
                    }
                    if i >= value {
                        node "greater_equal_{i}";
                    }
                    if i == value {
                        node "equal_{i}";
                    }
                    if i != value {
                        node "not_equal_{i}";
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute all comparison operators: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Test specific cases for i=5 (value=5)
        assert!(nodes.contains_key("less_equal_5"));
        assert!(nodes.contains_key("greater_equal_5"));
        assert!(nodes.contains_key("equal_5"));
        assert!(!nodes.contains_key("not_equal_5"));
        assert!(!nodes.contains_key("less_than_5"));
        assert!(!nodes.contains_key("greater_than_5"));

        // Test specific cases for i=3 (< value)
        assert!(nodes.contains_key("less_than_3"));
        assert!(nodes.contains_key("less_equal_3"));
        assert!(nodes.contains_key("not_equal_3"));
        assert!(!nodes.contains_key("greater_than_3"));
        assert!(!nodes.contains_key("greater_equal_3"));
        assert!(!nodes.contains_key("equal_3"));
    }

    #[test]
    fn test_nested_conditionals() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                for i in 0..5 {
                    if i > 1 {
                        if i < 4 {
                            node "nested_{i}";
                            if i == 2 {
                                node "special_{i}";
                            }
                        }
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute nested conditionals: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Should create nested nodes for i=2,3 (1 < i < 4)
        assert!(nodes.contains_key("nested_2"));
        assert!(nodes.contains_key("nested_3"));
        assert!(!nodes.contains_key("nested_0"));
        assert!(!nodes.contains_key("nested_1"));
        assert!(!nodes.contains_key("nested_4"));

        // Should create special node only for i=2
        assert!(nodes.contains_key("special_2"));
        assert!(!nodes.contains_key("special_3"));
    }

    #[test]
    fn test_conditional_with_complex_arithmetic() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                let size = 4;
                for i in 0..size {
                    for j in 0..size {
                        // Create nodes in upper triangle only
                        if i + j < size {
                            node "upper_{i}_{j}";
                        }
                        // Create edges with complex condition
                        if i * 2 + j > 3 {
                            if i != j {
                                edge: "upper_{i}_{j}" -> "upper_{j}_{i}" [type="complex"];
                            }
                        }
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute complex arithmetic conditionals: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Verify upper triangle nodes are created (i+j < 4)
        assert!(nodes.contains_key("upper_0_0")); // 0+0=0 < 4
        assert!(nodes.contains_key("upper_1_2")); // 1+2=3 < 4
        assert!(nodes.contains_key("upper_3_0")); // 3+0=3 < 4
        assert!(!nodes.contains_key("upper_3_3")); // 3+3=6 >= 4

        // Verify complex edges exist where they should
        assert!(!edges.is_empty());
        for (_, edge) in edges {
            assert_eq!(edge["metadata"]["type"], "complex");
        }
    }

    #[test]
    fn test_conditional_crystal_lattice_pattern() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph crystal {
                let lattice_size = 3;

                // Create atoms
                for x in 0..lattice_size {
                    for y in 0..lattice_size {
                        for z in 0..lattice_size {
                            node "C{x}_{y}_{z}" :atom [element="Carbon"];
                        }
                    }
                }

                // Create bonds with conditions
                for x in 0..lattice_size {
                    for y in 0..lattice_size {
                        for z in 0..lattice_size {
                            if x + 1 < lattice_size {
                                edge: "C{x}_{y}_{z}" -- "C{x+1}_{y}_{z}" [bond_type="covalent"];
                            }
                            if y + 1 < lattice_size {
                                edge: "C{x}_{y}_{z}" -- "C{x}_{y+1}_{z}" [bond_type="covalent"];
                            }
                            if z + 1 < lattice_size {
                                edge: "C{x}_{y}_{z}" -- "C{x}_{y}_{z+1}" [bond_type="covalent"];
                            }
                        }
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_ok(), "Failed to execute crystal lattice pattern: {:?}", result.err());

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();
        let edges = graph["edges"].as_object().unwrap();

        // Should have 27 atoms (3x3x3)
        assert_eq!(nodes.len(), 27);

        // Should have proper number of bonds
        // Each internal atom has 3 bonds, edge atoms have fewer
        // For 3x3x3: 18 bonds in x-direction + 18 in y-direction + 18 in z-direction = 54 bonds
        assert_eq!(edges.len(), 54);

        // Verify specific atoms exist
        assert!(nodes.contains_key("C0_0_0"));
        assert!(nodes.contains_key("C2_2_2"));

        // Verify all nodes are carbon atoms
        for (_, node) in nodes {
            assert_eq!(node["metadata"]["element"], "Carbon");
            assert_eq!(node["type"], "atom");
        }

        // Verify all edges are covalent bonds
        for (_, edge) in edges {
            assert_eq!(edge["metadata"]["bond_type"], "covalent");
        }
    }

    #[test]
    fn test_conditional_error_handling() {
        let mut engine = GGLEngine::new();

        // Test division by zero in condition
        let ggl_code = r#"
            graph test {
                let zero = 0;
                for i in 1..3 {
                    if i / zero > 1 {
                        node "never_created";
                    }
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err(), "Should fail with division by zero");

        let error_msg = result.err().unwrap();
        assert!(error_msg.contains("Division by zero"));
    }

    #[test]
    fn test_conditional_with_undefined_variable() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph test {
                if undefined_var < 5 {
                    node "should_not_exist";
                }
            }
        "#;

        let result = engine.generate_from_ggl(ggl_code);
        assert!(result.is_err(), "Should fail with undefined variable");

        let error_msg = result.err().unwrap();
        // Check for the actual error message that would be returned
        assert!(error_msg.contains("Cannot evaluate term") ||
               error_msg.contains("Undefined variable") ||
               error_msg.contains("Expected numeric value"),
               "Expected undefined variable error, got: {error_msg}");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_graph_generation() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph large {
                generate complete {
                    nodes: 50;
                    prefix: "node";
                }
            }
        "#;

        let start = std::time::Instant::now();
        let result = engine.generate_from_ggl(ggl_code);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_secs() < 5); // Should complete within 5 seconds

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Verify large graph was created
        assert_eq!(graph["nodes"].as_object().unwrap().len(), 50);
        assert_eq!(graph["edges"].as_object().unwrap().len(), 1225); // 50*49/2
    }

    #[test]
    fn test_complex_program_performance() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph complex {
                generate grid {
                    rows: 10;
                    cols: 10;
                    prefix: "cell";
                }

                rule add_metadata {
                    lhs { node N; }
                    rhs { node N [processed=true, timestamp=123456]; }
                }

                apply add_metadata 2 times;
            }
        "#;

        let start = std::time::Instant::now();
        let result = engine.generate_from_ggl(ggl_code);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_secs() < 10); // Should complete within 10 seconds

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        // Verify complex program executed correctly
        let nodes = graph["nodes"].as_object().unwrap();
        assert_eq!(nodes.len(), 100); // 10x10 grid

        // All nodes should have metadata from rule
        for (_, node) in nodes {
            assert_eq!(node["metadata"]["processed"], true);
            assert_eq!(node["metadata"]["timestamp"], 123456);
        }
    }

    #[test]
    fn test_conditional_performance() {
        let mut engine = GGLEngine::new();

        let ggl_code = r#"
            graph perf_test {
                for i in 0..100 {
                    for j in 0..100 {
                        if i + j < 150 {
                            if i * j > 100 {
                                node "perf_{i}_{j}";
                            }
                        }
                    }
                }
            }
        "#;

        let start = std::time::Instant::now();
        let result = engine.generate_from_ggl(ggl_code);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Performance test failed: {:?}", result.err());
        assert!(duration.as_secs() < 15); // Should complete within 15 seconds

        let json_str = result.unwrap();
        let graph: Value = serde_json::from_str(&json_str).unwrap();

        let nodes = graph["nodes"].as_object().unwrap();

        // Should have created a significant number of nodes
        assert!(nodes.len() > 100);

        // All created nodes should meet both conditions
        for (key, _) in nodes {
            if let Some(parts) = key.strip_prefix("perf_").and_then(|s| s.split_once('_')) {
                let i: i64 = parts.0.parse().unwrap();
                let j: i64 = parts.1.parse().unwrap();
                assert!(i + j < 150, "Node {key} failed first condition");
                assert!(i * j > 100, "Node {key} failed second condition");
            }
        }
    }
}
