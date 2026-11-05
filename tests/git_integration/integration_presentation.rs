use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand};

// Both profiles review full targets and policies, accept the approved explicit
// submit aliases, and reconcile clean source through the existing file owner.
#[test]
fn native_merge() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        repository.git(["switch", "--create", "topic"]);
        fs::write(repository.root.join("source.txt"), "changed source\n").unwrap();
        repository.commit();
        repository.git(["switch", "main"]);
        let target = repository.git(["rev-parse", "topic"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, '=');
        let frame = render(editor).join("\n");
        assert!(frame.contains("fast-forward-only merge"), "{frame}");
        assert!(
            frame.contains(String::from_utf8_lossy(&target).trim()),
            "{frame}"
        );
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_ne!(repository.git(["rev-parse", "HEAD"]), target);
        if profile == "non-modal" {
            key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        } else {
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        }
        wait(editor, |s| s.echo().contains("Integration verified"));
        assert_eq!(repository.git(["rev-parse", "HEAD"]), target);
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), "changed source\n");
        entry(editor);
        wait(editor, |s| s.echo().contains("Git Status refreshed"));
        editor.execute(Command::Git(GitCommand::OperationStatus));
        wait(editor, |s| s.echo().contains("sequence state checked"));
        assert!(
            render(editor)
                .join("\n")
                .contains("No active Git-owned sequence")
        );
    }
}

// Modified repository-backed buffers reject integration before review or a
// child; entering/leaving Git preserves their exact unsaved text and point.
#[test]
fn unsaved_integration() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    repository.git(["branch", "topic"]);
    plain(editor, '!');
    let source = text(editor);
    let point = editor.view().point;
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'b');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    plain(editor, 'e');
    assert!(
        editor.echo().contains("modified repository-backed buffer"),
        "{}",
        editor.echo()
    );
    plain(editor, 'q');
    plain(editor, 'q');
    assert_eq!(text(editor), source);
    assert_eq!(editor.view().point, point);
}
