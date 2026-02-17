//! Flowmark CLI: Markdown auto-formatter.

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use flowmark::config::FlowmarkConfig;
#[cfg(feature = "cli")]
use flowmark::reformat_api::{reformat_file, reformat_text};

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(name = "flowmark", about = "Markdown auto-formatter")]
#[command(version)]
struct Cli {
    /// Input files (reads from stdin if none specified)
    files: Vec<String>,

    /// Write output back to the input file
    #[arg(short = 'i', long)]
    inplace: bool,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Wrap width
    #[arg(short, long, default_value_t = 88)]
    width: usize,

    /// Use semantic line breaks (break at sentence boundaries)
    #[arg(long, default_value_t = true)]
    semantic: bool,

    /// Apply cleanups
    #[arg(long, default_value_t = true)]
    cleanups: bool,

    /// Convert straight quotes to typographic quotes
    #[arg(long)]
    smartquotes: bool,

    /// Convert ... to proper ellipsis
    #[arg(long)]
    ellipses: bool,

    /// Check mode: exit with error if files would change
    #[arg(long)]
    check: bool,

    /// List spacing mode: preserve, loose, tight
    #[arg(long, default_value = "preserve")]
    list_spacing: String,
}

#[cfg(feature = "cli")]
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let config = FlowmarkConfig {
        width: cli.width,
        semantic: cli.semantic,
        cleanups: cli.cleanups,
        smartquotes: cli.smartquotes,
        ellipses: cli.ellipses,
        list_spacing: cli.list_spacing,
    };

    if cli.files.is_empty() {
        // Read from stdin
        use std::io::Read;
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let output = reformat_text(&input, &config);
        print!("{output}");
    } else if cli.inplace {
        let mut any_changed = false;
        for file in &cli.files {
            let path = std::path::Path::new(file);
            let changed = reformat_file(path, &config)?;
            if changed {
                any_changed = true;
                if cli.check {
                    eprintln!("Would reformat: {file}");
                } else {
                    eprintln!("Reformatted: {file}");
                }
            }
        }
        if cli.check && any_changed {
            std::process::exit(1);
        }
    } else if cli.files.len() == 1 {
        let content = std::fs::read_to_string(&cli.files[0])?;
        let output = reformat_text(&content, &config);
        if let Some(out_path) = &cli.output {
            std::fs::write(out_path, &output)?;
        } else {
            print!("{output}");
        }
    } else {
        eprintln!("Error: Cannot process multiple files without --inplace");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI not available: compile with --features cli");
    std::process::exit(1);
}
