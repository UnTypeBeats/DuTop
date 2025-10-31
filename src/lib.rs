//! DuTop - High-performance disk usage analysis library
//!
//! This library provides fast, parallel disk usage analysis with configurable
//! traversal options, exclusion patterns, and output formatting.

use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub mod format;
pub mod output;

/// Configuration options for disk usage analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,
    /// Glob patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Whether to follow symbolic links
    pub follow_links: bool,
    /// Number of threads for parallel processing (None = auto-detect)
    pub num_threads: Option<usize>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            exclude_patterns: Vec::new(),
            follow_links: false,
            num_threads: None,
        }
    }
}

/// Represents a directory entry with its size information
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// Path to the directory
    pub path: PathBuf,
    /// Total size in bytes
    pub size: u64,
    /// Number of files in directory
    pub file_count: usize,
    /// Number of subdirectories
    pub dir_count: usize,
}

/// Results of disk usage analysis
#[derive(Debug)]
pub struct AnalysisResult {
    /// Path that was analyzed
    pub root_path: PathBuf,
    /// Total size of all files in bytes
    pub total_size: u64,
    /// Total number of files
    pub total_files: usize,
    /// Total number of directories
    pub total_dirs: usize,
    /// Top directories sorted by size
    pub top_directories: Vec<DirectoryEntry>,
}

/// Analyzes disk usage for the given path with specified configuration
///
/// # Arguments
/// * `path` - Root path to analyze
/// * `config` - Configuration options for the analysis
/// * `top_n` - Number of top directories to return
///
/// # Returns
/// * `Result<AnalysisResult>` - Analysis results or error
pub fn analyze_disk_usage(
    path: &Path,
    config: &AnalysisConfig,
    top_n: usize,
) -> Result<AnalysisResult> {
    // Validate path exists and is accessible
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    if !path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", path.display());
    }

    // Configure rayon thread pool if specified
    if let Some(threads) = config.num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .context("Failed to configure thread pool")?;
    }

    log::info!("Starting disk usage analysis for: {}", path.display());

    // Build exclusion matcher
    let exclusions = build_exclusion_matcher(&config.exclude_patterns)?;

    // Collect immediate subdirectories and their entries
    let mut dir_sizes: HashMap<PathBuf, DirectoryStats> = HashMap::new();
    let mut total_files = 0;
    let mut total_dirs = 0;
    let mut seen_inodes: HashSet<(u64, u64)> = HashSet::new(); // (dev, ino) pairs
    let mut error_count = 0;

    // Walk the directory tree
    let walker = WalkDir::new(path)
        .follow_links(config.follow_links)
        .max_depth(config.max_depth.unwrap_or(usize::MAX));

    for entry in walker.into_iter().filter_entry(|e| !is_excluded(e, &exclusions)) {
        match entry {
            Ok(entry) => {
                if let Err(e) = process_entry(&entry, path, &mut dir_sizes, &mut total_files, &mut total_dirs, &mut seen_inodes) {
                    log::debug!("Error processing {}: {}", entry.path().display(), e);
                    error_count += 1;
                }
            }
            Err(e) => {
                // Only log non-transient errors in debug mode
                let err_str = e.to_string();
                if !err_str.contains("Interrupted system call") {
                    log::debug!("Error accessing path: {}", e);
                }
                error_count += 1;
            }
        }
    }

    if error_count > 0 {
        log::info!("Skipped {} items due to errors (use --debug to see details)", error_count);
    }

    // Calculate total size
    let total_size: u64 = dir_sizes.values().map(|s| s.size).sum();

    // Convert to DirectoryEntry and sort by size
    let mut directories: Vec<DirectoryEntry> = dir_sizes
        .into_par_iter()
        .map(|(path, stats)| DirectoryEntry {
            path,
            size: stats.size,
            file_count: stats.file_count,
            dir_count: stats.dir_count,
        })
        .collect();

    directories.sort_by(|a, b| b.size.cmp(&a.size));

    // Take top N
    let top_directories = directories.into_iter().take(top_n).collect();

    log::info!(
        "Analysis complete: {} bytes, {} files, {} directories",
        total_size,
        total_files,
        total_dirs
    );

    Ok(AnalysisResult {
        root_path: path.to_path_buf(),
        total_size,
        total_files,
        total_dirs,
        top_directories,
    })
}

/// Statistics for a directory
#[derive(Debug, Default, Clone)]
struct DirectoryStats {
    size: u64,
    file_count: usize,
    dir_count: usize,
}

