fn main() {
    // Read parity version from Cargo.toml [package.metadata.parity]
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let table: toml::Table = cargo_toml.parse().expect("Failed to parse Cargo.toml");

    if let Some(parity) = table
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("parity"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        println!("cargo::rustc-env=PARITY_VERSION={parity}");
    }

    // Canonical docs source for runtime `--docs`: Rust README.
    // The Rust README is generated from a shared docs body + Rust wrapper preface.
    // Fallback to shared docs source if README is unavailable.
    let rust_readme = std::path::Path::new("README.md");
    let shared_docs = std::path::Path::new("repos/flowmark/docs/shared/flowmark-readme-shared.md");
    let docs_source = if rust_readme.exists() { rust_readme } else { shared_docs };
    let docs_content = std::fs::read_to_string(docs_source)
        .unwrap_or_else(|e| panic!("Failed to read docs source {}: {e}", docs_source.display()));
    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let generated_docs = std::path::Path::new(&out_dir).join("flowmark_docs.md");
    std::fs::write(&generated_docs, docs_content).unwrap_or_else(|e| {
        panic!("Failed to write generated docs {}: {e}", generated_docs.display())
    });

    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=README.md");
    println!("cargo::rerun-if-changed=repos/flowmark/docs/shared/flowmark-readme-shared.md");
}
