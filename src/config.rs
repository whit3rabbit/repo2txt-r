use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "default_ignore_types")]
    pub default_ignore_types_vec: Vec<String>,
    #[serde(skip)]
    pub default_ignore_types: HashSet<String>,
}

impl Config {
    pub fn new(mut config: Config) -> Self {
        config.default_ignore_types = config.default_ignore_types_vec.iter().cloned().collect();
        config
    }
}