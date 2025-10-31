# Product Requirements Document: DuTop

**Version:** 1.0  
**Last Updated:** 2024-01-XX  
**Status:** Draft  
**Project Lead:** [Your Name]  
**Document Owner:** AI Development Agent

---

## Executive Summary

**DuTop** is a high-performance disk usage analysis tool designed to replace the legacy `wdu` ZSH script with a production-grade, compiled solution. Built in Rust, DuTop provides fast, accurate workspace storage analysis with an intuitive CLI interface, enabling developers and system administrators to quickly identify and manage disk space consumption.

**Key Value Proposition:**
- **10-100x faster** than shell script implementations through parallel processing
- **Zero dependencies** - single static binary deployment
- **Cross-platform** - Windows, Linux, macOS support
- **Production-ready** - comprehensive error handling, logging, and monitoring

---

## 1. Project Overview

### 1.1 Background

The existing `wdu` ZSH script has served as a functional wrapper around the Unix `du` command but suffers from:
- Performance limitations on large directory trees
- Platform-specific shell dependencies
- Limited error handling and recovery
- Difficulty in maintenance and extension
- No structured output formats for automation

### 1.2 Goals

**Primary Goals:**
1. Deliver a robust, performant disk usage analysis tool
2. Maintain backward compatibility with `wdu` core functionality
3. Provide extensible architecture for future enhancements
4. Achieve production-grade reliability and error handling

**Success Metrics:**
- Performance: Analyze 1M files in < 5 seconds (on SSD)
- Reliability: 99.9% successful execution rate
- Adoption: 80% of `wdu` users migrate within 3 months
- Satisfaction: 4.5+ rating from user feedback

### 1.3 Naming Convention

**Official Name:** `dutop` (lowercase for Unix conventions)  
**Display Name:** DuTop  
**Rationale:** Follows Unix naming conventions (lowercase binary names) while maintaining branding in documentation

---

## 2. User Personas

### 2.1 Primary Personas

#### **Developer Dave**
- **Role:** Software Engineer
- **Context:** Manages multiple project workspaces, needs to clean up node_modules, build artifacts
- **Pain Points:** Slow disk analysis, difficulty identifying space hogs
- **Needs:** Fast results, clear visualization, integration with cleanup scripts

#### **SysAdmin Sarah**
- **Role:** System Administrator
- **Context:** Manages server storage, monitors disk usage across teams
- **Pain Points:** Manual auditing, lack of automation, inconsistent reporting
- **Needs:** JSON output for monitoring, scheduled analysis, threshold alerts

#### **Manager Mike**
- **Role:** Engineering Manager
- **Context:** Reviews team storage usage, enforces policies
- **Pain Points:** No historical tracking, manual report generation
- **Needs:** Summary reports, trend analysis, cost attribution

### 2.2 Secondary Personas

- **DevOps Engineer:** CI/CD integration, automated cleanup
- **Data Scientist:** Large dataset management
- **IT Auditor:** Compliance and storage policy enforcement

---

## 3. Functional Requirements

### 3.1 Core Features (MVP - Version 1.0)

#### FR-1: Directory Size Analysis
**Priority:** P0 (Critical)

```bash
dutop [path]
```

**Requirements:**
- Calculate total size of all files in directory tree
- Support symbolic link handling (follow/ignore options)
- Handle permission errors gracefully (skip with warning)
- Support multiple simultaneous path analysis

**Acceptance Criteria:**
- Accurately reports size matching `du -sh` output (±1%)
- Handles directories with 1M+ files
- Completes analysis within 5 seconds for 100K files (SSD)

---

#### FR-2: Top N Directories Display
**Priority:** P0 (Critical)

```bash
dutop --top 10 [path]
dutop -n 10 [path]
```

**Requirements:**
- Display configurable top N largest subdirectories
- Default to top 10 results
- Sort by size (descending)
- Show both absolute size and percentage of total

**Output Format:**
```
Total: 4.5 GB

Top 10 Largest Directories:
  1.  1.8 GB  (40%)  ./node_modules
  2.  892 MB  (19%)  ./target/debug
  3.  654 MB  (14%)  ./.git
  4.  320 MB  ( 7%)  ./build
  5.  180 MB  ( 4%)  ./dist
  ...
```

---

#### FR-3: Human-Readable Size Formatting
**Priority:** P0 (Critical)

**Requirements:**
- Display sizes in human-readable format (B, KB, MB, GB, TB)
- Support SI units (1000) and binary units (1024) - configurable
- Default to binary units (matching `du -h` behavior)
- Precision: 2 decimal places for clarity

**Flags:**
```bash
--si          # Use SI units (1000-based)
--bytes       # Raw byte output
--block-size  # Custom block size
```

---

#### FR-4: Depth Control
**Priority:** P0 (Critical)

