use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::fixture::Fixture;
use rev::lsp::catalog::{Executable, RECIPES, executable, inventory, launch};

// The finite acquisition catalog has unique recipes/profiles and the declared
// runtime/launch requirements. It never constrains arbitrary language routing.
#[test]
fn recipe_contracts() {
    let mut recipes = BTreeSet::new();
    let mut profiles = BTreeSet::new();
    for recipe in RECIPES {
        assert!(recipes.insert(recipe.id));
        assert!(recipe.source.starts_with("https://"));
        assert!(!recipe.acquisition.is_empty());
        assert!(!recipe.integrity.is_empty());
        assert!(!recipe.prerequisites.is_empty());
        for profile in recipe.launches {
            assert!(profiles.insert(profile.id));
            for language in profile.languages {
                assert!(launch(profile.id, language).is_some());
            }
        }
        assert!(recipe.supports("linux", "x86_64"));
        assert!(!recipe.supports("unknown", "unknown"));
    }
    assert_eq!(recipes.len(), 8);
    assert_eq!(profiles.len(), 10);
    assert!(launch("ols", "odin").is_some());
    assert!(launch("ols", "org").is_none());
    assert!(
        launch("vscode-css-language-server", "css")
            .unwrap()
            .args
            .contains(&"--stdio")
    );
}

// Existence checks are local and do not execute tools. Relative paths cannot
// borrow the editor's incidental CWD; search and output work remain bounded.
#[test]
fn executable_observations() {
    let f = Fixture::new();
    let command = f.write("bin/server", "#!/bin/sh\nexit 99\n");
    fs::set_permissions(&command, fs::Permissions::from_mode(0o700)).unwrap();
    let path = f.0.join("bin").into_os_string();
    assert_eq!(
        executable("server", Some(&path)),
        Executable::Available(command.clone())
    );
    assert_eq!(
        executable(command.to_str().unwrap(), None),
        Executable::Available(command.clone())
    );
    fs::set_permissions(&command, fs::Permissions::from_mode(0o600)).unwrap();
    for (name, command, path, expected) in [
        (
            "no permission",
            "server",
            Some(path.as_os_str()),
            Executable::Missing,
        ),
        (
            "absent",
            "absent",
            Some(path.as_os_str()),
            Executable::Missing,
        ),
        ("no PATH", "server", None, Executable::Missing),
        (
            "relative command",
            "./server",
            None,
            Executable::UnknownRoot,
        ),
        (
            "relative PATH",
            "server",
            Some(OsStr::new(".")),
            Executable::UnknownRoot,
        ),
    ] {
        assert_eq!(executable(command, path), expected, "{name}");
    }
    let excessive = std::env::join_paths(std::iter::repeat_n(f.0.join("missing"), 129)).unwrap();
    assert_eq!(
        executable("server", Some(&excessive)),
        Executable::SearchLimit
    );
}

// Missing tools, unsupported recipe platforms and malformed installation facts
// are observations only. The admitted arbitrary-language profile stays listed.
#[test]
fn inventory_independence() {
    let f = Fixture::new();
    f.write("data/lsp/installations.toml", "schema = 'broken'\n");
    let root = f.write("config.toml", "[lsp.preferred]\ngleam = 'custom'\n[lsp.servers.custom]\nlanguages = ['gleam']\ncommand = 'custom-server'\n[files.associations]\n'*.gleam' = 'gleam'\n");
    let startup = rev::config::prepare(&root);
    assert!(startup.diagnostic.is_none());
    let native = inventory(&startup.snapshot, None, "linux", "x86_64");
    assert!(native.contains("gleam | custom | external/configured | unknown | missing executable"));
    assert!(native.contains("missing or unresolved prerequisite: go"));
    assert!(native.contains("Managed payload readiness/version: unverified"));
    let other = inventory(&startup.snapshot, None, "other", "other");
    assert!(other.contains("unsupported platform"));
    assert!(other.contains("gleam | custom"));
    assert_eq!(
        fs::read_to_string(f.0.join("data/lsp/installations.toml")).unwrap(),
        "schema = 'broken'\n"
    );
}
