use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use serde::Deserialize;
use ignore::WalkBuilder;
use std::collections::HashSet;

mod args;
mod write;
use crate::args::Args;

#[derive(Deserialize)]
struct Config {
    image_extensions: Vec<String>,
    video_extensions: Vec<String>,
    audio_extensions: Vec<String>,
    document_extensions: Vec<String>,
    executable_extensions: Vec<String>,
    settings_extensions: Vec<String>,
    additional_ignore_types: Vec<String>,
    default_output_file: String,
    default_ignore_types: HashSet<String>,
}

fn load_default_config() -> Config {
    let json_str = include_str!("config.json");
    let mut config: Config = serde_json::from_str(json_str).expect("Failed to parse default config.json");
    
    // Combine all default ignore types into a single HashSet
    config.default_ignore_types = config.image_extensions.iter()
        .chain(config.video_extensions.iter())
        .chain(config.audio_extensions.iter())
        .chain(config.document_extensions.iter())
        .chain(config.executable_extensions.iter())
        .chain(config.additional_ignore_types.iter())
        .cloned()
        .collect();
    
    config
}

fn load_config_from_file(file_path: &Path) -> io::Result<Config> {
    let file_content = std::fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&file_content)?;
    Ok(config)
}

fn should_ignore(item: &Path, args: &Args, config: &Config, output_file: &str) -> bool {
    let item_name = match item.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return true,
    };

    let file_ext = item.extension()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Check if the file extension is in the ignore list
    if item.is_file() && (
        args.ignore_files.contains(&item_name.to_string()) ||
        args.ignore_types.contains(&file_ext) ||
        (args.ignore_types.is_empty() && config.default_ignore_types.contains(&format!(".{}", file_ext)))
    ) {
        return true;
    }

    // Ignore the output file itself
    if item.canonicalize().map_or(false, |p| p == Path::new(output_file).canonicalize().unwrap()) {
        return true;
    }

    // Ignore hidden files and directories (starting with '.')
    if item_name.starts_with('.') {
        return true;
    }

    // Ignore directories listed in exclude_dir
    if item.is_dir() && args.exclude_dir.contains(&item_name.to_string()) {
        return true;
    }

    // Only include files in include_dir if specified
    if let Some(include_dir) = &args.include_dir {
        if !item.canonicalize().map_or(false, |p| p.starts_with(include_dir)) {
            return true;
        }
    }

    // Collect all file extensions to be ignored
    let ignore_file_types: Vec<String> = [
        &config.image_extensions,
        &config.video_extensions,
        &config.audio_extensions,
        &config.document_extensions,
        &config.executable_extensions,
        &config.additional_ignore_types,
    ]
    .iter()
    .flat_map(|v| v.iter().cloned())
    .collect();

    // Ignore files with extensions specified in the configuration
    if item.is_file() && (args.ignore_files.contains(&item_name.to_string()) || ignore_file_types.contains(&file_ext)) {
        return true;
    }

    // Ignore common settings files if the flag is set
    if args.ignore_settings && config.settings_extensions.contains(&file_ext) {
        return true;
    }

    false
}

fn walk_repo(repo_path: &Path, args: &Args) -> io::Result<ignore::Walk> {
    let mut walker = WalkBuilder::new(repo_path);

    if args.use_gitignore {
        walker.git_ignore(true);
    }

    walker.standard_filters(true).follow_links(false);
    Ok(walker.build())
}

fn process_single_file(file_path: &Path, output_file: &mut File) -> io::Result<()> {
    writeln!(output_file, "\nFile Content Begins -->\n")?;
    let relative_path = file_path.strip_prefix(".").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(output_file, "[File Begins] {}", relative_path.display())?;
    write::write_file_content(file_path, output_file)?;
    writeln!(output_file, "\n[File Ends] {}", relative_path.display())?;
    writeln!(output_file, "\n<-- File Content Ends\n")?;
    Ok(())
}

fn main() -> io::Result<()> {
    let default_config = load_default_config();
    let args = args::parse_args(&default_config.default_output_file);
    let max_depth = 100;

    let config = if let Some(config_path) = &args.config_path {
        match load_config_from_file(config_path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: Failed to load config from file: {}. Using default config.\nError: {}", config_path.display(), e);
                default_config
            }
        }
    } else {
        default_config
    };

    let mut output_file = File::create(&args.output_file)
        .map_err(|e| {
            eprintln!("Error: Unable to create output file: {}\nError: {}", args.output_file, e);
            e
        })?;

    writeln!(output_file, "Repository Documentation")?;
    writeln!(output_file, "This document provides an overview of the repository's structure and contents.")?;
    writeln!(output_file, "The 'Directory/File Tree' section displays the repository's hierarchy.")?;
    writeln!(output_file, "The 'File Content' section details the contents of each file.")?;
    writeln!(output_file, "File contents are marked with '[File Begins]' and '[File Ends]' tags.\n")?;

    let repo_path = if args.repo_path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        &args.repo_path
    };

    if let Some(file_path) = &args.file_path {
        // If a file path is provided, process the single file
        if !file_path.exists() {
            eprintln!("Error: The specified file does not exist: {}", file_path.display());
            return Ok(());
        }

        process_single_file(file_path, &mut output_file)?;
    } else {
        // If no file path is provided, process the directory
        if !repo_path.is_dir() {
            eprintln!("Error: The specified directory does not exist, path is wrong or is not a directory: {}", repo_path.display());
            return Ok(());
        }

        writeln!(output_file, "Directory/File Tree Begins -->\n")?;
        let walker = walk_repo(repo_path, &args)?;
        let mut seen = std::collections::HashSet::new();
        write::write_tree(walker, &mut output_file, &args, &config, "", true, max_depth, 0, &mut seen)?;
        writeln!(output_file, "\n<-- Directory/File Tree Ends")?;

        writeln!(output_file, "\nFile Content Begins -->\n")?;
        let walker = walk_repo(repo_path, &args)?;
        write::write_file_contents_in_order(walker, &mut output_file, &args, &config, &mut seen)?;
        writeln!(output_file, "\n<-- File Content Ends\n")?;
    }

    Ok(())
}
