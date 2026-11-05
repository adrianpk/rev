use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand, Motion};

fn replace(editor: &mut rev::session::Session, value: &str) {
    editor.execute(Command::Motion(Motion::BufferEnd));
    let count = text(editor).chars().count();
    for _ in 0..count {
        key(editor, KeyCode::Backspace, KeyModifiers::NONE);
    }
    editor.handle(Event::Paste(value.into()));
}
fn accept(editor: &mut rev::session::Session) {
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
}

// Both profiles edit literal arguments, separate preparation from execution,
// reject Return as consent and restore the original native source on cancel.
#[test]
fn native_preparation() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let source = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        plain(editor, 'I');
        plain(editor, 'I');
        assert!(text(editor).contains("init"));
        let destination = repository.fixture.0.join("native-created");
        replace(
            editor,
            &serde_json::to_string(&vec!["init", "--", destination.to_str().unwrap()]).unwrap(),
        );
        accept(editor);
        wait(editor, |e| e.echo().contains("Exact prepared Git review"));
        assert!(!destination.exists());
        let review = render(editor).join("\n");
        assert!(review.contains("literal Git argv"), "{review}");
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!destination.exists());
        plain(editor, 'q');
        assert_eq!(text(editor), source);
        assert!(!destination.exists());
        plain(editor, 'I');
        plain(editor, 'I');
        replace(
            editor,
            &serde_json::to_string(&vec!["init", "--", destination.to_str().unwrap()]).unwrap(),
        );
        accept(editor);
        wait(editor, |e| e.echo().contains("Exact prepared Git review"));
        accept(editor);
        wait(editor, |e| {
            e.echo().contains("destination created and verified")
        });
        assert!(destination.join(".git/HEAD").exists());
        plain(editor, 'q');
        assert_eq!(text(editor), source);
    }
}

// Ignore entry visits the real ordinary file and preserves unsaved visiting
// identity; it inserts no pattern and performs no hidden save.
#[test]
fn ordinary_ignore() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    let ignore = repository.root.join(".gitignore");
    fs::write(&ignore, "build/\n").unwrap();
    editor.load(&ignore).unwrap();
    editor.insert('!');
    let before = text(editor);
    let id = editor.buffer().unwrap().id();
    editor.load(&repository.root.join("source.txt")).unwrap();
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    plain(editor, 'i');
    plain(editor, 't');
    assert!(!editor.git_visible());
    assert_eq!(editor.buffer().unwrap().id(), id);
    assert_eq!(text(editor), before);
    assert_eq!(fs::read(&ignore).unwrap(), b"build/\n");
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    editor.execute(Command::Git(GitCommand::IgnoreExclude));
    assert_eq!(
        editor.buffer().unwrap().file().unwrap().path(),
        repository.root.join(".git/info/exclude")
    );
}

// Named preparation needs no Git context: a chosen ordinary directory supplies
// cwd, cancellation restores tree/source focus and result inspection owns output.
#[test]
fn outside_repository() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", false);
    let editor = &mut owned.0;
    editor.load(&repository.fixture.0).unwrap();
    let source = text(editor);
    editor.execute(Command::Git(GitCommand::Init));
    assert!(text(editor).contains("init"));
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert!(!editor.git_visible());
    assert_eq!(text(editor), source);
    editor.execute(Command::Git(GitCommand::Init));
    let destination = repository.fixture.0.join("native-outside");
    replace(
        editor,
        &serde_json::to_string(&vec!["init", "--", destination.to_str().unwrap()]).unwrap(),
    );
    accept(editor);
    wait(editor, |e| e.echo().contains("Exact prepared Git review"));
    assert!(render(editor).join("\n").contains("native-outside"));
    accept(editor);
    editor.execute(Command::Git(GitCommand::Processes));
    wait(editor, |e| {
        e.echo().contains("destination created and verified")
    });
    assert!(render(editor).join("\n").contains("Verified"));
    assert!(destination.join(".git/HEAD").exists());
    plain(editor, 'q');
    assert!(!editor.git_visible());
    assert_eq!(text(editor), source);
}

