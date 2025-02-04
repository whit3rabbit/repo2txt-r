// src/config.rs
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[allow(dead_code)]
    #[serde(default = "default_settings_extensions")]
    pub settings_extensions: HashSet<String>,
    
    #[allow(dead_code)]
    #[serde(default = "default_ignore_types")]
    pub default_ignore_types: HashSet<String>,
    
    #[allow(dead_code)]
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_settings_extensions() -> HashSet<String> {
    [".json", ".yaml", ".yml", ".xml"]
        .iter().map(|s| s.to_string()).collect()
}

fn default_ignore_types() -> HashSet<String> {
    [
        "jpg", "jpeg", "png", "gif", "bmp", "svg", "tiff", "webp",
        "mp4", "avi", "mov", "wmv", "flv", "mkv", "webm",
        "mp3", "wav", "aac", "flac", "ogg", "m4a",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "exe", "dll", "so", "class", "jar", "pyc",
        "zip", "rar", "7z", "tar", "gz", "bz2", "bin", "dat", "db", "log"
    ].iter().map(|s| s.to_string()).collect()
}

fn default_max_depth() -> usize {
    100
}