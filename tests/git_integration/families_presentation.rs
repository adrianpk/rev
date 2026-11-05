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
        wait(editor, |e| e.echo().contains("submodule command verified"));
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
