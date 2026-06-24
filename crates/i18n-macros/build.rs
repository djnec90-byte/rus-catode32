fn main() {
    println!("cargo:rerun-if-changed=translations");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("translations");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
