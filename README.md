# repo2txt

`repo2txt` is a command-line tool that documents the structure of a GitHub repository, which is useful to pass to LLM's. It generates documentation containing a directory/file tree and the contents of each file in the repository, with support for multiple output formats.

## Features

- Documents repository structure and file contents
- Multiple output formats (text, markdown, HTML)
- Customizable file and directory filtering:
  - Ignore specific file types and names using glob patterns
  - Exclude directories
  - Include/exclude hidden files
  - Respect .gitignore rules
- Advanced traversal options:
  - Control directory traversal depth
  - Follow symbolic links
  - Focus on specific directories
- Configurable settings via JSON configuration files
- Single file documentation mode

## Installation

### Using Pre-built Binaries

1. Visit the [releases page](https://github.com/whit3rabbit/repo2txt-r/releases)
2. Download the binary for your operating system
3. Move the `repo2txt` binary to a directory in your system's PATH

### Building from Source

1. Install Rust using the [official installation guide](https://www.rust-lang.org/tools/install)
2. Clone the repository:
   ```bash
   git clone https://github.com/whit3rabbit/repo2txt-r.git
   ```
3. Navigate to the repository:
   ```bash
   cd repo2txt
   ```
4. Build the project:
   ```bash
   cargo build --release
   ```
5. The binary will be at `target/release/repo2txt`. Move it to your PATH for easier access.

## Usage

```
repo2txt [OPTIONS]

Options:
  -r, --repo-path <REPO_PATH>        Repository path [default: current directory]
  -o, --output-file <OUTPUT_FILE>    Output filename [default: output.txt]
  -f, --file-path <FILE_PATH>        Document a single file
      --output-format <FORMAT>       Output format: text|markdown|html [default: text]
      --max-depth <MAX_DEPTH>        Maximum directory traversal depth [default: 100]

Filtering Options:
      --ignore-files <PATTERNS>      Glob patterns for files to ignore
      --ignore-types <EXTENSIONS>    File extensions to ignore
      --exclude-dir <DIRECTORIES>    Directories to exclude [default: node_modules,vendor,dist,build,target]
      --include-dir <DIRECTORY>      Only document this directory and its contents
      --include-hidden               Include hidden files/directories [default: false]

Behavior Flags:
      --ignore-settings             Ignore common settings files [default: true]
      --use-gitignore              Use .gitignore rules [default: true]
      --follow-symlinks            Follow symbolic links [default: false]
      --config-path <PATH>         Custom configuration file path

General:
  -h, --help                       Print help
  -V, --version                    Print version
```

## Examples

Document a repository:

```bash
repo2txt -r /path/to/repo
```

Generate HTML documentation:

```bash
repo2txt -r /path/to/repo --output-format html -o documentation.html
```

Document specific directory with depth limit:

```bash
repo2txt -r /path/to/repo --include-dir src --max-depth 2
```

Document single file:

```bash
repo2txt -f /path/to/file.txt
```

Ignore specific file types and patterns:

```bash
repo2txt -r /path/to/repo --ignore-types "txt,log" --ignore-files "temp_*,*.bak"
```

Include hidden files and follow symlinks:

```bash
repo2txt -r /path/to/repo --include-hidden --follow-symlinks
```

## Configuration

The default configuration is built into the binary, but you can provide a custom `config.json`:

```json
{
  "settings_extensions": [
    ".json", ".yaml", ".yml", ".xml"
  ],
  "default_ignore_types": [
    "jpg", "jpeg", "png", "gif", "svg",
    "mp4", "avi", "mov",
    "mp3", "wav", "flac",
    "pdf", "doc", "docx",
    "exe", "dll", "so",
    "zip", "tar", "gz"
  ]
}
```

Use a custom config:

```bash
repo2txt -r /path/to/repo --config-path my-config.json
```

## License

This project is licensed under the [MIT License](LICENSE).
