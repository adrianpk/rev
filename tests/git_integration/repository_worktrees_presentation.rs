use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand, Split};

// Both profiles expose the approved keys and explicit review, preserve native
// sources/windows, and visit a distinct worktree context without changing files.
#[test]
fn native_worktrees() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        editor.execute(Command::Split(Split::Below));
        let source = text(editor);
        let windows: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        plain(editor, 'Z');
        plain(editor, if profile == "modal" { 'l' } else { 'Z' });
        wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
        let path = repository.fixture.0.join("native-linked");
        plain(editor, 'c');
        editor.handle(Event::Paste(path.to_str().unwrap().into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        editor.handle(Event::Paste("native-linked".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm worktree create"));
        let review = render(editor).join("\n");
        assert!(review.contains("New branch: native-linked"), "{review}");
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(!path.exists());
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        assert!(selected(editor).contains("native-linked"));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(render(editor).join("\n").contains("Common directory"));
        plain(editor, 'q');
        plain(editor, 'L');
        editor.handle(Event::Paste("offline storage".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm worktree lock"));
        assert!(
            render(editor)
                .join("\n")
                .contains("Lock reason: offline storage")
        );
        plain(editor, 'n');
        assert!(editor.echo().contains("Worktree action canceled"));
        plain(editor, 'L');
        editor.handle(Event::Paste("offline storage".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm worktree lock"));
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
        assert!(selected(editor).contains("[locked]"));
        plain(editor, 'U');
        wait(editor, |e| e.echo().contains("Confirm worktree unlock"));
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
        plain(editor, 'v');
        wait(editor, |e| {
            e.echo()
                .contains("Visited checked linked worktree Git Status")
        });
        assert_eq!(
            editor.git_observation().unwrap().identity.worktree.as_ref(),
            Some(&path)
        );
        assert_eq!(text(editor), source);
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            windows
        );
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
    }
}

// A linked worktree retained by either a clean or unsaved native source cannot
// be removed, even when another repository buffer is currently selected.
#[test]
fn visiting_owner() {
    for modified in [false, true] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, "non-modal", "", true);
        let path = linked(&repository, "linked");
        let editor = &mut owned.0;
        editor.load(&path.join("source.txt")).unwrap();
        if modified {
            editor.insert('!');
        }
        let source = text(editor);
        editor.load(&repository.root.join("source.txt")).unwrap();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        editor.execute(Command::Git(GitCommand::WorktreeList));
        wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, 'k');
        wait(editor, |e| {
            e.echo().contains(if modified {
                "protected by an unsaved"
            } else {
                "protected by a retained visiting"
            })
        });
        assert!(path.join("source.txt").exists());
        plain(editor, 'q');
        plain(editor, 'q');
        editor.load(&path.join("source.txt")).unwrap();
        assert_eq!(text(editor), source);
        assert_eq!(editor.buffer().unwrap().is_modified(), modified);
    }
}
