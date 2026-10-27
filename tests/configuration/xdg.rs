use std::ffi::OsString;
use std::io;
use std::path::Path;

use rev::config::Roots;

use crate::fixture::Fixture;

fn resolve(values: &[(&str, &str)]) -> io::Result<Roots> {
    Roots::resolve(|name| {
        values
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| OsString::from(value))
    })
}

// Each XDG root has its own override and HOME fallback. Empty and relative
// values cannot redirect discovery into a working directory or another root.
#[test]
fn independent_roots() {
    let cases = [
        (
            "defaults",
            vec![("HOME", "/users/example")],
            [
                "/users/example/.config/rev",
                "/users/example/.local/share/rev",
                "/users/example/.local/state/rev",
                "/users/example/.cache/rev",
            ],
        ),
        (
            "config only",
            vec![("HOME", "/users/example"), ("XDG_CONFIG_HOME", "/settings")],
            [
                "/settings/rev",
                "/users/example/.local/share/rev",
                "/users/example/.local/state/rev",
                "/users/example/.cache/rev",
            ],
        ),
        (
            "all overrides",
            vec![
                ("XDG_CONFIG_HOME", "/settings"),
                ("XDG_DATA_HOME", "/data"),
                ("XDG_STATE_HOME", "/state"),
                ("XDG_CACHE_HOME", "/cache"),
            ],
            ["/settings/rev", "/data/rev", "/state/rev", "/cache/rev"],
        ),
        (
            "invalid overrides",
            vec![
                ("HOME", "/users/example"),
                ("XDG_CONFIG_HOME", "relative"),
                ("XDG_DATA_HOME", ""),
                ("XDG_STATE_HOME", "~/state"),
                ("XDG_CACHE_HOME", "../cache"),
            ],
            [
                "/users/example/.config/rev",
                "/users/example/.local/share/rev",
                "/users/example/.local/state/rev",
                "/users/example/.cache/rev",
            ],
        ),
    ];
    for (name, values, expected) in cases {
        let roots = resolve(&values).unwrap();
        let actual = [&roots.config, &roots.data, &roots.state, &roots.cache]
            .map(|path| path.to_str().unwrap());
        assert_eq!(actual, expected, "{name}");
        assert_eq!(
            roots.config_file(),
            roots.config.join("config.toml"),
            "{name}"
        );
    }
}

// Missing HOME is harmless when all roots are explicit. Otherwise report the
// first unresolved root precisely, including roots not owned by configuration.
#[test]
fn unresolved_roots() {
    let cases = [
        ("missing HOME", vec![], "XDG_CONFIG_HOME"),
        (
            "relative HOME",
            vec![("HOME", "relative")],
            "XDG_CONFIG_HOME",
        ),
        (
            "missing data",
            vec![("XDG_CONFIG_HOME", "/settings")],
            "XDG_DATA_HOME",
        ),
        (
            "missing state",
            vec![("XDG_CONFIG_HOME", "/settings"), ("XDG_DATA_HOME", "/data")],
            "XDG_STATE_HOME",
        ),
        (
            "missing cache",
            vec![
                ("XDG_CONFIG_HOME", "/settings"),
                ("XDG_DATA_HOME", "/data"),
                ("XDG_STATE_HOME", "/state"),
            ],
            "XDG_CACHE_HOME",
        ),
    ];
    for (name, values, variable) in cases {
        let error = resolve(&values).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput, "{name}");
        assert!(error.to_string().contains(variable), "{name}: {error}");
    }
}

// Looking up all four destinations must not create any directory or settings
// file, and OS-native path bytes do not require UTF-8 conversion.
#[test]
fn passive_locations() {
    use std::os::unix::ffi::OsStringExt;
    let fixture = Fixture::new();
    let home = fixture.0.join("home");
    let roots =
        Roots::resolve(|name| (name == "HOME").then(|| home.clone().into_os_string())).unwrap();
    assert!(!home.exists());
    assert!(!roots.config_file().exists());
    let native = OsString::from_vec(b"/settings-\xff".to_vec());
    let roots = Roots::resolve(|_| Some(native.clone())).unwrap();
    assert_eq!(roots.config, Path::new(&native).join("rev"));
}
