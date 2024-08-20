use std::path::Path;
use globset::GlobSet;
use std::sync::Arc;
use ignore::{WalkBuilder, Walk};
use crate::args::Args;
use crate::config::Config;

pub fn is_ignored(path: &Path, globset: &Arc<GlobSet>, args: &Args, _config: &Config) -> bool {
    let path_str = path.to_str().unwrap_or("");

    // Check custom ignore patterns
    if globset.is_match(path_str) {
        println!("Debug: Ignored by globset: {:?}", path);
        return true;
    }

    // Check excluded directories
    if path.is_dir() && args.exclude_dir.contains(&path.file_name().unwrap_or_default().to_string_lossy().to_string()) {
        println!("Debug: Ignored directory: {:?}", path);
        return true;
    }

    // Check if path is within the included directory (if specified)
    if let Some(include_dir) = &args.include_dir {
        if !path.starts_with(include_dir) {
            println!("Debug: Not in included directory: {:?}", path);
            return true;
        }
    }

    // Ignore hidden files and directories unless explicitly included
    if !args.include_hidden {
        if let Some(file_name) = path.file_name() {
            if file_name.to_str().map(|s| s.starts_with('.')).unwrap_or(false) {
                println!("Debug: Ignored hidden file/directory: {:?}", path);
                return true;
            }
        }
    }

    println!("Debug: Not ignored: {:?}", path);
    false
}

pub fn walk_repo(repo_path: &Path, args: &Args, config: &Config, globset: Arc<GlobSet>) -> Walk {
    println!("Debug: Entering walk_repo function");
    let mut builder = WalkBuilder::new(repo_path);
    println!("Debug: WalkBuilder created for path: {:?}", repo_path);

    if args.use_gitignore {
        builder.git_ignore(true);
        builder.ignore(true);
        println!("Debug: Using .gitignore");
    }

    let args_clone = args.clone();
    let config_clone = config.clone();
    let globset_clone = Arc::clone(&globset);

    builder.filter_entry(move |entry| {
        let should_include = !is_ignored(entry.path(), &globset_clone, &args_clone, &config_clone);
        println!("Debug: Checking entry: {:?}, Include: {}", entry.path(), should_include);
        should_include
    });
    builder.standard_filters(false).follow_links(args.follow_symlinks);

    println!("Debug: Returning Walk object");
    builder.build()
}