//! Embeds `content/index.yaml` and every `content/<chapter>/LNN.yaml` into
//! the app. Chunk 9 replaces this with a compressed offline pack.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let content = root.join("content");
    println!("cargo:rerun-if-changed={}", content.display());

    let mut files = Vec::new();
    for dir in std::fs::read_dir(&content).expect("content/ exists").flatten() {
        if !dir.path().is_dir() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", dir.path().display());
        for f in std::fs::read_dir(dir.path()).unwrap().flatten() {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "yaml") {
                files.push(p);
            }
        }
    }
    files.sort();

    let mut out = String::new();
    let index = content.join("index.yaml").canonicalize().unwrap();
    writeln!(out, "pub const INDEX: &str = include_str!({:?});", index.display().to_string()).unwrap();
    out.push_str("pub const LECTURES: &[&str] = &[\n");
    for f in files {
        let f = f.canonicalize().unwrap();
        writeln!(out, "    include_str!({:?}),", f.display().to_string()).unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded.rs");
    std::fs::write(dest, out).unwrap();
}
