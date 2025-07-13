# GGL Test Suite Documentation

This directory contains a comprehensive test suite for the Graph Generation Language (GGL) that ensures all grammar features defined in `ggl.pest` are functional and working correctly.

## Test Structure Overview

The test suite is organized into four main test files, totaling **3,627 lines of test code**:

### 1. Parser Tests (`parser_tests.rs` - 884 lines)
Tests the core parsing functionality and grammar rules:

#### Lexical Tests
- **Identifiers**: Valid identifier patterns, case sensitivity, underscores, numbers
- **Strings**: String literals, escaping, empty strings, special characters
- **Numbers**: Integers, floats, negative numbers, scientific notation
- **Booleans**: `true` and `false` values
- **Comments**: Line comments (`//`) and block comments (`/* */`)

#### Grammar Feature Tests
- **Node Declarations**: Simple nodes, typed nodes, nodes with attributes
- **Edge Declarations**: Directed (`->`) and undirected (`--`) edges, edge attributes
- **Generator Statements**: All generator types with various parameter combinations
- **Rule Definitions**: Simple and complex pattern matching rules
- **Rule Applications**: Various iteration counts and edge cases
- **Complex Programs**: Mixed statements, nested structures, real-world scenarios

#### Error Handling Tests
- **Syntax Errors**: Invalid grammar, missing tokens, malformed expressions
- **Missing Semicolons**: Required punctuation validation
- **Invalid Numbers**: Malformed numeric literals
- **Unclosed Strings**: String termination validation
- **Unclosed Comments**: Comment block validation

### 2. Generator Tests (`generator_tests.rs` - 689 lines)
Comprehensive testing of all 7 built-in graph generators:

#### Individual Generator Tests
- **Complete Graph**: All-to-all connectivity, directed/undirected variants
- **Path Graph**: Linear chains, degree validation
- **Cycle Graph**: Circular structures, self-loops, triangles
- **Grid Graph**: 2D lattices, periodic boundaries, rectangular grids
- **Star Graph**: Hub-and-spoke topology, directed variants
- **Tree Graph**: Hierarchical structures, branching factors, depth limits
- **Barabasi-Albert**: Scale-free networks, preferential attachment

#### Parameter Validation Tests
- **Type Checking**: Numeric, string, and boolean parameter validation
- **Range Validation**: Minimum/maximum values, edge cases
- **Error Handling**: Missing parameters, invalid combinations
- **Default Values**: Fallback behavior for optional parameters

#### Graph Properties Tests
- **Connectivity**: Expected edge counts and graph structure
- **Node Naming**: Prefix consistency and numbering schemes
- **Edge Properties**: Source/target validation, no invalid references

### 3. Rule System Tests (`rule_tests.rs` - 1,058 lines)
Extensive testing of the pattern matching and graph transformation system:

#### Simple Rule Tests
- **Node Replacement**: Single node transformations
- **Edge Addition**: Creating new connections
- **Node Deletion**: Removing isolated nodes
- **Attribute Updates**: Metadata modifications

#### Type-Based Matching Tests
- **Type-Specific Rules**: Matching nodes by type
- **Type Preservation**: Maintaining types through transformations
- **Type Conversion**: Changing node types

#### Attribute-Based Tests
- **String Attributes**: Text-based matching and updates
- **Numeric Attributes**: Number-based conditions and modifications
- **Boolean Attributes**: True/false state changes
- **Complex Attributes**: Multi-attribute patterns

#### Edge Pattern Tests
- **Edge Transformation**: Changing edge properties
- **Triangle Closure**: Path-to-triangle transformations
- **Complex Patterns**: Multi-node, multi-edge matching

#### Rule Application Tests
- **Multiple Iterations**: Repeated rule applications
- **Termination Conditions**: Natural stopping points
- **No Matches**: Rules with no applicable patterns
- **Performance**: Large graph rule applications

### 4. Integration Tests (`integration_tests.rs` - 996 lines)
End-to-end testing of complete GGL programs:

#### Basic Integration Tests
- **Simple Programs**: Node and edge declarations
- **Mixed Content**: Manual and generated elements
- **Complex Attributes**: Multi-type metadata
- **Empty Graphs**: Edge case handling

