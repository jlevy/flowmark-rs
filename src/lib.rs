//! Flowmark: A Markdown auto-formatter for clean diffs and semantic line breaks.
//!
//! Ported from Python: [flowmark](https://github.com/jlevy/flowmark)

pub mod config;
pub mod error;
pub mod file_resolver;
pub mod formatter;
pub mod parser;
pub mod skills;
pub mod transform;
pub mod typography;
pub mod wrapping;

use std::path::Path;

pub use config::{DEFAULT_WRAP_WIDTH, FormatOptions, ListSpacing};
pub use error::{Error, Result};
pub use formatter::filling::fill_markdown;
pub use wrapping::line_wrappers::{line_wrap_by_sentence, line_wrap_to_width};
pub use wrapping::sentence::split_sentences_regex;
pub use wrapping::text_filling::{Wrap, fill_text};
pub use wrapping::text_wrapping::{
    html_md_word_split, simple_word_split, wrap_paragraph, wrap_paragraph_lines,
};

impl FormatOptions {
    /// Reformat a Markdown or plain text string.
    ///
    /// # Examples
    ///
    /// ```
    /// use flowmark::{FormatOptions, ListSpacing};
    ///
    /// // With default options (width 88, no semantic breaks)
    /// let opts = FormatOptions::default();
    /// let result = opts.reformat_text("# Hello\n\nSome text.");
    /// assert_eq!(result, "# Hello\n\nSome text.\n");
    ///
    /// // With semantic line breaks and typography
    /// let opts = FormatOptions {
    ///     semantic: true,
    ///     smartquotes: true,
    ///     ..FormatOptions::default()
    /// };
    /// let result = opts.reformat_text("He said \"hello.\" She said \"goodbye.\"");
    /// assert!(result.contains('\u{201c}')); // curly quotes applied
    /// ```
    pub fn reformat_text(&self, text: &str) -> String {
        if self.plaintext {
            // Python uses Wrap.WRAP (not WRAP_FULL) with the HTML/Markdown-aware
            // word splitter. This is likely a bug in Python (fmr-5u8i): plaintext
            // mode should use simple_word_splitter, but instead uses
            // html_md_word_splitter which incidentally treats markdown links and
            // fenced code blocks as atomic tokens. We match this behavior for parity.
            let wrap = if self.width > 0 { Wrap::Wrap } else { Wrap::None };
            fill_text(text, wrap, self.width, "", "", 0, None)
        } else {
            fill_markdown(
                text,
                true,
                self.width,
                self.semantic,
                self.cleanups,
                self.smartquotes,
                self.ellipses,
                None,
                self.list_spacing,
            )
        }
    }

    /// Reformat a Markdown or plain text file.
    pub fn reformat_file(
        &self,
        path: &Path,
        output: Option<&Path>,
        inplace: bool,
        nobackup: bool,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let formatted = self.reformat_text(&content);

        // Skip write if content is unchanged (preserves mtime, avoids I/O)
        if inplace && formatted == content {
            return Ok(());
        }

        if inplace {
            if !nobackup {
                let backup_path = path.with_extension("bak");
                std::fs::copy(path, &backup_path)?;
            }
            atomic_write(path, &formatted)?;
        } else if let Some(out) = output {
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            atomic_write(out, &formatted)?;
        } else {
            print!("{formatted}");
        }

        Ok(())
    }
}

/// Reformat a Markdown or plain text string.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn reformat_text(
    text: &str,
    width: usize,
    plaintext: bool,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
    list_spacing: ListSpacing,
) -> String {
    let opts =
        FormatOptions { width, plaintext, semantic, cleanups, smartquotes, ellipses, list_spacing };
    opts.reformat_text(text)
}

/// Reformat a Markdown or plain text file.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub fn reformat_file(
    path: &Path,
    output: Option<&Path>,
    width: usize,
    inplace: bool,
    nobackup: bool,
    plaintext: bool,
    semantic: bool,
    cleanups: bool,
    smartquotes: bool,
    ellipses: bool,
    list_spacing: ListSpacing,
) -> Result<()> {
    let opts =
        FormatOptions { width, plaintext, semantic, cleanups, smartquotes, ellipses, list_spacing };
    opts.reformat_file(path, output, inplace, nobackup)
}

/// Write content to a file atomically via a temporary file.
///
/// Writes to a temp file in the same directory, then persists (renames) to the
/// target path. This prevents file corruption if the process is interrupted.
/// On Unix, preserves the original file's permissions if it already exists.
fn atomic_write(path: &Path, content: &str) -> Result<()> {
    use std::io::Write;

    // Read original permissions before overwriting (Unix only).
    #[cfg(unix)]
    let original_permissions = path.metadata().ok().map(|m| m.permissions());

    let dir = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(content.as_bytes())?;
    tmp.persist(path).map_err(|e| Error::Io(e.error))?;

    // Restore original permissions after persist.
    #[cfg(unix)]
    if let Some(perms) = original_permissions {
        std::fs::set_permissions(path, perms)?;
    }

    Ok(())
}
