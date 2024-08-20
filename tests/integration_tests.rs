use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;

fn default_args() -> Vec<String> {
    vec![
        "--ignore-files".into(),
        "".into(),  // Default empty value for ignore files
        "--ignore-types".into(),
        "".into(),  // Default empty value for ignore types
        "--exclude-dir".into(),
        "".into(),  // Default empty value for exclude dir
    ]
}

#[test]
fn test_default_output() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    std::fs::create_dir_all(&repo_path).unwrap();
    
    let file_path = repo_path.join("README.md");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "# Sample Repo").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");
    
    let mut cmd = Command::cargo_bin("repo2txt").unwrap();
    cmd.args(&default_args())
        .arg("--repo-path")
        .arg(repo_path.to_str().unwrap())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap());
    
    cmd.assert().success();
    
    let output = std::fs::read_to_string(output_file).unwrap();
    assert!(output.contains("README.md"));
    assert!(output.contains("[File Begins] README.md"));
    assert!(output.contains("# Sample Repo"));
    assert!(output.contains("[File Ends] README.md"));
}

#[test]
fn test_ignore_files() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    std::fs::create_dir_all(&repo_path).unwrap();
    
    let file_path = repo_path.join("README.md");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "# Sample Repo").unwrap();
    
    let ignored_file_path = repo_path.join("ignored.txt");
    let mut ignored_file = File::create(&ignored_file_path).unwrap();
    writeln!(ignored_file, "This should be ignored").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");
    
    let mut cmd = Command::cargo_bin("repo2txt").unwrap();
    cmd.args(&default_args())
        .arg("--repo-path")
        .arg(repo_path.to_str().unwrap())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("--ignore-files")
        .arg("ignored.txt");
    
    cmd.assert().success();
    
    let output = std::fs::read_to_string(output_file).unwrap();
    assert!(output.contains("README.md"));
    assert!(!output.contains("ignored.txt"));
}

#[test]
fn test_custom_config() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    std::fs::create_dir_all(&repo_path).unwrap();
    
    let file_path = repo_path.join("README.md");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "# Sample Repo").unwrap();
    
    let config_file_path = temp_dir.path().join("custom_config.json");
    let mut config_file = File::create(&config_file_path).unwrap();
    writeln!(config_file, r#"{{
        "default_ignore_types": [".md"],
        "image_extensions": [],
        "video_extensions": [],
        "audio_extensions": [],
        "document_extensions": [],
        "executable_extensions": [],
        "settings_extensions": [],
        "additional_ignore_types": [],
        "default_output_file": "output.txt"
    }}"#).unwrap();
    
    let output_file = temp_dir.path().join("output.txt");
    
    let mut cmd = Command::cargo_bin("repo2txt").unwrap();
    cmd.args(&default_args())
        .arg("--repo-path")
        .arg(repo_path.to_str().unwrap())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap())
        .arg("--config-path")
        .arg(config_file_path.to_str().unwrap());
    
    cmd.assert().success();
    
    let output = std::fs::read_to_string(output_file).unwrap();
    assert!(!output.contains("README.md"));
}

#[test]
fn test_single_file() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path().join("repo");
    std::fs::create_dir_all(&repo_path).unwrap();
    
    let file_path = repo_path.join("README.md");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "# Sample Repo").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");
    
    let mut cmd = Command::cargo_bin("repo2txt").unwrap();
    cmd.args(&default_args())
        .arg("--repo-path")
        .arg(repo_path.to_str().unwrap())
        .arg("--file-path")
        .arg(file_path.to_str().unwrap())
        .arg("--output-file")
        .arg(output_file.to_str().unwrap());
    
    cmd.assert().success();
    
    let output = std::fs::read_to_string(output_file).unwrap();
    assert!(output.contains("README.md"));
    assert!(output.contains("[File Begins] README.md"));
    assert!(output.contains("# Sample Repo"));
    assert!(output.contains("[File Ends] README.md"));
}
