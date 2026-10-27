use rev::config::{Grammar, IndentStyle, Snapshot, load, resolve};

use super::*;

// The visible alias selects the identity even when its canonical referent has
// a different suffix. Unknown language IDs inherit globals without a grammar.
#[test]
fn visiting_languages() {
    let fixture = Fixture::new();
    let root = fixture.file("config.toml", b"editor.tab_width = 8\nlanguage.go.indent_width = 2\n[files.associations]\n'*.h' = 'cpp'\n'*.note' = 'custom'");
    let settings = resolve(&load(&root).unwrap()).unwrap();
    let cases = [
        ("body.rs", "alias.go", "go", "Text", 2),
        ("body.txt", "alias.rs", "rust", "Rust", 4),
        ("body.h", "alias.h", "cpp", "Text", 4),
        ("other.rs", "alias.note", "custom", "Text", 4),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (target, alias, language, mode, width) in cases {
        let path = fixture.file(target, b"fn main() {}\n");
        let alias_path = fixture.0.join(alias);
        symlink(&path, &alias_path).unwrap();
        let id = buffers.visit(&alias_path).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(buffer.file().unwrap().path(), path);
        assert_eq!(buffer.file().unwrap().basename(), alias);
        assert_eq!(buffer.name(), alias);
        assert_eq!(buffer.language(&settings), language);
        let policy = settings.editor(buffer.language(&settings));
        assert_eq!(policy.indent_width, width);
        assert_eq!(policy.indent_style, IndentStyle::Spaces);
        assert_eq!(policy.tab_width, 8);
        assert_eq!(buffer.mode(), mode);
        assert_eq!(settings.grammar(language) == Grammar::Rust, mode == "Rust");
        assert_eq!(text(buffer), "fn main() {}\n");
        assert_eq!(buffers.visit(&path).unwrap(), id);
        assert_eq!(buffers.get(id).unwrap().language(&settings), language);
    }
    assert_eq!(
        buffers.get(buffers.initial()).unwrap().language(&settings),
        "plain"
    );
    assert_eq!(Snapshot::default().editor("go").indent_width, 4);
}

// Generated label suffixes and later alias visits never select another policy.
// Ordinary saves retain that first alias and do not invent a disk-change prompt.
#[test]
fn retained_alias() {
    let fixture = Fixture::new();
    let target = fixture.file("body.txt", b"fn main() {}\n");
    let alias = fixture.0.join("visible.rs");
    symlink(&target, &alias).unwrap();
    fs::create_dir(fixture.0.join("other")).unwrap();
    let other = fixture.file("other/visible.rs", b"fn other() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&alias).unwrap();
    let second = buffers.visit(&other).unwrap();
    let settings = Snapshot::default();
    assert_eq!(buffers.get(second).unwrap().name(), "visible.rs<2>");
    assert_eq!(buffers.get(second).unwrap().language(&settings), "rust");
    assert_eq!(buffers.visit(&target).unwrap(), id);
    let request = buffers.request_save(id, None).unwrap();
    let rev::buffer::SaveRequest::Ready(save) = request else {
        panic!("unchanged alias target requires no confirmation");
    };
    assert!(buffers.publish_save(save).is_complete());
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.name(), "visible.rs");
    assert_eq!(buffer.file().unwrap().basename(), "visible.rs");
    assert_eq!(buffer.language(&settings), "rust");
    assert_eq!(buffer.mode(), "Rust");
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert_eq!(fs::read(&target).unwrap(), b"fn main() {}\n");
    assert!(fs::symlink_metadata(&alias).unwrap().is_symlink());
}
