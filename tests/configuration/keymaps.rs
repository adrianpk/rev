use std::path::Path;

use rev::command::{Binding, Command, PromptAction};
use rev::config::{Snapshot, parse, prepare};
use rev::input::keymap::{Context, Sequence};

fn lookup(snapshot: &Snapshot, context: Context, sequence: &str) -> Option<Binding> {
    snapshot
        .keymaps()
        .map(context)
        .lookup(&Sequence::parse(sequence).unwrap())
}

// Exact user replacement retains unrelated inherited chords and context maps.
// Prompt-only targets use fixed action identities without entering the M-x catalog.
#[test]
fn inherited_maps() {
    let fixture = super::fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "[keys.editor]\n'M-j' = 'next-line'\n'C-x C-s' = 'undo'\n[keys.minibuffer]\n'M-j' = 'minibuffer-accept'\n[keys.tree]\n'M-j' = 'tree-next-line'"));
    assert!(startup.diagnostic.is_none());
    let snapshot = startup.snapshot;
    assert_eq!(
        lookup(&snapshot, Context::Editor, "M-j"),
        Binding::named("next-line")
    );
    assert_eq!(
        lookup(&snapshot, Context::Editor, "C-x C-s"),
        Binding::named("undo")
    );
    assert_eq!(
        lookup(&snapshot, Context::Editor, "C-x C-f"),
        Binding::named("find-file")
    );
    assert_eq!(
        lookup(&snapshot, Context::Minibuffer, "M-j"),
        Some(Binding::Prompt(PromptAction::Accept))
    );
    assert_eq!(
        lookup(&snapshot, Context::Tree, "M-j"),
        Binding::named("tree-next-line")
    );
    assert!(Command::named("minibuffer-accept").is_none());
    assert!(
        snapshot
            .keymaps()
            .map(Context::Editor)
            .is_prefix(&Sequence::parse("C-x").unwrap())
    );
    assert!(
        !snapshot
            .keymaps()
            .map(Context::Editor)
            .is_prefix(&Sequence::parse("C-x C-s").unwrap())
    );
}

