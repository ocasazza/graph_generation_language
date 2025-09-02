//! Graph Generation Language (GGL) Core Library
//!
//! This library provides parsing and AST definitions for the Graph Generation Language,
//! which combines declarative graph definitions with the Baba Yaga functional programming language.

extern crate peg;

// --- Abstract Syntax Tree (AST) Definitions ---
// These enums represent the parsed structure of GGL/Baba Yaga code.

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Expr>),
    Table(Vec<(String, Expr)>), // Represented as a list of key-value pairs
    Constant(String), // For PI, INFINITY
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    Number, // Int ⊂ Float ⊂ Number
    String,
    Bool,
    List,
    Table,
    Result(Box<Type>), // e.g., Result<Int>
    Function(Vec<Type>, Box<Type>), // Argument types and return type
    Any, // Wildcard type, or inferred type '_'
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    // Unary Operators
    UnaryMinus,
    Not,
    // Arithmetic Operators
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    // Comparison Operators
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    // Logical Operators
    And,
    Or,
    // String Concatenation Operator
    StringConcat,
    // Functional Programming Operators
    Pipe, // |> operator for function composition
}

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Literal(Literal),
    Type(Type),
    Wildcard, // '_'
    Variable(String), // 'n' in 'n if (condition) then ...'
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    FunctionCall {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
    UnaryOp {
        op: Operator,
        expr: Box<Expr>,
    },
    BinaryOp {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    // Control Flow Expressions
    When {
        discriminants: Vec<Expr>,
        cases: Vec<(Vec<Pattern>, Option<Box<Expr>>, Box<Expr>)>, // (patterns, optional_guard_condition, result_expression)
    },
    // Function Definitions (can be anonymous/lambda expressions)
    FunctionDefinition {
        name: String, // Empty string for anonymous functions
        params: Vec<(String, Option<Type>)>, // (param_name, optional_param_type)
        return_type: Option<Type>,
        body: Box<Expr>,
        is_curried: bool, // Placeholder for curried vs. multi-param; needs deeper analysis
        with_locals: Option<Vec<(String, Expr)>>, // `with (locals)` clause
        with_rec_fns: Option<Vec<Stmt>>,     // `with rec (fns)` clause for mutual recursion
    },
    // Generic wrappers for Baba Yaga's extensive built-in operations
    ArrayOp { name: String, args: Vec<Expr> },
    TableOp { name: String, args: Vec<Expr> },
    StringOp { name: String, args: Vec<Expr> },
    MathOp { name: String, args: Vec<Expr> },
    FunctionCombinator { name: String, args: Vec<Expr> },
    ValidationDebugOp { name: String, args: Vec<Expr> },
    IoJsInteropOp { name: String, args: Vec<Expr> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expr(Expr), // Any expression can be a statement if its result is ignored
    Assignment(String, Expr), // `var : value;`
    Declaration(String, Type), // `var Type;`
    Function(
        String, // name
        Vec<(String, Option<Type>)>, // params
        Option<Type>, // return_type
        Expr, // body
        Option<Vec<(String, Expr)>>, // with_locals
        Option<Vec<Stmt>>, // with_rec_fns
    ),
    // GGL specific top-level graph definition (placeholder)
    GraphDeclaration {
        nodes: Vec<Expr>, // Should resolve to list of Tables
        edges: Vec<Expr>, // Should resolve to list of Tables
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

// --- PEG Grammar Definition ---
// This uses the `peg::parser!` macro to define the parsing rules for GGL.
peg::parser! {
    pub grammar ggl_parser() for str {
        // --- Utility Rules ---

        // Whitespace and comments rule: matches zero or more spaces, tabs, newlines, carriage returns, and comments.
        // `quiet!` suppresses this rule from appearing in error messages, making them cleaner.
        rule _ = quiet!{ (whitespace_char() / comment())* }

        // Individual whitespace characters
        rule whitespace_char() = " " / "\t" / "\n" / "\r"

        // Comment rule: matches single-line comments starting with //
        rule comment() = "//" (!"\n" [_])*

        // Identifiers: start with a letter or underscore, followed by letters, numbers, underscore, or dots for namespaced identifiers.
        rule identifier() -> String
            = s:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.']*) { s.to_string() }

        // End Of File (EOF): ensures the parser consumes the entire input.
        rule EOF() = ![_]

        // --- Type Parsing Rules ---

        rule r#type() -> Type
            = "Int" { Type::Int }
            / "Float" { Type::Float }
            / "Number" { Type::Number }
            / "String" { Type::String }
            / "Bool" { Type::Bool }
            / "List" { Type::List }
            / "Table" { Type::Table }
            / "Result" _ "<" _ inner:r#type() _ ">" { Type::Result(Box::new(inner)) }
            // For Function type syntax: Function(Int, String) -> Bool
            / "Function" _ "(" _ params:(r#type() ** (_ "," _)) _ ")" _ "->" _ return_type:r#type() { Type::Function(params, Box::new(return_type)) }
            / "_" { Type::Any } // Wildcard type
            / i:identifier() {? // Allow custom types or type variables
                // In a full implementation, you'd resolve 'i' to a known type or type variable.
                // For boilerplate, we'll return an error if it's not a built-in.
                match i.as_str() {
                    "Int" | "Float" | "Number" | "String" | "Bool" | "List" | "Table" | "Result" | "Function" => Err("Redundant match, should be caught by direct rules"),
                    _ => Err("Unknown or custom type, not fully implemented in boilerplate") // Placeholder
                }
            }


        // --- Literal Parsing Rules ---

        rule int_literal() -> Literal
            = n:$(['0'..='9']+) {? n.parse::<i64>().or(Err("Invalid integer")).map(Literal::Int) }

        rule float_literal() -> Literal
            = n:$(['0'..='9']+ "." ['0'..='9']*) {? n.parse::<f64>().or(Err("Invalid float")).map(Literal::Float) }

        rule string_literal() -> Literal
            = "\"" s:$(([^ '"' | '\\'] / "\\" [_])*) "\"" { Literal::String(s.to_string()) }

        rule bool_literal() -> Literal
            = "true" { Literal::Bool(true) }
            / "false" { Literal::Bool(false) }

        rule list_literal() -> Literal
            = "[" _ elements:(expression() ** (_ "," _)) _ "]" { Literal::List(elements) }

        rule table_entry() -> (String, Expr)
            = k:string_literal() _ ":" _ v:expression() _ ";"? {
                match k {
                    Literal::String(s) => (s, v),
                    _ => unreachable!("string_literal should only return String literals")
                }
            }
            / k:identifier() _ ":" _ v:expression() _ ";"? { (k, v) }

        // Special rule for 'with' clause local variable entries (uses semicolons)
        rule with_local_entry() -> (String, Expr)
            = k:identifier() _ ":" _ v:expression() { (k, v) }

        // Special rule for 'with rec' clause function entries (semicolons handled by separator)
        rule with_rec_entry() -> Stmt
            = name:identifier() _ ":" _ params:function_params() _ "->" _ body:with_rec_body_expression() {
                Stmt::Function(name, params, None, body, None, None)
            }
            / name:identifier() _ ":" _ param:identifier() _ "->" _ body:with_rec_body_expression() {
                Stmt::Function(name, vec![(param, None)], None, body, None, None)
            }

        // Expression rule for with_rec function bodies - allows when expressions but prevents left recursion
        rule with_rec_body_expression() -> Expr = precedence!{
            // Pipe operator (lowest precedence for functional composition)
            x:(@) _ "|>" _ y:@ { Expr::BinaryOp { op: Operator::Pipe, left: Box::new(x), right: Box::new(y) } }
            --
            // Logical Operators
            x:(@) _ "or" _ y:@ { Expr::BinaryOp { op: Operator::Or, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "and" _ y:@ { Expr::BinaryOp { op: Operator::And, left: Box::new(x), right: Box::new(y) } }
            --
            // Comparison Operators
            x:(@) _ "=" _ y:@ { Expr::BinaryOp { op: Operator::Equal, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "!=" _ y:@ { Expr::BinaryOp { op: Operator::NotEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<" _ y:@ { Expr::BinaryOp { op: Operator::LessThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<=" _ y:@ { Expr::BinaryOp { op: Operator::LessThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">=" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            --
            // Additive Arithmetic Operators
            x:(@) _ "+" _ y:@ { Expr::BinaryOp { op: Operator::Add, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "-" _ y:@ { Expr::BinaryOp { op: Operator::Subtract, left: Box::new(x), right: Box::new(y) } }
            --
            // Multiplicative Arithmetic Operators
            x:(@) _ "*" _ y:@ { Expr::BinaryOp { op: Operator::Multiply, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "/" _ y:@ { Expr::BinaryOp { op: Operator::Divide, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "%" _ y:@ { Expr::BinaryOp { op: Operator::Modulo, left: Box::new(x), right: Box::new(y) } }
            --
            // String Concatenation Operator
            x:(@) _ ".." _ y:@ { Expr::BinaryOp { op: Operator::StringConcat, left: Box::new(x), right: Box::new(y) } }
            --
            // Unary Operators (highest precedence among binary/unary ops)
            "-" _ e:@ { Expr::UnaryOp { op: Operator::UnaryMinus, expr: Box::new(e) } }
            "!" _ e:@ { Expr::UnaryOp { op: Operator::Not, expr: Box::new(e) } }
            --
            // Atom-level expressions (includes control flow like when expressions, but prevents function def recursion)
            e:with_rec_atom_expression() { e }
        }

        // Atom-level expressions for with_rec bodies - includes when expressions but prevents recursion
        rule with_rec_atom_expression() -> Expr
            = control_flow_expression()          // when expressions - MUST come first to avoid parsing "when" as identifier
            / with_rec_call_access_expression()  // Function calls, property access (specialized for with_rec)
            / array_op_call()                    // Array operations
            / table_op_call()                    // Table operations
            / string_op_call()                   // String operations
            / math_op_call()                     // Math operations
            / func_combinator_call()             // Function combinators
            / validation_debug_op_call()         // Validation and debug operations
            / io_js_interop_op_call()            // I/O and JS interop operations
            // Note: function_definition_expression() is excluded to prevent recursion in with_rec

        // Specialized call_access_expression for with_rec contexts - avoids control flow recursion
        rule with_rec_call_access_expression() -> Expr =
            head:with_rec_primary_expression()
            tail:(
                  "." _ prop:identifier() { Expr::PropertyAccess { object: Box::new(Expr::Identifier("".to_string())), property: prop } }
                / !("," / ";" / "}" / "]" / ")" / "|>" / "then" / "else" / "and" / "or") _ arg:with_rec_primary_expression() { arg } // Space-separated function arguments, but not if followed by terminators
            )* {
                // Fold the suffixes into the head expression to build the AST
                tail.into_iter().fold(head, |acc, suffix_expr| {
                    match suffix_expr {
                        Expr::PropertyAccess { object: _, property } => Expr::PropertyAccess { object: Box::new(acc), property },
                        arg => {
                            // Handle space-separated function application
                            match acc {
                                Expr::FunctionCall { function, mut args } => {
                                    // If it's already a function call, add more arguments
                                    args.push(arg);
                                    Expr::FunctionCall { function, args }
                                }
                                _ => {
                                    // Convert to a function call with the first argument
                                    Expr::FunctionCall {
                                        function: Box::new(acc),
                                        args: vec![arg]
                                    }
                                }
                            }
                        }
                    }
                })
            }

        // Primary expressions for with_rec contexts - excludes control flow to prevent recursion
        rule with_rec_primary_expression() -> Expr
            = l:literal() { Expr::Literal(l) }
            / variable_declaration_expression() // Variable declarations as expressions
            / function_call_expression() // Function calls
            / math_op_call() // Math operations with space-separated args
            / array_op_call() // Array operations
            / table_op_call() // Table operations
            / string_op_call() // String operations
            / func_combinator_call() // Function combinators
            / validation_debug_op_call() // Validation and debug operations
            / io_js_interop_op_call() // I/O and JS interop operations
            / i:identifier() { Expr::Identifier(i) }
            / "(" _ e:with_rec_body_expression() _ ")" { e }  // Use with_rec_body_expression instead of expression
            / lambda_expression() // Anonymous functions can be primary expressions
            // Note: control_flow_expression() and function_definition_expression() are excluded

        rule table_literal() -> Literal
            = "{" _ entries:(table_entry() ** (_ ","? _)) _ "}" { Literal::Table(entries) }

        rule constant_literal() -> Literal
            = "PI" { Literal::Constant("PI".to_string()) }
            / "INFINITY" { Literal::Constant("INFINITY".to_string()) }

        rule literal() -> Literal
            = float_literal()
            / int_literal()
            / string_literal()
            / bool_literal()
            / list_literal()
            / table_literal()
            / constant_literal()

        // --- Expression Parsing Rules ---

        // Variable declaration as expression: `var Type; var : value;` (semicolon optional)
        rule variable_declaration_expression() -> Expr
            = "var" _ t:r#type() _ ";" _ "var" _ ":" _ value:expression() {
                // For now, represent this as a function call to a special "var_decl" function
                // In a full implementation, this would be handled by the type system
                Expr::FunctionCall {
                    function: Box::new(Expr::Identifier("var_decl".to_string())),
                    args: vec![
                        Expr::Identifier(format!("{:?}", t)), // Type as string
                        value
                    ]
                }
            }

        // Function definition as expression: handles various function syntaxes
        rule function_definition_expression() -> Expr
            = name:identifier() _ ":" _ params:function_params() _ "->" _ return_type:r#type() _ "->" _ body:expression() {
                // Typed function: f : (x: Type) -> ReturnType -> body
                Expr::FunctionDefinition {
                    name: name,
                    params: params,
                    return_type: Some(return_type),
                    body: Box::new(body),
                    is_curried: false,
                    with_locals: None,
                    with_rec_fns: None,
                }
            }
            / name:identifier() _ ":" _ param:identifier() _ "->" _ "with" _ "rec" _ "(" _ rec_fns:(with_rec_entry() ** (_ ";" _)) _ ";" _? ")" _ "->" _ body:expression() {
                // Function with mutual recursion: f : x -> with rec (fns) -> body
                Expr::FunctionDefinition {
                    name: name,
                    params: vec![(param, None)],
                    return_type: None,
                    body: Box::new(body),
                    is_curried: false,
                    with_locals: None,
                    with_rec_fns: Some(rec_fns),
                }
            }
            / name:identifier() _ ":" _ param:identifier() _ "->" _ "with" _ "(" _ locals:(with_local_entry() ** (_ ";" _)) _ ";" _? ")" _ "->" _ body:expression() {
                // Function with locals: f : x -> with (locals) -> body
                Expr::FunctionDefinition {
                    name: name,
                    params: vec![(param, None)],
                    return_type: None,
                    body: Box::new(body),
                    is_curried: false,
                    with_locals: Some(locals),
                    with_rec_fns: None,
                }
            }
            / name:identifier() _ ":" _ first_param:identifier() _ "->" _ rest:curried_function_body() {
                // Curried function: f : x -> y -> body
                Expr::FunctionDefinition {
                    name: name,
                    params: vec![(first_param, None)],
                    return_type: None,
                    body: Box::new(rest),
                    is_curried: true,
                    with_locals: None,
                    with_rec_fns: None,
                }
            }
            / name:identifier() _ ":" _ params:function_params() _ "->" _ body:expression() {
                // Regular function: f : (x, y) -> body or f : x -> body
                Expr::FunctionDefinition {
                    name: name,
                    params: params,
                    return_type: None,
                    body: Box::new(body),
                    is_curried: false,
                    with_locals: None,
                    with_rec_fns: None,
                }
            }

        // Helper rule for parsing curried function bodies
        rule curried_function_body() -> Expr
            = param:identifier() _ "->" _ rest:curried_function_body() {
                // Another parameter in the curry chain: y -> z -> body
                Expr::FunctionDefinition {
                    name: "".to_string(), // Anonymous function
                    params: vec![(param, None)],
                    return_type: None,
                    body: Box::new(rest),
                    is_curried: true,
                    with_locals: None,
                    with_rec_fns: None,
                }
            }
            / body:curried_body_expression() {
                // Final body of the curried function
                body
            }

        // Special expression rule for curried function bodies that includes math operations
        rule curried_body_expression() -> Expr = precedence!{
            // Logical Operators (lowest precedence among these)
            x:(@) _ "or" _ y:@ { Expr::BinaryOp { op: Operator::Or, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "and" _ y:@ { Expr::BinaryOp { op: Operator::And, left: Box::new(x), right: Box::new(y) } }
            --
            // Comparison Operators
            x:(@) _ "=" _ y:@ { Expr::BinaryOp { op: Operator::Equal, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "!=" _ y:@ { Expr::BinaryOp { op: Operator::NotEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<" _ y:@ { Expr::BinaryOp { op: Operator::LessThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<=" _ y:@ { Expr::BinaryOp { op: Operator::LessThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">=" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            --
            // Additive Arithmetic Operators
            x:(@) _ "+" _ y:@ { Expr::BinaryOp { op: Operator::Add, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "-" _ y:@ { Expr::BinaryOp { op: Operator::Subtract, left: Box::new(x), right: Box::new(y) } }
            --
            // Multiplicative Arithmetic Operators
            x:(@) _ "*" _ y:@ { Expr::BinaryOp { op: Operator::Multiply, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "/" _ y:@ { Expr::BinaryOp { op: Operator::Divide, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "%" _ y:@ { Expr::BinaryOp { op: Operator::Modulo, left: Box::new(x), right: Box::new(y) } }
            --
            // String Concatenation Operator
            x:(@) _ ".." _ y:@ { Expr::BinaryOp { op: Operator::StringConcat, left: Box::new(x), right: Box::new(y) } }
            --
            // Unary Operators (highest precedence among binary/unary ops)
            "-" _ e:@ { Expr::UnaryOp { op: Operator::UnaryMinus, expr: Box::new(e) } }
            "!" _ e:@ { Expr::UnaryOp { op: Operator::Not, expr: Box::new(e) } }
            --
            // Atom-level expressions: function calls, property access, built-in ops, control flow
            e:curried_atom_expression() { e }
        }

        // Atom-level expressions for curried function bodies that includes math operations
        rule curried_atom_expression() -> Expr
            = call_access_expression() // Handles `f x`, `obj.prop`, `f(a,b)`
            / control_flow_expression() // 'when ... is ...'
            / array_op_call()
            / table_op_call()
            / string_op_call()
            / math_op_call() // Include math operations here
            / func_combinator_call()
            / validation_debug_op_call()
            / io_js_interop_op_call()

        // Primary expressions: basic building blocks like literals, identifiers, or parenthesized expressions.
        rule primary_expression() -> Expr
            = l:literal() { Expr::Literal(l) } // Parse literals FIRST (includes table_literal)
            / "(" _ e:expression() _ ")" { e } // Parenthesized expressions
            / variable_declaration_expression() // Variable declarations as expressions
            / with_expression() // Standalone with expressions
            / control_flow_expression() // 'when ... is ...' - parse at primary level to avoid precedence conflicts
            / function_call_expression() // Function calls
            / math_op_call() // Math operations with space-separated args
            / array_op_call() // Array operations
            / table_op_call() // Table operations
            / string_op_call() // String operations
            / func_combinator_call() // Function combinators
            / validation_debug_op_call() // Validation and debug operations
            / io_js_interop_op_call() // I/O and JS interop operations
            / function_definition_expression() // Function definitions as expressions - MOVED AFTER other rules
            / lambda_expression() // Anonymous functions can be primary expressions
            / i:identifier() { Expr::Identifier(i) } // Simple identifiers LAST

        // Function calls and property access: left-associative chaining (e.g., `obj.prop.method(arg)`)
        rule call_access_expression() -> Expr =
            head:primary_expression()
            tail:(
                  "." _ prop:identifier() { Expr::PropertyAccess { object: Box::new(Expr::Identifier("".to_string())), property: prop } }
                / !("," / ";" / "}" / "]" / ")" / "|>" / "then" / "else" / "and" / "or") _ arg:primary_expression() { arg } // Space-separated function arguments, but not if followed by terminators
            )* {
                // Fold the suffixes into the head expression to build the AST
                tail.into_iter().fold(head, |acc, suffix_expr| {
                    match suffix_expr {
                        Expr::PropertyAccess { object: _, property } => Expr::PropertyAccess { object: Box::new(acc), property },
                        arg => {
                            // Handle space-separated function application
                            match acc {
                                Expr::FunctionCall { function, mut args } => {
                                    // If it's already a function call, add more arguments
                                    args.push(arg);
                                    Expr::FunctionCall { function, args }
                                }
                                _ => {
                                    // Convert to a function call with the first argument
                                    Expr::FunctionCall {
                                        function: Box::new(acc),
                                        args: vec![arg]
                                    }
                                }
                            }
                        }
                    }
                })
            }

        rule function_call_expression() -> Expr
            = function:identifier() _ "(" _ args:(expression() ** (_ "," _)) _ ")" {
                Expr::FunctionCall {
                    function: Box::new(Expr::Identifier(function)),
                    args: args,
                }
            }

        // Main expression rule, handles operator precedence using `precedence!` macro.
        // Operators are grouped by precedence levels, from lowest binding to highest.
        pub rule expression() -> Expr = precedence!{
            // Pipe operator (lowest precedence for functional composition)
            x:(@) _ "|>" _ y:@ { Expr::BinaryOp { op: Operator::Pipe, left: Box::new(x), right: Box::new(y) } }
            --
            // Logical Operators
            x:(@) _ "or" _ y:@ { Expr::BinaryOp { op: Operator::Or, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "and" _ y:@ { Expr::BinaryOp { op: Operator::And, left: Box::new(x), right: Box::new(y) } }
            --
            // Comparison Operators
            x:(@) _ "=" _ y:@ { Expr::BinaryOp { op: Operator::Equal, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "!=" _ y:@ { Expr::BinaryOp { op: Operator::NotEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<" _ y:@ { Expr::BinaryOp { op: Operator::LessThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<=" _ y:@ { Expr::BinaryOp { op: Operator::LessThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">=" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            --
            // Additive Arithmetic Operators
            x:(@) _ "+" _ y:@ { Expr::BinaryOp { op: Operator::Add, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "-" _ y:@ { Expr::BinaryOp { op: Operator::Subtract, left: Box::new(x), right: Box::new(y) } }
            --
            // Multiplicative Arithmetic Operators
            x:(@) _ "*" _ y:@ { Expr::BinaryOp { op: Operator::Multiply, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "/" _ y:@ { Expr::BinaryOp { op: Operator::Divide, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "%" _ y:@ { Expr::BinaryOp { op: Operator::Modulo, left: Box::new(x), right: Box::new(y) } }
            --
            // String Concatenation Operator
            x:(@) _ ".." _ y:@ { Expr::BinaryOp { op: Operator::StringConcat, left: Box::new(x), right: Box::new(y) } }
            --
            // Unary Operators (highest precedence among binary/unary ops)
            "-" _ e:@ { Expr::UnaryOp { op: Operator::UnaryMinus, expr: Box::new(e) } }
            "!" _ e:@ { Expr::UnaryOp { op: Operator::Not, expr: Box::new(e) } }
            --
            // Atom-level expressions: function calls, property access, built-in ops, control flow
            e:atom_expression() { e }
        }

        // Atom-level expressions that don't participate in operator precedence
        rule atom_expression() -> Expr
            = array_op_call()
            / table_op_call()
            / string_op_call()
            / math_op_call()
            / func_combinator_call()
            / validation_debug_op_call()
            / io_js_interop_op_call()
            / call_access_expression() // Handles `f x`, `obj.prop`, `f(a,b)` - MOVED TO END

        // --- Function Parameter and Lambda Rules ---

        // Parses parameters in parentheses `(x: Int, y)` or space-separated `x y`.
        rule function_params() -> Vec<(String, Option<Type>)>
            = "(" _ params:((i:identifier() _ ":" _ t:r#type() { (i, Some(t)) } / i:identifier() { (i, None) }) ** (_ "," _)) _ ")" { params }
            / params:(identifier() ** _) {
                // For `f : x y -> body;` style, params are space-separated untyped identifiers
                params.into_iter().map(|s| (s, None)).collect()
            }

        // Anonymous function definition (lambda) `x -> body;`
        rule lambda_expression() -> Expr
            = params:function_params() _ "->" _ body:expression() {
                Expr::FunctionDefinition {
                    name: "".to_string(), // Anonymous function
                    params: params,
                    return_type: None, // Simplified: return type is inferred or explicitly parsed later
                    body: Box::new(body),
                    is_curried: false, // This requires deeper semantic analysis
                    with_locals: None,
                    with_rec_fns: None,
                }
            }

        // Standalone with expression: `with (locals) -> body`
        rule with_expression() -> Expr
            = "with" _ "(" _ locals:(with_local_entry() ** (_ ";" _)) _ ";" _? ")" _ "->" _ body:expression() {
                // Represent as a function definition with locals
                Expr::FunctionDefinition {
                    name: "".to_string(), // Anonymous function with locals
                    params: vec![], // No parameters
                    return_type: None,
                    body: Box::new(body),
                    is_curried: false,
                    with_locals: Some(locals),
                    with_rec_fns: None,
                }
            }


        // --- Control Flow Parsing Rules ---
        rule pattern_case_pattern() -> Pattern
            = l:literal() { Pattern::Literal(l) }
            / t:r#type() { Pattern::Type(t) } // `Int`, `String`, etc. as patterns
            / "_" { Pattern::Wildcard }
            / i:identifier() { Pattern::Variable(i) } // Variable capture for guards, e.g., `n if ...`

        rule when_case() -> (Vec<Pattern>, Option<Box<Expr>>, Box<Expr>)
            = pattern:pattern_case_pattern() _ // Match a single pattern per case
              guard:("if" _ cond:expression() _ { cond })? _ // Optional pattern guard
              "then" _ result:when_result_expression() {
                (vec![pattern], guard.map(Box::new), Box::new(result))
            }

        // Enhanced expression rule for when case results - allows binary operations without control flow recursion
        rule when_result_expression() -> Expr
            = binary_or_atom_expression()

        // Binary operations without control flow - prevents left recursion while enabling complex expressions
        rule binary_or_atom_expression() -> Expr = precedence!{
            // Pipe operator (lowest precedence for functional composition)
            x:(@) _ "|>" _ y:@ { Expr::BinaryOp { op: Operator::Pipe, left: Box::new(x), right: Box::new(y) } }
            --
            // Logical Operators
            x:(@) _ "or" _ y:@ { Expr::BinaryOp { op: Operator::Or, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "and" _ y:@ { Expr::BinaryOp { op: Operator::And, left: Box::new(x), right: Box::new(y) } }
            --
            // Comparison Operators
            x:(@) _ "=" _ y:@ { Expr::BinaryOp { op: Operator::Equal, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "!=" _ y:@ { Expr::BinaryOp { op: Operator::NotEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<" _ y:@ { Expr::BinaryOp { op: Operator::LessThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "<=" _ y:@ { Expr::BinaryOp { op: Operator::LessThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThan, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ ">=" _ y:@ { Expr::BinaryOp { op: Operator::GreaterThanOrEqual, left: Box::new(x), right: Box::new(y) } }
            --
            // Additive Arithmetic Operators
            x:(@) _ "+" _ y:@ { Expr::BinaryOp { op: Operator::Add, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "-" _ y:@ { Expr::BinaryOp { op: Operator::Subtract, left: Box::new(x), right: Box::new(y) } }
            --
            // Multiplicative Arithmetic Operators
            x:(@) _ "*" _ y:@ { Expr::BinaryOp { op: Operator::Multiply, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "/" _ y:@ { Expr::BinaryOp { op: Operator::Divide, left: Box::new(x), right: Box::new(y) } }
            x:(@) _ "%" _ y:@ { Expr::BinaryOp { op: Operator::Modulo, left: Box::new(x), right: Box::new(y) } }
            --
            // String Concatenation Operator
            x:(@) _ ".." _ y:@ { Expr::BinaryOp { op: Operator::StringConcat, left: Box::new(x), right: Box::new(y) } }
            --
            // Unary Operators (highest precedence among binary/unary ops)
            "-" _ e:@ { Expr::UnaryOp { op: Operator::UnaryMinus, expr: Box::new(e) } }
            "!" _ e:@ { Expr::UnaryOp { op: Operator::Not, expr: Box::new(e) } }
            --
            // Atom-level expressions (but excluding control flow to prevent recursion)
            e:non_control_atom_expression() { e }
        }

        // Atom-level expressions that exclude control flow to prevent left recursion
        rule non_control_atom_expression() -> Expr
            = call_access_expression()     // Function calls, property access
            / array_op_call()              // Array operations
            / table_op_call()              // Table operations
            / string_op_call()             // String operations
            / math_op_call()               // Math operations
            / func_combinator_call()       // Function combinators
            / validation_debug_op_call()   // Validation and debug operations
            / io_js_interop_op_call()      // I/O and JS interop operations
            // Note: control_flow_expression() is excluded to prevent recursion

        // Arguments for when result expressions - use full expression parsing
        rule when_result_arg() -> Expr
            = expression()

        rule control_flow_expression() -> Expr
            = "when" _ discriminant:when_discriminant() _ "is" _ cases:when_cases() {
                Expr::When {
                    discriminants: vec![discriminant],
                    cases
                }
            }

        // Simple discriminant expression for when clauses - avoid precedence parsing
        rule when_discriminant() -> Expr
            = i:identifier() { Expr::Identifier(i) }
            / l:literal() { Expr::Literal(l) }
            / "(" _ e:expression() _ ")" { e }

        rule when_cases() -> Vec<(Vec<Pattern>, Option<Box<Expr>>, Box<Expr>)>
            = cases:(when_case() ** _) { cases }

        // --- Built-in Operations (Boilerplate) ---
        // These rules parse calls to Baba Yaga's standard library functions.
        // They are generalized to match `namespace.function_name args` or `function_name args`.

        rule op_call_args() -> Vec<Expr> =
            "(" _ args:(expression() ** (_ "," _)) _ ")" { args } // Parenthesized, comma-separated args
            / args:(space_separated_arg() ** _) { args } // Space-separated args (for simpler calls like `map f xs`)

        rule array_op_call() -> Expr
            = "map" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:lambda_expression() _ xs:expression() { Expr::ArrayOp { name: "map".to_string(), args: vec![f, xs] } } // Specific rule for map to take a lambda
            / "filter" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ p:lambda_expression() _ xs:expression() { Expr::ArrayOp { name: "filter".to_string(), args: vec![p, xs] } } // Specific rule for filter
            / "where" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ p:lambda_expression() _ xs:expression() { Expr::ArrayOp { name: "where".to_string(), args: vec![p, xs] } } // Specific rule for where (similar to filter)
            / "reduce" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:lambda_expression() _ z:expression() _ xs:expression() { Expr::ArrayOp { name: "reduce".to_string(), args: vec![f, z, xs] } } // Specific rule for reduce
            / "scan" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:lambda_expression() _ z:expression() _ xs:expression() { Expr::ArrayOp { name: "scan".to_string(), args: vec![f, z, xs] } } // Specific rule for scan
            / "broadcast" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:lambda_expression() _ scalar:expression() _ xs:expression() { Expr::ArrayOp { name: "broadcast".to_string(), args: vec![f, scalar, xs] } } // Specific rule for broadcast
            / "zipWith" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:lambda_expression() _ xs:expression() _ ys:expression() { Expr::ArrayOp { name: "zipWith".to_string(), args: vec![f, xs, ys] } } // Specific rule for zipWith
            / "flatMap" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ f:expression() _ xs:expression() { Expr::ArrayOp { name: "flatMap".to_string(), args: vec![f, xs] } } // Specific rule for flatMap
            // Other array operations
            / op_name:$("cumsum" / "cumprod" / "at" / "take" / "drop" / "reshape" / "append" / "prepend" / "concat" / "update" / "removeAt" / "slice" / "length" / "chunk" / "range" / "repeat") !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ args:op_call_args() { Expr::ArrayOp { name: op_name.to_string(), args } }
            / "sort.by" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ xs:expression() _ f:expression() { Expr::ArrayOp { name: "sort.by".to_string(), args: vec![xs, f] } }
            / "group.by" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ xs:expression() _ f:expression() { Expr::ArrayOp { name: "group.by".to_string(), args: vec![xs, f] } }

        rule table_op_call() -> Expr
            = "set" _ tbl:expression() _ k:expression() _ v:expression() { Expr::TableOp { name: "set".to_string(), args: vec![tbl, k, v] } }
            // Other table operations
            / op_name:$("remove" / "merge" / "keys" / "values" / "shape") _ args:op_call_args() { Expr::TableOp { name: op_name.to_string(), args } }

        rule string_op_call() -> Expr
            = "str.concat" _ args:(expression() ** _) { Expr::StringOp { name: "str.concat".to_string(), args } } // Allows multiple arguments without parens
            // Other string operations
            / op_name:$("str.split" / "str.join" / "str.length" / "str.substring" / "str.replace" / "str.trim" / "str.upper" / "str.lower" / "text.lines" / "text.words") _ args:op_call_args() { Expr::StringOp { name: op_name.to_string(), args } }

        rule math_op_call() -> Expr
            = op_name:$("math.abs" / "math.sign" / "math.min" / "math.max" / "math.clamp" / "math.floor" / "math.ceil" / "math.round" / "math.trunc" / "math.pow" / "math.sqrt" / "math.exp" / "math.log" / "math.sin" / "math.cos" / "math.tan" / "math.asin" / "math.acos" / "math.atan" / "math.atan2" / "math.deg" / "math.rad" / "math.random" / "math.randomInt") _ args:math_args() { Expr::MathOp { name: op_name.to_string(), args } }

        // Helper rule for math operation arguments (simpler than space_separated_args to avoid recursion)
        rule math_args() -> Vec<Expr>
            = args:(math_arg() ** _) { args }

        // Individual math argument - excludes math_op_call to prevent recursion
        rule math_arg() -> Expr
            = l:literal() { Expr::Literal(l) }
            / i:identifier() { Expr::Identifier(i) }
            / "(" _ e:expression() _ ")" { e }

        // Helper rule for space-separated arguments (used in other function calls)
        rule space_separated_args() -> Vec<Expr>
            = args:(space_separated_arg() ** _) { args }

        // Individual space-separated argument - excludes operations that could cause recursion
        rule space_separated_arg() -> Expr
            = l:literal() { Expr::Literal(l) }
            / i:identifier() { Expr::Identifier(i) }
            / "(" _ e:expression() _ ")" { e }

        rule func_combinator_call() -> Expr
            = op_name:$("flip" / "apply" / "pipe" / "compose") _ args:op_call_args() { Expr::FunctionCombinator { name: op_name.to_string(), args } }

        rule validation_debug_op_call() -> Expr
            = "assert" !['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.'] _ condition:primary_expression() _ message:primary_expression() { Expr::ValidationDebugOp { name: "assert".to_string(), args: vec![condition, message] } }
            / op_name:$("validate.notEmpty" / "validate.range" / "validate.type" / "validate.email" / "debug.print" / "debug.inspect") _ args:op_call_args() { Expr::ValidationDebugOp { name: op_name.to_string(), args } }

        rule io_js_interop_op_call() -> Expr
            = op_name:$("io.out" / "io.in" / "io.callJS" / "io.callJSAsync" / "io.getProperty" / "io.setProperty" / "io.hasProperty" / "io.jsArrayToList" / "io.listToJSArray" / "io.objectToTable" / "io.tableToObject" / "io.getLastJSError" / "io.clearJSError") _ args:op_call_args() { Expr::IoJsInteropOp { name: op_name.to_string(), args } }


        // --- Statement Parsing Rules ---
        // Variable assignment: `var : value;`
        rule assignment_statement() -> Stmt
            = var:identifier() _ ":" _ value:expression() _ ";" { Stmt::Assignment(var, value) }

        // Typed variable declaration: `var Type;`
        rule type_declaration_statement() -> Stmt
            = var:identifier() _ t:r#type() _ ";" { Stmt::Declaration(var, t) }

        // Named function definition: `f : x -> body;`
        rule function_statement() -> Stmt
            = name:identifier() _ ":" _
              params:function_params()
              _ "->" _
              with_locals_clause:("with" _ "(" _ locals:(table_entry() ** (_ "," _)) _ ")" { locals })? // Optional `with (locals)`
              _ with_rec_clause:("with" _ "rec" _ "(" _ rec_fns:(function_statement() ** (_ "," _)) _ ")" { rec_fns })? // Optional `with rec (fns)` for mutual recursion
              _ body:expression() _ ";" {
                Stmt::Function(name, params, None, body, with_locals_clause, with_rec_clause)
            }

        // An expression followed by a semicolon can also be a statement.
        rule expression_as_statement() -> Stmt
            = e:expression() _ ";" { Stmt::Expr(e) }

        // GGL-specific graph declaration (boilerplate placeholder for top-level JSON-like structure)
        rule graph_declaration_statement() -> Stmt
            = "graph" _ "{" _
              "nodes" _ ":" _ nodes_list:list_literal() _ "," _
              "edges" _ ":" _ edges_list:list_literal() _
              "}" _ ";" {
                // `nodes_list` and `edges_list` are Literal::List, containing Exprs
                // They need to be unwrapped into Vec<Expr> to fit the AST.
                let nodes_expr = match nodes_list {
                    Literal::List(exprs) => exprs,
                    _ => unreachable!("Nodes must be a list literal.")
                };
                let edges_expr = match edges_list {
                    Literal::List(exprs) => exprs,
                    _ => unreachable!("Edges must be a list literal.")
                };
                Stmt::GraphDeclaration { nodes: nodes_expr, edges: edges_expr }
            }

        // A general rule for any statement type.
        pub rule statement() -> Stmt
            = _ s:(function_statement() / type_declaration_statement() / assignment_statement() / graph_declaration_statement() / expression_as_statement()) _ { s }

        // The top-level program is a sequence of zero or more statements, or a single expression (JSON-like)
        pub rule program() -> Program
            = _ statements:statement()* _ EOF() { Program { statements } }
            / _ expr:expression() _ EOF() { Program { statements: vec![Stmt::Expr(expr)] } }
    }
}

// --- Public API ---
/// Parse a GGL program from a string
pub fn parse_program(input: &str) -> Result<Program, peg::error::ParseError<peg::str::LineCol>> {
    ggl_parser::program(input)
}

/// Parse a single expression from a string
pub fn parse_expression(input: &str) -> Result<Expr, peg::error::ParseError<peg::str::LineCol>> {
    ggl_parser::expression(input)
}

/// Parse a single statement from a string
pub fn parse_statement(input: &str) -> Result<Stmt, peg::error::ParseError<peg::str::LineCol>> {
    ggl_parser::statement(input)
}

// The parser module is generated by the peg::parser! macro above
// Individual parser rules can be accessed via ggl_parser::rule_name()

// --- High-Level Engine API ---

/// High-level engine for processing GGL code and generating JSON output
/// This provides backward compatibility with the existing CLI, WASM, and browser demo interfaces
pub struct GGLEngine {
    base_path: Option<String>,
}

impl GGLEngine {
    /// Create a new GGL engine
    pub fn new() -> Self {
        Self { base_path: None }
    }

    /// Set the base path for resolving relative file paths
    pub fn with_base_path(mut self, path: &str) -> Self {
        self.base_path = Some(path.to_string());
        self
    }

    /// Parse GGL code and generate JSON output
    /// This is the main entry point that the CLI, WASM, and browser demo expect
    pub fn generate_from_ggl(&mut self, ggl_code: &str) -> Result<String, String> {
        // Parse the GGL program
        let program = parse_program(ggl_code)
            .map_err(|e| format!("Parse error: {}", e))?;

        // Convert the AST to JSON
        // For now, we'll create a simple JSON representation of the parsed AST
        // In a full implementation, this would evaluate the program and generate the actual graph
        let json_output = self.ast_to_json(&program)
            .map_err(|e| format!("Generation error: {}", e))?;

        Ok(json_output)
    }

    /// Convert the parsed AST to JSON representation
    /// This is a simplified implementation that creates a JSON structure from the AST
    fn ast_to_json(&self, program: &Program) -> Result<String, String> {
        use serde_json::json;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut metadata = serde_json::Map::new();

        // Process each statement in the program
        for stmt in &program.statements {
            match stmt {
                Stmt::GraphDeclaration { nodes: node_exprs, edges: edge_exprs } => {
                    // Convert node expressions to JSON
                    for node_expr in node_exprs {
                        if let Ok(node_json) = self.expr_to_json(node_expr) {
                            nodes.push(node_json);
                        }
                    }

                    // Convert edge expressions to JSON
                    for edge_expr in edge_exprs {
                        if let Ok(edge_json) = self.expr_to_json(edge_expr) {
                            edges.push(edge_json);
                        }
                    }
                }
                Stmt::Assignment(name, expr) => {
                    // Add assignments to metadata
                    if let Ok(value) = self.expr_to_json(expr) {
                        metadata.insert(name.clone(), value);
                    }
                }
                Stmt::Function(name, _params, _return_type, _body, _locals, _rec_fns) => {
                    // Add function definitions to metadata
                    metadata.insert(name.clone(), json!({"type": "function"}));
                }
                Stmt::Declaration(name, type_def) => {
                    // Add type declarations to metadata
                    metadata.insert(name.clone(), json!({"type": format!("{:?}", type_def)}));
                }
                Stmt::Expr(_) => {
                    // Expression statements don't directly contribute to graph structure
                }
            }
        }

        // If no explicit graph declaration was found, try to infer structure from the AST
        if nodes.is_empty() && edges.is_empty() {
            // Look for JSON-like structure in the top-level expressions
            for stmt in &program.statements {
                if let Stmt::Expr(expr) = stmt {
                    if let Ok(json_val) = self.expr_to_json(expr) {
                        if let serde_json::Value::Object(obj) = &json_val {
                            if let Some(serde_json::Value::Array(node_array)) = obj.get("nodes") {
                                nodes.extend(node_array.clone());
                            }
                            if let Some(serde_json::Value::Array(edge_array)) = obj.get("edges") {
                                edges.extend(edge_array.clone());
                            }
                            // Add other properties to metadata
                            for (key, value) in obj {
                                if key != "nodes" && key != "edges" {
                                    metadata.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create the final JSON structure
        let result = json!({
            "nodes": nodes,
            "edges": edges,
            "metadata": metadata
        });

        serde_json::to_string(&result)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }

    /// Convert an expression to JSON value
    fn expr_to_json(&self, expr: &Expr) -> Result<serde_json::Value, String> {
        use serde_json::json;

        match expr {
            Expr::Literal(lit) => self.literal_to_json(lit),
            Expr::Identifier(name) => Ok(json!(name)),
            Expr::BinaryOp { op, left, right } => {
                let left_val = self.expr_to_json(left)?;
                let right_val = self.expr_to_json(right)?;

                // For JSON output, we'll represent operations as objects
                Ok(json!({
                    "operation": format!("{:?}", op),
                    "left": left_val,
                    "right": right_val
                }))
            }
            Expr::UnaryOp { op, expr } => {
                let expr_val = self.expr_to_json(expr)?;
                Ok(json!({
                    "operation": format!("{:?}", op),
                    "operand": expr_val
                }))
            }
            Expr::FunctionCall { function, args } => {
                let func_val = self.expr_to_json(function)?;
                let arg_vals: Result<Vec<_>, _> = args.iter()
                    .map(|arg| self.expr_to_json(arg))
                    .collect();

                Ok(json!({
                    "function_call": func_val,
                    "arguments": arg_vals?
                }))
            }
            Expr::PropertyAccess { object, property } => {
                let obj_val = self.expr_to_json(object)?;
                Ok(json!({
                    "property_access": obj_val,
                    "property": property
                }))
            }
            _ => {
                // For other expression types, return a placeholder
                Ok(json!({
                    "expression_type": format!("{:?}", std::mem::discriminant(expr)),
                    "note": "Complex expression - simplified for JSON output"
                }))
            }
        }
    }

    /// Convert a literal to JSON value
    fn literal_to_json(&self, literal: &Literal) -> Result<serde_json::Value, String> {
        use serde_json::{json, Value};

        match literal {
            Literal::Int(i) => Ok(json!(i)),
            Literal::Float(f) => Ok(json!(f)),
            Literal::String(s) => Ok(json!(s)),
            Literal::Bool(b) => Ok(json!(b)),
            Literal::List(exprs) => {
                let values: Result<Vec<_>, _> = exprs.iter()
                    .map(|expr| self.expr_to_json(expr))
                    .collect();
                Ok(json!(values?))
            }
            Literal::Table(entries) => {
                let mut obj = serde_json::Map::new();
                for (key, expr) in entries {
                    let value = self.expr_to_json(expr)?;
                    obj.insert(key.clone(), value);
                }
                Ok(Value::Object(obj))
            }
            Literal::Constant(name) => {
                match name.as_str() {
                    "PI" => Ok(json!(std::f64::consts::PI)),
                    "INFINITY" => Ok(json!(f64::INFINITY)),
                    _ => Ok(json!(name))
                }
            }
        }
    }
}

impl Default for GGLEngine {
    fn default() -> Self {
        Self::new()
    }
}

mod test_math;
