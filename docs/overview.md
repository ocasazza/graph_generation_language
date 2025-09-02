# Graph Generation Language (GGL) Overview

## Description

The Graph Generation Language (GGL) is a domain-specific language for defining graph structures. It combines JSON-like declarative syntax with functional programming constructs from the Baba Yaga language.

## Core Components

### Parser Library

The core library (`src/lib`) provides:

- AST definitions for GGL syntax elements
- PEG-based parser using the `peg` crate
- Public API functions: `parse_program()`, `parse_expression()`, `parse_statement()`

### Language Syntax

#### Basic Structure

GGL files define graphs using a JSON-like syntax:

```ggl
graph {
  nodes : [
    {id: "A", type: "start"},
    {id: "B", type: "end"}
  ],
  edges : [
    {from: "A", to: "B", weight: 1.0}
  ]
};
```

#### Data Types

- Integers: `42`
- Floats: `3.14`
- Strings: `"text"`
- Booleans: `true`, `false`
- Lists: `[1, 2, 3]`
- Tables: `{key: value, other: 123}`
- Constants: `PI`, `INFINITY`

#### Variables and Functions

Variable assignment:
```ggl
x : 42;
name : "example";
```

Function definitions:
```ggl
add : x y -> x + y;
double : x -> x * 2;
```

#### Expressions

Binary operators with precedence:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `=`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `and`, `or`
- String concatenation: `..`

Unary operators:
- Negation: `-x`
- Logical not: `!x`

#### Control Flow

Pattern matching:
```ggl
when x is
  0 then "zero"
  Int then "number"
  _ then "other"
```

#### Built-in Operations

The parser recognizes calls to various operation categories:
- Array operations: `map`, `filter`, `reduce`, etc.
- Table operations: `set`, `remove`, `merge`, etc.
- String operations: `str.concat`, `str.split`, etc.
- Math operations: `math.abs`, `math.sin`, etc.
- Function combinators: `flip`, `apply`, `pipe`, `compose`
- Validation/debug: `validate.notEmpty`, `debug.print`, etc.
- I/O operations: `io.out`, `io.callJS`, etc.

## Implementation Status

The library currently provides:
- Complete AST representation
- PEG grammar for parsing GGL syntax
- Parsing functions that return structured AST
- Comprehensive test suite

The parser handles the full GGL syntax but does not include:
- Runtime evaluation
- Code generation
- JSON output generation