// Both profiles expose all five working submodule seeds through Status, keep
// cancellation native and retain actual list output under process inspection.
#[test]
fn native_submodules() {
    for profile in ["non-modal", "modal"] {
        let (repository, child, _owner) = super::submodule::added("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let original = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        for (key, verb) in [
            ('a', "add"),
            ('u', "update"),
            ('s', "sync"),
            ('d', "deinit"),
        ] {
            plain(editor, 'o');
            plain(editor, key);
            let seed: Vec<String> = serde_json::from_str(&text(editor)).unwrap();
            assert_eq!(seed[0], "submodule");
            assert_eq!(seed[1], verb);
            key_event_cancel(editor);
            assert_eq!(text(editor), original);
        }
        plain(editor, 'o');
        plain(editor, 'l');
        accept(editor);
        wait(editor, |e| e.echo().contains("Exact prepared Git review"));
        assert!(render(editor).join("\n").contains("--literal-pathspecs"));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(render(editor).join("\n").contains("Argument Review"));
        accept(editor);
        wait(editor, |e| e.echo().contains("repository command verified"));
        editor.execute(Command::Git(GitCommand::Processes));
        let rows = render(editor).join("\n");
        assert!(rows.contains("Verified"), "{rows}");
        editor.execute(Command::Git(GitCommand::ProcessShow));
        let detail = render(editor).join("\n");
        let head = String::from_utf8(child.git(["rev-parse", "HEAD"])).unwrap();
        assert!(detail.contains(&head[..12]), "{detail}");
        assert_eq!(text(editor), original);
    }
}
fn key_event_cancel(editor: &mut rev::session::Session) {
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
}

// Unsaved child text blocks update; a clean retained alias also protects deinit.
// Both preserve the visiting owner independently of clean saved disk bytes.
#[test]
fn submodule_sources() {
    for modified in [true, false] {
        let (repository, _child, _owner) = super::submodule::added("sha1");
        let mut owned = editor(&repository, "non-modal", "", true);
        let editor = &mut owned.0;
        let child = repository.root.join("module space/child");
        let alias = repository.fixture.0.join("visiting-alias");
        std::os::unix::fs::symlink(&child, &alias).unwrap();
        editor.load(if modified { &child } else { &alias }).unwrap();
        if modified {
            editor.insert('!');
        }
        let retained = text(editor);
        // Status resolves from an actual parent source, while the child/alias
        // remains retained in another ordinary native buffer.
        editor.load(&repository.root.join("source.txt")).unwrap();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        plain(editor, 'o');
        plain(editor, if modified { 'u' } else { 'd' });
        replace(
            editor,
            if modified {
                r#"["submodule","update","--init","--checkout","--","module space"]"#
            } else {
                r#"["submodule","deinit","--","module space"]"#
            },
        );
        accept(editor);
        if modified {
            assert!(editor.echo().contains("unsaved text"), "{}", editor.echo());
        } else {
            wait(editor, |e| e.echo().contains("Exact prepared Git review"));
            accept(editor);
            assert!(
                editor.echo().contains("retained visiting buffer"),
                "{}",
                editor.echo()
            );
        }
        key_event_cancel(editor);
        editor.load(if modified { &child } else { &alias }).unwrap();
        assert_eq!(text(editor), retained);
        assert_eq!(fs::read(child).unwrap(), b"first\n");
    }
}

// Both profiles expose all subtree seeds; preparation and cancellation remain
// read-only, while separate acceptance retains the actual split object output.
#[test]
fn native_subtrees() {
    for profile in ["non-modal", "modal"] {
        let (repository, _child, _owner) = super::subtree::added("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let original = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        for (key, verb) in [
            ('a', "add"),
            ('m', "merge"),
            ('f', "pull"),
            ('p', "push"),
            ('s', "split"),
        ] {
            plain(editor, 'O');
            plain(editor, key);
            let seed: Vec<String> = serde_json::from_str(&text(editor)).unwrap();
            assert_eq!(
                (&seed[0], &seed[1]),
                (&"subtree".to_owned(), &verb.to_owned())
            );
            key_event_cancel(editor);
            assert_eq!(text(editor), original);
        }
        let draft = r#"["subtree","split","--prefix","vendor module","--","HEAD"]"#;
        let objects = repository.git(["count-objects", "-v"]);
        let head = repository.git(["rev-parse", "HEAD"]);
        plain(editor, 'O');
        plain(editor, 's');
        replace(editor, draft);
        accept(editor);
        wait(editor, |e| e.echo().contains("Exact prepared Git review"));
        assert!(render(editor).join("\n").contains("Pinned Git object"));
        assert_eq!(repository.git(["count-objects", "-v"]), objects);
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(repository.git(["count-objects", "-v"]), objects);
        plain(editor, 'q');
        assert_eq!(text(editor), original);
        plain(editor, 'O');
        plain(editor, 's');
        replace(editor, draft);
        accept(editor);
        wait(editor, |e| e.echo().contains("Exact prepared Git review"));
        accept(editor);
        wait(editor, |e| e.echo().contains("repository command verified"));
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        editor.execute(Command::Git(GitCommand::Processes));
        editor.execute(Command::Git(GitCommand::ProcessShow));
        let output = render(editor).join("\n");
        assert!(
            output.contains("Verified") && output.contains("stdout:"),
            "{output}"
        );
        assert_eq!(text(editor), original);
    }
}

