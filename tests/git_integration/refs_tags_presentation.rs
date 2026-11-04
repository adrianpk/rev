use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::{
    command::{Command, GitCommand},
    session::Session,
};

fn accept(editor: &mut Session) {
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
}
fn annotated(editor: &mut Session) {
    key(editor, KeyCode::Home, KeyModifiers::NONE);
    for _ in "lightweight".chars() {
        key(editor, KeyCode::Delete, KeyModifiers::NONE);
    }
    editor.handle(Event::Paste("annotated".into()));
    accept(editor);
    accept(editor);
}

// Both profiles create a default lightweight tag, compose/review an annotated
// tag, delete its actual tag object and return to unchanged native source.
#[test]
fn native_tags() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let source = text(editor);
        let point = editor.view().point;
        let head = repository.git(["rev-parse", "HEAD"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        plain(editor, 't');
        editor.handle(Event::Paste("plain".into()));
        accept(editor);
        accept(editor);
        accept(editor);
        wait(editor, |s| {
            s.echo().contains("Create local lightweight tag")
        });
        let frame = render(editor).join("\n");
        assert!(frame.contains("Kind: lightweight"));
        assert!(frame.contains(String::from_utf8_lossy(&head).trim()));
        accept(editor);
        assert!(repository.git(["tag", "--list"]).is_empty());
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert_eq!(repository.git(["rev-parse", "refs/tags/plain"]), head);
        editor.execute(Command::Git(GitCommand::TagCreate));
        editor.handle(Event::Paste("annotated".into()));
        accept(editor);
        annotated(editor);
        wait(editor, |s| s.echo().contains("Compose tag annotation"));
        assert!(!editor.git_visible());
        plain(editor, 'c');
        accept(editor);
        editor.handle(Event::Paste("Annotation body λ雪".into()));
        let annotation = text(editor);
        editor.execute(Command::SaveBuffer);
        assert_eq!(text(editor), annotation);
        assert_eq!(
            fs::read_to_string(repository.root.join("source.txt")).unwrap(),
            source
        );
        key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        assert!(editor.git_visible());
        let frame = render(editor).join("\n");
        assert!(frame.contains("Kind: annotated"), "{frame}");
        assert!(frame.contains("Annotation body λ雪"), "{frame}");
        assert!(
            frame.contains(String::from_utf8_lossy(&head).trim()),
            "{frame}"
        );
        accept(editor);
        assert!(repository.git(["tag", "--list", "annotated"]).is_empty());
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert_eq!(text(editor), source);
        let raw = repository.git(["cat-file", "tag", "refs/tags/annotated"]);
        assert!(String::from_utf8_lossy(&raw).ends_with(&annotation));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, 'T');
        let object = repository.git(["rev-parse", "refs/tags/annotated"]);
        let frame = render(editor).join("\n");
        assert!(
            frame.contains(String::from_utf8_lossy(&object).trim()),
            "{frame}"
        );
        assert!(
            frame.contains(String::from_utf8_lossy(&head).trim()),
            "{frame}"
        );
        plain(editor, 'n');
        plain(editor, 'T');
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        assert!(repository.git(["tag", "--list", "annotated"]).is_empty());
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    }
}

// Cancellation leaves session-only annotation text available for an explicit
// annotated retry; composer text never becomes a commit or writable source.
#[test]
fn retained_annotation() {
    let repository = Repository::new("sha1");
    repository.commit();
    let mut owned = editor(&repository, "modal", "", true);
    let editor = &mut owned.0;
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'y');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    plain(editor, 't');
    editor.handle(Event::Paste("draft".into()));
    accept(editor);
    annotated(editor);
    wait(editor, |s| s.echo().contains("Compose tag annotation"));
    editor.handle(Event::Paste("Retained tag text\n".into()));
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(editor, KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert!(
        editor.echo().contains("annotation retained"),
        "{}",
        editor.echo()
    );
    assert!(editor.git_visible());
    assert!(repository.git(["tag", "--list"]).is_empty());
    plain(editor, 't');
    accept(editor);
    annotated(editor);
    wait(editor, |s| s.echo().contains("Compose tag annotation"));
    assert_eq!(text(editor), "Retained tag text\n");
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert!(editor.git_visible());
    assert!(repository.git(["tag", "--list"]).is_empty());
}
