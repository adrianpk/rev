use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command as Native, GitCommand};

fn open(editor: &mut rev::session::Session) {
    editor.execute(Native::Git(GitCommand::Conflicts));
    wait(editor, |e| {
        e.echo().contains("Git-owned sequence state checked")
    });
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |e| {
        e.echo().contains("Checked conflict sides retained")
    });
}

// Both profiles inspect immutable full sides, return through Operation without
// aborting, and visit the native result while preserving existing unsaved text.
#[test]
fn native_sides() {
    for (profile, format) in [("non-modal", "sha1"), ("modal", "sha256")] {
        let repository = Repository::new(format);
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        topic(
            &repository,
            Path::new("file"),
            b"BASE_SIDE\n",
            b"CURRENT_SIDE\n",
            b"APPLIED_SIDE\n",
        );
        stopped(&repository, &["merge", "topic"]);
        let disk = fs::read(repository.root.join("file")).unwrap();
        let index = repository.git(["ls-files", "--stage", "-z"]);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        open(editor);
        let mut captured = String::new();
        for _ in 0..4 {
            captured.push_str(&render(editor).join("\n"));
            key(editor, KeyCode::PageDown, KeyModifiers::NONE);
        }
        for expected in [
            "Git Conflict: file",
            "Base / index stage 1",
            "Current / index stage 2",
            "Applied / index stage 3",
            "BASE_SIDE",
            "CURRENT_SIDE",
            "APPLIED_SIDE",
        ] {
            assert!(
                captured.contains(expected),
                "{profile}: {expected}: {captured}"
            );
        }
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        plain(editor, 'q');
        assert!(render(editor).join("\n").contains("Git Operation: Merge"));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| {
            e.echo().contains("Checked conflict sides retained")
        });
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!editor.git_visible());
        assert_eq!(text(editor).as_bytes(), disk);
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        let unsaved = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        open(editor);
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!editor.git_visible());
        assert_eq!(text(editor), unsaved);
        assert!(editor.buffer().unwrap().is_modified());
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
}

// Selection uses exact conflict paths across navigation, refresh and pointer
// rows; neither wrapped display positions nor stale painted rows supply a path.
#[test]
fn selected_path() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "modal", "", true);
    let editor = &mut owned.0;
    for path in ["a", "z"] {
        fs::write(repository.root.join(path), "base\n").unwrap();
    }
    repository.commit();
    repository.git(["switch", "--create", "topic"]);
    for path in ["a", "z"] {
        fs::write(repository.root.join(path), "applied\n").unwrap();
    }
    repository.commit();
    repository.git(["switch", "main"]);
    for path in ["a", "z"] {
        fs::write(repository.root.join(path), "current\n").unwrap();
    }
    repository.commit();
    stopped(&repository, &["merge", "topic"]);
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    editor.execute(Native::Git(GitCommand::Conflicts));
    wait(editor, |e| {
        e.echo().contains("Git-owned sequence state checked")
    });
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    assert!(render(editor).iter().any(|s| s.starts_with("> z")));
    plain(editor, 'g');
    wait(editor, |e| {
        e.echo().contains("Git-owned sequence state checked")
    });
    let rows = render(editor);
    let row = rows.iter().position(|s| s.starts_with("  a")).unwrap();
    crate::status::presentation::click(editor, row as u16);
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |e| {
        e.echo().contains("Checked conflict sides retained")
    });
    assert!(render(editor).join("\n").contains("Git Conflict: a"));
}
