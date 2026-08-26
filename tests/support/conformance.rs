//! Independent Rust adapter for Flowmark's language-neutral process corpus.

use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use toml::Value;

const SCHEMA_VERSION: i64 = 1;
const CORPUS_NAME: &str = "flowmark-language-neutral-conformance";
const MAX_DIAGNOSTIC_BYTES: usize = 8_192;
const ENV_ALLOWLIST: &[&str] =
    &["PATH", "PATHEXT", "SYSTEMROOT", "WINDIR", "TMPDIR", "TEMP", "TMP"];
const ALLOWED_PATH_ROOTS: &[&str] =
    &["tests/parity_corpus/", "tests/tryscript/fixtures/", "tests/testdocs/"];

static ID_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9]+(?:[.-][a-z0-9]+)*$").expect("valid conformance ID regex")
});
static CHANGE_ID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^FM-[A-Z0-9]+(?:-[A-Z0-9]+)*$").expect("valid change ID regex"));
static ENV_NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").expect("valid environment name regex")
});

type Result<T> = std::result::Result<T, ConformanceError>;
type Table = toml::map::Map<String, Value>;

#[derive(Debug)]
pub struct ConformanceError {
    pub code: &'static str,
    message: String,
}

impl ConformanceError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        let message = bounded_text(message.into());
        Self { code, message }
    }
}

impl fmt::Display for ConformanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ConformanceError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseKind {
    Stdin,
    Files,
}

#[derive(Clone, Debug)]
pub struct ConformanceCase {
    pub id: String,
    pub change_id: String,
    pub description: String,
    pub kind: CaseKind,
    pub tags: Vec<String>,
    pub args: Vec<String>,
    pub expected_stdout: PathBuf,
    pub expected_stderr: PathBuf,
    pub expected_exit: i32,
    pub idempotent: bool,
    pub stdin: Option<PathBuf>,
    pub before_tree: Option<PathBuf>,
    pub after_tree: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct ConformanceManifest {
    pub corpus: String,
    pub default_env: BTreeMap<String, String>,
    pub cases: Vec<ConformanceCase>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileSnapshot {
    path: PathBuf,
    content: Vec<u8>,
}

#[derive(Debug)]
struct ProcessResult {
    command: Vec<String>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
    tree: Vec<FileSnapshot>,
}

#[derive(Clone, Copy)]
enum PassInput<'a> {
    First,
    Second { stdout: &'a [u8], tree: &'a [FileSnapshot] },
}

#[derive(Debug)]
struct CaseFields {
    id: String,
    change_id: String,
    description: String,
    kind: CaseKind,
    tags: Vec<String>,
    args: Vec<String>,
    expected_stdout: String,
    expected_stderr: String,
    expected_exit: i32,
    idempotent: bool,
    stdin: Option<String>,
    before_tree: Option<String>,
    after_tree: Option<String>,
}

fn bounded_text(mut message: String) -> String {
    if message.len() <= MAX_DIAGNOSTIC_BYTES {
        return message;
    }
    let mut boundary = MAX_DIAGNOSTIC_BYTES;
    while !message.is_char_boundary(boundary) {
        boundary -= 1;
    }
    message.truncate(boundary);
    message.push_str("\n... <diagnostic truncated>");
    message
}

fn table<'a>(value: &'a Value, location: &str) -> Result<&'a Table> {
    value
        .as_table()
        .ok_or_else(|| ConformanceError::new("invalid-type", format!("{location} must be a table")))
}

fn reject_unknown_fields(table: &Table, allowed: &[&str], location: &str) -> Result<()> {
    let allowed: BTreeSet<&str> = allowed.iter().copied().collect();
    if let Some(field) = table.keys().find(|field| !allowed.contains(field.as_str())) {
        return Err(ConformanceError::new(
            "unknown-field",
            format!("{location} has unknown field {field:?}"),
        ));
    }
    Ok(())
}

fn required<'a>(table: &'a Table, field: &str, location: &str) -> Result<&'a Value> {
    table.get(field).ok_or_else(|| {
        ConformanceError::new("missing-field", format!("{location} is missing {field:?}"))
    })
}

fn nonempty_string(table: &Table, field: &str, location: &str) -> Result<String> {
    let value = required(table, field, location)?;
    match value.as_str() {
        Some(text) if !text.is_empty() => Ok(text.to_owned()),
        _ => Err(ConformanceError::new(
            "invalid-type",
            format!("{location}.{field} must be a nonempty string"),
        )),
    }
}

fn string_array(table: &Table, field: &str, location: &str) -> Result<Vec<String>> {
    let value = required(table, field, location)?;
    let values = value.as_array().ok_or_else(|| {
        ConformanceError::new(
            "invalid-type",
            format!("{location}.{field} must be an array of strings"),
        )
    })?;
    values
        .iter()
        .map(|value| {
            value.as_str().map(str::to_owned).ok_or_else(|| {
                ConformanceError::new(
                    "invalid-type",
                    format!("{location}.{field} must be an array of strings"),
                )
            })
        })
        .collect()
}

