# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DuTop** is a high-performance disk usage analysis tool being built to replace the legacy `wdu` ZSH script. The goal is to create a production-grade, compiled Rust application that provides fast, accurate workspace storage analysis with 10-100x performance improvement through parallel processing.

**Current Status**: Early development stage - PRD complete, no Rust implementation yet. This is a greenfield project.

## Key Architecture Decisions (from PRD)

### Technology Stack
- **Language**: Rust 1.75+
- **CLI Parsing**: clap 4.4
- **Directory Traversal**: walkdir 2.4
- **Parallel Processing**: rayon 1.8
- **Output Formats**: serde + serde_json for JSON/structured output
- **Human-Readable Sizes**: humansize 2.1
- **Progress Indication**: indicatif 0.17
- **Error Handling**: anyhow 1.0
- **Logging**: log + env_logger
- **Config**: TOML-based configuration files

### Core Architecture Principles
1. **Modular Design**: Separate library crate (`dutop-lib`) from CLI binary
2. **Zero Dependencies**: Single static binary deployment
3. **Cross-Platform**: Windows, Linux, macOS, FreeBSD support
4. **Performance First**: Target < 5s for 100K files on SSD
5. **Production-Grade**: Comprehensive error handling, proper exit codes, structured logging

### Binary Name Convention
- **Binary name**: `dutop` (lowercase, following Unix conventions)
- **Display name**: DuTop (in documentation)

## Development Commands

Since this is a greenfield project, these commands will be used once implementation begins:

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- [path]

# Benchmarks (when implemented)
cargo bench

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt
```

## Critical Project Rules

**These rules from `.claude/prompts/project-rules.md` MUST be followed strictly:**

### Rule 1: Context Preservation
- MUST maintain context across ALL interactions
- Before starting ANY task: review previous decisions and implementations
- If context is lost: acknowledge immediately and reconnect to existing patterns

### Rule 2: Documentation Management
- ALL documentation files (*.md) MUST be in `docs/` folder
- ONLY exceptions: `README.md` and `CLAUDE.md` in root
- Any violation MUST be fixed immediately

### Rule 7: Bug Fix Verification
- NEVER claim a fix works without running tests
- NEVER tell user to "try it" - test it first
- MANDATORY testing protocol: Understand → Reproduce → Diagnose → Fix → TEST → Verify → Report
- Show actual test output proving it works

## Key Features to Implement (from PRD)

### MVP (Version 1.0) - P0 Priority
1. **Directory Size Analysis**: Calculate total size with symbolic link handling
2. **Top N Display**: Show configurable top N largest subdirectories (default 10)
3. **Human-Readable Formatting**: Display in B/KB/MB/GB/TB with SI and binary units
4. **Depth Control**: Limit traversal depth from root directory
5. **Exclusion Patterns**: Support glob patterns for exclusions
6. **Parallel Processing**: Multi-threaded analysis with configurable thread count

### Enhanced Features (Version 1.1+)
- JSON/CSV/YAML structured output
- Progress indication for large directories
- Size threshold filtering
- Summary statistics
- Configuration file support (~/.config/dutop/config.toml)

## Legacy Reference

The original `wdu.sh` script in `archive/` provides the baseline functionality:
- Default shows top 10 largest directories
- Color-coded bar chart visualization (green → yellow → red)
- Human-readable size formatting
- Special handling for home directory (too slow to scan)
- Terminal width detection for responsive layout

**Key behaviors to preserve:**
- Same basic output format and user experience
- `-n NUMBER` flag for custom list length
- `-d DEPTH` flag for depth control
- Graceful error handling for permission issues
- Visual bar chart representation

## Performance Targets

From the PRD, these are the success metrics:

| Metric | Target |
|--------|--------|
| Cold start time | < 100ms |
| Analysis (100K files) | < 5s |
| Analysis (1M files) | < 30s |
| Memory usage | < 50MB (peak RSS for 1M files) |
| Binary size | < 5MB (stripped release) |

## Exit Codes

Must implement these specific exit codes:
- 0: Success
- 1: General error
- 2: Invalid arguments
- 3: Permission denied
- 4: Path not found
- 5: Disk I/O error

## Configuration Strategy

When implementing config support:
- Location: `~/.config/dutop/config.toml` (Linux/macOS), `%APPDATA%\dutop\config.toml` (Windows)
- Environment variables: `DUTOP_THREADS`, `DUTOP_EXCLUDE`, `DUTOP_CONFIG`
- Priority: CLI args > Environment vars > Config file > Defaults

## Testing Requirements

From project rules:
- Unit test coverage > 80%
- Integration test coverage > 60%
- Clippy lints enforced (pedantic level)
- Test fixtures using tempfile crate
- CLI testing with assert_cmd + predicates
- Benchmark suite with criterion

## Important Implementation Notes

1. **Hard Link Handling**: Count hard-linked files only once to match `du` behavior
2. **Symlink Traversal**: Make configurable (follow/ignore)
3. **Sparse Files**: Handle correctly to match actual disk usage
4. **Permission Errors**: Skip with warning, continue analysis (graceful degradation)
5. **Signal Handling**: Proper cleanup on SIGINT/SIGTERM
6. **Thread Safety**: All concurrent operations must be thread-safe
7. **Security**: Read-only operations, no privilege escalation, no network access

## Future Extension Points

The PRD identifies these as potential future features:
- Interactive TUI mode for directory navigation
- Watch mode for continuous monitoring
- Alert system for threshold breaches
- Plugin system for extensibility
- Integration with file managers

## Working with This Codebase

1. **Before implementing features**: Review the PRD for detailed requirements and acceptance criteria
2. **Follow existing patterns**: Once code exists, maintain consistency with established conventions
3. **Performance matters**: Profile and benchmark any changes that could impact performance
4. **Cross-platform testing**: Test on Linux, macOS, and Windows when possible
5. **Document architectural decisions**: Use comments and rustdoc for non-obvious design choices
