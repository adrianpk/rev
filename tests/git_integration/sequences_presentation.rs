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

// Read actual native availability after a background source-History refresh;
// process ownership remains serialized and the fixture keeps its 5-second bound.
fn sequence_idle(editor: &mut rev::session::Session) {
    use std::time::{Duration, Instant};
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        let frame = render(editor).join("\n");
        if frame.contains("Admitted controls:") {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native sequence availability deadline: {frame}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

// Both profiles expose family-specific keys through the shared native review.
// Return/cancel preserve the stop; checked continuation/abort preserve sources.
#[test]
fn native_controls() {
    for (profile, kind) in [
        ("non-modal", ApplyKind::CherryPick),
        ("modal", ApplyKind::Revert),
    ] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let targets = super::controls::conflict(&repository, kind);
        let original = object(&repository, "HEAD");
        let source = crate::status::presentation::text(editor);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        editor.execute(Command::Git(GitCommand::Log));
        wait(editor, |s| s.echo().contains("Git history inspected"));
        // History lists newest first; explicit append order is first, second, last.
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, 'a');
        key(editor, KeyCode::Up, KeyModifiers::NONE);
        plain(editor, 'a');
        key(editor, KeyCode::Up, KeyModifiers::NONE);
        plain(editor, 'a');
        plain(
            editor,
            if kind == ApplyKind::CherryPick {
                'P'
            } else {
                'V'
            },
        );
        wait(editor, |s| {
            s.echo().starts_with("Confirm") && s.echo().contains("exact ordered revisions")
        });
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("stopped in progress"));
        sequence_idle(editor);
        editor.execute(Command::Git(GitCommand::OperationStatus));
        wait(editor, |s| s.echo().contains("sequence state checked"));
        sequence_idle(editor);
        let frame = render(editor).join("\n");
        assert!(frame.contains("Step 2 / 3"), "{frame}");
        // Labels plus SHA-256 IDs wrap at the native 80-column fixture width.
        assert!(
            frame
                .split_whitespace()
                .collect::<String>()
                .contains(&targets[1].hex()),
            "{frame}"
        );
        let before = object(&repository, "HEAD");
        editor.execute(Command::Git(if kind == ApplyKind::CherryPick {
            GitCommand::RevertAbort
        } else {
            GitCommand::CherryPickAbort
        }));
        wait(editor, |s| {
            s.echo().contains("different Git sequence family")
        });
        let abort_key = if kind == ApplyKind::CherryPick {
            'Y'
        } else {
            'Q'
        };
        plain(editor, abort_key);
        wait(editor, |s| s.echo().contains("Confirm abort"));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(object(&repository, "HEAD"), before);
        plain(editor, 'n');
        assert_eq!(object(&repository, "HEAD"), before);
        if kind == ApplyKind::CherryPick {
            fs::write(repository.root.join("file"), "native resolution\n").unwrap();
            repository.git(["add", "--", "file"]);
            plain(editor, '{');
            wait(editor, |s| s.echo().contains("Confirm continue"));
        } else {
            plain(editor, abort_key);
            wait(editor, |s| s.echo().contains("Confirm abort"));
        }
        let review = render(editor).join("\n");
        assert!(
            review
                .split_whitespace()
                .collect::<String>()
                .contains(&original.hex()),
            "{review}"
        );
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        wait(editor, |s| s.echo().contains("Git history inspected"));
        assert_eq!(crate::status::presentation::text(editor), source);
        assert_eq!(
            repository.git(["symbolic-ref", "HEAD"]),
            b"refs/heads/main\n"
        );
        if kind == ApplyKind::CherryPick {
            assert_eq!(
                fs::read(repository.root.join("file")).unwrap(),
                b"native resolution\n"
            );
            assert!(repository.root.join("last").exists());
        } else {
            assert_eq!(object(&repository, "HEAD"), original);
        }
    }
}
