use super::*;
use crate::conflicts::presentation::open;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};

// Both profiles require exact review, distinguish Return from acceptance,
// cancel safely and keep side replacement, staging and continuation separate.
#[test]
fn native_resolution() {
    for (profile, format) in [("non-modal", "sha1"), ("modal", "sha256")] {
        let repository = Repository::new(format);
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        stopped(&repository, &["merge", "topic"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let disk = fs::read(repository.root.join("file")).unwrap();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        open(editor);
        plain(editor, '2');
        wait(editor, |e| {
            e.echo().contains("Confirm replace result with Current")
        });
        assert!(
            render(editor)
                .join("\n")
                .contains("Checked conflict resolution review")
        );
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        plain(editor, 'n');
        assert!(editor.echo().contains("Conflict resolution canceled"));
        plain(editor, '3');
        wait(editor, |e| {
            e.echo().contains("Confirm replace result with Applied")
        });
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        wait(editor, |e| e.echo().contains("Conflict action verified"));
        assert_eq!(
            fs::read(repository.root.join("file")).unwrap(),
            b"applied\n"
        );
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
        open(editor);
        plain(editor, 'x');
        wait(editor, |e| {
            e.echo().contains("Confirm stage exact saved result")
        });
        key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        wait(editor, |e| e.echo().contains("Conflict action verified"));
        assert_eq!(repository.git(["show", ":file"]), b"applied\n");
        assert!(repository.git(["ls-files", "--unmerged", "-z"]).is_empty());
        assert!(repository.root.join(".git/MERGE_HEAD").exists());
    }
}

// Unsaved result text blocks changing requests. An unrelated unsaved native
// buffer is retained while a separately checked exact conflict is resolved.
#[test]
fn unsaved_result() {
    for target in [false, true] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, "modal", "", true);
        let editor = &mut owned.0;
        topic(
            &repository,
            Path::new("file"),
            b"base\n",
            b"current\n",
            b"applied\n",
        );
        stopped(&repository, &["merge", "topic"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        let disk = fs::read(repository.root.join("file")).unwrap();
        if target {
            entry(editor);
            wait(editor, |e| e.echo().contains("Git Status refreshed"));
            open(editor);
            key(editor, KeyCode::Enter, KeyModifiers::NONE);
        }
        plain(editor, 'i');
        plain(editor, 'x');
        let unsaved = text(editor);
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        open(editor);
        plain(editor, '2');
        if target {
            assert!(editor.echo().contains("unsaved text"));
            assert_eq!(fs::read(repository.root.join("file")).unwrap(), disk);
        } else {
            wait(editor, |e| {
                e.echo().contains("Confirm replace result with Current")
            });
            plain(editor, 'y');
            wait(editor, |e| e.echo().contains("Conflict action verified"));
            assert_eq!(
                fs::read(repository.root.join("file")).unwrap(),
                b"current\n"
            );
        }
        assert_eq!(text(editor), unsaved);
        assert!(editor.buffer().unwrap().is_modified());
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
}