fn integer(table: &Table, field: &str, location: &str) -> Result<i64> {
    required(table, field, location)?.as_integer().ok_or_else(|| {
        ConformanceError::new("invalid-type", format!("{location}.{field} must be an integer"))
    })
}

fn boolean(table: &Table, field: &str, location: &str) -> Result<bool> {
    required(table, field, location)?.as_bool().ok_or_else(|| {
        ConformanceError::new("invalid-type", format!("{location}.{field} must be a boolean"))
    })
}

fn lexical_path(raw_path: &str, location: &str) -> Result<PathBuf> {
    let components: Vec<&str> = raw_path.split('/').collect();
    if raw_path.starts_with('/')
        || raw_path.contains('\\')
        || components.iter().any(|component| matches!(*component, "" | "." | ".."))
        || !ALLOWED_PATH_ROOTS.iter().any(|root| raw_path.starts_with(root))
    {
        return Err(ConformanceError::new(
            "invalid-path",
            format!("{location} is not a confined path"),
        ));
    }
    Ok(components.iter().collect())
}

fn validate_existing_path(
    relative_path: &Path,
    location: &str,
    repo_root: &Path,
    expect_directory: bool,
) -> Result<()> {
    let mut current = repo_root.to_path_buf();
    for component in relative_path.components() {
        let Component::Normal(component) = component else {
            return Err(ConformanceError::new(
                "invalid-path",
                format!("{location} is not a confined path"),
            ));
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            let code = if error.kind() == std::io::ErrorKind::NotFound {
                "missing-path"
            } else {
                "invalid-path-kind"
            };
            ConformanceError::new(
                code,
                format!("{location} cannot inspect {}: {error}", current.display()),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ConformanceError::new(
                "symlink-path",
                format!("{location} traverses a symlink"),
            ));
        }
    }

    let metadata = fs::metadata(&current).map_err(|error| {
        ConformanceError::new(
            "missing-path",
            format!("{location} does not exist: {} ({error})", relative_path.display()),
        )
    })?;
    if (expect_directory && !metadata.is_dir()) || (!expect_directory && !metadata.is_file()) {
        return Err(ConformanceError::new(
            "invalid-path-kind",
            format!(
                "{location} must name a {}",
                if expect_directory { "directory" } else { "file" }
            ),
        ));
    }
    if expect_directory {
        validate_directory_entries(&current, location)?;
    }
    Ok(())
}