// Both profiles expose all notes/sparse seeds through their contextual
// prefixes and execute checked metadata and materialization on real files.
#[test]
fn native_notes_sparse() {
    for profile in ["non-modal", "modal"] {
        let repository = super::metadata::sparse_repository("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let original = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        for (prefix, suffix, family) in [
            ('T', 'a', Family::NotesAdd),
            ('T', 'r', Family::NotesRemove),
            ('T', 'p', Family::NotesPrune),
            ('>', 'e', Family::SparseEnable),
            ('>', 'd', Family::SparseDisable),
            ('>', 'r', Family::SparseReapply),
            ('>', 's', Family::SparseSet),
            ('>', 'a', Family::SparseAdd),
        ] {
            plain(editor, prefix);
            plain(editor, suffix);
            let seed: Vec<String> = serde_json::from_str(&text(editor)).unwrap();
            assert_eq!(seed, family.seed());
            key_event_cancel(editor);
            assert_eq!(text(editor), original);
        }
        for (prefix, suffix, draft) in [
            (
                'T',
                'a',
                r#"["notes","--ref","refs/notes/native","add","--message","native note","--","HEAD"]"#,
            ),
            (
                'T',
                'r',
                r#"["notes","--ref","refs/notes/native","remove","--","HEAD"]"#,
            ),
            (
                '>',
                's',
                r#"["sparse-checkout","set","--cone","--no-sparse-index","--skip-checks","--","alpha"]"#,
            ),
            ('>', 'd', r#"["sparse-checkout","disable"]"#),
        ] {
            plain(editor, prefix);
            plain(editor, suffix);
            replace(editor, draft);
            accept(editor);
            wait(editor, |e| e.echo().contains("Exact prepared Git review"));
            if prefix == 'T' {
                assert!(render(editor).join("\n").contains("Pinned Git object"));
            }
            key(editor, KeyCode::Enter, KeyModifiers::NONE);
            assert!(render(editor).join("\n").contains("Argument Review"));
            accept(editor);
            wait(editor, |e| e.echo().contains("repository command verified"));
            if prefix == 'T' && suffix == 'a' {
                let note = String::from_utf8(repository.git([
                    "notes",
                    "--ref",
                    "refs/notes/native",
                    "list",
                    "HEAD",
                ]))
                .unwrap();
                assert!(editor.echo().contains(note.trim()), "{}", editor.echo());
                assert_eq!(
                    repository.git(["notes", "--ref", "refs/notes/native", "show", "HEAD"]),
                    b"native note\n"
                );
            } else if prefix == 'T' {
                assert!(
                    repository
                        .git(["notes", "--ref", "refs/notes/native", "list"])
                        .is_empty()
                );
            } else {
                assert_eq!(repository.root.join("beta/file").exists(), suffix == 'd');
            }
            assert_eq!(text(editor), original);
        }
        plain(editor, 'q');
        editor.insert('!');
        let modified = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        plain(editor, '>');
        plain(editor, 'e');
        accept(editor);
        assert!(editor.echo().contains("unsaved text"), "{}", editor.echo());
        key_event_cancel(editor);
        plain(editor, 'q');
        assert_eq!(text(editor), modified);
        assert!(repository.root.join("beta/file").is_file());
    }
}
