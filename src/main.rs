//! CLI entry point for flowmark.

#[cfg(feature = "cli")]
mod cli {
    use anyhow::{Context, Result};
    use clap::Parser;
    use std::io::{BufWriter, Read, Write};
    use std::path::PathBuf;

    use flowmark::{DEFAULT_WRAP_WIDTH, FormatOptions, ListSpacing};

    #[derive(Parser, Debug)]
    #[command(name = "flowmark", version, about = "Markdown auto-formatter for clean diffs")]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Args {
        /// Input files or directories; use `-` for stdin
        #[arg(default_value = "-")]
        pub files: Vec<String>,

        /// Output file (- for stdout)
        #[arg(short, long, default_value = "-")]
        pub output: String,

        /// Line width (0 to disable wrapping)
        #[arg(short, long, default_value_t = DEFAULT_WRAP_WIDTH)]
        pub width: usize,

        /// Plaintext mode (no Markdown parsing)
        #[arg(short, long)]
        pub plaintext: bool,

        /// Semantic (sentence-based) line breaks
        #[arg(short, long)]
        pub semantic: bool,

        /// Safe cleanups (e.g., unbold headings)
        #[arg(short, long)]
        pub cleanups: bool,

        /// Convert straight quotes to curly quotes
        #[arg(long)]
        pub smartquotes: bool,

        /// Convert ... to ellipsis character
        #[arg(long)]
        pub ellipses: bool,

        /// Control list item spacing
        #[arg(long, value_enum, default_value_t = ListSpacing::Preserve)]
        pub list_spacing: ListSpacing,

        /// Edit files in place
        #[arg(short, long)]
        pub inplace: bool,

        /// No backup when using --inplace
        #[arg(long)]
        pub nobackup: bool,

        /// Shortcut for --inplace --nobackup --semantic --cleanups --smartquotes --ellipses
        #[arg(long)]
        pub auto: bool,

        /// Show verbose output (e.g., which files are being formatted)
        #[arg(short, long)]
        pub verbose: bool,
    }

    pub fn run() -> Result<()> {
        let mut args = Args::parse();

        // Handle --auto shortcut
        if args.auto {
            args.inplace = true;
            args.nobackup = true;
            args.semantic = true;
            args.cleanups = true;
            args.smartquotes = true;
            args.ellipses = true;
        }

        let opts = FormatOptions {
            width: args.width,
            plaintext: args.plaintext,
            semantic: args.semantic,
            cleanups: args.cleanups,
            smartquotes: args.smartquotes,
            ellipses: args.ellipses,
            list_spacing: args.list_spacing,
        };

        for file in &args.files {
            if file == "-" {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input).context("failed to read stdin")?;

                let output = opts.reformat_text(&input);
                let stdout = std::io::stdout().lock();
                let mut writer = BufWriter::new(stdout);
                writer.write_all(output.as_bytes()).context("failed to write to stdout")?;
            } else {
                let path = PathBuf::from(file);
                let output_path =
                    if args.output == "-" { None } else { Some(PathBuf::from(&args.output)) };

                if args.verbose {
                    eprintln!("formatting {}", path.display());
                }

                opts.reformat_file(&path, output_path.as_deref(), args.inplace, args.nobackup)
                    .with_context(|| format!("failed to format {}", path.display()))?;
            }
        }

        Ok(())
    }
}

fn main() -> std::process::ExitCode {
    // Reset SIGPIPE to default behavior so piping to `head` etc. works correctly.
    #[cfg(unix)]
    #[allow(unsafe_code)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    #[cfg(feature = "cli")]
    {
        if let Err(e) = cli::run() {
            eprintln!("error: {e:#}");
            return std::process::ExitCode::FAILURE;
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("error: CLI feature not enabled. Build with --features cli");
        return std::process::ExitCode::FAILURE;
    }

    std::process::ExitCode::SUCCESS
}