fn sorted_directory_entries(root: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| {
            ConformanceError::new(
                "invalid-path-kind",
                format!("cannot read directory {}: {error}", root.display()),
            )
        })?
        .map(|entry| {
            entry.map(|value| value.path()).map_err(|error| {
                ConformanceError::new(
                    "invalid-path-kind",
                    format!("cannot read entry in {}: {error}", root.display()),
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn validate_directory_entries(root: &Path, location: &str) -> Result<()> {
    for entry in sorted_directory_entries(root)? {
        let metadata = fs::symlink_metadata(&entry).map_err(|error| {
            ConformanceError::new(
                "invalid-path-kind",
                format!("cannot inspect {}: {error}", entry.display()),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ConformanceError::new(
                "symlink-path",
                format!("{location} contains a symlink: {}", entry.display()),
            ));
        }
        if metadata.is_dir() {
            validate_directory_entries(&entry, location)?;
        } else if !metadata.is_file() {
            return Err(ConformanceError::new(
                "invalid-path-kind",
                format!("{location} contains a non-regular entry: {}", entry.display()),
            ));
        }
    }
    Ok(())
}

fn validate_case_fields(table: &Table, index: usize) -> Result<CaseFields> {
    const ALLOWED_FIELDS: &[&str] = &[
        "id",
        "change_id",
        "description",
        "kind",
        "tags",
        "args",
        "expected_stdout",
        "expected_stderr",
        "expected_exit",
        "idempotent",
        "stdin",
        "before_tree",
        "after_tree",
    ];
    let location = format!("case[{index}]");
    reject_unknown_fields(table, ALLOWED_FIELDS, &location)?;

    let kind = match required(table, "kind", &location)?.as_str() {
        Some("stdin") => CaseKind::Stdin,
        Some("files") => CaseKind::Files,
        _ => {
            return Err(ConformanceError::new(
                "invalid-kind",
                format!("{location}.kind must be 'stdin' or 'files'"),
            ));
        }
    };
    let has_stdin = table.contains_key("stdin");
    let has_before = table.contains_key("before_tree");
    let has_after = table.contains_key("after_tree");
    let valid_kind_fields = match kind {
        CaseKind::Stdin => has_stdin && !has_before && !has_after,
        CaseKind::Files => !has_stdin && has_before && has_after,
    };
    if !valid_kind_fields {
        return Err(ConformanceError::new(
            "invalid-kind-fields",
            format!("{location} has fields invalid for {kind:?}"),
        ));
    }

    let id = nonempty_string(table, "id", &location)?;
    if !ID_PATTERN.is_match(&id) {
        return Err(ConformanceError::new(
            "invalid-id",
            format!("{location}.id is invalid: {id:?}"),
        ));
    }
    let change_id = nonempty_string(table, "change_id", &location)?;
    if !CHANGE_ID_PATTERN.is_match(&change_id) {
        return Err(ConformanceError::new(
            "invalid-change-id",
            format!("{location}.change_id is invalid: {change_id:?}"),
        ));
    }
    let description = nonempty_string(table, "description", &location)?;
    let tags = string_array(table, "tags", &location)?;
    if tags.is_empty() || tags.iter().any(|tag| !ID_PATTERN.is_match(tag)) {
        return Err(ConformanceError::new(
            "invalid-tags",
            format!("{location}.tags contains an invalid tag"),
        ));
    }
    if tags.iter().collect::<BTreeSet<_>>().len() != tags.len() {
        return Err(ConformanceError::new(
            "duplicate-tags",
            format!("{location}.tags contains a duplicate"),
        ));
    }

    let args = string_array(table, "args", &location)?;
    let dash_count = args.iter().filter(|argument| argument.as_str() == "-").count();
    let invalid_args = args.iter().any(String::is_empty)
        || match kind {
            CaseKind::Stdin => dash_count != 1,
            CaseKind::Files => dash_count != 0,
        };
    if invalid_args {
        return Err(ConformanceError::new(
            "invalid-args",
            format!("{location}.args is invalid for {kind:?}"),
        ));
    }

    let expected_exit_raw = integer(table, "expected_exit", &location)?;
    if !(0..=255).contains(&expected_exit_raw) {
        return Err(ConformanceError::new(
            "invalid-exit",
            format!("{location}.expected_exit must be 0..255"),
        ));
    }
    let expected_exit = i32::try_from(expected_exit_raw).map_err(|error| {
        ConformanceError::new(
            "invalid-exit",
            format!("{location}.expected_exit is invalid: {error}"),
        )
    })?;
    let idempotent = boolean(table, "idempotent", &location)?;
    if idempotent && expected_exit != 0 {
        return Err(ConformanceError::new(
            "invalid-idempotence",
            format!("{location} cannot repeat a failing case"),
        ));
    }

    Ok(CaseFields {
        id,
        change_id,
        description,
        kind,
        tags,
        args,
        expected_stdout: nonempty_string(table, "expected_stdout", &location)?,
        expected_stderr: nonempty_string(table, "expected_stderr", &location)?,
        expected_exit,
        idempotent,
        stdin: has_stdin.then(|| nonempty_string(table, "stdin", &location)).transpose()?,
        before_tree: has_before
            .then(|| nonempty_string(table, "before_tree", &location))
            .transpose()?,
        after_tree: has_after
            .then(|| nonempty_string(table, "after_tree", &location))
            .transpose()?,
    })
}

fn resolve_case_paths(fields: CaseFields, index: usize) -> Result<ConformanceCase> {
    let location = format!("case[{index}]");
    Ok(ConformanceCase {
        id: fields.id,
        change_id: fields.change_id,
        description: fields.description,
        kind: fields.kind,
        tags: fields.tags,
        args: fields.args,
        expected_stdout: lexical_path(
            &fields.expected_stdout,
            &format!("{location}.expected_stdout"),
        )?,
        expected_stderr: lexical_path(
            &fields.expected_stderr,
            &format!("{location}.expected_stderr"),
        )?,
        expected_exit: fields.expected_exit,
        idempotent: fields.idempotent,
        stdin: fields
            .stdin
            .as_deref()
            .map(|path| lexical_path(path, &format!("{location}.stdin")))
            .transpose()?,
        before_tree: fields
            .before_tree
            .as_deref()
            .map(|path| lexical_path(path, &format!("{location}.before_tree")))
            .transpose()?,
        after_tree: fields
            .after_tree
            .as_deref()
            .map(|path| lexical_path(path, &format!("{location}.after_tree")))
            .transpose()?,
    })
}

fn validate_case_paths(case: &ConformanceCase, index: usize, repo_root: &Path) -> Result<()> {
    let location = format!("case[{index}]");
    validate_existing_path(
        &case.expected_stdout,
        &format!("{location}.expected_stdout"),
        repo_root,
        false,
    )?;
    validate_existing_path(
        &case.expected_stderr,
        &format!("{location}.expected_stderr"),
        repo_root,
        false,
    )?;
    if let Some(path) = &case.stdin {
        validate_existing_path(path, &format!("{location}.stdin"), repo_root, false)?;
    }
    if let Some(path) = &case.before_tree {
        validate_existing_path(path, &format!("{location}.before_tree"), repo_root, true)?;
    }
    if let Some(path) = &case.after_tree {
        validate_existing_path(path, &format!("{location}.after_tree"), repo_root, true)?;
    }
    Ok(())
}

fn validate_manifest_value(
    value: &Value,
    repo_root: &Path,
    allow_case_registries: bool,
) -> Result<ConformanceManifest> {
    const TOP_LEVEL_FIELDS: &[&str] =
        &["schema_version", "corpus", "defaults", "case_registry", "case"];
    let root = table(value, "manifest")?;
    reject_unknown_fields(root, TOP_LEVEL_FIELDS, "manifest")?;

    let schema_version = integer(root, "schema_version", "manifest")?;
    if schema_version != SCHEMA_VERSION {
        return Err(ConformanceError::new(
            "unsupported-schema-version",
            format!("schema version {schema_version} is unsupported; expected {SCHEMA_VERSION}"),
        ));
    }
    let corpus = nonempty_string(root, "corpus", "manifest")?;
    if corpus != CORPUS_NAME {
        return Err(ConformanceError::new(
            "invalid-corpus",
            format!("manifest.corpus must be {CORPUS_NAME:?}"),
        ));
    }

    let defaults_value =
        root.get("defaults").cloned().unwrap_or_else(|| Value::Table(Table::new()));
    let defaults = table(&defaults_value, "manifest.defaults")?;
    reject_unknown_fields(defaults, &["env"], "manifest.defaults")?;
    let env_value = defaults.get("env").cloned().unwrap_or_else(|| Value::Table(Table::new()));
    let env = table(&env_value, "manifest.defaults.env")?;
    let mut default_env = BTreeMap::new();
    for (name, value) in env {
        let Some(value) = value.as_str() else {
            return Err(ConformanceError::new(
                "invalid-environment",
                "manifest.defaults.env must map names to strings",
            ));
        };
        if !ENV_NAME_PATTERN.is_match(name) {
            return Err(ConformanceError::new(
                "invalid-environment",
                "manifest.defaults.env must map names to strings",
            ));
        }
        default_env.insert(name.clone(), value.to_owned());
    }

    let raw_cases = match root.get("case") {
        None => Vec::new(),
        Some(value) => value.as_array().cloned().ok_or_else(|| {
            ConformanceError::new("invalid-type", "manifest.case must be an array")
        })?,
    };
    let mut case_fields = Vec::with_capacity(raw_cases.len());
    for (index, value) in raw_cases.iter().enumerate() {
        case_fields.push(validate_case_fields(table(value, &format!("case[{index}]"))?, index)?);
    }
    let mut seen = BTreeSet::new();
    for fields in &case_fields {
        if !seen.insert(fields.id.clone()) {
            return Err(ConformanceError::new(
                "duplicate-case-id",
                format!("duplicate case ID {:?}", fields.id),
            ));
        }
    }

    let mut cases = Vec::with_capacity(case_fields.len());
    for (index, fields) in case_fields.into_iter().enumerate() {
        cases.push(resolve_case_paths(fields, index)?);
    }
    for (index, case) in cases.iter().enumerate() {
        validate_case_paths(case, index, repo_root)?;
    }

    let raw_registries = match root.get("case_registry") {
        None => Vec::new(),
        Some(value) => value
            .as_array()
            .ok_or_else(|| {
                ConformanceError::new(
                    "invalid-type",
                    "manifest.case_registry must be an array of strings",
                )
            })?
            .clone(),
    };
    let mut registry_paths = Vec::with_capacity(raw_registries.len());
    for value in raw_registries {
        let raw_path = value.as_str().ok_or_else(|| {
            ConformanceError::new(
                "invalid-type",
                "manifest.case_registry must be an array of strings",
            )
        })?;
        registry_paths.push(raw_path.to_owned());
    }
    if !allow_case_registries && !registry_paths.is_empty() {
        return Err(ConformanceError::new(
            "nested-case-registry",
            "case registries cannot include registries",
        ));
    }

    let mut seen_registries = BTreeSet::new();
    for (index, raw_path) in registry_paths.iter().enumerate() {
        let location = format!("manifest.case_registry[{index}]");
        let relative_path = lexical_path(raw_path, &location)?;
        validate_existing_path(&relative_path, &location, repo_root, false)?;
        if !seen_registries.insert(relative_path.clone()) {
            return Err(ConformanceError::new(
                "duplicate-case-registry",
                format!("duplicate registry path {}", relative_path.display()),
            ));
        }
        let registry_path = repo_root.join(&relative_path);
        let registry_text = fs::read_to_string(&registry_path).map_err(|error| {
            ConformanceError::new(
                "invalid-case-registry",
                format!("cannot load {}: {error}", registry_path.display()),
            )
        })?;
        let registry_value = toml::from_str::<Value>(&registry_text).map_err(|error| {
            ConformanceError::new(
                "invalid-case-registry",
                format!("cannot load {}: {error}", registry_path.display()),
            )
        })?;
        let registry = validate_manifest_value(&registry_value, repo_root, false)?;
        if table(&registry_value, "included registry")?.contains_key("defaults") {
            return Err(ConformanceError::new(
                "invalid-case-registry",
                format!("included registry {} cannot define defaults", relative_path.display()),
            ));
        }
        cases.extend(registry.cases);
    }

    if cases.is_empty() {
        return Err(ConformanceError::new(
            "invalid-type",
            "manifest must define one or more cases",
        ));
    }
    seen.clear();
    for case in &cases {
        if !seen.insert(case.id.clone()) {
            return Err(ConformanceError::new(
                "duplicate-case-id",
                format!("duplicate case ID {:?}", case.id),
            ));
        }
    }
    Ok(ConformanceManifest { corpus, default_env, cases })
}

pub fn load_manifest(path: &Path, repo_root: &Path) -> Result<ConformanceManifest> {
    let text = fs::read_to_string(path).map_err(|error| {
        ConformanceError::new(
            "invalid-manifest",
            format!("cannot load {}: {error}", path.display()),
        )
    })?;
    let value = toml::from_str::<Value>(&text).map_err(|error| {
        ConformanceError::new(
            "invalid-manifest",
            format!("cannot load {}: {error}", path.display()),
        )
    })?;
    validate_manifest_value(&value, repo_root, true)
}

pub fn select_cases<'a>(
    manifest: &'a ConformanceManifest,
    ids: &[&str],
    change_ids: &[&str],
    tags: &[&str],
) -> Result<Vec<&'a ConformanceCase>> {
    let known_ids: BTreeSet<&str> = manifest.cases.iter().map(|case| case.id.as_str()).collect();
    let known_change_ids: BTreeSet<&str> =
        manifest.cases.iter().map(|case| case.change_id.as_str()).collect();
    let known_tags: BTreeSet<&str> =
        manifest.cases.iter().flat_map(|case| case.tags.iter().map(String::as_str)).collect();
    for (name, requested, known) in [
        ("case ID", ids, &known_ids),
        ("change ID", change_ids, &known_change_ids),
        ("tag", tags, &known_tags),
    ] {
        if let Some(value) = requested.iter().find(|value| !known.contains(**value)) {
            return Err(ConformanceError::new(
                "unknown-selector",
                format!("unknown {name} selector {value:?}"),
            ));
        }
    }

    let explicitly_selected = !(ids.is_empty() && change_ids.is_empty() && tags.is_empty());
    let selected: Vec<&ConformanceCase> = manifest
        .cases
        .iter()
        .filter(|case| {
            (explicitly_selected || !case.tags.iter().any(|tag| tag == "deferred"))
                && (ids.is_empty() || ids.contains(&case.id.as_str()))
                && (change_ids.is_empty() || change_ids.contains(&case.change_id.as_str()))
                && (tags.is_empty()
                    || tags.iter().all(|tag| case.tags.iter().any(|value| value == tag)))
        })
        .collect();
    if selected.is_empty() {
        return Err(ConformanceError::new(
            "empty-selection",
            "selectors matched no conformance cases",
        ));
    }
    Ok(selected)
}

fn copy_tree_contents(source: &Path, destination: &Path) -> Result<()> {
    for path in sorted_directory_entries(source)? {
        let name = path.file_name().ok_or_else(|| {
            ConformanceError::new(
                "invalid-path-kind",
                format!("path has no file name: {}", path.display()),
            )
        })?;
        let target = destination.join(name);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            ConformanceError::new(
                "invalid-path-kind",
                format!("cannot inspect {}: {error}", path.display()),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(ConformanceError::new(
                "symlink-path",
                format!("cannot materialize symlink {}", path.display()),
            ));
        }
        if metadata.is_dir() {
            fs::create_dir(&target).map_err(|error| {
                ConformanceError::new(
                    "materialization-failure",
                    format!("cannot create {}: {error}", target.display()),
                )
            })?;
            copy_tree_contents(&path, &target)?;
        } else if metadata.is_file() {
            fs::copy(&path, &target).map_err(|error| {
                ConformanceError::new(
                    "materialization-failure",
                    format!("cannot copy {} to {}: {error}", path.display(), target.display()),
                )
            })?;
        } else {
            return Err(ConformanceError::new(
                "invalid-path-kind",
                format!("cannot materialize non-regular entry {}", path.display()),
            ));
        }
    }
    Ok(())
}

fn write_snapshot_tree(snapshots: &[FileSnapshot], destination: &Path) -> Result<()> {
    for snapshot in snapshots {
        let target = destination.join(&snapshot.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                ConformanceError::new(
                    "materialization-failure",
                    format!("cannot create {}: {error}", parent.display()),
                )
            })?;
        }
        fs::write(&target, &snapshot.content).map_err(|error| {
            ConformanceError::new(
                "materialization-failure",
                format!("cannot write {}: {error}", target.display()),
            )
        })?;
    }
    Ok(())
}

fn snapshot_tree(root: &Path) -> Result<Vec<FileSnapshot>> {
    fn visit(root: &Path, current: &Path, snapshots: &mut Vec<FileSnapshot>) -> Result<()> {
        for path in sorted_directory_entries(current)? {
            let metadata = fs::symlink_metadata(&path).map_err(|error| {
                ConformanceError::new(
                    "sandbox-entry",
                    format!("cannot inspect {}: {error}", path.display()),
                )
            })?;
            let relative = path.strip_prefix(root).map_err(|error| {
                ConformanceError::new(
                    "sandbox-entry",
                    format!("cannot relativize {}: {error}", path.display()),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(ConformanceError::new(
                    "sandbox-entry",
                    format!("sandbox contains symlink {}", relative.display()),
                ));
            }
            if metadata.is_dir() {
                visit(root, &path, snapshots)?;
            } else if metadata.is_file() {
                let content = fs::read(&path).map_err(|error| {
                    ConformanceError::new(
                        "sandbox-entry",
                        format!("cannot read {}: {error}", path.display()),
                    )
                })?;
                snapshots.push(FileSnapshot { path: relative.to_path_buf(), content });
            } else {
                return Err(ConformanceError::new(
                    "sandbox-entry",
                    format!("sandbox contains non-file {}", relative.display()),
                ));
            }
        }
        Ok(())
    }

    let mut snapshots = Vec::new();
    visit(root, root, &mut snapshots)?;
    Ok(snapshots)
}

fn materialize_case(
    case: &ConformanceCase,
    repo_root: &Path,
    sandbox: &Path,
    second_pass: bool,
    previous_stdout: Option<&[u8]>,
    previous_tree: Option<&[FileSnapshot]>,
) -> Result<Option<Vec<u8>>> {
    if !sorted_directory_entries(sandbox)?.is_empty() {
        return Err(ConformanceError::new(
            "invalid-sandbox",
            "case sandbox must be an empty directory",
        ));
    }

    match case.kind {
        CaseKind::Stdin => {
            if second_pass {
                return previous_stdout.map(|bytes| Some(bytes.to_vec())).ok_or_else(|| {
                    ConformanceError::new(
                        "invalid-idempotence",
                        format!("case {:?} has no first-pass stdout", case.id),
                    )
                });
            }
            let path = case.stdin.as_ref().ok_or_else(|| {
                ConformanceError::new(
                    "invalid-kind-fields",
                    format!("case {:?} has no stdin", case.id),
                )
            })?;
            fs::read(repo_root.join(path)).map(Some).map_err(|error| {
                ConformanceError::new(
                    "materialization-failure",
                    format!("cannot read {}: {error}", path.display()),
                )
            })
        }
        CaseKind::Files => {
            if second_pass {
                if let Some(snapshots) = previous_tree {
                    write_snapshot_tree(snapshots, sandbox)?;
                    return Ok(None);
                }
            }
            let source = if second_pass { &case.after_tree } else { &case.before_tree };
            let source = source.as_ref().ok_or_else(|| {
                ConformanceError::new(
                    "invalid-kind-fields",
                    format!("case {:?} has no input tree", case.id),
                )
            })?;
            copy_tree_contents(&repo_root.join(source), sandbox)?;
            Ok(None)
        }
    }
}

fn controlled_environment(manifest: &ConformanceManifest) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::new();
    for name in ENV_ALLOWLIST {
        if let Ok(value) = std::env::var(name) {
            environment.insert((*name).to_owned(), value);
        }
    }
    environment.extend(manifest.default_env.clone());
    environment
}

fn status_code(status: ExitStatus) -> i32 {
    status.code().unwrap_or(-1)
}

fn join_reader(
    handle: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| {
            ConformanceError::new("execution-failure", format!("{stream} reader thread panicked"))
        })?
        .map_err(|error| {
            ConformanceError::new(
                "execution-failure",
                format!("cannot read child {stream}: {error}"),
            )
        })
}

fn join_writer(handle: thread::JoinHandle<std::io::Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|_| ConformanceError::new("execution-failure", "stdin writer thread panicked"))?
        .map_err(|error| {
            ConformanceError::new("execution-failure", format!("cannot write child stdin: {error}"))
        })
}

fn run_process(
    command: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    stdin_bytes: Option<Vec<u8>>,
    timeout: Duration,
) -> Result<ProcessResult> {
    let (program, arguments) = command.split_first().ok_or_else(|| {
        ConformanceError::new("execution-failure", "process command cannot be empty")
    })?;
    let mut child = Command::new(program)
        .args(arguments)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            ConformanceError::new("execution-failure", format!("cannot spawn {program:?}: {error}"))
        })?;

    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| ConformanceError::new("execution-failure", "child stdin was not piped"))?;
    let stdin_handle = thread::spawn(move || -> std::io::Result<()> {
        if let Some(bytes) = stdin_bytes {
            child_stdin.write_all(&bytes)?;
        }
        Ok(())
    });
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| ConformanceError::new("execution-failure", "child stdout was not piped"))?;
    let stdout_handle = thread::spawn(move || {
        let mut bytes = Vec::new();
        child_stdout.read_to_end(&mut bytes)?;
        Ok(bytes)
    });
    let mut child_stderr = child
        .stderr
        .take()
        .ok_or_else(|| ConformanceError::new("execution-failure", "child stderr was not piped"))?;
    let stderr_handle = thread::spawn(move || {
        let mut bytes = Vec::new();
        child_stderr.read_to_end(&mut bytes)?;
        Ok(bytes)
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| {
            ConformanceError::new(
                "execution-failure",
                format!("cannot inspect child process: {error}"),
            )
        })? {
            break status;
        }
        if Instant::now() >= deadline {
            let kill_result = child.kill();
            let wait_result = child.wait();
            let stdin_result = join_writer(stdin_handle);
            let stdout_result = join_reader(stdout_handle, "stdout");
            let stderr_result = join_reader(stderr_handle, "stderr");
            kill_result.map_err(|error| {
                ConformanceError::new(
                    "timeout",
                    format!(
                        "command {command:?} exceeded {} seconds and could not be killed: {error}",
                        timeout.as_secs_f64()
                    ),
                )
            })?;
            wait_result.map_err(|error| {
                ConformanceError::new(
                    "timeout",
                    format!("cannot reap timed-out command {command:?}: {error}"),
                )
            })?;
            stdin_result?;
            stdout_result?;
            stderr_result?;
            return Err(ConformanceError::new(
                "timeout",
                format!("command {command:?} exceeded {} seconds", timeout.as_secs_f64()),
            ));
        }
        thread::sleep(Duration::from_millis(5));
    };

    join_writer(stdin_handle)?;
    let stdout = join_reader(stdout_handle, "stdout")?;
    let stderr = join_reader(stderr_handle, "stderr")?;
    Ok(ProcessResult {
        command: command.to_vec(),
        stdout,
        stderr,
        exit_code: status_code(status),
        tree: snapshot_tree(cwd)?,
    })
}