/// Process a single directory entry
fn process_entry(
    entry: &DirEntry,
    root_path: &Path,
    dir_sizes: &mut HashMap<PathBuf, DirectoryStats>,
    total_files: &mut usize,
    total_dirs: &mut usize,
    seen_inodes: &mut HashSet<(u64, u64)>,
) -> Result<()> {
    let path = entry.path();

    if entry.file_type().is_file() {
        let metadata = entry.metadata()
            .context("Failed to read file metadata")?;

        // Get inode information to track hard links
        let inode_key = get_inode_key(&metadata);

        // Skip if we've already counted this inode (hard link)
        if !seen_inodes.insert(inode_key) {
            log::trace!("Skipping hard link: {}", path.display());
            return Ok(());
        }

        // Use actual disk usage (blocks) instead of apparent size
        let size = get_disk_usage(&metadata);
        *total_files += 1;

        // Find the immediate subdirectory under root (or file directly in root)
        let subdir = find_immediate_subdir(path, root_path);

        let stats = dir_sizes.entry(subdir).or_default();
        stats.size += size;
        stats.file_count += 1;
    } else if entry.file_type().is_dir() && path != root_path {
        *total_dirs += 1;

        // Track this as a subdirectory
        let subdir = find_immediate_subdir(path, root_path);
        let stats = dir_sizes.entry(subdir).or_default();
        stats.dir_count += 1;
    }

    Ok(())
}

/// Get a unique key for an inode (handles hard links correctly)
#[cfg(unix)]
fn get_inode_key(metadata: &std::fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}

/// Get a unique key for a file on Windows (using file index)
#[cfg(windows)]
fn get_inode_key(metadata: &std::fs::Metadata) -> (u64, u64) {
    use std::os::windows::fs::MetadataExt;
    // On Windows, use volume serial number and file index
    // This is a simplified approach; for full correctness we'd need nFileIndexHigh/Low
    (metadata.volume_serial_number().unwrap_or(0), metadata.file_index().unwrap_or(0))
}

/// Fallback for other platforms
#[cfg(not(any(unix, windows)))]
fn get_inode_key(_metadata: &std::fs::Metadata) -> (u64, u64) {
    // On unsupported platforms, return a dummy key (will count hard links separately)
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    (0, COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Get actual disk usage in bytes (matching du behavior)
/// This uses blocks allocated on disk, not apparent file size
#[cfg(unix)]
fn get_disk_usage(metadata: &std::fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    // blocks() returns the number of 512-byte blocks allocated
    metadata.blocks() * 512
}

/// Get disk usage on Windows
#[cfg(windows)]
fn get_disk_usage(metadata: &std::fs::Metadata) -> u64 {
    use std::os::windows::fs::MetadataExt;
    // On Windows, use file_size which is the actual size on disk
    // This is an approximation; Windows uses cluster sizes
    metadata.file_size()
}

/// Fallback for other platforms
#[cfg(not(any(unix, windows)))]
fn get_disk_usage(metadata: &std::fs::Metadata) -> u64 {
    // Fall back to apparent size
    metadata.len()
}

/// Find the immediate subdirectory under root for a given path
fn find_immediate_subdir(path: &Path, root: &Path) -> PathBuf {
    // Strip the root prefix and get the first component
    match path.strip_prefix(root) {
        Ok(relative) => {
            if let Some(first) = relative.components().next() {
                root.join(first)
            } else {
                // Path is the root itself
                root.to_path_buf()
            }
        }
        Err(_) => {
            // Path is not under root (shouldn't happen)
            path.to_path_buf()
        }
    }
}

/// Build exclusion matcher from patterns
fn build_exclusion_matcher(patterns: &[String]) -> Result<Vec<glob::Pattern>> {
    patterns
        .iter()
        .map(|p| glob::Pattern::new(p).context(format!("Invalid glob pattern: {}", p)))
        .collect()
}

/// Check if a directory entry should be excluded
fn is_excluded(entry: &DirEntry, exclusions: &[glob::Pattern]) -> bool {
    if exclusions.is_empty() {
        return false;
    }

    let path = entry.path();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    exclusions.iter().any(|pattern| pattern.matches(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = AnalysisConfig::default();

        let result = analyze_disk_usage(temp_dir.path(), &config, 10).unwrap();

        assert_eq!(result.total_size, 0);
        assert_eq!(result.total_files, 0);
        assert_eq!(result.top_directories.len(), 0);
    }

    #[test]
    fn test_analyze_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let config = AnalysisConfig::default();
        let result = analyze_disk_usage(temp_dir.path(), &config, 10).unwrap();

        assert_eq!(result.total_files, 1);
        assert!(result.total_size > 0);
    }

    #[test]
    fn test_exclusion_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create some directories
        fs::create_dir_all(temp_dir.path().join("node_modules")).unwrap();
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();

        // Add files
        fs::write(temp_dir.path().join("node_modules/test.js"), "test").unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let config = AnalysisConfig {
            exclude_patterns: vec!["node_modules".to_string()],
            ..Default::default()
        };

        let result = analyze_disk_usage(temp_dir.path(), &config, 10).unwrap();

        // Should only see src directory
        assert_eq!(result.total_files, 1);
        assert_eq!(result.top_directories.len(), 1);
        assert!(result.top_directories[0].path.ends_with("src"));
    }

    #[test]
    fn test_find_immediate_subdir() {
        let root = Path::new("/home/user");
        let path = Path::new("/home/user/projects/rust/src/main.rs");

        let result = find_immediate_subdir(path, root);
        assert_eq!(result, Path::new("/home/user/projects"));
    }
}
