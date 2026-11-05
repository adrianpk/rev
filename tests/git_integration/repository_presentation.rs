use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand};

// Both profiles use the approved z prefix and stash-specific p pop binding;
// review shows full authority, Return cannot accept and ordinary source returns.
#[test]
fn native_stashes() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        fs::write(repository.root.join("other"), "saved stash bytes\n").unwrap();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        plain(editor, 'z');
        plain(editor, 'z');
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        assert!(render(editor).join("\n").contains("Git Stashes"));
        plain(editor, 'c');
        editor.handle(Event::Paste("native example".into()));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm stash create"));
        assert!(
            render(editor)
                .join("\n")
                .contains("Includes tracked and untracked")
        );
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(repository.git(["stash", "list"]).is_empty());
        plain(editor, 'n');
        assert!(editor.echo().contains("Stash action canceled"));
        plain(editor, 'c');
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm stash create"));
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        assert!(!repository.root.join("other").exists());
        let object = String::from_utf8(repository.git(["rev-parse", "refs/stash"])).unwrap();
        assert!(selected(editor).contains(object.trim()));
        plain(editor, 'v');
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        assert!(render(editor).join("\n").contains("saved stash bytes"));
        let source = text(editor);
        editor.insert('!');
        assert_eq!(text(editor), source);
        plain(editor, 'q');
        plain(editor, 'p');
        wait(editor, |e| {
            e.echo().contains("Confirm stash apply exact object, then")
        });
        assert!(render(editor).join("\n").contains(object.trim()));
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        assert_eq!(
            fs::read(repository.root.join("other")).unwrap(),
            b"saved stash bytes\n"
        );
        assert!(repository.git(["stash", "list"]).is_empty());
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
    }
}

// A modified affected native source blocks saved-only stash creation, while
// an unrelated unsaved visiting file is preserved without a hidden save.
#[test]
fn native_source_guards() {
    for variant in ["affected", "unrelated", "rename"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, "modal", "", true);
        let editor = &mut owned.0;
        plain(editor, 'i');
        plain(editor, 'x');
        let source = text(editor);
        if variant == "rename" {
            repository.git(["mv", "--", "source.txt", "renamed.txt"]);
        } else {
            fs::write(
                repository.root.join(if variant == "affected" {
                    "source.txt"
                } else {
                    "other"
                }),
                "saved disk\n",
            )
            .unwrap();
        }
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        editor.execute(Command::Git(GitCommand::StashList));
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        plain(editor, 'c');
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        if variant != "unrelated" {
            wait(editor, |e| {
                e.echo()
                    .contains("affected visiting buffer has unsaved text")
            });
            assert!(repository.git(["stash", "list"]).is_empty());
        } else {
            wait(editor, |e| e.echo().contains("Confirm stash create"));
            plain(editor, 'y');
            wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
            assert!(!repository.git(["stash", "list"]).is_empty());
        }
        plain(editor, 'q');
        plain(editor, 'q');
        assert_eq!(text(editor), source);
        assert!(editor.buffer().unwrap().is_modified());
    }
}