fn run_once(
    case: &ConformanceCase,
    manifest: &ConformanceManifest,
    executable: &Path,
    repo_root: &Path,
    timeout: Duration,
    pass: PassInput<'_>,
) -> Result<ProcessResult> {
    let sandbox = TempDir::with_prefix("flowmark-conformance-").map_err(|error| {
        ConformanceError::new(
            "invalid-sandbox",
            format!("cannot create conformance sandbox: {error}"),
        )
    })?;
    let stdin = match pass {
        PassInput::First => materialize_case(case, repo_root, sandbox.path(), false, None, None)?,
        PassInput::Second { stdout, tree } => {
            materialize_case(case, repo_root, sandbox.path(), true, Some(stdout), Some(tree))?
        }
    };
    let mut command = Vec::with_capacity(case.args.len() + 1);
    command.push(executable.to_string_lossy().into_owned());
    command.extend(case.args.iter().cloned());
    run_process(&command, sandbox.path(), &controlled_environment(manifest), stdin, timeout)
        .map_err(|error| {
            ConformanceError::new(error.code, format!("case {:?}; {}", case.id, error.message))
        })
}

fn path_bytes(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|error| {
        ConformanceError::new(
            "comparison-failure",
            format!("cannot read expectation {}: {error}", path.display()),
        )
    })
}

