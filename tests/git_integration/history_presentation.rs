use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand, Split};

// Both native profiles retain exact selection across pages, detail and refresh.
// Read-only history input cannot save or edit the canonical modified source.
#[test]
fn native_history() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        for n in 0..4 {
            fs::write(
                repository.root.join("history.txt"),
                format!("version {n}\n"),
            )
            .unwrap();
            repository.commit();
        }
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let source = text(editor);
        let point = editor.view().point;
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        let head = repository.git(["rev-parse", "HEAD"]);
        let index = repository.git(["ls-files", "--stage", "-z"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'l');
        plain(editor, 'l');
        wait(editor, |s| s.echo().contains("Git history inspected"));
        let root = selected(editor);
        assert!(
            root.contains(String::from_utf8_lossy(&head).trim()),
            "{root}"
        );
        plain(editor, 'n');
        let older = selected(editor);
        assert_ne!(root, older);
        plain(editor, 'g');
        wait(editor, |s| s.echo().contains("Git history inspected"));
        assert_eq!(selected(editor), older);
        key(editor, KeyCode::Tab, KeyModifiers::NONE);
        wait(editor, |s| s.echo().contains("Git history inspected"));
        let frame = render(editor).join("\n");
        assert!(frame.contains("Git Revision:"), "{frame}");
        editor.handle(Event::Paste("forbidden".into()));
        editor.execute(Command::SaveBuffer);
        editor.insert('!');
        assert_eq!(text(editor), source);
        plain(editor, 'q');
        assert_eq!(selected(editor), older);
        plain(editor, 'q');
        assert!(render(editor).join("\n").contains("Git:"));
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    }
}

// History F captures its selected full target without a text prompt; A keeps
// amend bound to checked HEAD. Both use the existing explicit review owner.
#[test]
fn contextual_composition() {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source"), "base\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("source"), "next\n").unwrap();
    repository.commit();
    fs::write(repository.root.join("source"), "staged\n").unwrap();
    repository.git(["add", "source"]);
    let mut owned = editor(&repository, "modal", "", false);
    let editor = &mut owned.0;
    editor.load(&repository.root.join("source")).unwrap();
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'h');
    wait(editor, |s| s.echo().contains("Git history inspected"));
    plain(editor, 'n');
    plain(editor, 'F');
    wait(editor, |s| s.echo().contains("Fixup exact target"));
    assert!(!editor.git_visible(), "{}", editor.echo());
    assert!(text(editor).starts_with("fixup! Fixture"));
    key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
    assert!(editor.git_visible());
    assert!(render(editor).join("\n").contains("Fixup"));
    key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
    wait(editor, |s| s.echo().contains("Commit created and verified"));
    editor.execute(Command::Git(GitCommand::Log));
    wait(editor, |s| s.echo().contains("Git history inspected"));
    plain(editor, 'A');
    wait(editor, |s| s.echo().contains("Amend exact HEAD"));
    assert!(!editor.git_visible());
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    editor.execute(Command::Git(GitCommand::Log));
    wait(editor, |s| s.echo().contains("Git history inspected"));
    repository.commit();
    plain(editor, 'g');
    wait(editor, |s| s.echo().contains("History inspection failed"));
    plain(editor, 'F');
    assert!(editor.git_visible(), "{}", editor.echo());
    assert!(
        editor.echo().contains("Select an exact History revision"),
        "{}",
        editor.echo()
    );
}
