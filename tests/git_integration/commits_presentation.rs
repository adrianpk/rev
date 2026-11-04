use crate::{
    status::presentation::{editor, entry, key, plain, text, wait},
    support::Repository,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, Split};
use std::fs;

// A single c opens a native editable owner in both profiles; following c is
// literal, Return is a newline, both submit aliases create exact messages, and
// the modified source, split identities and point survive the retained view.
#[test]
fn normal_composer() {
    for (profile, chord) in [("non-modal", false), ("modal", true)] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "base\n").unwrap();
        repository.commit();
        fs::write(&path, "staged\n").unwrap();
        repository.git(["add", "source"]);
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&path).unwrap();
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let before = text(editor);
        let point = editor.view().point;
        let id = editor.buffer().unwrap().id();
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'c');
        assert!(!editor.git_visible(), "{}", editor.echo());
        plain(editor, 'c');
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        editor.handle(Event::Paste("body λ雪".into()));
        assert_eq!(text(editor), "c\nbody λ雪");
        assert!(
            !editor
                .git_observation()
                .unwrap()
                .status
                .as_ref()
                .unwrap()
                .paths
                .is_empty()
        );
        if chord {
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
        } else {
            key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        }
        wait(editor, |s| s.echo().contains("Commit created and verified"));
        assert_eq!(repository.git(["show", "HEAD:source"]), b"staged\n");
        assert_eq!(
            repository.git(["show", "--format=%B", "--no-patch", "HEAD"]),
            "c\nbody λ雪\n".as_bytes()
        );
        plain(editor, 'q');
        assert_eq!(editor.buffer().unwrap().id(), id);
        assert_eq!(text(editor), before);
        assert_eq!(editor.view().point, point);
        assert!(editor.buffer().unwrap().is_modified());
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
    }
}

// Both cancellation aliases preserve the same session message without a
// subprocess. Reopening after Status uses that text; filesystem publication and
// unrelated workspace commands cannot escape the transient view owner.
#[test]
fn normal_cancel() {
    for (profile, chord) in [("non-modal", true), ("modal", false)] {
        let repository = Repository::new("sha1");
        let path = repository.root.join("source");
        fs::write(&path, "base\n").unwrap();
        repository.commit();
        let head = repository.git(["rev-parse", "HEAD"]);
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&path).unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'c');
        editor.handle(Event::Paste("retained λ\n\nbody".into()));
        editor.execute(Command::SaveBuffer);
        assert!(editor.echo().contains("composer owns"));
        if chord {
            key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
            key(editor, KeyCode::Char('k'), KeyModifiers::CONTROL);
        } else {
            key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        assert!(editor.git_visible());
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(fs::read(&path).unwrap(), b"base\n");
        plain(editor, 'c');
        assert_eq!(text(editor), "retained λ\n\nbody");
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    }
}
