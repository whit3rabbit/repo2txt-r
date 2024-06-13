use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use ignore::DirEntry;
use crate::args::Args;
use crate::Config;
use crate::should_ignore;


pub fn write_tree(walker: ignore::Walk, output_file: &mut File, args: &Args, config: &Config, prefix: &str, is_root: bool, max_depth: usize, current_depth: usize, seen: &mut std::collections::HashSet<PathBuf>) -> io::Result<()> {
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

pub fn write_file_content(file_path: &Path, output_file: &mut File) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        writeln!(output_file, "{}", line?)?;
    }

    Ok(())
}

pub fn write_file_contents_in_order(walker: ignore::Walk, output_file: &mut File, args: &Args, config: &Config, seen: &mut std::collections::HashSet<PathBuf>) -> io::Result<()> {
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