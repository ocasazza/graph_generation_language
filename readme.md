# Graph Generation Language (GGL)

A domain-specific language for creating and manipulating graphs through declarative syntax. GGL allows you to define graph structures, generate common graph topologies, and apply transformation rules to evolve graphs over time.

[![Documentation](https://img.shields.io/badge/docs-latest-blue)](https://ocasazza.github.io/graph_generation_language/rustdoc/share/doc/graph_generation_language/index.html)
[![Demo](https://img.shields.io/badge/demo)](https://ocasazza.github.io/graph_generation_language/demo/index.html)

### Installation

Using Nix Flake

You need [Nix](https://nixos.org/download.html) and [direnv](https://direnv.net/) installed:

```bash
git clone https://github.com/ocasazza/graph-generation-language.git
cd graph-generation-language
direnv allow  # This will automatically set up the development environment
```

This repo uses [Flakes](https://nixos.asia/en/flakes).

```bash
# Dev shell
nix develop

# or run via cargo
nix develop -c cargo run

# build
nix build
```

For building documentation:

```bash
cargo doc --document-private-items --package graph_generation_language --all-features
```

Once built, these docs can be viewed locally at `./graph_generation_language/target/doc/graph_generation_language/index.html`

For more, see nix apps:

```bash
> om show flake.nix

📦 Packages (nix build flake.nix#<name>)
╭───────────────────────────────┬─────────────╮
│ name                          │ description │
├───────────────────────────────┼─────────────┤
│ graphGenerationLanguageCli    │ N/A         │
│ graphGenerationLanguageWasm   │ N/A         │
│ default                       │ N/A         │
╰───────────────────────────────┴─────────────╯

🐚 Devshells (nix develop flake.nix#<name>)
╭─────────┬─────────────╮
│ name    │ description │
├─────────┼─────────────┤
│ default │ N/A         │
╰─────────┴─────────────╯

🚀 Apps (nix run flake.nix#<name>)
╭─────────────────────┬─────────────╮
│ name                │ description │
├─────────────────────┼─────────────┤
│ fmt                 │ N/A         │
│ server              │ N/A         │
│ get-build-artifacts │ N/A         │
│ update-repo-info    │ N/A         │
╰─────────────────────┴─────────────╯

🔍 Checks (nix flake check)
╭───────────────────────────────┬─────────────╮
│ name                          │ description │
├───────────────────────────────┼─────────────┤
│ clippy                        │ N/A         │
│ docs                          │ N/A         │
│ graphGenerationLanguageClient │ N/A         │
│ graphGenerationLanguage       │ N/A         │
│ graphGenerationLanguageCli    │ N/A         │
│ graphGenerationLanguageWasm   │ N/A         │
╰───────────────────────────────┴─────────────╯
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Pest](https://pest.rs/) parser
- [Zulip](https://nixos.zulipchat.com/#narrow/stream/413950-nix)
- [nixos.wiki: Packaging Rust projects with nix](https://nixos.wiki/wiki/Rust#Packaging_Rust_projects_with_nix)
