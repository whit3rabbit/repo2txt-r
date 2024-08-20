use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::collections::HashSet;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::sync::Arc;

mod args;
mod config;
mod utils;
mod write;
use crate::args::{Args, OutputFormat, parse_args};
use crate::config::Config;
use crate::utils::walk_repo;
use crate::write::{write_tree, write_file_contents_in_order, write_file_content};

fn load_default_config() -> Config {
    let json_str = include_str!("config.json");
    serde_json::from_str(json_str)
        .expect("Failed to parse default config.json")
}

fn load_config_from_file(file_path: &Path) -> io::Result<Config> {
    let file_content = std::fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&file_content)?;
    Ok(Config::new(config))
}

fn create_globset(args: &Args, config: &Config) -> io::Result<Arc<GlobSet>> {
    let mut glob_builder = GlobSetBuilder::new();
    
    for pattern in &args.ignore_files {
        glob_builder.add(Glob::new(pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid ignore file pattern '{}': {}", pattern, e)))?);
    }
    for pattern in &args.ignore_types {
        glob_builder.add(Glob::new(&format!("*.{}", pattern)).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid ignore type pattern '{}': {}", pattern, e)))?);
    }
    for pattern in &config.default_ignore_types {
        if !args.ignore_types.contains(pattern) {
            glob_builder.add(Glob::new(pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid default ignore type pattern '{}': {}", pattern, e)))?);
        }
    }
    
    glob_builder.build()
        .map(Arc::new)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Failed to build GlobSet: {}", e)))
}

fn write_header(output_file: &mut File, args: &Args) -> io::Result<()> {
    match args.output_format {
        OutputFormat::Text => {
            writeln!(output_file, "Repository Documentation")?;
            writeln!(output_file, "This document provides an overview of the repository's structure and contents.")?;
            writeln!(output_file, "The 'Directory/File Tree' section displays the repository's hierarchy.")?;
            writeln!(output_file, "The 'File Content' section details the contents of each file.")?;
            writeln!(output_file, "File contents are marked with '[File Begins]' and '[File Ends]' tags.\n")?;
        },
        OutputFormat::Markdown => {
            writeln!(output_file, "# Repository Documentation")?;
            writeln!(output_file, "This document provides an overview of the repository's structure and contents.\n")?;
            writeln!(output_file, "## Directory/File Tree")?;
            writeln!(output_file, "The following section displays the repository's hierarchy.\n")?;
            writeln!(output_file, "## File Content")?;
            writeln!(output_file, "This section details the contents of each file.\n")?;
        },
        OutputFormat::HTML => {
            writeln!(output_file, "<!DOCTYPE html>")?;
            writeln!(output_file, "<html lang=\"en\">")?;
            writeln!(output_file, "<head>")?;
            writeln!(output_file, "    <meta charset=\"UTF-8\">")?;
            writeln!(output_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
            writeln!(output_file, "    <title>Repository Documentation</title>")?;
            writeln!(output_file, "</head>")?;
            writeln!(output_file, "<body>")?;
            writeln!(output_file, "    <h1>Repository Documentation</h1>")?;
            writeln!(output_file, "    <p>This document provides an overview of the repository's structure and contents.</p>")?;
            writeln!(output_file, "    <h2>Directory/File Tree</h2>")?;
            writeln!(output_file, "    <p>The following section displays the repository's hierarchy.</p>")?;
            writeln!(output_file, "    <h2>File Content</h2>")?;
            writeln!(output_file, "    <p>This section details the contents of each file.</p>")?;
        },
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = parse_args();
    println!("Debug: args = {:?}", args);

    let config = if let Some(config_path) = &args.config_path {
        load_config_from_file(config_path).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config from file: {}. Using default config.\nError: {}", config_path.display(), e);
            load_default_config()
        })
    } else {
        load_default_config()
    };
    println!("Debug: config loaded");

    let mut output_file = File::create(&args.output_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create output file '{}': {}", args.output_file, e)))?;
    println!("Debug: output file created: {}", args.output_file);

    write_header(&mut output_file, &args)?;

    let repo_path = if args.repo_path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        &args.repo_path
    };
    println!("Debug: Using repo_path = {:?}", repo_path);

    if let Some(file_path) = &args.file_path {
        if !file_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("The specified file does not exist: {}", file_path.display())));
        }
        write_file_content(file_path, &mut output_file, &args)?;
    } else {
        if !repo_path.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("The specified directory does not exist or is not a directory: {}", repo_path.display())));
        }

        let globset = create_globset(&args, &config)?;
        println!("Debug: GlobSet created");

        writeln!(output_file, "Directory/File Tree Begins -->\n")?;
        let mut seen_tree = HashSet::new();
        let walker_tree = walk_repo(repo_path, &args, &config, Arc::clone(&globset));
        println!("Debug: Walker created for tree");
        write_tree(walker_tree, &mut output_file, &args, &config, "", args.max_depth, 0, &mut seen_tree, Arc::clone(&globset), &args.output_file)?;
        writeln!(output_file, "\n<-- Directory/File Tree Ends")?;

        writeln!(output_file, "\nFile Content Begins -->\n")?;
        let mut seen_content = HashSet::new();
        let walker_content = walk_repo(repo_path, &args, &config, Arc::clone(&globset));
        println!("Debug: Walker created for content");
        write_file_contents_in_order(walker_content, &mut output_file, &args, &config, &mut seen_content, globset, &args.output_file)?;
        writeln!(output_file, "\n<-- File Content Ends\n")?;
    }

    println!("Documentation generated successfully. Output written to: {}", args.output_file);
    Ok(())
}