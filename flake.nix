{
  description = "Build a cargo project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;
        rustToolchainFor =
          p:
          p.rust-bin.stable.latest.default.override {
            # Set the build targets supported by the toolchain,
            # wasm32-unknown-unknown is required for trunk.
            targets = [ "wasm32-unknown-unknown" ];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;
        # When filtering sources, we want to allow assets other than .rs files
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "html"
                "scss"
                "pest"
                "ggl"
              ]
            ) unfilteredRoot)
          ];
        };

        commonArgs = {
          # ! this can be optimized
          inherit src;
          strictDeps = true;
          buildInputs = [
            pkgs.cacert
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.openssl
            # Additional darwin specific inputs can be set here
          ] ++ lib.optionals pkgs.stdenv.isLinux [
            pkgs.openssl
            pkgs.pkg-config
          ];
          # Set SSL certificate environment variables
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
        };

        # ---------------------------------------
        # build the native packages
        # i.e. non-wasm / non-browser stuff first
        # ---------------------------------------
        nativeArgs = commonArgs // {
          pname = "graph_generation_language";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly nativeArgs;

        # ---------------------------------
        # build the library / ggl rust crate
        # that can be published to crates.io
        # ---------------------------------
        graphGenerationLanguage = craneLib.buildPackage (
          nativeArgs
          // {
            inherit cargoArtifacts;
          }
        );

        # -----------------------------
        # build a native cli application
        # that uses the ggl library
        # -----------------------------
        graphGenerationLanguageCli = craneLib.buildPackage (
          nativeArgs
          // {
            pname = "ggl_cli";
            inherit cargoArtifacts;
          }
        );

        # ----------------------------
        # build the WASM library that
        # can be published to npm
        # ----------------------------
        wasmArgs = commonArgs // {
          pname = "ggl_wasm";
          cargoExtraArgs = "--package=ggl_wasm";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
        wasmCargoArtifacts = craneLib.buildDepsOnly (
          wasmArgs
          // {
            doCheck = false;
          }
        );
        graphGenerationLanguageWasm = craneLib.mkCargoDerivation (wasmArgs // {
          cargoArtifacts = wasmCargoArtifacts;
          doCheck = false;
          buildPhaseCargoCommand = ''
            HOME=$(mktemp -d fake-homeXXXX)
            cd src/wasm
            wasm-pack build --target web --out-dir pkg
            cd ../..
          '';
          installPhaseCommand = ''
            mkdir -p $out
            cp -r ./src/wasm/pkg $out/
          '';
          nativeBuildInputs = with pkgs; [
            binaryen
            wasm-bindgen-cli
            wasm-pack
            nodejs
            cacert
          ] ++ lib.optionals stdenv.isLinux [
            # Linux-specific tools for debugging if needed
            # pkgs.strace
          ] ++ lib.optionals stdenv.isDarwin [
            pkgs.libiconv
            pkgs.openssl
          ];
        });

      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit graphGenerationLanguage graphGenerationLanguageCli graphGenerationLanguageWasm;
          docs = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
        };

        packages.default = graphGenerationLanguage;
        packages.graphGenerationLanguageCli = graphGenerationLanguageCli;
        packages.graphGenerationLanguageWasm = graphGenerationLanguageWasm;

        apps.ggl = flake-utils.lib.mkApp {
          name = "graphGenerationLanguageCli";
          drv = graphGenerationLanguageCli;
        };

        # app to copy all outpaths from omnix result to local ./artifacts folder
        apps.get-build-artifacts = flake-utils.lib.mkApp {
          name = "get-build-artifacts";
          drv = pkgs.writeShellScriptBin "get-build-artifacts" ''
            set -euo pipefail
            rm -rf artifacts
            mkdir -p artifacts
            jq -r '.result.ROOT.build.byName | to_entries[] | "\(.key):\(.value)"' result | while IFS=':' read -r name path; do
              [ -e "$path" ] && mkdir -p "artifacts/$name" && cp -r "$path" "artifacts/$name/"
            done
          '';
        };

        # app to update repository statistics, coverage info, etc
        apps.update-repo-info = flake-utils.lib.mkApp {
          name = "update-repo-info";
          drv = pkgs.writeShellScriptBin "update-repo-info" ''
            set -euo pipefail
            echo "" > COVERAGE.md
            echo "# Project Information and Code Coverage" >> COVERAGE.md
            echo "## Code Statistics" >> COVERAGE.md
            nix develop -c tokei --hidden -C >> COVERAGE.md
          '';
        };

        apps.fmt = flake-utils.lib.mkApp {
          name = "fmt";
          drv = pkgs.writeShellScriptBin "fmt" ''
            set -euo pipefail
            nix develop -c cargo clippy --all-targets --fix --allow-dirty
          '';
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
          shellHook = ''
            # Ensure rust-analyzer can find the toolchain
            export RUST_SRC_PATH="${rustToolchainFor pkgs}/lib/rustlib/src/rust/library";
          '';
          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.act
            pkgs.rust-analyzer
            pkgs.rustup
            pkgs.trunk
            pkgs.wasm-pack
            pkgs.tokei
          ];
        };
      }
    );
}