```bash
dutop --depth 2 [path]
dutop -d 2 [path]
```

**Requirements:**
- Limit traversal depth from root directory
- Default: unlimited depth
- Respect depth when calculating top N results
- Show aggregated sizes for directories beyond depth limit

---

#### FR-5: Exclusion Patterns
**Priority:** P1 (High)

```bash
dutop --exclude "node_modules" --exclude "*.log"
dutop -x "node_modules" -x "*.log"
```

**Requirements:**
- Support glob patterns for exclusions
- Multiple exclusion patterns allowed
- Case-sensitive and case-insensitive modes
- .gitignore-style pattern file support

**Pattern Examples:**
- `node_modules` - exact name match
- `*.log` - extension match
- `**/target` - any depth match
- `.git` - hidden file/directory

---

#### FR-6: Parallel Processing
**Priority:** P1 (High)

```bash
dutop --threads 8 [path]
dutop -j 8 [path]
```

**Requirements:**
- Automatic CPU core detection
- Configurable thread count
- Default: CPU count - 1 (leave one core free)
- Thread-safe aggregation of results

**Performance Target:**
- 5-10x speedup on multi-core systems vs. single-threaded

---

### 3.2 Enhanced Features (Version 1.1+)

#### FR-7: Structured Output Formats
**Priority:** P1 (High)

```bash
dutop --format json [path]
dutop --format csv [path]
dutop --format yaml [path]
```

**JSON Output Example:**
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "path": "/home/user/workspace",
  "total_size": 4831838208,
  "total_size_human": "4.5 GB",
  "file_count": 45230,
  "directory_count": 1820,
  "top_directories": [
    {
      "path": "./node_modules",
      "size": 1932735283,
      "size_human": "1.8 GB",
      "percentage": 40.0
    }
  ]
}
```

---

#### FR-8: Progress Indication
**Priority:** P2 (Medium)

```bash
dutop --progress [path]
```

**Requirements:**
- Show real-time progress for large directories
- Display: files processed, current directory, estimated time
- Optional: disable for CI/CD environments (auto-detect TTY)

**Display:**
```
Analyzing: /home/user/workspace
Progress: 45,230 files | 1,820 dirs | 4.5 GB | 32% [=====>    ] ETA: 8s
```

---

#### FR-9: Filter by Size Threshold
**Priority:** P2 (Medium)

```bash
dutop --min-size 100M [path]
dutop --max-size 5G [path]
```

**Requirements:**
- Filter results by minimum size threshold
- Filter results by maximum size threshold
- Combine with other filters (depth, exclusions)

---

#### FR-10: Sorting Options
**Priority:** P2 (Medium)

```bash
dutop --sort size [path]      # Default
dutop --sort name [path]
dutop --sort count [path]     # File count
```

**Requirements:**
- Sort by size (default)
- Sort by name (alphabetical)
- Sort by file count
- Ascending/descending modes

---

#### FR-11: Summary Statistics
**Priority:** P2 (Medium)

```bash
dutop --summary [path]
```

**Output:**
```
Summary Statistics:
  Total Size:       4.5 GB
  File Count:       45,230
  Directory Count:  1,820
  Average File:     104 KB
  Largest File:     32 MB (./video.mp4)
  Oldest File:      ./legacy/data.csv (2019-03-12)
  Newest File:      ./build/output.js (2024-01-15)
```

---

#### FR-12: Interactive Mode (Future)
**Priority:** P3 (Low)

```bash
dutop --interactive [path]
dutop -i [path]
```

**Requirements:**
- TUI (Text User Interface) for navigation
- Keyboard shortcuts for drill-down
- Real-time sorting and filtering
- Integration with file managers (open in Finder/Explorer)

---

### 3.3 Configuration & Persistence

#### FR-13: Configuration File
**Priority:** P1 (High)

**Config Location:**
- `~/.config/dutop/config.toml` (Linux/macOS)
- `%APPDATA%\dutop\config.toml` (Windows)

**Example Configuration:**
```toml
[defaults]
top = 10
depth = 3
threads = "auto"
format = "human"
exclude = ["node_modules", "target", ".git"]

[display]
show_percentage = true
color_enabled = true
progress = true

