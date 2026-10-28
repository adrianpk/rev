use std::collections::BTreeMap;
use std::path::Path;

use rev::config::{Document, Grammar, IndentStyle, Keymaps, Settings, parse, prepare};
use rev::input::keymap::Context;

use super::fixture::Fixture;

const SAMPLE: &str = rev::config::default_template();

fn document() -> Document {
    parse(Path::new("reference/config.toml"), SAMPLE.as_bytes()).unwrap()
}

fn declared(keys: &Keymaps, context: Context) -> &BTreeMap<String, String> {
    keys.map(context)
}

fn example(heading: &str) -> String {
    let text = SAMPLE
        .lines()
        .skip_while(|line| !line.starts_with(heading))
        .skip(1)
        .map_while(|line| line.strip_prefix("# "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!text.is_empty(), "missing example: {heading}");
    text
}

// Packaged and on-disk bytes must agree, then resolve through the real loader
// to compiled scalar, format and palette defaults independently of examples.
#[test]
fn sample_defaults() {
    let reference =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/reference/config.toml");
    assert_eq!(SAMPLE.as_bytes(), std::fs::read(reference).unwrap());
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", SAMPLE);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut actual = startup.snapshot.settings().clone();
    actual.keys = Keymaps::default();
    assert_eq!(actual, Settings::default());
    let document = document();
    assert_eq!(document.version, Some(1));
    assert_eq!(document.include, Some(Vec::new()));
    assert_eq!(std::fs::read_to_string(root).unwrap(), SAMPLE);
    assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
}

// Every context's active sample entries and effective targets must agree with
// compiled defaults. Comparing only inherited lookup would miss omitted entries.
#[test]
fn sample_maps() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", SAMPLE);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let source = document();
    let defaults = rev::config::Snapshot::default();
    for context in Context::ALL {
        let expected: BTreeMap<_, _> = context
            .local_defaults()
            .iter()
            .map(|(key, name)| (key.to_string(), name.to_string()))
            .collect();
        assert_eq!(declared(&source.keys, context), &expected, "{context:?}");
        let actual: Vec<_> = startup.snapshot.keymaps().map(context).bindings().collect();
        let compiled: Vec<_> = defaults.keymaps().map(context).bindings().collect();
        assert_eq!(actual, compiled, "{context:?}");
    }
}

// The commented include array must load real owned files in its declared order;
// it cannot be a placeholder for unsupported implicit or remote configuration.
#[test]
fn optional_includes() {
    let fixture = Fixture::new();
    fixture.write("editing.toml", "editor.indent_width = 2");
    fixture.write("colors.toml", "editor.indent_width = 3");
    let text = example("# Optional includes:");
    let root = fixture.write("config.toml", &text);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(startup.snapshot.settings().editor.indent_width, 3);
    assert_eq!(std::fs::read_to_string(root).unwrap(), text);
    assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 3);
}

// Uncommented language leaves must select only the advertised buffer policy
// while global defaults and the fixed grammar catalog remain unchanged.
#[test]
fn optional_language() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", &example("# Optional language:"));
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let policy = startup.snapshot.editor("go");
    assert_eq!(policy.indent_style, IndentStyle::Tabs);
    assert_eq!((policy.indent_width, policy.tab_width), (4, 8));
    assert_eq!(startup.snapshot.grammar("go"), Grammar::Plain);
    assert_eq!(
        startup.snapshot.settings().editor,
        Settings::default().editor
    );
}

// The commented literal/suffix association entries must identify their language
// without requiring a grammar install or changing unrelated built-in recognition.
#[test]
fn optional_associations() {
    let fixture = Fixture::new();
    let text = format!(
        "[files.associations]\n{}",
        example("# Optional associations:")
    );
    let root = fixture.write("config.toml", &text);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(startup.snapshot.language(Path::new("unit.go")), "go");
    assert_eq!(startup.snapshot.language(Path::new("Buildfile")), "go");
    assert_eq!(startup.snapshot.language(Path::new("lib.rs")), "rust");
    assert_eq!(startup.snapshot.grammar("go"), Grammar::Plain);
}
