use rev::config::load;

use crate::fixture::Fixture;

// Each included source precedes its declaring parent; siblings retain declared
// order and relative children use their own declaring directory.
#[test]
fn declared_order() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['parts/first.toml', 'last.toml']");
    fixture.write(
        "parts/first.toml",
        "include = ['leaf.toml']\neditor.indent_width = 2",
    );
    fixture.write("parts/leaf.toml", "editor.tab_width = 8");
    fixture.write("last.toml", "syntax.enabled = false");
    fixture.write("unused.toml", "malformed unrelated file");
    let graph = load(&root).unwrap();
    let names: Vec<_> = graph
        .order
        .iter()
        .map(|id| {
            graph.sources[*id]
                .path
                .strip_prefix(&fixture.0)
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        names
            .iter()
            .map(|path| path.to_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "parts/leaf.toml",
            "parts/first.toml",
            "last.toml",
            "config.toml"
        ]
    );
    assert_eq!(
        graph
            .sources
            .iter()
            .map(|source| source.parent)
            .collect::<Vec<_>>(),
        [None, Some(0), Some(1), Some(0)]
    );
    assert_eq!(graph.sources[3].declaration, 1);
    assert_eq!(graph.sources[3].document.syntax.enabled, Some(false));
}

// Missing optional roots alone yield the empty input graph for defaults. A
// missing explicit source rejects the graph instead of returning accepted siblings.
#[test]
fn optional_root() {
    let fixture = Fixture::new();
    let root = fixture.0.join("settings/config.toml");
    let graph = load(&root).unwrap();
    assert!(graph.sources.is_empty());
    assert!(!root.parent().unwrap().exists());
    let root = fixture.write("config.toml", "include = ['missing.toml']");
    let error = load(&root).unwrap_err();
    assert_eq!(error.source, root);
    assert_eq!(error.key, ["include"]);
    assert!(error.message.contains("missing.toml"));
}

// A repeated shared leaf is an ordinary occurrence after its earlier branch
// has left the active stack; a global visited-file set would reject this graph.
#[test]
fn shared_leaves() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['left.toml', 'right.toml']");
    fixture.write("left.toml", "include = ['shared.toml']");
    fixture.write("right.toml", "include = ['shared.toml']");
    fixture.write("shared.toml", "editor.indent_width = 2");
    let graph = load(&root).unwrap();
    assert_eq!(graph.sources.len(), 5);
    assert_eq!(graph.order, [2, 1, 4, 3, 0]);
    assert_eq!(
        graph.sources[2].canonical_path,
        graph.sources[4].canonical_path
    );
    assert_eq!(graph.sources[2].parent, Some(1));
    assert_eq!(graph.sources[4].parent, Some(3));
}
