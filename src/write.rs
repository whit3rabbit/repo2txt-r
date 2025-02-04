// write.rs
use std::fs::File;
use std::io::{self, Write, BufRead, BufReader};
use std::path::Path;
use walkdir::DirEntry;
use crate::args::Args;

pub fn write_tree(entries: &[DirEntry], output_file: &mut File) -> io::Result<()> {
    for entry in entries {
        let depth = entry.depth();
        let prefix = "    ".repeat(depth);
        let marker = if depth > 0 { "└── " } else { "" };
        
        if let Some(file_name) = entry.file_name().to_str() {
            writeln!(output_file, "{}{}{}", prefix, marker, file_name)?;
        }
    }
    Ok(())
}

pub fn write_file_contents(entries: &[DirEntry], output_file: &mut File, args: &Args) -> io::Result<()> {
    for entry in entries.iter().filter(|e| e.file_type().is_file()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(&args.repo_path)
            .unwrap_or(path)
            .to_path_buf();

        writeln!(output_file, "[File Begins] {}", relative_path.display())?;
        write_file_content(path, output_file)?;
        writeln!(output_file, "[File Ends] {}", relative_path.display())?;
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