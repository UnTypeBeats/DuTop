//! DuTop - High-performance disk usage analysis tool
//!
//! A fast, parallel disk usage analyzer built in Rust to replace legacy shell scripts.

use anyhow::{Context, Result};
use clap::Parser;
use dutop::{analyze_disk_usage, output, AnalysisConfig};
use std::path::PathBuf;
use std::process;

/// High-performance disk usage analysis tool
#[derive(Parser, Debug)]
#[command(name = "dutop")]
#[command(author = "DuTop Contributors")]
#[command(version)]
#[command(about = "Analyze disk usage and display top directories", long_about = None)]
struct Args {
    /// Directory to analyze (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Number of top directories to display
    #[arg(short = 'n', long = "top", default_value = "10")]
    top: usize,

    /// Maximum depth to traverse (default: unlimited)
    #[arg(short = 'd', long = "depth")]
    depth: Option<usize>,

    /// Exclude patterns (glob syntax, can be specified multiple times)
    #[arg(short = 'x', long = "exclude")]
    exclude: Vec<String>,

    /// Follow symbolic links
    #[arg(short = 'L', long = "follow-links")]
    follow_links: bool,

    /// Number of threads to use (default: auto-detect)
    #[arg(short = 'j', long = "threads")]
    threads: Option<usize>,

    /// Output format: human (default), json
    #[arg(short = 'f', long = "format", default_value = "human")]
    format: OutputFormat,

    /// Disable colored output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Enable verbose logging
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Enable debug logging
    #[arg(long = "debug")]
    debug: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() {
    let exit_code = match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {:#}", e);

            // Return appropriate exit code based on error type
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                match io_err.kind() {
                    std::io::ErrorKind::NotFound => 4,        // Path not found
                    std::io::ErrorKind::PermissionDenied => 3, // Permission denied
                    _ => 5,                                    // Disk I/O error
                }
            } else {
                1 // General error
            }
        }
    };

    process::exit(exit_code);
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args)?;

    log::debug!("Starting DuTop with args: {:?}", args);

    // Validate path
    let path = args.path.canonicalize().context(format!(
        "Failed to access path: {}",
        args.path.display()
    ))?;

    log::info!("Analyzing path: {}", path.display());

    // Build configuration
    let config = AnalysisConfig {
        max_depth: args.depth,
        exclude_patterns: args.exclude,
        follow_links: args.follow_links,
        num_threads: args.threads,
    };

    // Perform analysis
    let result = analyze_disk_usage(&path, &config, args.top)?;

    // Output results
    match args.format {
        OutputFormat::Human => {
            let output_config = output::OutputConfig {
                use_colors: !args.no_color && atty::is(atty::Stream::Stdout),
                ..Default::default()
            };
            output::print_results(&result, &output_config);
        }
        OutputFormat::Json => {
            output::print_json(&result)?;
        }
    }

    log::info!("Analysis complete");

    Ok(())
}

fn init_logging(args: &Args) -> Result<()> {
    let log_level = if args.debug {
        "debug"
    } else if args.verbose {
        "info"
    } else {
        "warn"
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    Ok(())
}