fn escaped_window(bytes: &[u8], center: usize) -> String {
    let start = center.saturating_sub(32);
    let end = bytes.len().min(center.saturating_add(96));
    bytes[start..end].iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ")
}

#[allow(clippy::naive_bytecount)]
fn bounded_diff(expected: &[u8], actual: &[u8]) -> String {
    let first_difference = expected
        .iter()
        .zip(actual)
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    match (std::str::from_utf8(expected), std::str::from_utf8(actual)) {
        (Ok(expected_text), Ok(actual_text)) => {
            let expected_lines: Vec<&str> = expected_text.split_inclusive('\n').collect();
            let actual_lines: Vec<&str> = actual_text.split_inclusive('\n').collect();
            let line = expected[..first_difference.min(expected.len())]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count();
            let start = line.saturating_sub(2);
            let end = (line + 3).max(start + 1);
            let mut rendered = format!(
                "first byte difference at {first_difference}; expected {} bytes, actual {} bytes\n--- expected\n+++ actual\n",
                expected.len(),
                actual.len()
            );
            let context_end = end.min(expected_lines.len().max(actual_lines.len()));
            for index in start..context_end {
                match (expected_lines.get(index), actual_lines.get(index)) {
                    (Some(left), Some(right)) if left == right => {
                        rendered.push(' ');
                        rendered.push_str(left);
                    }
                    (Some(left), Some(right)) => {
                        rendered.push('-');
                        rendered.push_str(left);
                        rendered.push('+');
                        rendered.push_str(right);
                    }
                    (Some(left), None) => {
                        rendered.push('-');
                        rendered.push_str(left);
                    }
                    (None, Some(right)) => {
                        rendered.push('+');
                        rendered.push_str(right);
                    }
                    (None, None) => {}
                }
            }
            bounded_text(rendered)
        }
        _ => bounded_text(format!(
            "binary difference at byte {first_difference}; expected {} bytes, actual {} bytes\nexpected window: {}\nactual window:   {}",
            expected.len(),
            actual.len(),
            escaped_window(expected, first_difference),
            escaped_window(actual, first_difference)
        )),
    }
}

