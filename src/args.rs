use clap::{Arg, Command, ArgAction};
use std::path::PathBuf;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Args {
    pub repo_path: PathBuf,
    pub output_file: String,
    pub ignore_files: HashSet<String>,
    pub ignore_types: HashSet<String>,
    pub exclude_dir: HashSet<String>,
    pub include_dir: Option<PathBuf>,
    pub ignore_settings: bool,
    pub use_gitignore: bool,
}

pub fn parse_args(default_output_file: &str) -> Args {
    let matches = Command::new("repo2txt")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Document the structure of a GitHub repository.")
        .arg(
            Arg::new("repo_path")
                .short('r')
                .long("repo_path")
                .value_name("REPO_PATH")
                .help("Path to the directory to process (i.e., cloned repo). If no path is specified, defaults to the current directory.")
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output_file")
                .value_name("OUTPUT_FILE")
                .help(&format!("Name for the output text file. Defaults to \"{}\".", default_output_file))
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("ignore_files")
                .long("ignore-files")
                .value_name("IGNORE_FILES")
                .help("List of file names to ignore. Omit this argument to ignore no file names.")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("ignore_types")
                .long("ignore-types")
                .value_name("IGNORE_TYPES")
                .help("List of file extensions to ignore. Defaults to list in config.json. Omit this argument to ignore no types.")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("exclude_dir")
                .long("exclude-dir")
                .value_name("EXCLUDE_DIR")
                .help("List of directory names to exclude or \"none\" for no directories.")
                .action(ArgAction::Append)
        )
        .arg(
            Arg::new("ignore_settings")
                .long("ignore-settings")
                .help("Flag to ignore common settings files.")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("include_dir")
                .long("include-dir")
                .value_name("INCLUDE_DIR")
                .help("Specific directory to include. Only contents of this directory will be documented.")
                .action(ArgAction::Set)
        )
        .arg(
            Arg::new("use_gitignore")
                .long("no-gitignore")
                .help("Flag to ignore .gitignore file.")
                .action(ArgAction::SetFalse)
        )
        .get_matches();

    let repo_path = matches.get_one::<String>("repo_path").unwrap_or(&String::from(".")).into();
    let output_file = matches.get_one::<String>("output_file").unwrap_or(&default_output_file.to_string()).to_string();
    let ignore_files: HashSet<String> = matches.get_many::<String>("ignore_files").unwrap_or_default().map(|s| s.to_string()).collect();
    let ignore_types: HashSet<String> = matches.get_many::<String>("ignore_types").unwrap_or_default().map(|s| s.to_string()).collect();
    let exclude_dir: HashSet<String> = matches.get_many::<String>("exclude_dir").unwrap_or_default().map(|s| s.to_string()).collect();
    let include_dir = matches.get_one::<String>("include_dir").map(PathBuf::from);
    let ignore_settings = matches.get_flag("ignore_settings");
    let use_gitignore = !matches.get_flag("use_gitignore");

    Args {
        repo_path,
        output_file,
        ignore_files,
        ignore_types,
        exclude_dir,
        include_dir,
        ignore_settings,
        use_gitignore,
    }
}
