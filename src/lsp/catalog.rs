//! Finite acquisition metadata and local executable observations. The catalog
//! never owns language identities, settings, installation facts or processes.
mod inventory;
pub use inventory::{inventory, with_installations};

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Launch {
    pub id: &'static str,
    pub languages: &'static [&'static str],
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub root_markers: &'static [&'static str],
}

/// Read-only metadata. Availability of a prerequisite or executable is not
/// installation verification, protocol readiness or evidence of semantic quality.
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub id: &'static str,
    pub name: &'static str,
    pub launches: &'static [Launch],
    pub prerequisites: &'static [&'static str],
    pub acquisition: &'static str,
    pub integrity: &'static str,
    pub source: &'static str,
}

impl Recipe {
    pub fn launch(&self, language: &str) -> Option<&Launch> {
        self.launches
            .iter()
            .find(|p| p.languages.contains(&language))
    }

    pub fn supports(&self, os: &str, arch: &str) -> bool {
        os == "linux" && arch == "x86_64"
    }
}

pub const RECIPES: [Recipe; 8] = [
    Recipe {
        id: "rust-analyzer",
        name: "Rust Analyzer",
        launches: &[Launch {
            id: "rust-analyzer",
            languages: &["rust"],
            command: "rust-analyzer",
            args: &[],
            root_markers: &["Cargo.toml"],
        }],
        prerequisites: &["curl", "gzip", "sha256sum"],
        acquisition: "official release gzip",
        integrity: "release SHA-256",
        source: "https://github.com/rust-lang/rust-analyzer",
    },
    Recipe {
        id: "gopls",
        name: "Go language server",
        launches: &[Launch {
            id: "gopls",
            languages: &["go"],
            command: "gopls",
            args: &[],
            root_markers: &["go.work", "go.mod"],
        }],
        prerequisites: &["go"],
        acquisition: "exact Go module; local toolchain",
        integrity: "module provenance",
        source: "https://go.dev/gopls/",
    },
    Recipe {
        id: "typescript-language-server",
        name: "TypeScript and JavaScript",
        launches: &[Launch {
            id: "typescript-language-server",
            languages: &["typescript", "javascript", "tsx"],
            command: "typescript-language-server",
            args: &["--stdio"],
            root_markers: &["package.json"],
        }],
        prerequisites: &["node", "npm"],
        acquisition: "isolated npm prefix; TypeScript 6",
        integrity: "package lock; scripts disabled",
        source: "https://github.com/typescript-language-server/typescript-language-server",
    },
    Recipe {
        id: "vscode-langservers-extracted",
        name: "HTML, CSS and JSON",
        launches: &[
            Launch {
                id: "vscode-html-language-server",
                languages: &["html"],
                command: "vscode-html-language-server",
                args: &["--stdio"],
                root_markers: &["package.json"],
            },
            Launch {
                id: "vscode-css-language-server",
                languages: &["css"],
                command: "vscode-css-language-server",
                args: &["--stdio"],
                root_markers: &["package.json"],
            },
            Launch {
                id: "vscode-json-language-server",
                languages: &["json"],
                command: "vscode-json-language-server",
                args: &["--stdio"],
                root_markers: &["package.json"],
            },
        ],
        prerequisites: &["node", "npm"],
        acquisition: "shared isolated npm prefix",
        integrity: "package lock; scripts disabled",
        source: "https://github.com/hrsh7th/vscode-langservers-extracted",
    },
    Recipe {
        id: "marksman",
        name: "Markdown",
        launches: &[Launch {
            id: "marksman",
            languages: &["markdown"],
            command: "marksman",
            args: &["server"],
            root_markers: &[".marksman.toml", ".git"],
        }],
        prerequisites: &["curl", "sha256sum"],
        acquisition: "official native release",
        integrity: "release SHA-256",
        source: "https://github.com/artempyanykh/marksman",
    },
    Recipe {
        id: "ruby-lsp",
        name: "Ruby",
        launches: &[Launch {
            id: "ruby-lsp",
            languages: &["ruby"],
            command: "ruby-lsp",
            args: &[],
            root_markers: &["Gemfile"],
        }],
        prerequisites: &["ruby", "gem"],
        acquisition: "isolated gem home",
        integrity: "exact gem and dependency resolution",
        source: "https://shopify.github.io/ruby-lsp/",
    },
    Recipe {
        id: "python-lsp-server",
        name: "Python",
        launches: &[Launch {
            id: "pylsp",
            languages: &["python"],
            command: "pylsp",
            args: &[],
            root_markers: &["pyproject.toml", "setup.py"],
        }],
        prerequisites: &["python3"],
        acquisition: "fixed-path venv; pip; no extras",
        integrity: "exact package provenance",
        source: "https://github.com/python-lsp/python-lsp-server",
    },
    Recipe {
        id: "ols",
        name: "Odin",
        launches: &[Launch {
            id: "ols",
            languages: &["odin"],
            command: "ols",
            args: &[],
            root_markers: &["ols.json"],
        }],
        prerequisites: &["git", "odin", "bash"],
        acquisition: "immutable Git revision and local build",
        integrity: "revision and toolchain identity; builtin paths",
        source: "https://github.com/DanielGavin/ols",
    },
];

pub fn launch(id: &str, language: &str) -> Option<&'static Launch> {
    RECIPES
        .iter()
        .flat_map(|r| r.launches)
        .find(|p| p.id == id && p.languages.contains(&language))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Executable {
    Available(PathBuf),
    Missing,
    UnknownRoot,
    SearchLimit,
}

/// Inspect at most 128 PATH entries. Relative commands/entries need an actual
/// client working root and are never resolved against an incidental editor CWD.
/// No executable is run, even when a file exists with execute permission.
pub fn executable(command: &str, search_path: Option<&OsStr>) -> Executable {
    let command = Path::new(command);
    if command.is_absolute() {
        return if executable_file(command) {
            Executable::Available(command.to_owned())
        } else {
            Executable::Missing
        };
    }
    if command.components().count() != 1 {
        return Executable::UnknownRoot;
    }
    let Some(search_path) = search_path else {
        return Executable::Missing;
    };
    let mut relative = false;
    for (index, directory) in std::env::split_paths(search_path).enumerate() {
        if index == 128 {
            return Executable::SearchLimit;
        }
        if !directory.is_absolute() {
            relative = true;
            continue;
        }
        let candidate = directory.join(command);
        if executable_file(&candidate) {
            return Executable::Available(candidate);
        }
    }
    if relative {
        Executable::UnknownRoot
    } else {
        Executable::Missing
    }
}

fn executable_file(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}
