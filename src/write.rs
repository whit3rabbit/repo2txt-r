use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use ignore::{Walk, DirEntry};
use std::path::{Path, PathBuf};
use globset::GlobSet;
use std::sync::Arc;
use crate::args::Args;
use crate::config::Config;
use crate::utils::is_ignored;

pub fn write_tree(
    walker: Walk,
    output_file: &mut File,
    args: &Args,
    config: &Config,
    prefix: &str,
    max_depth: usize,
    current_depth: usize,
    seen: &mut std::collections::HashSet<PathBuf>,
    globset: Arc<GlobSet>,
    output_file_name: &str,
) -> io::Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for (index, entry) in entries.iter().enumerate() {
        let item = entry.path();

        if item.file_name().map(|f| f.to_str().unwrap_or("")) == Some(output_file_name) {
            continue;
        }

        if is_ignored(item, &globset, args, config) {
            continue;
        }

        let canonicalized_item = item.canonicalize().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to canonicalize path: {:?}. Error: {}", item, e))
        })?;
        if seen.contains(&canonicalized_item) {
            continue;
        }
        seen.insert(canonicalized_item);

        let is_last_item = index == entries.len() - 1;
        let new_prefix = if is_last_item { "└── " } else { "├── " };
        let child_prefix = if is_last_item { "    " } else { "│   " };

        if let Some(file_name) = item.file_name() {
            writeln!(output_file, "{}{}{}", prefix, new_prefix, file_name.to_string_lossy())?;
        }

        if item.is_dir() && current_depth < max_depth {
            let sub_walker = Walk::new(item);
            write_tree(
                sub_walker,
                output_file,
                args,
                config,
                &format!("{}{}", prefix, child_prefix),
                max_depth,
                current_depth + 1,
                seen,
                Arc::clone(&globset),
                output_file_name,
            )?;
        }
    }

    Ok(())
}

pub fn write_file_contents_in_order(
    walker: Walk,
    output_file: &mut File,
    args: &Args,
    config: &Config,
    seen: &mut std::collections::HashSet<PathBuf>,
    globset: Arc<GlobSet>,
    output_file_name: &str,
) -> io::Result<()> {
    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let item = entry.path();

        if item.file_name().map(|f| f.to_str().unwrap_or("")) == Some(output_file_name) {
            continue;
        }

        if is_ignored(item, &globset, args, config) {
            continue;
        }

        let canonicalized_item = item.canonicalize().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to canonicalize path: {:?}. Error: {}", item, e))
        })?;
        if seen.contains(&canonicalized_item) {
            continue;
        }
        seen.insert(canonicalized_item);

        let relative_path = item.strip_prefix(&args.repo_path).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to get relative path for: {:?}. Error: {}", item, e))
        })?;

        if item.is_file() {
            writeln!(output_file, "[File Begins] {}", relative_path.display())?;
            write_file_content(item, output_file, args)?;
            writeln!(output_file, "\n[File Ends] {}", relative_path.display())?;
        }
    }

    Ok(())
}

pub fn write_file_content(file_path: &Path, output_file: &mut File, _args: &Args) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        writeln!(output_file, "{}", line?)?;
    }

    Ok(())
}