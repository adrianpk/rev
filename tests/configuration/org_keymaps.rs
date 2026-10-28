use rev::command::Binding;
use rev::config::{parse, prepare};
use rev::input::keymap::{Context, Sequence};

use super::fixture::Fixture;

// Includes retain normalized Org-map overrides and their actual source. Empty
// local overlays inherit the admitted editor map, not another Org role's map.
#[test]
fn org_inheritance() {
    let fixture = Fixture::new();
    fixture.write(
        "first.toml",
        "org.enabled = false\n[keys.org]\n'ESC j' = 'newline'",
    );
    let root = fixture.write("config.toml", "include = ['first.toml']\norg.enabled = true\n[keys.editor]\n'M-k' = 'forward-char'\n[keys.org]\n'M-j' = 'backward-char'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let snapshot = startup.snapshot;
    assert!(snapshot.settings().org.enabled);
    assert_eq!(snapshot.origin(&["org", "enabled"]), Some(root.as_path()));
    assert_eq!(
        snapshot.origin(&["keys", "org", "M-j"]),
        Some(root.as_path())
    );
    assert_eq!(snapshot.origin(&["keys", "org", "ESC j"]), None);
    let org = snapshot.keymaps().map(Context::Org);
    assert_eq!(
        org.lookup(&Sequence::parse("M-j").unwrap()),
        Binding::named("backward-char")
    );
    for context in [Context::Org, Context::OrgCapture, Context::OrgSourceEdit] {
        assert_eq!(
            snapshot
                .keymaps()
                .map(context)
                .lookup(&Sequence::parse("M-k").unwrap()),
            Binding::named("forward-char")
        );
    }
    assert_eq!(
        snapshot
            .keymaps()
            .map(Context::OrgCapture)
            .lookup(&Sequence::parse("M-j").unwrap()),
        None
    );
}

// All roles enforce reserved cancellation, typed command availability and
// inherited prefix conflicts. Invalid candidates cannot partially publish Org.
#[test]
fn org_map_admission() {
    let cases = [
        ("org prefix", "[keys.org]\n'C-x' = 'newline'", "org"),
        (
            "capture prefix",
            "[keys.org_capture]\n'C-x' = 'newline'",
            "org_capture",
        ),
        (
            "source prefix",
            "[keys.org_source_edit]\n'C-x' = 'newline'",
            "org_source_edit",
        ),
        (
            "agenda edit",
            "[keys.org_agenda]\n'x' = 'newline'",
            "org_agenda",
        ),
        (
            "calendar edit",
            "[keys.org_calendar]\n'x' = 'delete-char'",
            "org_calendar",
        ),
        ("cancel", "[keys.org]\n'C-g' = 'org-mode'", "org"),
        (
            "future action",
            "[keys.org]\n'C-c C-c' = 'org-clock-in'",
            "org",
        ),
    ];
    let fixture = Fixture::new();
    for (name, source, context) in cases {
        let path = fixture.write("config.toml", source);
        let error = parse(&path, source.as_bytes()).unwrap_err();
        assert_eq!(&error.key[..2], ["keys", context], "{name}");
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot, rev::config::Snapshot::default(), "{name}");
    }
}
