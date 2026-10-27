use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::symlink;
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

// Identical rules merge by source order, while distinct overlapping rules keep
// their specificity even when the shorter rule comes from the root layer.
#[test]
fn layered_patterns() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['first.toml', 'last.toml']\n[files.associations]\n'*.h' = 'header'\n'*.ts' = 'root'\n'*.toml' = 'custom'");
    fixture.write("first.toml", "[files.associations]\n'*.h' = 'cpp'\n'*.ts' = 'earlier'\n'*.test.ts' = 'test-typescript'\n'Gemfile' = 'custom-gems'");
    let last = fixture.write("last.toml", "[files.associations]\n'*.h' = 'c'\n'special.h' = 'special'\n'*.test.ts' = 'checked-typescript'");
    let snapshot = resolve(&load(&root).unwrap()).unwrap();
    let cases = [
        ("other.h", "header"),
        ("special.h", "special"),
        ("file.ts", "root"),
        ("file.test.ts", "checked-typescript"),
        ("Gemfile", "custom-gems"),
        ("Cargo.toml", "custom"),
    ];
    for (path, expected) in cases {
        assert_eq!(snapshot.language(Path::new(path)), expected, "{path}");
    }
    assert_eq!(
        snapshot.origin(&["files", "associations", "*.h"]),
        Some(root.as_path())
    );
    assert_eq!(
        snapshot.origin(&["files", "associations", "*.test.ts"]),
        Some(last.as_path())
    );
}

// Matching compares OS bytes without lossy labels or wildcard expansion.
// Unicode suffix length, literal punctuation and invalid UTF-8 prefixes retain
// exact identity; only the admitted leading star has wildcard meaning.
#[test]
fn literal_names() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "[files.associations]\n'*.rs' = 'short'\n'*.界.rs' = 'unicode'\n'*.multi.界.rs' = 'compound'\n'file+(one).rs' = 'literal'\n'raw-�.rs' = 'replacement'");
    let snapshot = resolve(&load(&root).unwrap()).unwrap();
    let cases = [
        ("file.界.rs", "unicode"),
        ("file.multi.界.rs", "compound"),
        ("file+(one).rs", "literal"),
        ("file+one.rs", "short"),
        ("raw-�.rs", "replacement"),
        ("file.RS", "plain"),
        (".rs", "short"),
        ("file.rs.backup", "plain"),
    ];
    for (path, expected) in cases {
        assert_eq!(snapshot.language(Path::new(path)), expected, "{path}");
    }
    let raw = OsString::from_vec(b"raw-\xff.rs".to_vec());
    assert_eq!(snapshot.language(Path::new(&raw)), "short");
}

// A directory or dangling symlink is still just a supplied name to this query.
// Malformed project-local config and nonexistent parents cannot trigger IO,
// implicit configuration discovery, validation of the target or file creation.
#[test]
fn name_only() {
    let fixture = Fixture::new();
    fixture.write("config.toml", "malformed project configuration = [");
    let directory = fixture.0.join("directory.rs");
    fs::create_dir(&directory).unwrap();
    let dangling = fixture.0.join("alias.go");
    symlink("missing.rs", &dangling).unwrap();
    let absent = fixture.0.join("absent/child.py");
    let snapshot = Snapshot::default();
    for (path, expected) in [(&directory, "rust"), (&dangling, "go"), (&absent, "python")] {
        assert_eq!(snapshot.language(path), expected);
    }
    assert!(!fixture.0.join("absent").exists());
    assert!(!fixture.0.join("missing.rs").exists());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 3);
}
