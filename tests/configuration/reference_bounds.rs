use std::path::Path;

use rev::config::parse;

const REFERENCE: &str = include_str!("../../docs/reference/defaults.md");

fn rows(prefix: &str) -> Vec<(&str, &str)> {
    REFERENCE
        .lines()
        .filter_map(|line| row(line, prefix))
        .collect()
}

fn row<'a>(line: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    let cells: Vec<_> = line.split('|').map(str::trim).collect();
    let key = cells.get(1)?.strip_prefix('`')?.strip_suffix('`')?;
    let kind = cells.get(2)?.strip_prefix(prefix)?;
    Some((key, kind))
}

fn admitted(key: &str, value: &str) -> bool {
    let key = key.replace("<id>", "go");
    parse(
        Path::new("reference.toml"),
        format!("{key} = {value}").as_bytes(),
    )
    .is_ok()
}

// Drive the actual parser from every published integer interval. Both endpoints
// must work and adjacent out-of-range values must reject, including language leaves.
#[test]
fn integer_bounds() {
    let rows = rows("Integer ");
    assert!(!rows.is_empty());
    for (key, range) in rows {
        let (lower, upper) = range.split_once("..=").unwrap();
        let lower: i64 = lower.parse().unwrap();
        let upper: i64 = upper.parse().unwrap();
        for (value, expected) in [
            (lower, true),
            (upper, true),
            (lower - 1, false),
            (upper + 1, false),
        ] {
            assert_eq!(admitted(key, &value.to_string()), expected, "{key}={value}");
        }
    }
}

// Published layout shares are finite floating-point intervals, with inclusive
// endpoints. Excess, infinity and NaN cannot be admitted as valid preferences.
#[test]
fn float_bounds() {
    let rows = rows("Finite float ");
    assert!(!rows.is_empty());
    for (key, range) in rows {
        let (lower, upper) = range.split_once("..=").unwrap();
        let lower: f64 = lower.parse().unwrap();
        let upper: f64 = upper.parse().unwrap();
        for (value, expected) in [
            (format!("{lower:?}"), true),
            (format!("{upper:?}"), true),
            (format!("{:?}", lower - 0.01), false),
            (format!("{:?}", upper + 0.01), false),
            ("inf".into(), false),
            ("-inf".into(), false),
            ("nan".into(), false),
        ] {
            assert_eq!(admitted(key, &value), expected, "{key}={value}");
        }
    }
}

// Every enum spelling advertised by the catalog must be accepted verbatim;
// arbitrary values reject instead of silently selecting a compiled default.
#[test]
fn enum_values() {
    let rows = rows("String: ");
    assert!(!rows.is_empty());
    for (key, choices) in rows {
        let values = choices.split('"').skip(1).step_by(2);
        for value in values {
            assert!(admitted(key, &format!("\"{value}\"")), "{key}={value}");
        }
        assert!(!admitted(key, "\"unsupported-reference-value\""), "{key}");
    }
}
