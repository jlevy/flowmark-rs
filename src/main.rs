//! CLI entry point for flowmark.

#[cfg(feature = "cli")]
mod cli {
    use clap::Parser;
    use std::io::Read;
    use std::path::PathBuf;

    use flowmark::{ListSpacing, DEFAULT_WRAP_WIDTH};

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
        #[arg(long, default_value = "preserve")]
        pub list_spacing: String,

        /// Edit files in place
        #[arg(short, long)]
        pub inplace: bool,

        /// No backup when using --inplace
        #[arg(long)]
        pub nobackup: bool,

        /// Shortcut for --inplace --nobackup --semantic --cleanups --smartquotes --ellipses
        #[arg(long)]
        pub auto: bool,
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
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

        let list_spacing: ListSpacing = args
            .list_spacing
            .parse()
            .map_err(|e: String| flowmark::Error::Config(e))?;

        for file in &args.files {
            if file == "-" {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;

                let output = flowmark::reformat_text(
                    &input,
                    args.width,
                    args.plaintext,
                    args.semantic,
                    args.cleanups,
                    args.smartquotes,
                    args.ellipses,
                    list_spacing,
                );
                print!("{output}");
            } else {
                let path = PathBuf::from(file);
                let output_path = if args.output == "-" {
                    None
                } else {
                    Some(PathBuf::from(&args.output))
                };

                flowmark::reformat_file(
                    &path,
                    output_path.as_deref(),
                    args.width,
                    args.inplace,
                    args.nobackup,
                    args.plaintext,
                    args.semantic,
                    args.cleanups,
                    args.smartquotes,
                    args.ellipses,
                    list_spacing,
                )?;
            }
        }

        Ok(())
    }
}

fn main() {
    #[cfg(feature = "cli")]
    {
        if let Err(e) = cli::run() {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("CLI feature not enabled. Build with --features cli");
        std::process::exit(1);
    }
}