#### Generator Integration Tests
- **All Generators**: End-to-end generator functionality
- **Parameter Combinations**: Various generator configurations
- **Error Scenarios**: Invalid generators and parameters

#### Rule Integration Tests
- **Simple Rules**: Basic transformations
- **Complex Rules**: Multi-pattern matching
- **Generator + Rules**: Combined functionality
- **Rule Chains**: Multiple rule applications

#### Real-World Scenarios
- **Social Networks**: User relationships and metadata
- **Organizational Hierarchies**: Management structures
- **Infrastructure Networks**: Server and datacenter modeling

#### Performance Tests
- **Large Graphs**: 50+ node complete graphs
- **Complex Programs**: Grid generation with rule applications
- **Timing Validation**: Performance benchmarks

## Grammar Coverage

The test suite ensures **100% coverage** of all grammar features defined in `ggl.pest`:

### Core Language Elements ✅
- [x] Whitespace and comments
- [x] Identifiers (`ident`)
- [x] String literals (`string`)
- [x] Numeric literals (`number`)
- [x] Boolean literals (`boolean`)
- [x] Values (`value`)

### Graph Structure Elements ✅
- [x] Attributes (`attribute`, `attribute_list`, `attributes`)
- [x] Node declarations (`node_type`, `node_decl`)
- [x] Edge declarations (`edge_op`, `edge_decl`)
- [x] Graph containers (`graph`)

### Generator System ✅
- [x] Generator parameters (`param`, `param_list`)
- [x] Generator statements (`generate_stmt`)
- [x] All 7 built-in generators

### Rule System ✅
- [x] Pattern definitions (`node_pattern`, `edge_pattern`, `pattern`)
- [x] Rule definitions (`rule_def`)
- [x] Rule applications (`apply_rule`)

### Program Structure ✅
- [x] Statement composition (`statement`)
- [x] Program entry point (`program`)

## Test Execution

To run the complete test suite:

```bash
cargo test
```

To run specific test modules:

```bash
cargo test parser_tests
cargo test generator_tests
cargo test rule_tests
cargo test integration_tests
```

To run tests with output:

```bash
cargo test -- --nocapture
```

## Test Categories

### Unit Tests
- Individual grammar rule parsing
- Generator function validation
- Rule pattern matching
- Type system validation

### Integration Tests
- Complete GGL program execution
- Multi-component interactions
- Error propagation
- JSON output validation

### Performance Tests
- Large graph generation (50+ nodes)
- Complex rule applications (100+ iterations)
- Memory usage validation
- Execution time benchmarks

### Property Tests
- Graph invariant preservation
- Generator mathematical properties
- Rule application convergence
- Edge case boundary testing

## Expected Test Results

When all tests pass, you should see:

- **Parser Tests**: ~50 test cases covering all grammar features
- **Generator Tests**: ~35 test cases covering all 7 generators
- **Rule Tests**: ~25 test cases covering pattern matching and transformations
- **Integration Tests**: ~30 test cases covering end-to-end functionality

**Total**: ~140 comprehensive test cases ensuring complete GGL functionality.

## Test Data and Fixtures

The `fixtures/` directory contains:

- `valid_programs/`: Example GGL programs that should parse successfully
- `invalid_programs/`: Example programs that should fail parsing
- `expected_outputs/`: JSON outputs for validation

## Continuous Integration

These tests are designed to:

1. **Validate Grammar**: Ensure all `ggl.pest` rules work correctly
2. **Prevent Regressions**: Catch breaking changes early
3. **Document Behavior**: Serve as executable documentation
4. **Guide Development**: Provide examples for new features

## Contributing

When adding new features to GGL:

1. Add corresponding tests to the appropriate test file
2. Include both positive and negative test cases
3. Test edge cases and error conditions
4. Update this documentation

## Test Philosophy

This test suite follows these principles:

- **Comprehensive Coverage**: Every grammar rule is tested
- **Real-World Examples**: Tests use practical scenarios
- **Error Validation**: Both success and failure cases are tested
- **Performance Awareness**: Large graphs and complex operations are tested
- **Documentation Value**: Tests serve as usage examples

The goal is to ensure that every feature defined in the GGL grammar is not only parseable but also functionally correct and performant.
