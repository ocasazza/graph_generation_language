#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_simple_with_expression() {
        // Test a basic with expression - need semicolon after each local variable
        let input = r#"with (
            x : 5;
        ) -> x + 1"#;

        let result = parse_expression(input);
        match result {
            Ok(ast) => println!("Successfully parsed simple with: {:?}", ast),
            Err(e) => {
                println!("Failed to parse simple with: {}", e);
                panic!("Failed to parse simple with: {}", e);
            }
        }
    }

    #[test]
    fn test_range_syntax_in_list() {
        // Test the range syntax that's causing issues around line 170 in chimera.ggl
        let input = r#"chunk [5-7, 9, 10, 14, 17, 19] 3"#;

        let result = parse_expression(input);
        match result {
            Ok(ast) => println!("Successfully parsed range syntax: {:?}", ast),
            Err(e) => {
                println!("Failed to parse range syntax: {}", e);
                // This is likely the issue - the parser doesn't understand range syntax like 5-7
                println!("This appears to be the issue around line 170 in chimera.ggl");
                panic!("Failed to parse range syntax: {}", e);
            }
        }
    }

    #[test]
    fn test_at_indices_expression() {
        // Test the at_indices_data expression that's failing at line 177:49
        let input = r#"at_indices_data"#;

        let result = parse_expression(input);
        match result {
            Ok(ast) => println!("Successfully parsed at_indices_data: {:?}", ast),
            Err(e) => {
                println!("Failed to parse at_indices_data: {}", e);
                panic!("Failed to parse at_indices_data: {}", e);
            }
        }
    }

    #[test]
    fn test_parenthesized_range_call() {
        let input = "(range 0 max_nodes_to_generate)";
        match parse_expression(input) {
            Ok(expr) => {
                println!("Parsed parenthesized range call: {:#?}", expr);

                // Verify it's parsed as a parenthesized ArrayOp
                if let Expr::ArrayOp { name, args } = expr {
                    assert_eq!(name, "range");
                    assert_eq!(args.len(), 2);

                    // First arg should be literal 0
                    assert_eq!(args[0], Expr::Literal(Literal::Int(0)));

                    // Second arg should be identifier
                    assert_eq!(args[1], Expr::Identifier("max_nodes_to_generate".to_string()));
                } else {
                    panic!("Expected ArrayOp, got: {:#?}", expr);
                }
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_flatmap_issue() {
        let input = r#"flatMap (node_idx -> [
    {
        "id": "test",
        "value": node_idx
    }
]) (range 0 5)"#;
        match parse_expression(input) {
            Ok(expr) => {
                println!("Parsed flatMap expression: {:#?}", expr);
                // This should parse successfully as a flatMap call with two arguments
            }
            Err(e) => panic!("Parse error: {}", e),
        }
    }

    #[test]
    fn test_exact_problematic_section() {
        // Test the exact section that includes the problematic line 177 area
        let input = r#"with (
            raw_data : range 1 6;
            cum_sum_data : cumsum raw_data;
            cum_prod_data : cumprod raw_data;
            scanned_data : scan (acc x -> acc + x) 0 raw_data;
            indices_gt_3 : where (x -> x > 3) raw_data;
            at_indices_data : at indices_gt_3 raw_data;
            take_first_2 : take 2 raw_data;
            drop_last_2 : drop 2 raw_data;
            broadcast_op : broadcast (x y -> x * y) 10 raw_data;
            reshaped_matrix : reshape [6, 9] [5-7, 9, 17, 19];
            chunked_list : chunk [5-7, 9, 10, 14, 17, 19] 3;
            repeated_element_list : repeat 4 "X";
        ) -> {
            "id": "advanced_array_ops_node",
            "cumulative_sums": cum_sum_data,
            "cumulative_products": cum_prod_data,
            "scanned_results": scanned_data,
            "selected_by_where": at_indices_data,
            "taken_elements": take_first_2,
            "dropped_elements": drop_last_2,
            "broadcasted_values": broadcast_op,
            "matrix_view": reshaped_matrix,
            "chunked_parts": chunked_list,
            "repeated_items": repeated_element_list
        }"#;

        let result = parse_expression(input);
        match result {
            Ok(ast) => println!("Successfully parsed problematic section: {:?}", ast),
            Err(e) => {
                println!("Failed to parse problematic section: {}", e);
                // Let's see what line and column this gives us
                let lines: Vec<&str> = input.lines().collect();
                println!("Input has {} lines", lines.len());
                if let Some(error_location) = format!("{}", e).split(" at ").nth(1) {
                    if let Some(line_col) = error_location.split(":").collect::<Vec<_>>().get(0..2) {
                        if let (Ok(line_num), Ok(col_num)) = (line_col[0].parse::<usize>(), line_col[1].parse::<usize>()) {
                            println!("Error at line {}, column {}", line_num, col_num);
                            if line_num > 0 && line_num <= lines.len() {
                                let line = lines[line_num - 1];
                                println!("Line {}: '{}'", line_num, line);
                                if col_num > 0 && col_num <= line.len() {
                                    let char_at_error = line.chars().nth(col_num - 1).unwrap_or(' ');
                                    println!("Character at error position: '{}'", char_at_error);
                                    // Show context around the error
                                    let prefix = " ".repeat(col_num.saturating_sub(1));
                                    println!("Position marker: {}^", prefix);
                                }
                            }
                        }
                    }
                }
                panic!("Failed to parse problematic section: {}", e);
            }
        }
    }

    #[test]
    fn test_line_177_debug() {
        // Let me test around line 177 area - around the list manipulation section
        let input = r#"
        // List Manipulation [15, 19, 21]
        with (
            original_list : ["a", "b", "c"];
            appended_item : append original_list "d"; // append
            prepended_item : prepend "z" appended_item; // prepend
            concatenated_list : concat prepended_item ["e", "f"]; // concat
            updated_item : update concatenated_list 2 "new_c"; // update
            removed_item : removeAt updated_item 0; // removeAt
            sliced_part : slice removed_item 1 3; // slice
            current_length : length sliced_part; // length
        ) -> {
            "id": "list_manipulation_node",
            "final_list_state": sliced_part,
            "list_final_length": current_length
        }
        "#;

        let result = parse_expression(input);
        match result {
            Ok(ast) => println!("Successfully parsed line 177 area: {:?}", ast),
            Err(e) => {
                println!("Failed to parse line 177 area: {}", e);
                // Let's try debugging what's at that exact position
                let lines: Vec<&str> = input.lines().collect();
                println!("Input has {} lines", lines.len());
                for (i, line) in lines.iter().enumerate() {
                    println!("Line {}: {}", i + 1, line);
                }
                panic!("Failed to parse line 177 area: {}", e);
            }
        }
    }

    #[test]
    fn test_chimera_top_level_structure() {
        let input = r#"// filename: comprehensive_example.ggl

{
    "name": "Comprehensive GGL Feature Showcase Graph",
    "description": "This GGL file demonstrates every Baba Yaga language feature."
}"#;

        match parse_program(input) {
            Ok(program) => {
                println!("Parsed program successfully: {:#?}", program);
                assert_eq!(program.statements.len(), 1);
            }
            Err(e) => {
                // Extract line and column from the error
                let error_str = format!("{}", e);
                let parts: Vec<&str> = error_str.split(" at ").collect();
                if parts.len() > 1 {
                    let location_part = parts[1];
                    let loc_parts: Vec<&str> = location_part.split(":").collect();
                    if loc_parts.len() >= 2 {
                        let line_num: usize = loc_parts[0].parse().unwrap_or(0);
                        let col_num: usize = loc_parts[1].split(",").next().unwrap_or("0").parse().unwrap_or(0);

                        println!("Error at line {}, column {}", line_num, col_num);

                        let lines: Vec<&str> = input.lines().collect();
                        if line_num > 0 && line_num <= lines.len() {
                            let line = lines[line_num - 1];
                            println!("Line {}: {}", line_num, line);
                            if col_num > 0 && col_num <= line.len() {
                                let char_at_error = line.chars().nth(col_num - 1).unwrap_or(' ');
                                println!("Character at error position: '{}'", char_at_error);
                            }
                        }
                    }
                }
                panic!("Parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_minimal_chimera_structure() {
        let input = r#"// filename: comprehensive_example.ggl

{
    "name": "Test",
    "variables": {
        "current_id_counter": "test"
    }
}"#;

        match parse_program(input) {
            Ok(program) => {
                println!("Parsed program successfully: {:#?}", program);
                assert_eq!(program.statements.len(), 1);
            }
            Err(e) => {
                // Extract line and column from the error
                let error_str = format!("{}", e);
                let parts: Vec<&str> = error_str.split(" at ").collect();
                if parts.len() > 1 {
                    let location_part = parts[1];
                    let loc_parts: Vec<&str> = location_part.split(":").collect();
                    if loc_parts.len() >= 2 {
                        let line_num: usize = loc_parts[0].parse().unwrap_or(0);
                        let col_num: usize = loc_parts[1].split(",").next().unwrap_or("0").parse().unwrap_or(0);

                        println!("Error at line {}, column {}", line_num, col_num);

                        let lines: Vec<&str> = input.lines().collect();
                        if line_num > 0 && line_num <= lines.len() {
                            let line = lines[line_num - 1];
                            println!("Line {}: {}", line_num, line);
                            if col_num > 0 && col_num <= line.len() {
                                let char_at_error = line.chars().nth(col_num - 1).unwrap_or(' ');
                                println!("Character at error position: '{}'", char_at_error);
                            }
                        }
                    }
                }
                panic!("Parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_table_with_function() {
        // Test the full table entry
        let input = r#"{"fibonacci": f : n -> with rec (
    fib_0 : x -> when x is 0 then 0 _ then fib_1 (x - 1);
) -> fib_0 n}"#;
        let result = parse_expression(input);
        println!("Table with function parse result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call_with_binary_op() {
        let input = "fib_1 (x - 1)";
        let result = parse_expression(input);
        println!("Function call with binary op result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse function call: {:?}", result.as_ref().err());
    }


    #[test]
    fn test_simple_identifier_in_table() {
        // Test simple identifier in table context
        let simple_input = r#"
        {
            "test": at_indices_data
        }
        "#;

        match crate::parse_expression(simple_input) {
            Ok(parsed) => {
                println!("Successfully parsed simple identifier: {:?}", parsed);
            }
            Err(e) => {
                println!("Failed to parse simple identifier: {}", e);
                panic!("Failed to parse simple identifier: {}", e);
            }
        }
    }

    #[test]
    fn test_identifier_with_comma() {
        // Test identifier followed by comma
        let comma_input = r#"
        {
            "test": at_indices_data,
        }
        "#;

        match crate::parse_expression(comma_input) {
            Ok(parsed) => {
                println!("Successfully parsed identifier with comma: {:?}", parsed);
            }
            Err(e) => {
                println!("Failed to parse identifier with comma: {}", e);

                // Extract line and column information
                let lines: Vec<&str> = comma_input.lines().collect();
                println!("Input has {} lines", lines.len());

                if let Some(pos) = e.location.line.checked_sub(1) {
                    if pos < lines.len() {
                        println!("Error at line {}, column {}", e.location.line, e.location.column);
                        println!("Line {}: '{}'", e.location.line, lines[pos]);

                        // Show character at error position if possible
                        let line_chars: Vec<char> = lines[pos].chars().collect();
                        if e.location.column > 0 && (e.location.column - 1) < line_chars.len() {
                            println!("Character at error position: '{}'", line_chars[e.location.column - 1]);

                            // Create a position marker
                            let marker = " ".repeat(e.location.column - 1) + "^";
                            println!("Position marker: {}", marker);
                        }
                    }
                }

                panic!("Failed to parse identifier with comma: {}", e);
            }
        }
    }

    #[test]
    fn test_simple_with_rec() {
        // Test parsing a simple with rec with just one function
        let input = r#"f : n -> with rec (
            fib_0 : x -> when x is 0 then 0 _ then fib_1 (x - 1);
        ) -> fib_0 n"#;
        let result = parse_expression(input);
        println!("Simple with rec result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
            println!("Character at error position: '{}'",
                input.chars().nth(e.location.offset).unwrap_or(' '));
            // Print context around error
            let lines: Vec<&str> = input.lines().collect();
            if e.location.line > 0 && e.location.line <= lines.len() {
                let line = lines[e.location.line - 1];
                println!("Line {}: {}", e.location.line, line);
                println!("Error at column {}: {}", e.location.column,
                    " ".repeat(e.location.column.saturating_sub(1)) + "^");
            }
        }
        assert!(result.is_ok(), "Failed to parse simple with rec: {:?}", result.as_ref().err());
    }

    #[test]
    fn test_space_function_call() {
        // Test if the issue is with space-separated function calls
        let input = "fib_1 (x - 1)";
        let result = parse_expression(input);
        println!("Space function call result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
        }
        assert!(result.is_ok(), "Failed to parse space function call: {:?}", result.as_ref().err());
    }

    #[test]
    fn test_var_declaration_in_table() {
        // Test variable declaration expression in table context
        let input = r#"{"current_id_counter": var Int; var : 0}"#;
        let result = parse_expression(input);
        println!("Var declaration in table result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
            println!("Character at error position: '{}'",
                input.chars().nth(e.location.offset).unwrap_or(' '));
        }
        assert!(result.is_ok(), "Failed to parse var declaration in table: {:?}", result.as_ref().err());
    }

    #[test]
    fn test_chimera_variables_section() {
        // Test the variables section from chimera.ggl more closely
        let input = r#"{
        "current_id_counter": var Int; var : 0,
        "greeting_message": "Hello GGL!",
        "processed_data_list": [var Int; var : 10, 20, 30]
    }"#;
        let result = parse_expression(input);
        println!("Chimera variables section result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
            println!("Character at error position: '{}'",
                input.chars().nth(e.location.offset).unwrap_or(' '));
            // Print context around error
            let lines: Vec<&str> = input.lines().collect();
            if e.location.line > 0 && e.location.line <= lines.len() {
                let line = lines[e.location.line - 1];
                println!("Line {}: {}", e.location.line, line);
                println!("Error at column {}: {}", e.location.column,
                    " ".repeat(e.location.column.saturating_sub(1)) + "^");
            }
        }
        assert!(result.is_ok(), "Failed to parse chimera variables section: {:?}", result.as_ref().err());
    }

    #[test]
    fn test_chimera_with_comments() {
        // Test parsing with C++-style comments like those in chimera.ggl
        let input = r#"{
    // Variable Assignment: var : value; and var Type; var : value; [5, 10, 11]
    "variables": {
        "current_id_counter": var Int; var : 0,
        "greeting_message": "Hello GGL!", // String
        "processed_data_list": [var Int; var : 10, 20, 30] // Example of typed assignment in a list context
    }
}"#;
        let result = parse_expression(input);
        println!("Chimera with comments result: {:?}", result);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
            println!("Error location: line {}, column {}", e.location.line, e.location.column);
            println!("Character at error position: '{}'",
                input.chars().nth(e.location.offset).unwrap_or(' '));
            // Print context around error
            let lines: Vec<&str> = input.lines().collect();
            if e.location.line > 0 && e.location.line <= lines.len() {
                let line = lines[e.location.line - 1];
                println!("Line {}: {}", e.location.line, line);
                println!("Error at column {}: {}", e.location.column,
                    " ".repeat(e.location.column.saturating_sub(1)) + "^");
            }
        }
        assert!(result.is_ok(), "Failed to parse chimera with comments: {:?}", result.as_ref().err());
    }

    #[test]
    fn test_chimera_full_structure() {
        // Test the exact top-level structure from chimera.ggl
        let input = r#"// filename: comprehensive_example.ggl

{
    "name": "Comprehensive GGL Feature Showcase Graph",
    "description": "This GGL file demonstrates every Baba Yaga language feature integrated into a JSON-like declarative structure for dynamic graph generation.",

    // ----------------------------------------------------------------------------------------------------
    // 1. Baba Yaga Language Constructs & Core Features
    //    (Variables, Functions, Literals, Operators, Control Flow, Types, Result Handling)
    // ----------------------------------------------------------------------------------------------------

    // Literals: Int, Float, String, Bool, List, Table, Constants PI, INFINITY [5-8]
    "global_declarations": {
        "max_nodes_to_generate": 5, // Int
        "connection_strength": 0.75, // Float
        "base_name_prefix": "Dynamic-", // String
        "is_graph_directed": true, // Bool
        "initial_id_list": [5, 6, 9], // List
        "node_templates": { "default_type": "generic", "priority": 1 }, // Table
        "math_constants": { "pi_val": PI, "infinity_val": INFINITY } // Constants
    },

    // Variable Assignment: var : value; and var Type; var : value; [5, 10, 11]
    "variables": {
        "current_id_counter": var Int; var : 0,
        "greeting_message": "Hello GGL!",
        "processed_data_list": [var Int; var : 10, 20, 30] // Example of typed assignment in a list context
    }
}"#;

        match parse_program(input) {
            Ok(program) => {
                println!("Parsed program successfully: {:#?}", program);
                assert_eq!(program.statements.len(), 1);
            }
            Err(e) => {
                // Extract line and column from the error
                let error_str = format!("{}", e);
                let parts: Vec<&str> = error_str.split(" at ").collect();
                if parts.len() > 1 {
                    let location_part = parts[1];
                    let loc_parts: Vec<&str> = location_part.split(":").collect();
                    if loc_parts.len() >= 2 {
                        let line_num: usize = loc_parts[0].parse().unwrap_or(0);
                        let col_num: usize = loc_parts[1].split(",").next().unwrap_or("0").parse().unwrap_or(0);

                        println!("Error at line {}, column {}", line_num, col_num);

                        let lines: Vec<&str> = input.lines().collect();
                        if line_num > 0 && line_num <= lines.len() {
                            let line = lines[line_num - 1];
                            println!("Line {}: {}", line_num, line);
                            if col_num > 0 && col_num <= line.len() {
                                let char_at_error = line.chars().nth(col_num - 1).unwrap_or(' ');
                                println!("Character at error position: '{}'", char_at_error);
                            }
                        }
                    }
                }
                panic!("Parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_chimera_engine_full_structure() {
        // Test using the exact same approach as the e2e test: using GGLEngine
        let input = r#"// filename: comprehensive_example.ggl

{
    "name": "Comprehensive GGL Feature Showcase Graph",
    "description": "This GGL file demonstrates every Baba Yaga language feature integrated into a JSON-like declarative structure for dynamic graph generation.",

    // ----------------------------------------------------------------------------------------------------
    // 1. Baba Yaga Language Constructs & Core Features
    //    (Variables, Functions, Literals, Operators, Control Flow, Types, Result Handling)
    // ----------------------------------------------------------------------------------------------------

    // Literals: Int, Float, String, Bool, List, Table, Constants PI, INFINITY [5-8]
    "global_declarations": {
        "max_nodes_to_generate": 5, // Int
        "connection_strength": 0.75, // Float
        "base_name_prefix": "Dynamic-", // String
        "is_graph_directed": true, // Bool
        "initial_id_list": [5, 6, 9], // List
        "node_templates": { "default_type": "generic", "priority": 1 }, // Table
        "math_constants": { "pi_val": PI, "infinity_val": INFINITY } // Constants
    },

    // Variable Assignment: var : value; and var Type; var : value; [5, 10, 11]
    "variables": {
        "current_id_counter": var Int; var : 0,
        "greeting_message": "Hello GGL!",
        "processed_data_list": [var Int; var : 10, 20, 30] // Example of typed assignment in a list context
    }
}"#;

        // Test with GGLEngine like the e2e test does
        let mut engine = GGLEngine::new();
        match engine.generate_from_ggl(input) {
            Ok(actual_json) => {
                println!("Generated JSON: {}", actual_json);

                // Validate that we can parse the JSON
                let actual_parsed: serde_json::Value = serde_json::from_str(&actual_json)
                    .expect("Generated JSON should be valid JSON");

                // Check that we have the basic structure
                assert!(actual_parsed.get("nodes").is_some(), "Generated JSON should have nodes");
                assert!(actual_parsed.get("edges").is_some(), "Generated JSON should have edges");
                assert!(actual_parsed.get("metadata").is_some(), "Generated JSON should have metadata");

                println!("Engine test passed! Generated JSON is valid.");
            }
            Err(e) => {
                println!("Engine failed with error: {}", e);
                panic!("Engine failed to process the input: {}", e);
            }
        }
    }

    #[test]
    fn test_chimera_exact_semicolon_issue() {
        // Test the exact issue from chimera.ggl: semicolon after var declaration
        let input = r#"// filename: comprehensive_example.ggl

{
    // Variable Assignment: var : value; and var Type; var : value; [5, 10, 11]
    "variables": {
        "current_id_counter": var Int; var : 0;
        "greeting_message": "Hello GGL!",
        "processed_data_list": [var Int; var : 10, 20, 30] // Example of typed assignment in a list context
    }
}"#;

        // Test with GGLEngine like the e2e test does
        let mut engine = GGLEngine::new();
        match engine.generate_from_ggl(input) {
            Ok(actual_json) => {
                println!("Generated JSON: {}", actual_json);
                println!("Engine test with semicolon passed!");
            }
            Err(e) => {
                println!("Engine failed with error: {}", e);
                // This should show us the exact error we're seeing in the e2e test
                println!("This is the exact issue preventing chimera e2e test from passing!");
                panic!("Engine failed to process the input with semicolon: {}", e);
            }
        }
    }

    #[test]
    fn test_range_function_call() {
        // Test the range function call that's failing at line 150
        let input = "range 0 max_nodes_to_generate";

        match crate::parse_expression(input) {
            Ok(parsed) => {
                println!("Successfully parsed range call: {:?}", parsed);
            }
            Err(e) => {
                println!("Failed to parse range call: {}", e);

                // Extract line and column information
                let lines: Vec<&str> = input.lines().collect();
                println!("Input has {} lines", lines.len());

                if let Some(pos) = e.location.line.checked_sub(1) {
                    if pos < lines.len() {
                        println!("Error at line {}, column {}", e.location.line, e.location.column);
                        println!("Line {}: '{}'", e.location.line, lines[pos]);

                        // Show character at error position if possible
                        let line_chars: Vec<char> = lines[pos].chars().collect();
                        if e.location.column > 0 && (e.location.column - 1) < line_chars.len() {
                            println!("Character at error position: '{}'", line_chars[e.location.column - 1]);

                            // Create a position marker
                            let marker = " ".repeat(e.location.column - 1) + "^";
                            println!("Position marker: {}", marker);
                        }
                    }
                }

                panic!("Failed to parse range call: {}", e);
            }
        }
    }

    #[test]
    fn test_minimal_line_177_issue() {
        // Test the exact minimal case that reproduces the line 177:36 error
        let input = r#"{
    "selected_by_where": at_indices_data
}"#;

        match crate::parse_expression(input) {
            Ok(parsed) => {
                println!("Successfully parsed minimal line 177 case: {:?}", parsed);
            }
            Err(e) => {
                println!("Failed to parse minimal line 177 case: {}", e);

                // Extract line and column information
                let lines: Vec<&str> = input.lines().collect();
                println!("Input has {} lines", lines.len());

                if let Some(pos) = e.location.line.checked_sub(1) {
                    if pos < lines.len() {
                        println!("Error at line {}, column {}", e.location.line, e.location.column);
                        println!("Line {}: '{}'", e.location.line, lines[pos]);

                        // Show character at error position if possible
                        let line_chars: Vec<char> = lines[pos].chars().collect();
                        if e.location.column > 0 && (e.location.column - 1) < line_chars.len() {
                            println!("Character at error position: '{}'", line_chars[e.location.column - 1]);

                            // Create a position marker
                            let marker = " ".repeat(e.location.column - 1) + "^";
                            println!("Position marker: {}", marker);
                        }
                    }
                }

                panic!("Failed to parse minimal line 177 case: {}", e);
            }
        }
    }

    #[test]
    fn test_zipwith_issue() {
        // Test the zipWith issue from line 283
        let input = r#"zipWith (source_id target_id -> { "test": source_id })
(map (i -> increment i) (range 0 5))
(map (i -> increment i) (range 1 6))"#;

        match crate::parse_expression(input) {
            Ok(parsed) => {
                println!("Successfully parsed zipWith case: {:?}", parsed);
            }
            Err(e) => {
                println!("Failed to parse zipWith case: {}", e);

                // Extract line and column information
                let lines: Vec<&str> = input.lines().collect();
                println!("Input has {} lines", lines.len());

                if let Some(pos) = e.location.line.checked_sub(1) {
                    if pos < lines.len() {
                        println!("Error at line {}, column {}", e.location.line, e.location.column);
                        println!("Line {}: '{}'", e.location.line, lines[pos]);

                        // Show character at error position if possible
                        let line_chars: Vec<char> = lines[pos].chars().collect();
                        if e.location.column > 0 && (e.location.column - 1) < line_chars.len() {
                            println!("Character at error position: '{}'", line_chars[e.location.column - 1]);

                            // Create a position marker
                            let marker = " ".repeat(e.location.column - 1) + "^";
                            println!("Position marker: {}", marker);
                        }
                    }
                }

                panic!("Failed to parse zipWith case: {}", e);
            }
        }
    }

}
