use clap::{Parser, ValueEnum};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::hash::Hash;

fn parse_set<T: FromStr + Eq + Hash>(s: &str) -> Result<HashSet<T>, String> {
    s.split(',')
     .map(str::trim)
     .map(|item| T::from_str(item).map_err(|_| format!("Failed to parse: {}", item)))
     .collect()
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "repo2txt",
    version = "0.074",
    author = "whiterabbit@protonmail.com",
    about = "Repo2txt helps you document the structure of a GitHub repository."
)]
pub struct Args {
    #[arg(short, long, value_name = "REPO_PATH", help = "Path to the directory to process (i.e., cloned repo). If no path is specified, defaults to the current directory.", default_value = ".")]
    pub repo_path: PathBuf,

    #[arg(short, long, value_name = "OUTPUT_FILE", help = "Name for the output file. Defaults to \"output.txt\".", default_value = "output.txt")]
    pub output_file: String,

    #[arg(long, value_name = "IGNORE_FILES", help = "List of file names or patterns to ignore. Use glob patterns for wildcards.", default_value = "", value_parser = parse_set::<String>)]
    pub ignore_files: HashSet<String>,

    #[arg(long, value_name = "IGNORE_TYPES", help = "List of file extensions to ignore.", default_value = "", value_parser = parse_set::<String>)]
    pub ignore_types: HashSet<String>,

    #[arg(long, value_name = "EXCLUDE_DIR", help = "List of directory names to exclude.", default_value = "node_modules,vendor,dist,build,target", value_parser = parse_set::<String>)]
    pub exclude_dir: HashSet<String>,

    #[arg(long, value_name = "INCLUDE_DIR", help = "Specific directory to include. Only contents of this directory will be documented.")]
    pub include_dir: Option<PathBuf>,

    #[arg(
        long,
        help = "Flag to ignore common settings files [possible values: true, false]",
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_value_t = true,
        default_missing_value = "true"
    )]
    pub ignore_settings: bool,

    #[arg(
        long,
        help = "Use .gitignore file for ignoring files [possible values: true, false]",
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_value_t = true,
        default_missing_value = "true"
    )]
    pub use_gitignore: bool,

    #[arg(short, long, value_name = "FILE_PATH", help = "Path to a single file to process.")]
    pub file_path: Option<PathBuf>,

    #[arg(long, value_name = "CONFIG_PATH", help = "Path to a custom configuration file. If not specified, the default configuration is used.")]
    pub config_path: Option<PathBuf>,

    #[arg(
        long,
        help = "Follow symbolic links when traversing the repository [possible values: true, false]",
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true"
    )]
    pub follow_symlinks: bool,

    #[arg(long, value_name = "MAX_DEPTH", help = "Maximum depth to traverse in the directory tree. Default is 100.", default_value_t = 100)]
    pub max_depth: usize,

    #[arg(long, value_name = "FORMAT", help = "Output format: text, markdown, or html. Default is text.", value_enum, default_value_t = OutputFormat::Text)]
    pub output_format: OutputFormat,

    #[arg(
        long,
        help = "Include hidden files and directories in the documentation [possible values: true, false]",
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        num_args = 0..=1,
        default_value_t = false,
        default_missing_value = "true"
    )]
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Markdown,
    HTML,
}

pub fn parse_args() -> Args {
    Args::parse()
}