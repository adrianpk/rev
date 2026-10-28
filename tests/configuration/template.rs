use std::path::Path;

use rev::config::{default_template, parse};

// The shipped bytes must remain complete UTF-8 TOML within the existing root
// source limit, without runtime file lookup.
#[test]
fn packaged_source() {
    let text = default_template();
    assert!(!text.is_empty());
    assert!(text.len() <= 256 * 1024);
    assert_eq!(
        text.as_bytes(),
        include_bytes!("../../docs/reference/config.toml")
    );
    assert!(parse(Path::new("packaged/config.toml"), text.as_bytes()).is_ok());
}

// Defaulting must not hide omitted colors/controls or enable optional examples.
// The normal typed equality and context-map tests resolve these same asset bytes.
#[test]
fn explicit_defaults() {
    let text = default_template();
    let raw: toml::Table = toml::from_str(text).unwrap();
    let palette = raw["palette"].as_table().unwrap();
    let mut roles = 0;
    for table in [
        palette,
        palette["syntax"].as_table().unwrap(),
        palette["gutter"].as_table().unwrap(),
    ] {
        for color in table.values().filter_map(toml::Value::as_str) {
            assert!(rev::config::Rgb::parse(color).is_some());
            roles += 1;
        }
    }
    assert_eq!(roles, 29);
    assert_eq!(raw["gutter"].as_table().unwrap().len(), 7);
    assert_eq!(raw["mouse"].as_table().unwrap().len(), 2);
    let document = parse(Path::new("packaged/config.toml"), text.as_bytes()).unwrap();
    assert_eq!(document.include, Some(Vec::new()));
    assert!(document.language.is_empty());
    assert!(document.files.associations.is_empty());
}
