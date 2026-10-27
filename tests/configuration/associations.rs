use std::path::Path;

use rev::config::{Snapshot, load, resolve};

use crate::fixture::Fixture;

// Every accepted built-in maps a basename to an editing identity independently
// of grammar availability. Case, unknown suffixes and parent names stay literal.
#[test]
fn builtins() {
    let snapshot = Snapshot::default();
    let cases = [
        ("main.rs", "rust"),
        ("main.go", "go"),
        ("main.rb", "ruby"),
        ("Gemfile", "ruby"),
        ("Rakefile", "ruby"),
        ("main.c", "c"),
        ("main.h", "c"),
        ("main.cc", "cpp"),
        ("main.cpp", "cpp"),
        ("main.cxx", "cpp"),
        ("main.hpp", "cpp"),
        ("main.py", "python"),
        ("main.js", "javascript"),
        ("main.mjs", "javascript"),
        ("main.cjs", "javascript"),
        ("main.ts", "typescript"),
        ("main.tsx", "typescript"),
        ("main.json", "json"),
        ("main.toml", "toml"),
        ("Cargo.toml", "toml"),
        ("main.md", "markdown"),
        ("main.sh", "shell"),
        ("Makefile", "make"),
        ("MAIN.RS", "plain"),
        ("gemfile", "plain"),
        ("unrecognized.xyz", "plain"),
        ("Makefile/notes", "plain"),
        ("/not-created/main.go", "go"),
        ("", "plain"),
        ("/", "plain"),
    ];
    for (path, expected) in cases {
        assert_eq!(snapshot.language(Path::new(path)), expected, "{path}");
    }
}

// User rules form the higher-priority class even when a built-in is exact.
// Within it, exact names and the longest literal suffix determine the result.
#[test]
fn user_precedence() {
    let fixture = Fixture::new();
    let root = fixture.write(
        "config.toml",
        "[files.associations]\n'*.h' = 'cpp'\n'special.h' = 'c'\n'*.ts' = 'short'\n'*.test.ts' = 'test-typescript'\n'*.deep.test.ts' = 'deep'\n'*.toml' = 'custom'\n'Makefile' = 'build'",
    );
    let snapshot = resolve(&load(&root).unwrap()).unwrap();
    let cases = [
        ("other.h", "cpp"),
        ("special.h", "c"),
        ("SPECIAL.h", "cpp"),
        ("file.ts", "short"),
        ("file.test.ts", "test-typescript"),
        ("file.deep.test.ts", "deep"),
        ("file.test.tsx", "typescript"),
        ("Cargo.toml", "custom"),
        ("Makefile", "build"),
        ("main.rs", "rust"),
    ];
    for (path, expected) in cases {
        assert_eq!(snapshot.language(Path::new(path)), expected, "{path}");
    }
}
