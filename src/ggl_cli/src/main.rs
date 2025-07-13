use clap::Parser;
use ggl::GGLEngine;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author = "Olive Casazza",
    version,
    about = "Graph Generation Language CLI"
)]
/// Command-line interface for the Graph Generation Language (GGL)
struct Args {
    /// Input GGL file to process
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file for the generated graph JSON (defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print the JSON output
    #[arg(short, long)]
    pretty: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.verbose {
        eprintln!(
            "Graph Generation Language CLI v{}",
            env!("CARGO_PKG_VERSION")
        );
    }

    // Read input
    let ggl_code = match args.input {
        Some(path) => {
            if args.verbose {
                eprintln!("Reading GGL code from: {}", path.display());
            }
            fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read input file '{}': {}", path.display(), e))?
        }
        None => {
            if args.verbose {
                eprintln!("Reading GGL code from stdin...");
            }
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .map_err(|e| format!("Failed to read from stdin: {e}"))?;
            buffer
        }
    };

    if args.verbose {
        eprintln!("Processing GGL code ({} characters)", ggl_code.len());
    }

    // Process with GGL engine
    let mut engine = GGLEngine::new();
    let result = engine
        .generate_from_ggl(&ggl_code)
        .map_err(|e| format!("GGL processing error: {e}"))?;

    // Format output
    let output = if args.pretty {
        let parsed: serde_json::Value = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse generated JSON: {e}"))?;
        serde_json::to_string_pretty(&parsed)
            .map_err(|e| format!("Failed to format JSON: {e}"))?
    } else {
        result
    };

    // Write output
    match args.output {
        Some(path) => {
            if args.verbose {
                eprintln!("Writing output to: {}", path.display());
            }
            fs::write(&path, &output)
                .map_err(|e| format!("Failed to write output file '{}': {}", path.display(), e))?;
        }
        None => {
            println!("{output}");
        }
    }

    if args.verbose {
        eprintln!("Processing completed successfully");
    }

    Ok(())
}
