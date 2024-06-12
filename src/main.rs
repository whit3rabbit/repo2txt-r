use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;
use serde::Deserialize;

mod args;
use crate::args::Args;

#[derive(Deserialize)]
struct Config {
    settings_extensions: Vec<String>,
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

    if item.is_file() && (args.ignore_files.contains(&item_name.to_string()) || args.ignore_types.contains(&file_ext)) {
        return true;
    }

    if args.ignore_settings && config.settings_extensions.contains(&file_ext) {
        return true;
    }

    false
}

fn write_tree(dir_path: &Path, output_file: &mut File, args: &Args, config: &Config, prefix: &str, is_root: bool) -> io::Result<()> {
    if is_root {
        writeln!(output_file, "{}/", dir_path.file_name().unwrap().to_string_lossy())?;
    }

    let mut items: Vec<_> = fs::read_dir(dir_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<_, io::Error>>()?;
    items.sort();

    for (index, item) in items.iter().enumerate() {
        if should_ignore(item, args, config, &args.output_file) {
            continue;
        }

        let is_last_item = index == items.len() - 1;
        let new_prefix = if is_last_item { "└── " } else { "├── " };
        let child_prefix = if is_last_item { "    " } else { "│   " };

        writeln!(output_file, "{}{}{}", prefix, new_prefix, item.file_name().unwrap().to_string_lossy())?;

        if item.is_dir() {
            write_tree(&item, output_file, args, config, &format!("{}{}", prefix, child_prefix), false)?;
        }
    }

    Ok(())
}

fn write_file_content(file_path: &Path, output_file: &mut File, depth: usize) -> io::Result<()> {
    let indentation = "  ".repeat(depth);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        writeln!(output_file, "{}{}", indentation, line?)?;
    }

    Ok(())
}

fn write_file_contents_in_order(dir_path: &Path, output_file: &mut File, args: &Args, config: &Config, depth: usize) -> io::Result<()> {
    let items: Vec<_> = fs::read_dir(dir_path)?
        .map(|res| res.map(|e| e.path()))
        .filter(|path| !should_ignore(path.as_ref().unwrap(), args, config, &args.output_file))
        .collect::<Result<_, io::Error>>()?;

    for item in items {
        let relative_path = item.strip_prefix(&args.repo_path).unwrap();

        if item.is_dir() {
            write_file_contents_in_order(&item, output_file, args, config, depth + 1)?;
        } else if item.is_file() {
            writeln!(output_file, "{}[File Begins] {}", "  ".repeat(depth), relative_path.display())?;
            write_file_content(&item, output_file, depth)?;
            writeln!(output_file, "\n{}[File Ends] {}", "  ".repeat(depth), relative_path.display())?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = args::parse_args();
    let config = load_config();

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
    write_tree(&args.repo_path, &mut output_file, &args, &config, "", true)?;
    writeln!(output_file, "\n<-- Directory/File Tree Ends")?;
    writeln!(output_file, "\nFile Content Begins -->\n")?;
    write_file_contents_in_order(&args.repo_path, &mut output_file, &args, &config, 0)?;
    writeln!(output_file, "\n<-- File Content Ends\n")?;
    
    Ok(())
}
