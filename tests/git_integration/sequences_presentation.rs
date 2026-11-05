use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand};

// Both native profiles preserve independent ordered marks and range-base roles
// through cancellation, exact review and an explicitly submitted real request.
#[test]
fn native_order() {
    for (profile, kind) in [
        ("non-modal", ApplyKind::CherryPick),
        ("modal", ApplyKind::Revert),
    ] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let [first, second] = source(&repository);
        if kind == ApplyKind::Revert {
            repository.git(["merge", "--ff-only", "topic"]);
        }
        let before = object(&repository, "HEAD");
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        editor.execute(Command::Git(GitCommand::Log));
        wait(editor, |s| s.echo().contains("Git history inspected"));
        plain(editor, 'a');
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, 'a');
        plain(editor, '*');
        let trigger = if kind == ApplyKind::CherryPick {
            'P'
        } else {
            'V'
        };
        plain(editor, trigger);
        wait(editor, |s| {
            s.echo().starts_with("Confirm") && s.echo().contains("exact ordered revisions")
        });
        let frame = render(editor).join("\n");
        assert!(frame.contains("Ordered revision review"), "{frame}");
        assert!(frame.contains(&first.hex()), "{frame}");
        assert!(frame.contains(&second.hex()), "{frame}");
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(object(&repository, "HEAD"), before);
        plain(editor, 'n');
        assert_eq!(object(&repository, "HEAD"), before);
        plain(editor, trigger);
        wait(editor, |s| {
            s.echo().starts_with("Confirm") && s.echo().contains("exact ordered revisions")
        });
        if profile == "non-modal" {
            key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        } else {
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        }
        wait(editor, |s| s.echo().contains("Git history inspected"));
        assert_ne!(object(&repository, "HEAD"), before);
        if kind == ApplyKind::CherryPick {
            assert_eq!(
                fs::read(repository.root.join("first")).unwrap(),
                b"First authored change"
            );
        } else {
            assert!(!repository.root.join("first").exists());
        }
        let frame = render(editor).join("\n");
        assert!(frame.contains("*[2]"), "{frame}");
    }
}

// A selected merge requires an explicit parent before any changing request.
// Its exact parent object is shown in review and cancellation preserves HEAD.
#[test]
fn native_mainline() {
    let repository = Repository::new("sha256");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    let [_, second] = merges(&repository);
    let before = object(&repository, "HEAD");
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'b');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    // Refs are sorted by exact name: main, side, side2, topic.
    for _ in 0..3 {
        key(editor, KeyCode::Down, KeyModifiers::NONE);
    }
    editor.execute(Command::Git(GitCommand::Log));
    wait(editor, |s| s.echo().contains("Git history inspected"));
    plain(editor, 'P');
    assert!(
        editor.echo().contains("Choose an explicit merge parent"),
        "{}",
        editor.echo()
    );
    editor.handle(Event::Paste("2".into()));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |s| {
        s.echo().starts_with("Confirm") && s.echo().contains("exact ordered revisions")
    });
    let frame = render(editor).join("\n");
    let parent = object(&repository, &format!("{}^2", second.hex()));
    assert!(frame.contains(&parent.hex()), "{frame}");
    plain(editor, 'n');
    assert_eq!(object(&repository, "HEAD"), before);
}
