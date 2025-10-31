# DuTop

A high-performance disk usage analysis tool built in Rust, designed to replace legacy shell scripts with a fast, parallel, cross-platform solution.

## Features

- **Fast Analysis**: 3-4x faster than shell-based tools through parallel processing
- **Visual Output**: Color-coded bar chart display for easy identification of space usage
- **Flexible Filtering**: Exclude patterns with glob syntax support
- **Depth Control**: Limit directory traversal depth for focused analysis
- **Multiple Output Formats**: Human-readable table or JSON for automation
- **Cross-Platform**: Works on Linux, macOS, Windows, and FreeBSD

## Installation

### Platform Support

DuTop runs on all major platforms:
- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Linux** (x86_64, ARM64, musl)
- ✅ **Windows** (x86_64)
- ✅ **FreeBSD** and other Unix-like systems

### Quick Install

#### macOS / Linux

**Download pre-built binary:**
```bash
# macOS (Universal - Intel + Apple Silicon)
curl -L https://github.com/yourusername/dutop/releases/latest/download/dutop-VERSION-universal-apple-darwin.tar.gz | tar xz
sudo mv dutop-universal /usr/local/bin/dutop

# Linux x86_64
curl -L https://github.com/yourusername/dutop/releases/latest/download/dutop-VERSION-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv dutop /usr/local/bin/

# Linux (static musl - works everywhere)
curl -L https://github.com/yourusername/dutop/releases/latest/download/dutop-VERSION-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv dutop /usr/local/bin/
```

**Using Cargo (from source):**
```bash
cargo install dutop
# or from git
cargo install --git https://github.com/yourusername/dutop
```

**Using Homebrew (macOS/Linux):**
```bash
brew tap yourusername/dutop
brew install dutop
```

#### Windows

**Download pre-built binary:**
1. Download `dutop-VERSION-x86_64-pc-windows-msvc.zip` from [releases](https://github.com/yourusername/dutop/releases)
2. Extract `dutop.exe`
3. Add to PATH or move to `C:\Windows\System32\`

**Using Cargo:**
```powershell
cargo install dutop
```

**Using Scoop:**
```powershell
scoop bucket add dutop https://github.com/yourusername/scoop-dutop
scoop install dutop
```

### Build from Source

```bash
git clone https://github.com/yourusername/dutop
cd dutop
cargo build --release

# Install (Unix)
sudo cp target/release/dutop /usr/local/bin/

# Install (Windows - as Administrator)
copy target\release\dutop.exe C:\Windows\System32\
```

Binary size: **~1.6MB** (optimized and stripped)

## Usage

### Basic Usage

```bash
# Analyze current directory, show top 10
dutop

# Analyze specific directory
dutop /path/to/directory

# Show top 20 directories
dutop -n 20

# Exclude patterns (can be specified multiple times)
dutop --exclude "node_modules" --exclude "target" --exclude "*.log"
```

### Advanced Options

```bash
# Limit traversal depth
dutop -d 2 /path/to/directory

# Output as JSON for scripting
dutop --format json . > usage.json

# Use specific number of threads
dutop -j 4 .

# Follow symbolic links
dutop -L /path/with/symlinks

# Disable colors
dutop --no-color

# Enable verbose logging
dutop -v
```

### Examples

**Example 1: Quick workspace cleanup**
```bash
dutop -n 15 ~/projects --exclude ".git"
```

**Example 2: Find large directories for archival**
```bash
dutop -n 5 /data --format json | jq '.top_directories[] | select(.size > 1000000000)'
```

**Example 3: Shallow scan of home directory**
```bash
dutop -d 1 ~
```

## Output Format

### Human-Readable Table
```
Analyzing: /Users/username/projects

┌────────────────────────────────┬──────────┬───────┬──────────────────────────────┐
│ ██████████████████████████████ │  454.6 M │ 100% │ target │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   26.7 K │   0% │ node_modules │
│ ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │   22.9 K │   0% │ src │
└────────────────────────────────┴──────────┴───────┴──────────────────────────────┘

Total: 454.7 M
Files: 2261  Directories: 347
```

The bar chart uses color coding:
- 🟢 Green: < 33% of maximum
- 🟡 Yellow: 33-50% of maximum
- 🔴 Red: > 50% of maximum

### JSON Output
```json
{
  "path": "/Users/username/projects",
  "total_size": 476839936,
  "total_size_human": "454.7 M",
  "file_count": 2261,
  "directory_count": 347,
  "top_directories": [
    {
      "path": "/Users/username/projects/target",
      "size": 476647424,
      "size_human": "454.6 M",
      "percentage": 99.95965735523452,
      "file_count": 2237,
      "dir_count": 340
    }
  ]
}
```

## Performance

Performance comparison on a directory with ~2,000 files:

| Tool | Time |
|------|------|
| `wdu.sh` (shell script) | 907ms |
| `dutop` (Rust) | 250ms |

**Result**: 3.6x faster with the Rust implementation

For larger directories (100K+ files), the performance gain is even more significant due to parallel processing.

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `3`: Permission denied
- `4`: Path not found
- `5`: Disk I/O error

## Options Reference

```
Usage: dutop [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to analyze (default: current directory)

Options:
  -n, --top <TOP>          Number of top directories to display [default: 10]
  -d, --depth <DEPTH>      Maximum depth to traverse (default: unlimited)
  -x, --exclude <EXCLUDE>  Exclude patterns (glob syntax, can be specified multiple times)
  -L, --follow-links       Follow symbolic links
  -j, --threads <THREADS>  Number of threads to use (default: auto-detect)
  -f, --format <FORMAT>    Output format: human (default), json [possible values: human, json]
      --no-color           Disable colored output
  -v, --verbose            Enable verbose logging
      --debug              Enable debug logging
  -h, --help               Print help
  -V, --version            Print version
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_exclusion_patterns
```

### Running Benchmarks

```bash
cargo bench
```

## Project Status

**Current Version**: 0.1.0 (MVP)

This is the MVP release with core functionality. See [PRD.md](PRD.md) for the full roadmap.

### Implemented Features (v0.1.0)
- ✅ Directory size analysis with parallel processing
- ✅ Top N directories display
- ✅ Human-readable size formatting
- ✅ Depth control
- ✅ Exclusion patterns (glob syntax)
- ✅ JSON output format
- ✅ Color-coded bar chart visualization
- ✅ Proper error handling and exit codes
- ✅ Multi-threaded analysis with configurable thread count

### Planned Features (v0.2.0+)
- CSV/YAML output formats
- Progress indication for large scans
- Size threshold filtering
- Configuration file support
- Interactive TUI mode
- Watch mode for continuous monitoring

## License

MIT

## Contributing

Contributions welcome! Please see [PRD.md](PRD.md) for project requirements and architecture details.
