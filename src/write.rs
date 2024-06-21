use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use ignore::DirEntry;
use crate::args::Args;
use crate::Config;
use crate::should_ignore;

pub fn write_tree(
    walker: ignore::Walk,
    output_file: &mut File,
    args: &Args,
    config: &Config,
    prefix: &str,
    _is_root: bool,
    max_depth: usize,
    current_depth: usize,
    seen: &mut std::collections::HashSet<PathBuf>,
) -> io::Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for (index, entry) in entries.iter().enumerate() {
        let item = entry.path();

        if should_ignore(item, args, config, &args.output_file) {
            continue;
        }

        // Skip if already seen
        let canonicalized_item = match item.canonicalize() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Warning: Unable to canonicalize directory: {:?}\nError: {}", item, e);
                continue;
            }
        };
        if seen.contains(&canonicalized_item) {
            continue;
        }
        seen.insert(canonicalized_item.clone());

        let is_last_item = index == entries.len() - 1;
        let new_prefix = if is_last_item { "└── " } else { "├── " };
        let child_prefix = if is_last_item { "    " } else { "│   " };

        if let Some(file_name) = item.file_name() {
            writeln!(output_file, "{}{}{}", prefix, new_prefix, file_name.to_string_lossy())?;
        } else {
            eprintln!("Warning: Unable to get file name for item: {:?}", item);
            continue;
        }

        if item.is_dir() {
            let sub_walker = WalkBuilder::new(item).build();
            write_tree(
                sub_walker,
                output_file,
                args,
                config,
                &format!("{}{}", prefix, child_prefix),
                false,
                max_depth,
                current_depth + 1,
                seen,
            )?;
        }
    }

    Ok(())
}

pub fn write_file_content(file_path: &Path, output_file: &mut File) -> io::Result<()> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Warning: Unable to open file: {:?}\nError: {}", file_path, e);
            return Ok(());
        }
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(content) => writeln!(output_file, "{}", content)?,
            Err(e) => eprintln!("Warning: Unable to read line in file: {:?}\nError: {}", file_path, e),
        }
    }

    Ok(())
}

pub fn write_file_contents_in_order(
    walker: ignore::Walk,
    output_file: &mut File,
    args: &Args,
    config: &Config,
    seen: &mut std::collections::HashSet<PathBuf>,
) -> io::Result<()> {
    let mut entries: Vec<DirEntry> = walker.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.path().to_path_buf());

    for entry in entries {
        let item = entry.path();
        if should_ignore(item, args, config, &args.output_file) {
            continue;
        }

        let relative_path = match item.strip_prefix(&args.repo_path) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Warning: Unable to get relative path for item: {:?}\nError: {}", item, e);
                continue;
            }
        };

        if item.is_dir() {
            let canonicalized_item = match item.canonicalize() {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Warning: Unable to canonicalize directory: {:?}\nError: {}", item, e);
                    continue;
                }
            };
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
