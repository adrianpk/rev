use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, GitCommand};

// Both profiles reach exact remote rows and invoke fetch through its direct
// native key/remote prefix without changing retained modified source state.
#[test]
fn native_fetch() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        remote(&repository, "origin");
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, '!');
        let source = text(editor);
        let point = editor.view().point;
        let head = repository.git(["rev-parse", "HEAD"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        plain(editor, '&');
        assert!(editor.echo().contains("Select one exact configured remote"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        assert!(render(editor).join("\n").contains("remote origin"));
        if profile == "non-modal" {
            plain(editor, '&');
        } else {
            plain(editor, 'M');
            plain(editor, 'f');
        }
        wait(editor, |s| s.echo().contains("Fetch completed"));
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        assert_eq!(
            repository.git(["rev-parse", "refs/remotes/origin/main"]),
            head
        );
        editor.execute(Command::Git(GitCommand::Quit));
        editor.execute(Command::Git(GitCommand::Quit));
        assert!(!editor.git_visible());
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
        assert!(editor.buffer().unwrap().is_modified());
    }
}

// A push mark survives navigation to the remote row. Both profiles review
// the exact full source/object/upstream, require y and retain unsaved source.
#[test]
fn native_push() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        let origin = remote(&repository, "origin");
        repository.git(["fetch", "origin"]);
        repository.git(["branch", "--set-upstream-to=origin/main", "main"]);
        let base = repository.git(["rev-parse", "HEAD"]);
        repository.commit();
        let tip = repository.git(["rev-parse", "HEAD"]);
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, '!');
        let source = text(editor);
        let point = editor.view().point;
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'b');
        wait(editor, |s| s.echo().contains("Refs and Status checked"));
        plain(editor, '*');
        assert!(editor.echo().contains("push source branch marked"));
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        key(editor, KeyCode::Down, KeyModifiers::NONE);
        plain(editor, 'M');
        plain(editor, 'p');
        wait(editor, |s| s.echo().starts_with("Push refs/heads/main at"));
        let frame = render(editor).join("\n");
        assert!(
            frame.contains("Single upstream branch push review"),
            "{frame}"
        );
        assert!(
            frame.contains(String::from_utf8_lossy(&tip).trim()),
            "{frame}"
        );
        assert!(
            frame.contains("origin") && frame.contains("refs/heads/main"),
            "{frame}"
        );
        key(editor, KeyCode::PageDown, KeyModifiers::NONE);
        key(editor, KeyCode::PageUp, KeyModifiers::NONE);
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            repository.git(["--git-dir", origin.to_str().unwrap(), "rev-parse", "main"]),
            base
        );
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(
            repository.git(["--git-dir", origin.to_str().unwrap(), "rev-parse", "main"]),
            base
        );
        editor.execute(Command::Git(GitCommand::Push));
        wait(editor, |s| s.echo().starts_with("Push refs/heads/main at"));
        plain(editor, 'y');
        wait(editor, |s| s.echo().contains("Push verified"));
        assert_eq!(
            repository.git(["--git-dir", origin.to_str().unwrap(), "rev-parse", "main"]),
            tip
        );
        key(editor, KeyCode::Char('>'), KeyModifiers::ALT);
        plain(editor, '-');
        plain(editor, 'p');
        assert!(
            editor.echo().contains("Mark one exact source branch"),
            "{}",
            editor.echo()
        );
        editor.execute(Command::Git(GitCommand::Quit));
        editor.execute(Command::Git(GitCommand::Quit));
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
        assert!(editor.buffer().unwrap().is_modified());
    }
}
