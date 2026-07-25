use anyhow::Result;
use clap::Parser;
use std::fs;
use std::io::{self, Write};

use xorb::cli::Cli;
use xorb::clipboard::copy_to_clipboard;
use xorb::formatter::{estimate_tokens, format_bundle};
use xorb::scanner::scan_directory;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if !cli.quiet {
        eprintln!("Scanning target: {}", cli.path.display());
    }

    let scan_result = scan_directory(&cli)?;
    let bundle = format_bundle(&scan_result, cli.format);
    let token_estimate = estimate_tokens(&bundle);

    let mut copied_to_clipboard = false;

    // Attempt copy to clipboard unless disabled
    if !cli.no_clipboard {
        match copy_to_clipboard(&bundle) {
            Ok(_) => {
                copied_to_clipboard = true;
                if !cli.quiet {
                    eprintln!("Repository bundle successfully copied to system clipboard.");
                }
            }
            Err(e) => {
                if !cli.quiet {
                    eprintln!("Warning: Clipboard unavailable ({})", e);
                }
            }
        }
    }

    // Write to output file if specified
    if let Some(ref output_path) = cli.output {
        fs::write(output_path, &bundle)?;
        if !cli.quiet {
            eprintln!("Repository bundle written to: {}", output_path.display());
        }
    }

    // Print to stdout if explicitly requested or if both clipboard and file output were not used
    if cli.stdout || (!copied_to_clipboard && cli.output.is_none()) {
        io::stdout().write_all(bundle.as_bytes())?;
    }

    if !cli.quiet {
        eprintln!(
            "Scanned {} file(s) | Estimated tokens: {}",
            scan_result.files.len(),
            token_estimate
        );
    }

    Ok(())
}
