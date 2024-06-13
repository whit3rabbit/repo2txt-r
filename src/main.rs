use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use serde::Deserialize;
use ignore::WalkBuilder;

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

fn main() -> io::Result<()> {
    let config = load_config();
    let args = args::parse_args(&config.default_output_file);
    let max_depth = 100;

    if !args.repo_path.is_dir() {
        eprintln!("Error: The specified directory does not exist, path is wrong or is not a directory: {}", args.repo_path.display());
        return Ok(());
    }

    let mut output_file = File::create(&args.output_file)?;

    writeln!(output_file, "Repository Documentation")?;
    writeln!(output_file, "...")?; // ... (write the introduction)

    writeln!(output_file, "Directory/File Tree Begins -->\n")?;
    let walker = walk_repo(&args.repo_path, &args)?;
    let mut seen = std::collections::HashSet::new();
    write::write_tree(walker, &mut output_file, &args, &config, "", true, max_depth, 0, &mut seen)?;
    writeln!(output_file, "\n<-- Directory/File Tree Ends")?;

    writeln!(output_file, "\nFile Content Begins -->\n")?;
    let walker = walk_repo(&args.repo_path, &args)?;
    write::write_file_contents_in_order(walker, &mut output_file, &args, &config, &mut seen)?;
    writeln!(output_file, "\n<-- File Content Ends\n")?;

    Ok(())
}