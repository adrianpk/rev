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