// Normalized aliases in later layers intentionally replace earlier bindings;
// removed spellings cannot retain stale provenance or resurrect inherited entries.
#[test]
fn layered_aliases() {
    let fixture = super::fixture::Fixture::new();
    fixture.write(
        "first.toml",
        "[keys.editor]\n'ESC j' = 'next-line'\n'C-x C-s' = 'save-buffer'",
    );
    let root = fixture.write(
        "config.toml",
        "include = ['first.toml']\n[keys.editor]\n'M-j' = 'previous-line'\n'C-x C-s' = 'undo'",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let snapshot = startup.snapshot;
    assert_eq!(
        lookup(&snapshot, Context::Editor, "ESC j"),
        Binding::named("previous-line")
    );
    assert_eq!(
        lookup(&snapshot, Context::Editor, "C-x C-s"),
        Binding::named("undo")
    );
    assert_eq!(
        snapshot.origin(&["keys", "editor", "M-j"]),
        Some(root.as_path())
    );
    assert_eq!(snapshot.origin(&["keys", "editor", "ESC j"]), None);
    assert!(!snapshot.settings().keys.editor.contains_key("ESC j"));
    assert_eq!(snapshot.settings().keys.editor.len(), 2);
}

// Syntax, context, duplicate aliases and inherited leaf/prefix ambiguity reject
// the entire candidate and select the original key/value span for diagnostics.
#[test]
fn rejected_maps() {
    for (name, source, message, span) in [
        (
            "syntax",
            "[keys.editor]\n'Q-j' = 'next-line'",
            "printable scalar",
            "'Q-j'",
        ),
        (
            "context",
            "[keys.minibuffer]\n'M-j' = 'save-buffer'",
            "unavailable",
            "'save-buffer'",
        ),
        (
            "prompt action",
            "[keys.tree]\n'M-j' = 'minibuffer-accept'",
            "unavailable",
            "'minibuffer-accept'",
        ),
        (
            "tree context",
            "[keys.tree]\n'M-j' = 'find-file'",
            "unavailable",
            "'find-file'",
        ),
        (
            "duplicate",
            "[keys.editor]\n'M-j' = 'next-line'\n'A-j' = 'previous-line'",
            "duplicates",
            "'M-j'",
        ),
        (
            "prefix leaf",
            "[keys.editor]\n'C-x' = 'next-line'",
            "prefix conflict",
            "'C-x'",
        ),
        (
            "leaf descendant",
            "[keys.editor]\n'C-f x' = 'next-line'",
            "prefix conflict",
            "'C-f x'",
        ),
        (
            "alias prefix",
            "[keys.editor]\n'ESC x k' = 'next-line'",
            "prefix conflict",
            "'ESC x k'",
        ),
    ] {
        let error = parse(Path::new("keys.toml"), source.as_bytes()).unwrap_err();
        assert!(error.message.contains(message), "{name}: {error}");
        assert_eq!(&source[error.span.unwrap()], span, "{name}");
        assert!(error.position.is_some(), "{name}");
    }
}

// Individually valid layers may form an ambiguous complete map. Rejection
// publishes complete defaults, including unrelated settings and all three maps.
#[test]
fn merged_conflict() {
    let fixture = super::fixture::Fixture::new();
    fixture.write("first.toml", "[keys.editor]\n'M-j k' = 'next-line'");
    let root = fixture.write(
        "config.toml",
        "include = ['first.toml']\neditor.tab_width = 8\n[keys.editor]\n'M-j' = 'previous-line'",
    );
    let startup = prepare(&root);
    let error = startup.diagnostic.unwrap();
    assert!(error.message.contains("M-j k") && error.message.contains("M-j"));
    assert_eq!(error.source, root);
    assert_eq!(error.key, ["keys", "editor", "M-j"]);
    assert_eq!(startup.snapshot, Snapshot::default());
    assert_eq!(
        lookup(&startup.snapshot, Context::Editor, "C-x C-s"),
        Binding::named("save-buffer")
    );
    assert_eq!(lookup(&startup.snapshot, Context::Editor, "M-j"), None);
}

// Removing an override restores the inherited command without modifying any
// other chord. Default maps retain every audited entry and available target.
#[test]
fn complete_defaults() {
    let snapshot = Snapshot::default();
    for context in Context::ALL {
        let map = snapshot.keymaps().map(context);
        assert_eq!(map.bindings().count(), context.defaults().len());
        for (sequence, name) in context.defaults() {
            let target = lookup(&snapshot, context, sequence).unwrap();
            assert_eq!(target.name(), *name);
            assert!(context.available(target));
        }
    }
    assert_eq!(
        lookup(&snapshot, Context::Editor, "C-x C-s"),
        Binding::named("save-buffer")
    );
}

// Cancel stays reachable in each complete map. Case aliases cannot reassign it,
// and a chord containing cancel later in a prefix is unreachable and rejected.
#[test]
fn cancel_guard() {
    for (context, replacement) in [
        ("editor", "undo"),
        ("minibuffer", "forward-char"),
        ("tree", "tree-next-line"),
    ] {
        let allowed = format!("[keys.{context}]\n'C-g' = 'keyboard-quit'");
        assert!(parse(Path::new("cancel.toml"), allowed.as_bytes()).is_ok());
        let blocked = format!("[keys.{context}]\n'C-G' = '{replacement}'");
        let error = parse(Path::new("cancel.toml"), blocked.as_bytes()).unwrap_err();
        assert!(error.message.contains("C-g is reserved"));
        assert_eq!(&blocked[error.span.unwrap()], format!("'{replacement}'"));
        let unreachable = format!("[keys.{context}]\n'M-j C-g' = 'keyboard-quit'");
        let error = parse(Path::new("cancel.toml"), unreachable.as_bytes()).unwrap_err();
        assert!(error.message.contains("cannot complete"));
        assert_eq!(&unreachable[error.span.unwrap()], "'M-j C-g'");
    }
}

// Terminal aliases cannot receive conflicting custom targets in one source,
// even when modern terminals can report some of those physical keys separately.
#[test]
fn indistinguishable_chords() {
    for (left, right) in [
        ("C-i", "TAB"),
        ("C-m", "RET"),
        ("C-h", "DEL"),
        ("C-@", "C-SPC"),
        ("C-/", "C-7"),
        ("ESC j", "M-j"),
    ] {
        let source = format!("[keys.editor]\n'{left}' = 'undo'\n'{right}' = 'next-line'");
        let error = parse(Path::new("aliases.toml"), source.as_bytes()).unwrap_err();
        assert!(
            error
                .message
                .contains("terminal aliases cannot be distinguished"),
            "{left}: {error}"
        );
        assert!(error.span.is_some());
    }
}

// A failed reconfiguration cannot partially replace any effective context map.
// Omitting the override on the next preparation restores the inherited mapping.
#[test]
fn rejected_replacement() {
    let fixture = super::fixture::Fixture::new();
    let path = fixture.write("config.toml", "[keys.editor]\n'C-x C-s' = 'undo'");
    let previous = prepare(&path).snapshot;
    fixture.write(
        "config.toml",
        "[keys.editor]\n'M-j' = 'next-line'\n[keys.minibuffer]\n'C-g' = 'forward-char'",
    );
    let rejected = prepare(&path);
    assert!(rejected.diagnostic.is_some());
    assert_eq!(rejected.snapshot, Snapshot::default());
    assert_eq!(
        lookup(&previous, Context::Editor, "C-x C-s"),
        Binding::named("undo")
    );
    fixture.write("config.toml", "# The override is omitted.\n");
    let restored = prepare(&path);
    assert!(restored.diagnostic.is_none());
    assert_eq!(
        lookup(&restored.snapshot, Context::Editor, "C-x C-s"),
        Binding::named("save-buffer")
    );
    assert_eq!(lookup(&restored.snapshot, Context::Editor, "M-j"), None);
}
