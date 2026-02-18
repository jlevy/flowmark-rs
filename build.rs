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

    println!("cargo::rerun-if-changed=Cargo.toml");
}
