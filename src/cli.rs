use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Markdown,
    Xml,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "xorb",
    author,
    version,
    about = "Bundle directory structure and file contents into LLM-friendly formats"
)]
pub struct Cli {
    /// Target directory or file path to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format (markdown, xml, json)
    #[arg(short = 'f', long = "format", value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    /// Write output to a specific file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print output to stdout
    #[arg(short, long)]
    pub stdout: bool,

    /// Skip copying output to system clipboard
    #[arg(short = 'n', long)]
    pub no_clipboard: bool,

    /// Include hidden files and directories (dotfiles)
    #[arg(long)]
    pub hidden: bool,

    /// Do not respect ignore files (.gitignore, .ignore, etc.)
    #[arg(long)]
    pub no_ignore: bool,

    /// Maximum file size in kilobytes to include (0 = unlimited)
    #[arg(long, default_value_t = 1024)]
    pub max_size_kb: u64,

    /// Include only files matching these glob patterns (e.g. -i "*.rs" -i "src/**" or -i src/*.rs)
    #[arg(short = 'i', long = "include", value_name = "GLOB", num_args = 1..)]
    pub include: Option<Vec<String>>,

    /// Exclude files matching these glob patterns (e.g. -e "tests/*" -e "*.tmp")
    #[arg(short = 'e', long = "exclude", value_name = "GLOB", num_args = 1..)]
    pub exclude: Option<Vec<String>>,

    /// Maximum directory recursion depth
    #[arg(short = 'd', long = "max-depth", value_name = "DEPTH")]
    pub max_depth: Option<usize>,

    /// Filter to only files changed according to git diff (optionally supply a ref, e.g. --diff main)
    #[arg(long = "diff", value_name = "REF", num_args = 0..=1, default_missing_value = "HEAD")]
    pub diff: Option<String>,

    /// Suppress diagnostic status messages
    #[arg(short, long)]
    pub quiet: bool,
}
