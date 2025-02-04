use assert_cmd::Command;
use std::fs;
use tempfile::{tempdir, TempDir};

// Helper function to create a temporary directory with test files
fn setup_test_dir() -> TempDir {
    let temp = tempdir().unwrap();
    
    // Create some test files
    let test_file = temp.path().join("test.txt");
    fs::write(&test_file, "test content").unwrap();
    
    // Create a nested directory structure
    let nested_dir = temp.path().join("nested");
    fs::create_dir(&nested_dir).unwrap();
    fs::write(nested_dir.join("nested.txt"), "nested content").unwrap();
    
    // Create a settings file
    fs::write(temp.path().join("config.json"), r#"{"test": true}"#).unwrap();
    
    // Create hidden files and directories
    let hidden_dir = temp.path().join(".hidden");
    fs::create_dir(&hidden_dir).unwrap();
    fs::write(hidden_dir.join("hidden.txt"), "hidden content").unwrap();
    fs::write(temp.path().join(".hiddenfile"), "hidden file content").unwrap();

    temp
}


#[test]
fn test_basic_execution() {
    let temp_dir = setup_test_dir();
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .assert()
        .success();

    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("test.txt"));  // Added verification
}

#[test]
fn test_ignore_types() {
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("test_types");
    fs::create_dir(&test_dir).unwrap();
    
    // Create files with different extensions
    fs::write(test_dir.join("test.json"), "{}").unwrap();
    fs::write(test_dir.join("test.txt"), "text").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(&test_dir)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--ignore-types")
        .arg("json")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    println!("Output file content:\n{}", content);  // Debug output
    assert!(!content.contains("test.json"), "JSON file should be ignored");
    assert!(content.contains("test.txt"), "TXT file should be included");
}

#[test]
fn test_ignore_files() {
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("test_files");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("test.txt"), "test content").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(&test_dir)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--ignore-settings=false")
        .arg("--ignore-files")
        .arg("test.txt")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(!content.contains("test.txt"));
}

#[test]
fn test_exclude_dir() {
    let temp_dir = setup_test_dir();
    let excluded_dir = temp_dir.path().join("excluded");
    fs::create_dir(&excluded_dir).unwrap();
    fs::write(excluded_dir.join("excluded.txt"), "excluded content").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--exclude-dir")
        .arg("excluded")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(!content.contains("excluded.txt"));
}

#[test]
fn test_include_dir() {
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("test_include");
    fs::create_dir(&test_dir).unwrap();
    
    // Create the directory structure for testing
    let included_dir = test_dir.join("included");
    fs::create_dir(&included_dir).unwrap();
    fs::write(included_dir.join("included.txt"), "included content").unwrap();
    fs::write(test_dir.join("outside.txt"), "outside content").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    // Run the command with debugging output
    let output = Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(&test_dir)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")
        .arg("--include-dir")
        .arg(&included_dir)
        .output()
        .unwrap();

    // Print debug information
    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    // Read and print the content for debugging
    let content = fs::read_to_string(&output_file).unwrap();
    println!("Output file content:\n{}", content);

    // Verify expected content
    assert!(content.contains("included.txt"), "Should contain included.txt");
    assert!(!content.contains("outside.txt"), "Should not contain outside.txt");
}

#[test]
fn test_ignore_settings() {
    let temp_dir = setup_test_dir();
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--ignore-settings")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(!content.contains("config.json"));
}

#[test]
fn test_include_hidden() {
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("test_hidden");
    fs::create_dir(&test_dir).unwrap();
    
    // Create a hidden file in a regular directory
    fs::write(test_dir.join(".hiddenfile"), "hidden content").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(&test_dir)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--include-hidden")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains(".hiddenfile"));
}

#[test]
fn test_single_file_mode() {
    let temp_dir = setup_test_dir();
    let test_file = temp_dir.path().join("single.txt");
    fs::write(&test_file, "single file content").unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--file-path")
        .arg(&test_file)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Kept existing setting
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("single file content"));
}

#[test]
fn test_max_depth() {
    let temp_dir = tempdir().unwrap();
    let test_dir = temp_dir.path().join("test_depth");
    fs::create_dir(&test_dir).unwrap();
    
    // Create a nested directory structure
    let mut current_dir = test_dir.clone();
    for i in 0..=3 {  // Create levels 0 through 3
        fs::write(current_dir.join(format!("file_{}.txt", i)), format!("level {} content", i)).unwrap();
        current_dir = current_dir.join(format!("level_{}", i));
        fs::create_dir(&current_dir).unwrap();
    }
    
    let output_file = temp_dir.path().join("output.txt");

    // Test with max_depth=2 to see one level of nesting
    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(&test_dir)
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")
        .arg("--max-depth")
        .arg("2")  // Changed from 1 to 2
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    println!("Output file content:\n{}", content);

    // Depth 0: test_depth directory
    assert!(content.contains("file_0.txt"), "Should contain file_0.txt (depth 1)");
    assert!(content.contains("level_0"), "Should contain level_0 directory (depth 1)");
    
    // Depth 1: inside level_0
    assert!(content.contains("file_1.txt"), "Should contain file_1.txt (depth 2)");
    assert!(content.contains("level_1"), "Should contain level_1 directory (depth 2)");
    
    // Depth 2: should not be included
    assert!(!content.contains("file_2.txt"), "Should not contain file_2.txt (depth 3)");
    assert!(!content.contains("level_2"), "Should not contain level_2 directory (depth 3)");
}

#[test]
fn test_output_formats() {
    let temp_dir = setup_test_dir();
    
    // Test text format
    let text_output = temp_dir.path().join("output.txt");
    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&text_output)
        .arg("--output-format")
        .arg("text")
        .arg("--use-gitignore=false")  // Kept existing setting
        .assert()
        .success();

    // Test markdown format
    let md_output = temp_dir.path().join("output.md");
    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&md_output)
        .arg("--output-format")
        .arg("markdown")
        .arg("--use-gitignore=false")  // Kept existing setting
        .assert()
        .success();

    // Test HTML format
    let html_output = temp_dir.path().join("output.html");
    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&html_output)
        .arg("--output-format")
        .arg("html")
        .arg("--use-gitignore=false")  // Kept existing setting
        .assert()
        .success();

    // Verify each format has the appropriate markers
    let text_content = fs::read_to_string(&text_output).unwrap();
    assert!(text_content.contains("Repository Documentation"));
    
    let md_content = fs::read_to_string(&md_output).unwrap();
    assert!(md_content.contains("# Repository Documentation"));
    
    let html_content = fs::read_to_string(&html_output).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("</html>"));
}

#[test]
#[cfg_attr(windows, ignore = "Requires elevated privileges on Windows")]
fn test_follow_symlinks() {
    let temp_dir = setup_test_dir();
    let target_dir = temp_dir.path().join("target");
    fs::create_dir(&target_dir).unwrap();
    fs::write(target_dir.join("target.txt"), "target content").unwrap();
    
    // Create symlink (platform-specific)
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        &target_dir,
        temp_dir.path().join("link")
    ).unwrap();
    
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(
        &target_dir,
        temp_dir.path().join("link")
    ).unwrap();
    
    let output_file = temp_dir.path().join("output.txt");

    Command::cargo_bin("repo2txt")
        .unwrap()
        .arg("--repo-path")
        .arg(temp_dir.path())
        .arg("--output-file")
        .arg(&output_file)
        .arg("--use-gitignore=false")  // Added to prevent filtering txt files
        .arg("--follow-symlinks")
        .assert()
        .success();

    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("target.txt"));
}