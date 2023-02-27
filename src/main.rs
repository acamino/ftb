use clap::Parser;
use ftb::TableFormatter;
use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};
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

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    const MAX_INPUT_SIZE: u64 = 10 * 1024 * 1024; // 10MB

    let input = if let Some(path) = cli.input {
        read_file(&path, MAX_INPUT_SIZE)?
    } else {
        read_stdin(MAX_INPUT_SIZE)?
    };

    let mut formatter = TableFormatter::new();
    // Use format_document to handle full Markdown files with tables
    let output = formatter.format_document(&input);

    print!("{output}");
    Ok(())
}

fn read_file(path: &Path, max_size: u64) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            format!(
                "File not found: {}\nHint: Check the file path is correct",
                path.display()
            )
        }
        ErrorKind::PermissionDenied => {
            format!(
                "Permission denied: {}\nHint: Check file permissions with: ls -l {}",
                path.display(),
                path.display()
            )
        }
        _ => format!("Cannot access file: {}\nError: {}", path.display(), e),
    })?;

    if metadata.len() > max_size {
        return Err(format!(
            "File too large: {:.2} MB (max {:.2} MB)",
            metadata.len() as f64 / 1_048_576.0,
            max_size as f64 / 1_048_576.0
        )
        .into());
    }

    if !metadata.is_file() {
        return Err(format!(
            "Not a regular file: {}\n\
            Hint: Provide a path to a text file containing a Markdown table",
            path.display()
        )
        .into());
    }

    fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}\nError: {}", path.display(), e).into())
}

fn read_stdin(max_size: u64) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock().take(max_size);

    handle
        .read_to_string(&mut buffer)
        .map_err(|e| format!("Failed to read from stdin: {e}"))?;

    if buffer.len() as u64 >= max_size {
        return Err(format!(
            "Input too large (max {:.2} MB)",
            max_size as f64 / 1_048_576.0
        )
        .into());
    }

    Ok(buffer)
}
