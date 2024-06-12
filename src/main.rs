use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use serde::Deserialize;
use ignore::{WalkBuilder, DirEntry};

mod args;
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
}

fn load_config() -> Config {
    let json_str = include_str!("config.json");
    serde_json::from_str(json_str).expect("Failed to parse config.json")
}

fn should_ignore(item: &Path, args: &Args, config: &Config, output_file: &str) -> bool {
    let item_name = match item.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return true,
    };

    let file_ext = item.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    
    if item.canonicalize().map_or(false, |p| p == Path::new(output_file).canonicalize().unwrap()) {
        return true;
    }

    if item_name.starts_with('.') {
        return true;
    }

    if item.is_dir() && args.exclude_dir.contains(&item_name.to_string()) {
        return true;
    }

    if let Some(include_dir) = &args.include_dir {
        if !item.canonicalize().map_or(false, |p| p.starts_with(include_dir)) {
            return true;
        }
    }

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

    if item.is_file() && (args.ignore_files.contains(&item_name.to_string()) || ignore_file_types.contains(&file_ext.to_string())) {
        return true;
    }

    if args.ignore_settings && config.settings_extensions.contains(&file_ext.to_string()) {
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

fn write_tree(walker: ignore::Walk, output_file: &mut File, args: &Args, config: &Config, prefix: &str, is_root: bool, max_depth: usize, current_depth: usize, seen: &mut std::collections::HashSet<PathBuf>) -> io::Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    if let Some(root_entry) = entries.get(0) {
        let dir_path = root_entry.path();
        if is_root {
            writeln!(output_file, "{}/", dir_path.file_name().unwrap().to_string_lossy())?;
        }

        for (index, entry) in entries.iter().enumerate() {
            let item = entry.path();
            if should_ignore(item, args, config, &args.output_file) {
                continue;
            }

            let is_last_item = index == entries.len() - 1;
            let new_prefix = if is_last_item { "└── " } else { "├── " };
            let child_prefix = if is_last_item { "    " } else { "│   " };

            writeln!(output_file, "{}{}{}", prefix, new_prefix, item.file_name().unwrap().to_string_lossy())?;

            if item.is_dir() {
                let canonicalized_item = item.canonicalize()?;
                if seen.contains(&canonicalized_item) {
                    continue; // Prevent infinite recursion
                }
                seen.insert(canonicalized_item);
                let sub_walker = WalkBuilder::new(item).build();
                write_tree(sub_walker, output_file, args, config, &format!("{}{}", prefix, child_prefix), false, max_depth, current_depth + 1, seen)?;
            }
        }
    }

    Ok(())
}

fn write_file_content(file_path: &Path, output_file: &mut File) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        writeln!(output_file, "{}", line?)?;
    }

    Ok(())
}

fn write_file_contents_in_order(walker: ignore::Walk, output_file: &mut File, args: &Args, config: &Config, seen: &mut std::collections::HashSet<PathBuf>) -> io::Result<()> {
    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let item = entry.path();
        if should_ignore(item, args, config, &args.output_file) {
            continue;
        }

        let relative_path = item.strip_prefix(&args.repo_path).unwrap();

        if item.is_dir() {
            let canonicalized_item = item.canonicalize()?;
            if seen.contains(&canonicalized_item) {
                continue; // Prevent infinite recursion
            }
            seen.insert(canonicalized_item);
            let sub_walker = WalkBuilder::new(item).build();
            write_file_contents_in_order(sub_walker, output_file, args, config, seen)?;
        } else if item.is_file() {
            writeln!(output_file, "[File Begins] {}", relative_path.display())?;
            write_file_content(item, output_file)?;
            writeln!(output_file, "\n[File Ends] {}", relative_path.display())?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let config = load_config();
    let args = args::parse_args(&config.default_output_file);
    let max_depth = 100; // Set a reasonable max depth to avoid stack overflow

    if !args.repo_path.is_dir() {
        eprintln!("Error: The specified directory does not exist, path is wrong or is not a directory: {}", args.repo_path.display());
        return Ok(());
    }

    let mut output_file = File::create(&args.output_file)?;

    writeln!(output_file, "Repository Documentation")?;
    writeln!(output_file,
        "This document provides a comprehensive overview of the repository's structure and contents.\n\
        The first section, titled 'Directory/File Tree', displays the repository's hierarchy in a tree format.\n\
        In this section, directories and files are listed using tree branches to indicate their structure and relationships.\n\
        Following the tree representation, the 'File Content' section details the contents of each file in the repository.\n\
        Each file's content is introduced with a '[File Begins]' marker followed by the file's relative path,\n\
        and the content is displayed verbatim. The end of each file's content is marked with a '[File Ends]' marker.\n\
        This format ensures a clear and orderly presentation of both the structure and the detailed contents of the repository.\n\n"
    )?;
    writeln!(output_file, "Directory/File Tree Begins -->\n")?;
    let walker = walk_repo(&args.repo_path, &args)?;
    let mut seen = std::collections::HashSet::new();
    write_tree(walker, &mut output_file, &args, &config, "", true, max_depth, 0, &mut seen)?;
    writeln!(output_file, "\n<-- Directory/File Tree Ends")?;
    writeln!(output_file, "\nFile Content Begins -->\n")?;
    let walker = walk_repo(&args.repo_path, &args)?;
    write_file_contents_in_order(walker, &mut output_file, &args, &config, &mut seen)?;
    writeln!(output_file, "\n<-- File Content Ends\n")?;
    
    Ok(())
}