[units]
binary = true  # Use 1024-based units
precision = 2
```

---

#### FR-14: Environment Variables
**Priority:** P2 (Medium)

```bash
DUTOP_THREADS=8
DUTOP_EXCLUDE="node_modules:target"
DUTOP_CONFIG=/custom/path/config.toml
```

**Requirements:**
- Override config file settings
- Support common CI/CD environment patterns
- Backward compatibility with `DU_` prefixed vars

---

### 3.4 Integration & Automation

#### FR-15: Exit Codes
**Priority:** P0 (Critical)

```
0  - Success
1  - General error
2  - Invalid arguments
3  - Permission denied
4  - Path not found
5  - Disk I/O error
```

---

#### FR-16: Logging
**Priority:** P1 (High)

```bash
dutop --log-level debug [path]
dutop --log-file /var/log/dutop.log [path]
```

**Log Levels:**
- `error` - Critical errors only
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging
- `trace` - Maximum verbosity

---

#### FR-17: Watch Mode (Future)
**Priority:** P3 (Low)

```bash
dutop --watch --interval 60 [path]
```

**Requirements:**
- Monitor directory for changes
- Re-analyze at specified intervals
- Alert on threshold breaches
- Daemon mode for continuous monitoring

---

## 4. Non-Functional Requirements

### 4.1 Performance

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Cold start time | < 100ms | `time dutop --version` |
| Analysis speed (100K files) | < 5s | Benchmark on SSD |
| Analysis speed (1M files) | < 30s | Benchmark on SSD |
| Memory usage | < 50MB | Peak RSS for 1M files |
| Binary size | < 5MB | Stripped release build |
| CPU usage | < 80% per core | Average during analysis |

**Performance Testing:**
- Automated benchmarks in CI/CD
- Comparison with `du`, `ncdu`, `dust`
- Regression testing for each release

---

### 4.2 Reliability

**Error Handling:**
- Graceful degradation on permission errors
- Continue analysis on corrupted filesystem entries
- Atomic operations for all file I/O
- Proper resource cleanup (file handles, memory)

**Data Integrity:**
- Accurate size calculations (match `du` within 1%)
- Proper handling of hard links (count once)
- Correct symlink traversal (configurable)
- Handle sparse files correctly

**Stability:**
- No panics/crashes on invalid input
- Memory-safe operations (Rust guarantees)
- Thread-safe concurrent operations
- Proper signal handling (SIGINT, SIGTERM)

---

### 4.3 Compatibility

**Operating Systems:**
- ✅ Linux (Ubuntu 20.04+, RHEL 8+, Arch)
- ✅ macOS (11.0+)
- ✅ Windows (10+, Server 2019+)
- ✅ FreeBSD (13.0+)

**Filesystems:**
- ext4, XFS, Btrfs (Linux)
- APFS, HFS+ (macOS)
- NTFS, ReFS (Windows)
- ZFS (FreeBSD)

**Architectures:**
- x86_64 (primary)
- ARM64 (Apple Silicon, Raspberry Pi)
- ARM (legacy support)

---

### 4.4 Usability

**Command-Line Interface:**
- POSIX-compliant argument parsing
- GNU-style long options (`--help`)
- Short option aliases (`-h`)
- Consistent with Unix tool conventions

**Documentation:**
- Comprehensive man page
- Built-in help (`--help`)
- Examples in documentation
- Error messages with suggestions

**Accessibility:**
- Color-blind friendly color schemes
- Screen reader compatible output
- Respect `NO_COLOR` environment variable

---

### 4.5 Security

**File System Access:**
- Read-only operations (no modifications)
- Proper permission handling
- No privilege escalation
- Sandbox-friendly (no system calls beyond file I/O)

**Data Privacy:**
- No telemetry or data collection
- No network access
- No cloud dependencies
- Local-only processing

**Vulnerability Management:**
- Regular dependency audits (`cargo audit`)
- Automated security scanning in CI/CD
- CVE monitoring and patching
- Secure coding practices (no `unsafe` without justification)

---

### 4.6 Maintainability

**Code Quality:**
- Rust idioms and best practices
- Comprehensive documentation (rustdoc)
- Unit test coverage > 80%
- Integration test coverage > 60%
- Clippy lints enforced (pedantic)

**Architecture:**
- Modular design (library + CLI)
- Clear separation of concerns
- Minimal external dependencies
- Extensible plugin system (future)

**Development Workflow:**
- CI/CD pipeline (GitHub Actions)
- Automated testing on all platforms
- Semantic versioning (SemVer)
- Changelog generation (keep-a-changelog)

---

## 5. Technical Architecture

### 5.1 Technology Stack

**Primary Language:** Rust 1.75+

**Core Dependencies:**
```toml
[dependencies]
clap = "4.4"              # CLI argument parsing
walkdir = "2.4"           # Directory traversal
rayon = "1.8"             # Parallel processing
serde = "1.0"             # Serialization
serde_json = "1.0"        # JSON output
humansize = "2.1"         # Human-readable sizes
indicatif = "0.17"        # Progress bars
anyhow = "1.0"            # Error handling
log = "0.4"               # Logging interface
env_logger = "0.11"       # Logging implementation
toml = "0.8"              # Config parsing
```

**Development Dependencies:**
```toml
[dev-dependencies]
criterion = "0.5"         # Benchmarking
tempfile = "3.8"          # Test fixtures
assert_cmd = "2.0"        # CLI testing
predicates = "3.0"        # Test assertions
```

---

###