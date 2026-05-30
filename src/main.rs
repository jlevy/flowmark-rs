//! CLI entry point for flowmark.

#[cfg(feature = "cli")]
mod cli {
    use anyhow::{Context, Result};
    use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, parser::ValueSource};
    use rayon::prelude::*;
    use std::fs;
    use std::io::{BufWriter, Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use flowmark::config::{
        ConfigValue, DEFAULT_WRAP_WIDTH, FormatOptions, ListSpacing, find_config_file, load_config,
        merge_cli_with_config,
    };
    use flowmark::file_resolver::{FileResolver, FileResolverConfig};
    use flowmark::formatter::filling::{
        get_fill_perf_stats, reset_fill_perf_stats, set_fill_perf_stats_enabled,
    };
    use flowmark::incremental_cache::{
        IncrementalCache, compute_formatter_fingerprint, manifest_hash_count, project_manifest_path,
    };
    use flowmark::settings::{
        CacheRootSource, INCREMENTAL_CACHE_SUBDIR, resolve_default_cache_root,
    };
    use flowmark::skills;

    /// Characters that indicate a path is a glob pattern.
    const GLOB_CHARS: &[char] = &['*', '?', '['];

    #[derive(Default)]
    struct CachePerfCounters {
        hits: AtomicUsize,
        misses: AtomicUsize,
    }

    #[derive(Parser, Debug)]
    #[command(
        name = "flowmark",
        version,
        disable_help_flag = true,
        long_version = env!("FLOWMARK_LONG_VERSION"),
        about = "Flowmark: Better auto-formatting for Markdown and plaintext",
        next_line_help = false,
        after_help = "Common usage:
  flowmark --auto README.md
  flowmark --auto docs/
  flowmark --auto .
  flowmark --list-files .

Agent usage:
  flowmark --skill
  Agents should run `flowmark --skill` for full Flowmark usage guidance.

Use `flowmark --docs` for full documentation.
"
    )]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Args {
        /// Input files or directories (required; use `-` for stdin, `.` for current directory)
        pub files: Vec<String>,

        /// Output file (use `-` for stdout)
        #[arg(short, long, default_value = "-")]
        pub output: String,

        /// Line width to wrap to, or 0 to disable line wrapping
        #[arg(short, long, default_value_t = DEFAULT_WRAP_WIDTH)]
        pub width: usize,

        /// Process as plaintext (no Markdown parsing)
        #[arg(short, long)]
        pub plaintext: bool,

        /// Enable semantic (sentence-based) line breaks (Markdown mode only)
        #[arg(short, long)]
        pub semantic: bool,

        /// Enable safe cleanups for common issues (Markdown mode only)
        #[arg(short, long)]
        pub cleanups: bool,

        /// Convert straight quotes to typographic (curly) quotes (Markdown mode only)
        #[arg(long)]
        pub smartquotes: bool,

        /// Convert three dots (...) to ellipsis character (…) (Markdown mode only)
        #[arg(long)]
        pub ellipses: bool,

        /// Control list spacing
        #[arg(long, value_enum, default_value_t = ListSpacing::Preserve)]
        pub list_spacing: ListSpacing,

        /// Edit files in place (ignores --output)
        #[arg(short, long)]
        pub inplace: bool,

        /// No backup when using --inplace
        #[arg(long)]
        pub nobackup: bool,

        /// Don't write any files; exit with a non-zero status if any file would be reformatted. Useful for CI and pre-commit validation
        #[arg(long)]
        pub check: bool,

        /// Convenience preset for full auto-formatting; requires at least one file or directory argument
        #[arg(long)]
        pub auto: bool,

        /// Show verbose output (e.g., which files are being formatted)
        #[arg(short, long)]
        pub verbose: bool,

        /// Print help
        #[arg(
            short = 'h',
            long = "help",
            action = clap::ArgAction::HelpShort
        )]
        pub help: Option<bool>,

        // --- File discovery options ---
        /// Additional file patterns to include (e.g., '*.mdx'). Can be repeated
        #[arg(long, value_name = "PATTERN", help_heading = "File Discovery Options")]
        pub extend_include: Vec<String>,

        /// Replace all default exclusion patterns. Can be repeated
        #[arg(long, value_name = "PATTERN", help_heading = "File Discovery Options")]
        pub exclude: Option<Vec<String>>,

        /// Add to default exclusion patterns (e.g., 'drafts/'). Can be repeated
        #[arg(long, value_name = "PATTERN", help_heading = "File Discovery Options")]
        pub extend_exclude: Vec<String>,

        /// Disable .gitignore integration
        #[arg(long, help_heading = "File Discovery Options")]
        pub no_respect_gitignore: bool,

        /// Apply exclusion patterns even to files named explicitly on the command line
        #[arg(long, help_heading = "File Discovery Options")]
        pub force_exclude: bool,

        /// Print resolved file paths without formatting; requires at least one file or directory argument
        #[arg(long, help_heading = "File Discovery Options")]
        pub list_files: bool,

        /// Skip files larger than this size in bytes (0 = no limit)
        #[arg(
            long,
            default_value_t = 1_048_576,
            value_name = "BYTES",
            help_heading = "File Discovery Options"
        )]
        pub files_max_size: u64,

        // --- Agent skill options ---
        /// Print skill instructions for coding agents (same as Claude Code SKILL.md)
        #[arg(long, help_heading = "Agent Options")]
        pub skill: bool,

        /// Install the flowmark agent skill (project-local .agents/skills + .claude/skills + AGENTS.md by default)
        #[arg(long, help_heading = "Agent Options")]
        pub install_skill: bool,

        /// Install the skill to a single explicit base dir (e.g. ~/.claude); for global/custom installs
        #[arg(long, value_name = "DIR", help_heading = "Agent Options")]
        pub agent_base: Option<String>,

        /// With --install-skill: comma-separated subset of surfaces to install. Values: 'portable', 'claude', 'agents-md', or 'all' (default if omitted)
        #[arg(long, value_name = "LIST", help_heading = "Agent Options")]
        pub surfaces: Option<String>,

        /// Print full documentation
        #[arg(long, help_heading = "Agent Options")]
        pub docs: bool,

        // --- Performance options ---
        /// Number of parallel formatting threads (0 = all cores, default)
        #[arg(long, default_value_t = 0, value_name = "N", help_heading = "Performance Options")]
        pub threads: usize,

        /// Enable incremental cache for unchanged-file fast paths (default: enabled)
        #[arg(
            long,
            default_value_t = true,
            num_args = 0..=1,
            default_missing_value = "true",
            value_name = "BOOL",
            help_heading = "Performance Options"
        )]
        pub incremental: bool,

        /// Disable incremental cache for this run
        #[arg(long = "no-cache", help_heading = "Performance Options")]
        pub no_incremental: bool,

        /// Override incremental cache directory
        #[arg(long = "cache-dir", value_name = "DIR", help_heading = "Performance Options")]
        pub incremental_cache_dir: Option<String>,

        /// Show cache directory, file count, and total size
        #[arg(long, help_heading = "Performance Options")]
        pub show_cache: bool,

        /// Delete the entire cache directory (non-interactive)
        #[arg(long, help_heading = "Performance Options")]
        pub clear_cache: bool,

        /// Print performance statistics summary
        #[arg(long, help_heading = "Performance Options")]
        pub perf_stats: bool,
    }

    /// Detect which flags the user explicitly passed on the command line.
    fn detect_explicit_flags(matches: &ArgMatches) -> Vec<&'static str> {
        let tracked: &[(&str, &str)] = &[
            ("width", "width"),
            ("semantic", "semantic"),
            ("cleanups", "cleanups"),
            ("smartquotes", "smartquotes"),
            ("ellipses", "ellipses"),
            ("list_spacing", "list_spacing"),
            ("extend_include", "extend_include"),
            ("exclude", "exclude"),
            ("extend_exclude", "extend_exclude"),
            ("no_respect_gitignore", "respect_gitignore"),
            ("force_exclude", "force_exclude"),
            ("files_max_size", "files_max_size"),
            ("incremental", "incremental"),
            ("no_incremental", "incremental"),
            ("incremental_cache_dir", "incremental_cache_dir"),
        ];

        let mut explicit = Vec::new();
        for &(arg_id, field_name) in tracked {
            if matches.value_source(arg_id) == Some(ValueSource::CommandLine) {
                explicit.push(field_name);
            }
        }
        explicit
    }

    /// Check if any input paths need file resolution (directories or globs).
    fn needs_file_resolution(files: &[String]) -> bool {
        for f in files {
            if f == "-" {
                continue;
            }
            if Path::new(f).is_dir() {
                return true;
            }
            if f.contains(GLOB_CHARS) {
                return true;
            }
        }
        false
    }

    /// Resolve files using the file resolver if needed.
    #[allow(clippy::too_many_arguments)]
    fn resolve_files(
        files: &[String],
        list_files: bool,
        extend_include: &[String],
        exclude: Option<&Vec<String>>,
        extend_exclude: &[String],
        respect_gitignore: bool,
        force_exclude: bool,
        files_max_size: u64,
    ) -> Vec<String> {
        // `force_exclude` makes exclusions apply to explicitly-named files too, so the
        // resolver must run even for a plain file list (matching Python's `_resolve_files`).
        if !needs_file_resolution(files) && !list_files && !force_exclude {
            // Validate all non-stdin paths exist (matches Python's file_resolver behavior).
            for f in files {
                if f != "-" && !Path::new(f).exists() {
                    eprintln!("Error: Path not found: {f}");
                    std::process::exit(1);
                }
            }
            return files.to_vec();
        }

        // Filter out stdin marker before passing to resolver
        let resolvable: Vec<&str> =
            files.iter().filter(|f| *f != "-").map(String::as_str).collect();
        let stdin_present = resolvable.len() < files.len();

        let config = FileResolverConfig {
            extend_include: extend_include.to_vec(),
            exclude: exclude.cloned(),
            extend_exclude: extend_exclude.to_vec(),
            respect_gitignore,
            force_exclude,
            files_max_size,
            ..FileResolverConfig::default()
        };

        let mut file_resolver = FileResolver::new(config);
        let found = match file_resolver.resolve(&resolvable) {
            Ok(paths) => paths,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        };

        let mut result: Vec<String> =
            found.iter().map(|p| p.to_string_lossy().to_string()).collect();

        if stdin_present {
            result.insert(0, "-".to_string());
        }

        result
    }

    #[derive(Debug, Default)]
    struct IncrementalConfigOverrides {
        incremental: Option<bool>,
        incremental_cache_dir: Option<String>,
    }

    fn load_incremental_config_overrides(config_path: &Path) -> IncrementalConfigOverrides {
        let Ok(text) = std::fs::read_to_string(config_path) else {
            return IncrementalConfigOverrides::default();
        };
        let Ok(data) = toml::from_str::<toml::Value>(&text) else {
            return IncrementalConfigOverrides::default();
        };

        let section = if config_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "pyproject.toml")
        {
            data.get("tool")
                .and_then(|t| t.get("flowmark"))
                .cloned()
                .unwrap_or(toml::Value::Table(toml::map::Map::new()))
        } else {
            data
        };

        let Some(table) = section.as_table() else {
            return IncrementalConfigOverrides::default();
        };

        let mut overrides = IncrementalConfigOverrides::default();
        for (key, value) in table {
            if let Some(sub_table) = value.as_table() {
                for (sub_key, sub_value) in sub_table {
                    apply_incremental_override(&mut overrides, sub_key, sub_value);
                }
            } else {
                apply_incremental_override(&mut overrides, key, value);
            }
        }
        overrides
    }

    fn apply_incremental_override(
        overrides: &mut IncrementalConfigOverrides,
        key: &str,
        value: &toml::Value,
    ) {
        let normalized = key.replace('-', "_");
        match normalized.as_str() {
            "incremental" | "cache" => overrides.incremental = value.as_bool(),
            "incremental_cache_dir" | "cache_dir" => {
                if let Some(v) = value.as_str() {
                    overrides.incremental_cache_dir = Some(v.to_string());
                }
            }
            _ => {}
        }
    }

    fn format_ns_as_ms(ns: u128) -> String {
        let whole_ms = ns / 1_000_000;
        let fractional_ms = (ns % 1_000_000) / 1_000;
        format!("{whole_ms}.{fractional_ms:03}")
    }

    fn format_hit_rate_percent_tenths(cache_hits: usize, cache_total: usize) -> String {
        if cache_total == 0 {
            return "0.0".to_string();
        }
        let scaled_percent =
            ((cache_hits as u128) * 1000 + (cache_total as u128 / 2)) / (cache_total as u128);
        let whole = scaled_percent / 10;
        let frac = scaled_percent % 10;
        format!("{whole}.{frac}")
    }

    fn resolve_cache_root(cache_dir_override: Option<&str>, warn_on_fallback: bool) -> PathBuf {
        if let Some(cache_dir) = cache_dir_override {
            return PathBuf::from(cache_dir);
        }

        let resolved = resolve_default_cache_root();
        if warn_on_fallback {
            match resolved.source {
                CacheRootSource::OsCacheDir => {}
                CacheRootSource::HomeFallback => {
                    eprintln!(
                        "Warning: OS cache directory unavailable; using home fallback at {}",
                        resolved.path.display()
                    );
                }
                CacheRootSource::TempFallback => {
                    eprintln!(
                        "Warning: OS and home cache directories unavailable; using temp fallback at {}",
                        resolved.path.display()
                    );
                }
            }
        }
        resolved.path
    }

    fn should_resolve_project_root_for_cache_ops(show_cache: bool) -> bool {
        show_cache
    }

    fn format_bytes_human(bytes: u64) -> String {
        const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
        let mut value = bytes;
        let mut scale = 1u64;
        let mut unit_index = 0usize;
        while value >= 1024 && unit_index + 1 < UNITS.len() {
            value /= 1024;
            scale *= 1024;
            unit_index += 1;
        }
        if unit_index == 0 {
            format!("{bytes} {}", UNITS[unit_index])
        } else {
            let whole = bytes / scale;
            let tenths = ((bytes % scale) * 10) / scale;
            format!("{whole}.{tenths} {}", UNITS[unit_index])
        }
    }

    fn cache_usage(cache_root: &Path) -> Result<(usize, u64)> {
        if !cache_root.exists() {
            return Ok((0, 0));
        }

        let mut file_count = 0usize;
        let mut total_bytes = 0u64;
        let mut stack = vec![cache_root.to_path_buf()];

        while let Some(dir) = stack.pop() {
            for entry in
                fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                let file_type = entry.file_type()?;
                if file_type.is_dir() {
                    stack.push(path);
                } else if file_type.is_file() {
                    file_count += 1;
                    total_bytes += entry.metadata()?.len();
                }
            }
        }

        Ok((file_count, total_bytes))
    }

    fn cache_manifest_count(cache_root: &Path) -> Result<usize> {
        let incremental_dir = cache_root.join(INCREMENTAL_CACHE_SUBDIR);
        if !incremental_dir.exists() {
            return Ok(0);
        }

        let mut manifest_count = 0usize;
        for entry in fs::read_dir(&incremental_dir)
            .with_context(|| format!("failed to read {}", incremental_dir.display()))?
        {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                manifest_count += 1;
            }
        }
        Ok(manifest_count)
    }

    fn run_show_cache(cache_root: &Path, project_root: &Path) -> Result<()> {
        let (file_count, total_bytes) = cache_usage(cache_root)?;
        let manifest_count = cache_manifest_count(cache_root)?;
        let project_manifest = project_manifest_path(cache_root, project_root);
        let project_entry_count = manifest_hash_count(&project_manifest).unwrap_or(0);

        println!("Cache directory: {}", cache_root.display());
        println!("Cache files: {file_count}");
        println!("Cache size: {}", format_bytes_human(total_bytes));
        println!("Cache manifests: {manifest_count}");
        println!("Current project manifest: {}", project_manifest.display());
        println!("Current project entries: {project_entry_count}");
        Ok(())
    }

    fn run_clear_cache(cache_root: &Path) -> Result<()> {
        println!("Cache directory: {}", cache_root.display());
        if cache_root.exists() {
            fs::remove_dir_all(cache_root)
                .with_context(|| format!("failed to remove cache at {}", cache_root.display()))?;
            println!("Cache cleared.");
        } else {
            println!("Cache already empty.");
        }
        Ok(())
    }

    fn open_incremental_cache(
        enabled: bool,
        cache_dir_override: Option<&str>,
        opts: &FormatOptions,
        config_path: Option<&Path>,
    ) -> Option<Arc<IncrementalCache>> {
        if !enabled {
            return None;
        }

        let project_root = std::env::current_dir().ok()?;
        let cache_root = resolve_cache_root(cache_dir_override, true);
        let fingerprint =
            compute_formatter_fingerprint(opts, env!("CARGO_PKG_VERSION"), config_path);

        match IncrementalCache::open(&cache_root, &project_root, fingerprint) {
            Ok(cache) => Some(Arc::new(cache)),
            Err(error) => {
                eprintln!(
                    "Warning: failed to initialize incremental cache at {}: {error}",
                    cache_root.display()
                );
                None
            }
        }
    }

    fn format_inplace_with_incremental_cache(
        opts: &FormatOptions,
        path: &Path,
        nobackup: bool,
        cache: &IncrementalCache,
    ) -> Result<bool> {
        let content = std::fs::read_to_string(path)?;
        if cache.is_known_formatted(path, content.as_bytes()) {
            return Ok(true);
        }

        let formatted = opts.reformat_text(&content);
        if formatted == content {
            cache.record_formatted(path, content.as_bytes());
            return Ok(false);
        }

        if !nobackup {
            let backup_path = path.with_extension("bak");
            std::fs::copy(path, &backup_path)?;
        }

        atomic_write(path, &formatted)?;
        cache.record_formatted(path, formatted.as_bytes());
        Ok(false)
    }

    fn atomic_write(path: &Path, content: &str) -> Result<()> {
        #[cfg(unix)]
        let original_permissions = path.metadata().ok().map(|metadata| metadata.permissions());

        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        let mut temp_file = tempfile::NamedTempFile::new_in(dir)?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.persist(path).map_err(|error| error.error)?;

        #[cfg(unix)]
        if let Some(permissions) = original_permissions {
            std::fs::set_permissions(path, permissions)?;
        }

        Ok(())
    }

    pub fn run() -> Result<()> {
        // Parse args, keeping matches for explicit flag detection
        let matches = Args::command().get_matches();
        let mut args = Args::from_arg_matches(&matches).context("failed to parse arguments")?;
        let explicit_flags = detect_explicit_flags(&matches);
        let is_auto = args.auto;

        // Handle --auto shortcut
        if args.auto {
            args.inplace = true;
            args.nobackup = true;
            args.semantic = true;
            args.cleanups = true;
            args.smartquotes = true;
            args.ellipses = true;
        }
        if args.no_incremental {
            args.incremental = false;
        }

        // Early exit: --install-skill
        if args.install_skill {
            let results = if let Some(base) = args.agent_base.as_deref() {
                if args.surfaces.is_some() {
                    eprintln!(
                        "Error: --surfaces is incompatible with --agent-base (the latter is \
                         an explicit single-base install and writes one skill location)."
                    );
                    std::process::exit(2);
                }
                match skills::install_skill(Some(base), None, None) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                let selected = match args.surfaces.as_deref() {
                    None => None,
                    Some(raw) => {
                        let tokens: Vec<&str> =
                            raw.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
                        if tokens.is_empty() {
                            eprintln!(
                                "Error: --surfaces given an empty value; pass a comma-separated \
                                 list of: portable, claude, agents-md, all."
                            );
                            std::process::exit(2);
                        }
                        let mut expanded: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        for tok in tokens {
                            if tok == "all" {
                                expanded
                                    .extend(skills::ALL_SURFACES.iter().map(|s| (*s).to_string()));
                            } else if skills::ALL_SURFACES.contains(&tok) {
                                expanded.insert(tok.to_string());
                            } else {
                                eprintln!(
                                    "Error: unknown surface '{tok}'; valid values: portable, \
                                     claude, agents-md, all."
                                );
                                std::process::exit(2);
                            }
                        }
                        Some(expanded)
                    }
                };
                match skills::install_skill(None, None, selected.as_ref()) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            };
            // Forward-compat guard: fail if any surface was newer than this build understands.
            if results.iter().any(|r| r.action == "blocked-newer") {
                std::process::exit(1);
            }
            return Ok(());
        }

        // Early exit: --skill
        if args.skill {
            println!("{}", skills::compose_skill(None));
            return Ok(());
        }

        // Early exit: --docs
        if args.docs {
            print!("{}", skills::get_docs_content());
            return Ok(());
        }

        // Derive respect_gitignore (inverted from --no-respect-gitignore)
        let mut respect_gitignore = !args.no_respect_gitignore;

        // Load and merge config file settings
        let mut resolved_config_path: Option<PathBuf> = None;
        let explicit_refs: Vec<&str> = explicit_flags.clone();
        if let Ok(cwd) = std::env::current_dir() {
            if let Some(config_path) = find_config_file(&cwd) {
                let config = load_config(&config_path);
                let incremental_overrides = load_incremental_config_overrides(&config_path);
                resolved_config_path = Some(config_path.clone());
                merge_cli_with_config(Some(&config), is_auto, &explicit_refs, |name, value| {
                    apply_config_field(&mut args, &mut respect_gitignore, name, value);
                });
                if !explicit_refs.contains(&"incremental") {
                    if let Some(v) = incremental_overrides.incremental {
                        args.incremental = v;
                    }
                }
                if !explicit_refs.contains(&"incremental_cache_dir") {
                    if let Some(v) = incremental_overrides.incremental_cache_dir {
                        args.incremental_cache_dir = Some(v);
                    }
                }
            }
        }

        // Cache lifecycle operations (non-interactive).
        if args.clear_cache || args.show_cache {
            let cache_root = resolve_cache_root(args.incremental_cache_dir.as_deref(), true);
            let project_root = if should_resolve_project_root_for_cache_ops(args.show_cache) {
                Some(
                    std::env::current_dir()
                        .context("failed to resolve current working directory")?,
                )
            } else {
                None
            };
            if args.clear_cache {
                run_clear_cache(&cache_root)?;
            }
            if args.show_cache {
                let project_root = project_root
                    .as_deref()
                    .context("failed to resolve project root for --show-cache")?;
                run_show_cache(&cache_root, project_root)?;
            }
            return Ok(());
        }

        // Validate: files required
        if args.files.is_empty() {
            if is_auto {
                eprintln!(
                    "Error: --auto requires at least one file or directory argument \
                     (use '.' for current directory, --help for more options)"
                );
                std::process::exit(1);
            }
            if args.list_files {
                eprintln!(
                    "Error: --list-files requires at least one file or directory argument \
                     (use '.' for current directory, --help for more options)"
                );
                std::process::exit(1);
            }
            eprintln!(
                "Error: No input specified. Provide files, directories \
                 (use '.' for current directory), or '-' for stdin. \
                 Use --help for more options."
            );
            std::process::exit(1);
        }

        // Resolve files
        let resolved_files = resolve_files(
            &args.files,
            args.list_files,
            &args.extend_include,
            args.exclude.as_ref(),
            &args.extend_exclude,
            respect_gitignore,
            args.force_exclude,
            args.files_max_size,
        );

        // Handle --list-files mode
        if args.list_files {
            for f in &resolved_files {
                println!("{f}");
            }
            return Ok(());
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

        // Check-only mode: never write. Report any file that would change and exit with
        // status 1 (matching Black/Ruff/Prettier). Bypasses the write-time validations
        // (multi-file output guard, inplace-with-stdin) since nothing is written.
        if args.check {
            let mut changed: Vec<&str> = Vec::new();
            for file in &resolved_files {
                let content = if file == "-" {
                    let mut input = String::new();
                    std::io::stdin().read_to_string(&mut input).context("failed to read stdin")?;
                    input
                } else {
                    std::fs::read_to_string(file)
                        .with_context(|| format!("failed to read {file}"))?
                };
                if opts.reformat_text(&content) != content {
                    changed.push(file);
                }
            }
            for f in &changed {
                let label = if *f == "-" { "<stdin>" } else { f };
                eprintln!("Would reformat: {label}");
            }
            if changed.is_empty() {
                return Ok(());
            }
            std::process::exit(1);
        }

        let incremental_enabled = args.inplace && args.incremental;
        let incremental_cache = open_incremental_cache(
            incremental_enabled,
            args.incremental_cache_dir.as_deref(),
            &opts,
            resolved_config_path.as_deref(),
        );
        set_fill_perf_stats_enabled(args.perf_stats);
        if args.perf_stats {
            reset_fill_perf_stats();
        }
        let cache_perf = Arc::new(CachePerfCounters::default());

        // Configure rayon thread pool
        if args.threads > 0 {
            rayon::ThreadPoolBuilder::new().num_threads(args.threads).build_global().ok();
        }

        // Validate: cannot use --inplace with stdin
        if args.inplace && resolved_files.iter().any(|f| f == "-") {
            eprintln!("Error: Cannot use `inplace` with stdin");
            std::process::exit(1);
        }

        // Validate: cannot use --output with multiple files
        let has_explicit_output = args.output != "-";
        if has_explicit_output && resolved_files.len() > 1 {
            eprintln!(
                "Error: Cannot specify output file when processing multiple files \
                 (use --inplace instead)"
            );
            std::process::exit(1);
        }

        // Partition stdin from regular files (stdin must be handled sequentially)
        let (stdin_files, regular_files): (Vec<&String>, Vec<&String>) =
            resolved_files.iter().partition(|f| *f == "-");

        // Handle stdin sequentially
        for _file in &stdin_files {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).context("failed to read stdin")?;

            let output = opts.reformat_text(&input);
            let stdout = std::io::stdout().lock();
            let mut writer = BufWriter::new(stdout);
            writer.write_all(output.as_bytes()).context("failed to write to stdout")?;
        }

        // Format regular files: parallel when inplace, sequential for stdout output
        if args.inplace {
            // Inplace: parallelize across files (order doesn't matter)
            let incremental_cache = incremental_cache.clone();
            let cache_perf = cache_perf.clone();
            regular_files.par_iter().try_for_each(|file| {
                let path = PathBuf::from(file);
                if args.verbose {
                    eprintln!("formatting {}", path.display());
                }
                if let Some(cache) = &incremental_cache {
                    let was_cache_hit =
                        format_inplace_with_incremental_cache(&opts, &path, args.nobackup, cache)
                            .with_context(|| format!("failed to format {}", path.display()))?;
                    if args.perf_stats {
                        if was_cache_hit {
                            cache_perf.hits.fetch_add(1, Ordering::Relaxed);
                        } else {
                            cache_perf.misses.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Ok(())
                } else {
                    if args.perf_stats {
                        cache_perf.misses.fetch_add(1, Ordering::Relaxed);
                    }
                    opts.reformat_file(&path, None, args.inplace, args.nobackup)
                        .with_context(|| format!("failed to format {}", path.display()))
                }
            })?;
        } else {
            // Stdout or explicit output: must preserve file order
            for file in &regular_files {
                let path = PathBuf::from(file);
                let output_path =
                    if has_explicit_output { Some(PathBuf::from(&args.output)) } else { None };
                if args.verbose {
                    eprintln!("formatting {}", path.display());
                }
                opts.reformat_file(&path, output_path.as_deref(), false, args.nobackup)
                    .with_context(|| format!("failed to format {}", path.display()))?;
            }
        }

        if let Some(cache) = &incremental_cache {
            cache.flush().context("failed to persist incremental cache")?;
        }
        if args.perf_stats {
            let stats = get_fill_perf_stats();
            let cache_hits = cache_perf.hits.load(Ordering::Relaxed);
            let cache_misses = cache_perf.misses.load(Ordering::Relaxed);
            let cache_total = cache_hits + cache_misses;
            let hit_rate = format_hit_rate_percent_tenths(cache_hits, cache_total);
            eprintln!("perf-stats:");
            eprintln!(
                "  fill_markdown files={} total={}ms preprocess={}ms parse={}ms transforms={}ms render={}ms postprocess={}ms",
                stats.files,
                format_ns_as_ms(stats.total_ns()),
                format_ns_as_ms(stats.preprocess_ns),
                format_ns_as_ms(stats.parse_ns),
                format_ns_as_ms(stats.transforms_ns),
                format_ns_as_ms(stats.render_ns),
                format_ns_as_ms(stats.postprocess_ns),
            );
            eprintln!("  incremental hits={cache_hits} misses={cache_misses} hit_rate={hit_rate}%");
        }
        set_fill_perf_stats_enabled(false);

        Ok(())
    }

    /// Apply a config field value to the args.
    fn apply_config_field(
        args: &mut Args,
        respect_gitignore: &mut bool,
        name: &str,
        value: &ConfigValue,
    ) {
        match name {
            "width" => {
                if let ConfigValue::Usize(v) = value {
                    args.width = *v;
                }
            }
            "semantic" => {
                if let ConfigValue::Bool(v) = value {
                    args.semantic = *v;
                }
            }
            "cleanups" => {
                if let ConfigValue::Bool(v) = value {
                    args.cleanups = *v;
                }
            }
            "smartquotes" => {
                if let ConfigValue::Bool(v) = value {
                    args.smartquotes = *v;
                }
            }
            "ellipses" => {
                if let ConfigValue::Bool(v) = value {
                    args.ellipses = *v;
                }
            }
            "list_spacing" => {
                if let ConfigValue::String(v) = value {
                    if let Ok(ls) = v.parse::<ListSpacing>() {
                        args.list_spacing = ls;
                    }
                }
            }
            "extend_include" => {
                if let ConfigValue::StringList(v) = value {
                    args.extend_include.clone_from(v);
                }
            }
            "exclude" => {
                if let ConfigValue::StringList(v) = value {
                    args.exclude = Some(v.clone());
                }
            }
            "extend_exclude" => {
                if let ConfigValue::StringList(v) = value {
                    args.extend_exclude.clone_from(v);
                }
            }
            "respect_gitignore" => {
                if let ConfigValue::Bool(v) = value {
                    *respect_gitignore = *v;
                }
            }
            "force_exclude" => {
                if let ConfigValue::Bool(v) = value {
                    args.force_exclude = *v;
                }
            }
            "files_max_size" => {
                if let ConfigValue::U64(v) = value {
                    args.files_max_size = *v;
                }
            }
            _ => {}
        }
    }

    #[cfg(test)]
    mod tests {
        use super::should_resolve_project_root_for_cache_ops;

        #[test]
        fn clear_cache_does_not_require_project_root() {
            assert!(
                !should_resolve_project_root_for_cache_ops(false),
                "--clear-cache should not require project root resolution"
            );
        }

        #[test]
        fn show_cache_requires_project_root() {
            assert!(
                should_resolve_project_root_for_cache_ops(true),
                "--show-cache should require project root resolution"
            );
        }
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
