use std::path::Path;
use globset::GlobSet;
use std::sync::Arc;
use walkdir::{DirEntry, WalkDir};
use crate::args::Args;
use crate::config::Config;

pub fn is_ignored(
    entry: &DirEntry,
    globset: &Arc<GlobSet>,
    args: &Args,
    _config: &Config,
    output_file_path: &Path
) -> bool {
    let path = entry.path();
    
    // Check output file using canonical path
    if let Ok(canonical_path) = path.canonicalize() {
        if canonical_path == output_file_path {
            return true;
        }
    }
    
    // Handle include_dir first - if specified, check both containment and ancestry
    if let Some(include_dir) = &args.include_dir {
        // Allow paths that are either:
        // 1. Inside the include_dir
        // 2. Are ancestors of include_dir (needed to traverse to it)
        if !path.starts_with(include_dir) && !include_dir.starts_with(path) {
            return true;
        }
    }

    // Check depth
    if entry.depth() > args.max_depth {
        return true;
    }

    // Skip excluded directories
    if path.is_dir() {
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if args.exclude_dir.contains(dir_name) {
                return true;
            }
        }
    }

    // Handle file-specific filters for non-directory entries
    if path.is_file() {
        // Check file name against ignore_files
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if args.ignore_files.contains(file_name) {
                return true;
            }
        }

        // Check extensions against ignore_types
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if args.ignore_types.contains(&ext.to_lowercase()) {
                return true;
            }
        }

        // Handle settings files
        if args.ignore_settings {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("json") || 
                   ext.eq_ignore_ascii_case("yaml") || 
                   ext.eq_ignore_ascii_case("yml") {
                    return true;
                }
            }
        }
    }

    // Check against globset patterns
    if !globset.is_empty() && globset.is_match(path) {
        return true;
    }

    false
}

pub fn walk_entries(
    path: &Path,
    args: &Args,
    config: &Config,
    globset: Arc<GlobSet>,
    output_file_path: &Path
) -> Vec<DirEntry> {
    WalkDir::new(path)
        .min_depth(0)
        .max_depth(args.max_depth)
        .follow_links(args.follow_symlinks)
        .into_iter()
        .filter_entry(|e| !is_ignored(e, &globset, args, config, output_file_path))
        .filter_map(|e| e.ok())
        .collect()
}