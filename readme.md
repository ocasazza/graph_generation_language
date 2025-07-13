# Graph Generation Language (GGL)

A domain-specific language for creating and manipulating graphs through declarative syntax. GGL allows you to define graph structures, generate common graph topologies, and apply transformation rules to evolve graphs over time.
st suites including parser, generator, rule, and integration tests.

[![Documentation](https://img.shields.io/badge/docs-latest-blue)](https://ocasazza.github.io/graph-generation-language/)

## Features

- **Declarative Syntax**: Define graphs using intuitive node and edge declarations
- **Built-in Generators**: Create common graph structures (complete, path, cycle, grid, star, tree, scale-free)
- **Transformation Rules**: Apply pattern-based rules to modify graph structure
- **Rich Attributes**: Support for typed nodes and edges with metadata
- **JSON Output**: Export graphs in standard JSON format

## Quick Start

### Commands

We also provide a [`justfile`](https://just.systems/) for Makefile'esque commands to be run inside of the devShell.

```zsh
❯ just
Available recipes:
    default
    pre-commit-all       # Run pre-commit hooks on all files, including autoformatting
    run *ARGS            # Run 'cargo run' on the project
    test *ARGS           # Run all tests
    test-coverage        # Run tests with coverage
    test-file FILE *ARGS # Run specific test file
    test-watch *ARGS     # Run tests with bacon (auto-recompiles and re-runs tests)
    watch *ARGS          # Run 'bacon' to run the project (auto-recompiles)
```

### Installation

#### Option 1: Using Nix Flake (Recommended)

If you have [Nix](https://nixos.org/download.html) and [direnv](https://direnv.net/) installed:

```bash
git clone https://github.com/ocasazza/graph-generation-language.git
cd graph-generation-language
direnv allow  # This will automatically set up the development environment
```

This repo uses [Flakes](https://nixos.asia/en/flakes) from the get-go.

```bash
# Dev shell
nix develop

# or run via cargo
nix develop -c cargo run

# build
nix build
```

#### Option 2: Manual Installation

```bash
git clone https://github.com/ocasazza/graph-generation-language.git
cd graph-generation-language
cargo build --release
```

For documentation building:

```bash
cargo doc --document-private-items --package graph_generation_language --all-features
```

Once built, these docs can be viewed locally at `./graph-generation-language/target/doc/graph_generation_language/index.html`

### Basic Example

```ggl
graph social_network {
    // Define nodes with types and attributes
    node alice :person [name="Alice", age=30];
    node bob :person [name="Bob", age=25];
    node company :organization [name="Tech Corp"];

    // Create relationships
    edge friendship: alice -- bob [strength=0.8];
    edge employment: alice -> company [role="Engineer"];

    // Generate additional structure
    generate complete {
        nodes: 5;
        prefix: "user";
    }

    // Apply transformation rules
    rule add_metadata {
        lhs { node N :person; }
        rhs { node N :person [active=true]; }
    }

    apply add_metadata 10 times;
}
```

### Running

```bash
# Run with your GGL file
cargo run -- your_graph.ggl

# Run tests
cargo test

```

## Tips

- Run `nix flake update` to update all flake inputs.
- Run `nix --accept-flake-config run github:juspay/omnix ci` to build _all_ outputs.
- [pre-commit] hooks will automatically be setup in Nix shell. You can also run `pre-commit run -a` manually to run the hooks (e.g.: to autoformat the project tree using `rustfmt`, `nixpkgs-fmt`, etc.).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Pest](https://pest.rs/) parser
- [Zulip](https://nixos.zulipchat.com/#narrow/stream/413950-nix)
- [nixos.wiki: Packaging Rust projects with nix](https://nixos.wiki/wiki/Rust#Packaging_Rust_projects_with_nix)