fn failure(
    code: &'static str,
    case: &ConformanceCase,
    result: &ProcessResult,
    detail: impl Into<String>,
) -> ConformanceError {
    ConformanceError::new(
        code,
        format!("case {:?}; command {:?}\n{}", case.id, result.command, detail.into()),
    )
}

fn compare_result(case: &ConformanceCase, result: &ProcessResult, repo_root: &Path) -> Result<()> {
    if result.exit_code != case.expected_exit {
        return Err(failure(
            "exit-mismatch",
            case,
            result,
            format!("expected exit {}, got {}", case.expected_exit, result.exit_code),
        ));
    }
    let expected_stdout = path_bytes(&repo_root.join(&case.expected_stdout))?;
    if result.stdout != expected_stdout {
        return Err(failure(
            "stdout-mismatch",
            case,
            result,
            bounded_diff(&expected_stdout, &result.stdout),
        ));
    }
    let expected_stderr = path_bytes(&repo_root.join(&case.expected_stderr))?;
    if result.stderr != expected_stderr {
        return Err(failure(
            "stderr-mismatch",
            case,
            result,
            bounded_diff(&expected_stderr, &result.stderr),
        ));
    }
    let expected_tree = match case.kind {
        CaseKind::Stdin => Vec::new(),
        CaseKind::Files => {
            snapshot_tree(&repo_root.join(case.after_tree.as_ref().ok_or_else(|| {
                ConformanceError::new(
                    "invalid-kind-fields",
                    format!("case {:?} has no after tree", case.id),
                )
            })?))?
        }
    };
    if result.tree != expected_tree {
        let expected_paths: BTreeMap<&Path, &[u8]> = expected_tree
            .iter()
            .map(|snapshot| (snapshot.path.as_path(), snapshot.content.as_slice()))
            .collect();
        let actual_paths: BTreeMap<&Path, &[u8]> = result
            .tree
            .iter()
            .map(|snapshot| (snapshot.path.as_path(), snapshot.content.as_slice()))
            .collect();
        let missing: Vec<String> = expected_paths
            .keys()
            .filter(|path| !actual_paths.contains_key(**path))
            .map(|path| path.display().to_string())
            .collect();
        let extra: Vec<String> = actual_paths
            .keys()
            .filter(|path| !expected_paths.contains_key(**path))
            .map(|path| path.display().to_string())
            .collect();
        let changed: Vec<String> = expected_paths
            .iter()
            .filter(|(path, bytes)| actual_paths.get(**path).is_some_and(|actual| actual != *bytes))
            .map(|(path, _)| path.display().to_string())
            .collect();
        let mut detail = format!("missing={missing:?}; extra={extra:?}; changed={changed:?}");
        if let Some(path) = changed.first() {
            let path = Path::new(path);
            if let (Some(expected), Some(actual)) =
                (expected_paths.get(path), actual_paths.get(path))
            {
                detail.push('\n');
                detail.push_str(&bounded_diff(expected, actual));
            }
        }
        return Err(failure("file-tree-mismatch", case, result, detail));
    }
    Ok(())
}

