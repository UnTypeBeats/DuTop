//! Output formatting for disk usage results

use crate::format::{format_percentage, format_size_auto};
use crate::AnalysisResult;
use std::cmp;

/// Configuration for output display
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Whether to use colors in output
    pub use_colors: bool,
    /// Width of the bar chart
    pub bar_width: usize,
    /// Width of the size column
    pub size_width: usize,
    /// Width of the percentage column
    pub percent_width: usize,
    /// Width of the name column
    pub name_width: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            use_colors: atty::is(atty::Stream::Stdout),
            bar_width: 30,
            size_width: 8,
            percent_width: 5,
            name_width: 30,
        }
    }
}

/// ANSI color codes
struct Colors {
    green: &'static str,
    yellow: &'static str,
    red: &'static str,
    reset: &'static str,
}

impl Colors {
    fn enabled() -> Self {
        Self {
            green: "\x1b[38;5;34m",
            yellow: "\x1b[38;5;220m",
            red: "\x1b[38;5;160m",
            reset: "\x1b[0m",
        }
    }

    fn disabled() -> Self {
        Self {
            green: "",
            yellow: "",
            red: "",
            reset: "",
        }
    }
}

/// Print analysis results in a formatted table with bar chart
pub fn print_results(result: &AnalysisResult, config: &OutputConfig) {
    let colors = if config.use_colors {
        Colors::enabled()
    } else {
        Colors::disabled()
    };

    println!("\nAnalyzing: {}", result.root_path.display());
    println!();

    if result.top_directories.is_empty() {
        println!("┌────────────────────┐");
        println!("│ No files found     │");
        println!("└────────────────────┘");
        return;
    }

    // Calculate maximum size for bar scaling
    let max_size = result
        .top_directories
        .first()
        .map(|d| d.size)
        .unwrap_or(1);

    // Print table header
    print_table_border(&config, true);

    // Print each directory
    for dir in &result.top_directories {
        print_directory_row(dir, max_size, result.total_size, &colors, config);
    }

    // Print table footer
    print_table_border(&config, false);

    // Print total
    println!("\nTotal: {}", format_size_auto(result.total_size));
    println!(
        "Files: {}  Directories: {}",
        result.total_files, result.total_dirs
    );
}

/// Print a directory row in the table
fn print_directory_row(
    dir: &crate::DirectoryEntry,
    max_size: u64,
    total_size: u64,
    colors: &Colors,
    config: &OutputConfig,
) {
    // Calculate bar length
    let bar_length = if max_size > 0 {
        ((dir.size as f64 / max_size as f64) * config.bar_width as f64) as usize
    } else {
        0
    };
    let bar_length = cmp::min(bar_length, config.bar_width);

    // Select color based on size percentage
    let color = select_color(bar_length, config.bar_width, colors);

    // Create bar
    let filled = "█".repeat(bar_length);
    let empty = "░".repeat(config.bar_width - bar_length);
    let bar = format!("{}{}{}", color, filled, empty);

    // Format size and percentage
    let size_str = format_size_auto(dir.size);
    let percent_str = format_percentage(dir.size, total_size);

    // Get directory name (relative to analyzed path)
    let name = dir
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    // Truncate name if needed to fit in column
    let display_name = if name.len() > config.name_width {
        format!("{}...", &name[..config.name_width - 3])
    } else {
        name.to_string()
    };

    // Print the row
    println!(
        "│ {}{} │ {:>size_w$} │ {:>pct_w$} │ {:<name_w$} │",
        bar,
        colors.reset,
        size_str,
        percent_str,
        display_name,
        size_w = config.size_width,
        pct_w = config.percent_width,
        name_w = config.name_width
    );
}

/// Select color based on bar fill percentage
fn select_color<'a>(bar_length: usize, bar_width: usize, colors: &'a Colors) -> &'a str {
    let threshold_yellow = bar_width * 33 / 100;
    let threshold_red = bar_width * 50 / 100;

    if bar_length >= threshold_red {
        colors.red
    } else if bar_length >= threshold_yellow {
        colors.yellow
    } else {
        colors.green
    }
}

/// Print table border
fn print_table_border(config: &OutputConfig, is_top: bool) {
    let bar_border = "─".repeat(config.bar_width + 2);
    let size_border = "─".repeat(config.size_width + 2);
    let percent_border = "─".repeat(config.percent_width + 2);
    let name_border = "─".repeat(config.name_width + 2);

    let (left, mid, right) = if is_top {
        ("┌", "┬", "┐")
    } else {
        ("└", "┴", "┘")
    };

    println!(
        "{}{}{}{}{}{}{}{}{}",
        left, bar_border, mid, size_border, mid, percent_border, mid, name_border, right
    );
}

/// Output results in JSON format
pub fn print_json(result: &AnalysisResult) -> anyhow::Result<()> {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonOutput<'a> {
        path: String,
        total_size: u64,
        total_size_human: String,
        file_count: usize,
        directory_count: usize,
        top_directories: Vec<JsonDirectory<'a>>,
    }

    #[derive(Serialize)]
    struct JsonDirectory<'a> {
        path: String,
        size: u64,
        size_human: String,
        percentage: f64,
        file_count: usize,
        dir_count: usize,
        #[serde(skip)]
        _marker: std::marker::PhantomData<&'a ()>,
    }

    let total = result.total_size as f64;
    let output = JsonOutput {
        path: result.root_path.display().to_string(),
        total_size: result.total_size,
        total_size_human: format_size_auto(result.total_size),
        file_count: result.total_files,
        directory_count: result.total_dirs,
        top_directories: result
            .top_directories
            .iter()
            .map(|d| JsonDirectory {
                path: d.path.display().to_string(),
                size: d.size,
                size_human: format_size_auto(d.size),
                percentage: if total > 0.0 {
                    (d.size as f64 / total) * 100.0
                } else {
                    0.0
                },
                file_count: d.file_count,
                dir_count: d.dir_count,
                _marker: std::marker::PhantomData,
            })
            .collect(),
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_color() {
        let colors = Colors::enabled();

        // Low usage - green
        assert_eq!(select_color(5, 30, &colors), colors.green);

        // Medium usage - yellow
        assert_eq!(select_color(12, 30, &colors), colors.yellow);

        // High usage - red
        assert_eq!(select_color(20, 30, &colors), colors.red);
    }

    #[test]
    fn test_colors_disabled() {
        let colors = Colors::disabled();
        assert_eq!(colors.green, "");
        assert_eq!(colors.yellow, "");
        assert_eq!(colors.red, "");
        assert_eq!(colors.reset, "");
    }
}
