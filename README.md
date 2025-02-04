# repo2txt

`repo2txt` is a command-line tool that documents the structure of a GitHub repository. It generates a text file containing a directory/file tree and the contents of each file in the repository.

## Features

- Generates a text file documenting the repository structure and file contents
- Supports customizable ignore lists for file names, extensions, and directories
- Allows specifying a specific directory to include
- Supports ignoring common settings files
- Can use the repository's `.gitignore` file to exclude files and directories
- Provides an option to document a single file instead of the entire repository

## Installation

### Using Pre-built Binaries

1. Visit the [releases page](https://github.com/whit3rabbit/repo2txt-r/releases) of this repository.
2. Download the appropriate binary for your operating system.
3. Extract the downloaded archive.
4. Move the `repo2txt` binary to a directory in your system's PATH.

### Building from Source

1. Ensure you have Rust installed on your system. If not, follow the [official installation guide](https://www.rust-lang.org/tools/install).
2. Clone this repository:
   ```
   git clone https://github.com/whit3rabbit/repo2txt-r.git
   ```
3. Navigate to the repository directory:
   ```
   cd repo2txt
   ```
4. Build the project:
   ```
   cargo build --release
   ```
5. The compiled binary will be located at `target/release/repo2txt-r`. You can move it to a directory in your system's PATH for easier access.

## Usage

```
USAGE:
    repo2txt-r [OPTIONS]

OPTIONS:
    -r, --repo_path <REPO_PATH>          Path to the directory to process (i.e., cloned repo). If no path is specified, defaults to the current directory.
    -o, --output_file <OUTPUT_FILE>      Name for the output text file. Defaults to "output.txt".
        --ignore-types <IGNORE_TYPES>    List of file extensions to ignore (without leading dots, e.g., "txt").
        --ignore-files <IGNORE_FILES>    List of glob patterns for file names/paths to ignore (e.g., "**/temp.txt").
        --exclude-dir <EXCLUDE_DIR>      List of directory names to exclude or "none" for no directories.
        --ignore-settings                Flag to ignore common settings files.
        --include-dir <INCLUDE_DIR>      Specific directory to include. Only contents of this directory will be documented.
        --no-gitignore                   Flag to ignore .gitignore file.
    -f, --file <FILE_PATH>               Path to the file to process.
    -h, --help                           Print help information
    -V, --version                        Print version information
```

## Examples

- Document the structure of a repository:
  ```
  repo2txt-r -r /path/to/repo
  ```

- Document a specific directory within a repository:
  ```
  repo2txt-r -r /path/to/repo --include-dir /path/to/specific/directory
  ```

- Document a single file:
  ```
  repo2txt-r -f /path/to/file
  ```

## Configuration

The `config.json` file allows you to customize the behavior of `repo2txt`. You can modify the following settings:

- `image_extensions`: List of image file extensions to ignore.
- `video_extensions`: List of video file extensions to ignore.
- `audio_extensions`: List of audio file extensions to ignore.
- `document_extensions`: List of document file extensions to ignore.
- `executable_extensions`: List of executable file extensions to ignore.
- `settings_extensions`: List of settings file extensions to ignore when using the `--ignore-settings` flag.
- `additional_ignore_types`: Additional file extensions to ignore.
- `default_output_file`: Default name for the output text file.

This config is built into the binary at build time so it won't do any good to alter it afterwards.

## License

This project is licensed under the [MIT License](LICENSE).