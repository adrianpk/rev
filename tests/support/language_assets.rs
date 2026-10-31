//! Owned real libraries are compiled before any editor instance is constructed.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, Weak};

use crate::fixture;

static COMPILED: Mutex<Weak<Compiled>> = Mutex::new(Weak::new());
pub struct Compiled {
    _owner: fixture::Fixture,
    paths: std::collections::BTreeMap<&'static str, PathBuf>,
}

impl Compiled {
    pub fn parser(&self, name: &str) -> &PathBuf {
        self.paths.get(name).expect("owned grammar fixture")
    }
}

pub fn libraries() -> Arc<Compiled> {
    let mut shared = COMPILED.lock().unwrap();
    if let Some(compiled) = shared.upgrade() {
        return compiled;
    }
    let owner = fixture::Fixture::new();
    let rust = package("tree-sitter-rust");
    let python = package("tree-sitter-python");
    let rust_lib = owner.0.join("rust-v1.so");
    let python_lib = owner.0.join("python-v1.so");
    let incompatible = owner.0.join("incompatible-v1.so");
    compile(&rust, &rust.join("parser.c"), &rust_lib);
    compile(&python, &python.join("parser.c"), &python_lib);
    let parser = fs::read_to_string(rust.join("parser.c")).unwrap();
    let bad = parser.replace(
        "#define LANGUAGE_VERSION 15",
        "#define LANGUAGE_VERSION 999",
    );
    assert_ne!(
        parser, bad,
        "fixture must change only the generated ABI constant"
    );
    let bad_parser = owner.write("incompatible-parser.c", &bad);
    compile(&rust, &bad_parser, &incompatible);
    let compiled = Arc::new(Compiled {
        _owner: owner,
        paths: std::collections::BTreeMap::from([
            ("rust", rust_lib),
            ("python", python_lib),
            ("incompatible", incompatible),
        ]),
    });
    *shared = Arc::downgrade(&compiled);
    compiled
}

fn package(name: &str) -> PathBuf {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--offline", "--format-version", "1"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let package = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == name)
        .unwrap();
    Path::new(package["manifest_path"].as_str().unwrap())
        .parent()
        .unwrap()
        .join("src")
}

fn compile(source: &Path, parser: &Path, output: &Path) {
    let result = Command::new("cc")
        .args(["-shared", "-fPIC", "-O0", "-I"])
        .arg(source)
        .arg(parser)
        .arg(source.join("scanner.c"))
        .arg("-o")
        .arg(output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

pub fn registration(parser: &Path, symbol: &str, highlights: &Path, completions: &Path) -> String {
    format!(
        "parser = {parser:?}\nsymbol = {symbol:?}\nhighlights = {highlights:?}\ncompletions = {completions:?}\n"
    )
}

pub fn settle(session: &mut rev::session::Session) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        session.expire_input(std::time::Instant::now());
        if session.remaining_input(std::time::Instant::now()).is_none() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "owned language/completion work did not settle"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
