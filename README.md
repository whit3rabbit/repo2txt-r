# repo2txt-r

`repo2txt-r` is a Rust-based command-line tool that generates a comprehensive documentation file for a given repository. This documentation includes a hierarchical tree view of directories and files, along with the content of each file.

## Download and run

Check the release tab to download the binary and run.

## Features

- Generates a directory and file tree in a readable format.
- Includes the content of each file in the documentation.
- Allows customization to ignore specific file types or directories.
- Honors `.gitignore` files for excluding paths.

## Configuration

The tool uses a `config.json` file for configuration, which includes the following settings:

```json
{
  "image_extensions": [".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".tiff", ".webp"],
  "video_extensions": [".mp4", ".avi", ".mov", ".wmv", ".flv", ".mkv", ".webm"],
  "audio_extensions": [".mp3", ".wav", ".aac", ".flac", ".ogg", ".m4a"],
  "document_extensions": [".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx"],
  "executable_extensions": [".exe", ".dll", ".so", ".class", ".jar", ".pyc"],
  "settings_extensions": [".json", ".yaml", ".yml", ".xml"],
  "additional_ignore_types": [".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".bin", ".dat", ".db", ".log"]
}
```

## Usage

Run the tool with the following command:

```sh
cargo run --release -- <repo-path> <output-file> [options]
Options:

--exclude-dir <dir>: Exclude a specific directory.
--ignore-files <file>: Ignore specific files.
--ignore-types <ext>: Ignore specific file types.
--use-gitignore: Honor .gitignore files.
```

Example
To generate documentation for a repository located at ./my-repo and save the output to output.txt, run:

```sh
cargo run --release -- ./my-repo output.txt --use-gitignore
```

## License

This project is licensed under the MIT License. See the LICENSE file for details.
