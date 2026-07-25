use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "xorb",
    author,
    version,
    about = "Bundle directory structure and file contents into LLM-friendly Markdown"
)]
pub struct Cli {
    /// Target directory or file path to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

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

    /// Include only files matching these glob patterns (e.g. -i "*.rs" -i "src/**")
    #[arg(short = 'i', long = "include", value_name = "GLOB")]
    pub include: Option<Vec<String>>,

    /// Exclude files matching these glob patterns (e.g. -e "tests/*" -e "*.tmp")
    #[arg(short = 'e', long = "exclude", value_name = "GLOB")]
    pub exclude: Option<Vec<String>>,

    /// Suppress diagnostic status messages
    #[arg(short, long)]
    pub quiet: bool,
}
