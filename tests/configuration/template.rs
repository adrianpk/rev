use std::path::Path;

use rev::config::{default_template, parse};

// The shipped bytes must remain complete UTF-8 TOML within the existing root
// source limit, with one stable borrowed resource and no runtime file lookup.
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
