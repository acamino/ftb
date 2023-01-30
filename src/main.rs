use clap::Parser;
use ftb::TableFormatter;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

/// Format and align Markdown tables
///
/// Reads a Markdown table from stdin or a file and outputs a properly aligned version.
/// Perfect for use with pipes: pbpaste | ftb
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input file (reads from stdin if not provided)
    input: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    // Read input from file or stdin
    let input = match cli.input {
        Some(path) => fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", path.display(), e);
            process::exit(1);
        }),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).unwrap_or_else(|e| {
                eprintln!("Error reading from stdin: {}", e);
                process::exit(1);
            });
            buffer
        }
    };

    // Format the table
    let mut formatter = TableFormatter::new();
    let output = formatter.format_table(&input);

    // Write to stdout
    print!("{}", output);
}
