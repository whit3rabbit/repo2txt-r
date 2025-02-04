use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::sync::Arc;

mod args;
mod config;
mod utils;
mod write;
use crate::args::{Args, OutputFormat, parse_args};
use crate::config::Config;
use crate::utils::walk_entries;
use crate::write::{write_tree, write_file_content, write_file_contents};

fn load_default_config() -> Config {
    let json_str = include_str!("config.json");
    serde_json::from_str(json_str)
        .expect("Failed to parse default config.json")
}

fn load_config_from_file(file_path: &Path) -> io::Result<Config> {
    let file_content = std::fs::read_to_string(file_path)?;
    serde_json::from_str(&file_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse config file: {}", e)))
}

fn create_globset(args: &Args) -> io::Result<Arc<GlobSet>> {
    let mut glob_builder = GlobSetBuilder::new();
    
    // Process ignore_files as exact glob patterns
    for pattern in &args.ignore_files {
        glob_builder.add(Glob::new(pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid ignore file pattern '{}': {}", pattern, e)))?);
    }
    
    // Process ignore_types by trimming leading dots and creating *.ext globs
    for pattern in &args.ignore_types {
        let trimmed = pattern.trim_start_matches('.');
        let glob_pattern = format!("*.{}", trimmed);
        glob_builder.add(Glob::new(&glob_pattern).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid ignore type pattern '{}': {}", pattern, e)))?);
    }
    
    // Build the glob set
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
    // Parse command line arguments
    let args = parse_args();
    println!("Debug: args = {:?}", args);

    // Load configuration
    let config = if let Some(config_path) = &args.config_path {
        load_config_from_file(config_path).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load config from file: {}. Using default config.\nError: {}", config_path.display(), e);
            load_default_config()
        })
    } else {
        load_default_config()
    };
    println!("Debug: config loaded");

    // Create output file
    let mut output_file = File::create(&args.output_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create output file '{}': {}", args.output_file, e)))?;
    println!("Debug: output file created: {}", args.output_file);

    // Get canonical path for output file
    let output_file_path = std::fs::canonicalize(&args.output_file)
        .unwrap_or_else(|_| PathBuf::from(&args.output_file));

    // Write documentation header
    write_header(&mut output_file, &args)?;

    // Determine repository path
    let repo_path = if args.repo_path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        &args.repo_path
    };
    println!("Debug: Using repo_path = {:?}", repo_path);

    // Create glob patterns for file filtering
    let globset = create_globset(&args)?;

    // Handle single file mode vs repository mode
    if let Some(file_path) = &args.file_path {
        if !file_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("The specified file does not exist: {}", file_path.display())));
        }
        write_file_content(file_path, &mut output_file)?;
    } else {
        // Repository mode
        if !repo_path.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("The specified directory does not exist or is not a directory: {}", repo_path.display())));
        }

        // Get all entries
        let entries = walk_entries(repo_path, &args, &config, Arc::clone(&globset), &output_file_path);

        // Write directory tree
        writeln!(output_file, "Directory/File Tree Begins -->\n")?;
        write_tree(&entries, &mut output_file)?;
        writeln!(output_file, "\n<-- Directory/File Tree Ends")?;

        // Write file contents
        writeln!(output_file, "\nFile Content Begins -->\n")?;
        write_file_contents(&entries, &mut output_file, &args)?;
        writeln!(output_file, "\n<-- File Content Ends\n")?;
    }

    if args.output_format == OutputFormat::HTML {
        writeln!(output_file, "</body>\n</html>")?;
    }

    println!("Documentation generated successfully. Output written to: {}", args.output_file);
    Ok(())
}