pub fn run_case(
    case: &ConformanceCase,
    manifest: &ConformanceManifest,
    executable: &Path,
    repo_root: &Path,
    timeout: Duration,
) -> Result<usize> {
    if !executable.is_file() {
        return Err(ConformanceError::new(
            "invalid-executable",
            format!("not executable: {}", executable.display()),
        ));
    }
    if timeout.is_zero() {
        return Err(ConformanceError::new("invalid-timeout", "timeout must be positive"));
    }
    let first = run_once(case, manifest, executable, repo_root, timeout, PassInput::First)?;
    compare_result(case, &first, repo_root)?;
    if !case.idempotent {
        return Ok(1);
    }
    let second = run_once(
        case,
        manifest,
        executable,
        repo_root,
        timeout,
        PassInput::Second { stdout: &first.stdout, tree: &first.tree },
    )?;
    compare_result(case, &second, repo_root)?;
    Ok(2)
}

#[derive(Clone, Debug)]
pub struct KnownDivergence {
    pub case_id: String,
    pub tracker: String,
    pub reason: String,
}

pub fn load_known_divergences(path: &Path) -> Result<Vec<KnownDivergence>> {
    let text = fs::read_to_string(path).map_err(|error| {
        ConformanceError::new(
            "invalid-divergence-ledger",
            format!("cannot load {}: {error}", path.display()),
        )
    })?;
    let value = toml::from_str::<Value>(&text).map_err(|error| {
        ConformanceError::new(
            "invalid-divergence-ledger",
            format!("cannot load {}: {error}", path.display()),
        )
    })?;
    let root = table(&value, "divergence ledger")?;
    reject_unknown_fields(root, &["divergence"], "divergence ledger")?;
    let entries = match root.get("divergence") {
        None => Vec::new(),
        Some(value) => value.as_array().cloned().ok_or_else(|| {
            ConformanceError::new(
                "invalid-divergence-ledger",
                "divergence must be an array of tables",
            )
        })?,
    };
    let mut divergences = Vec::with_capacity(entries.len());
    let mut seen = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let location = format!("divergence[{index}]");
        let entry = table(entry, &location)?;
        reject_unknown_fields(entry, &["case_id", "tracker", "reason"], &location)?;
        let case_id = nonempty_string(entry, "case_id", &location)?;
        let tracker = nonempty_string(entry, "tracker", &location)?;
        let reason = nonempty_string(entry, "reason", &location)?;
        if !seen.insert(case_id.clone()) {
            return Err(ConformanceError::new(
                "duplicate-divergence",
                format!("duplicate divergence for case {case_id:?}"),
            ));
        }
        divergences.push(KnownDivergence { case_id, tracker, reason });
    }
    Ok(divergences)
}

pub fn upstream_root(project_root: &Path) -> PathBuf {
    std::env::var_os("FLOWMARK_UPSTREAM_ROOT")
        .map_or_else(|| project_root.join("repos/flowmark"), PathBuf::from)
}
