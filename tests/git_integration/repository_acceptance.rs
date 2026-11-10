use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, selected, text, wait};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{BufferCommand, Command, GitCommand, TerminalCommand};

// Saved-only stash creation uses the ordinary file owner's reload/ask policy;
// it never saves buffer text or replaces the original native buffer identity.
#[test]
fn native_reload() {
    for policy in ["ask", "reload"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(
            &repository,
            "non-modal",
            &format!("[files]\nexternal_changes='{policy}'\nexternal_change_debounce_ms=0\n"),
            true,
        );
        let editor = &mut owned.0;
        let path = repository.root.join("source.txt");
        let base = text(editor);
        fs::write(&path, "saved changed source\n").unwrap();
        editor.execute(Command::RevertBuffer);
        wait(editor, |e| text(e) == "saved changed source\n");
        let buffer = editor.buffer().unwrap().id();
        entry(editor);
        wait(editor, |e| e.echo().contains("Git Status refreshed"));
        editor.execute(Command::Git(GitCommand::StashList));
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        plain(editor, 'c');
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |e| e.echo().contains("Confirm stash create"));
        plain(editor, 'y');
        wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
        assert_eq!(fs::read_to_string(&path).unwrap(), base);
        plain(editor, 'q');
        plain(editor, 'q');
        assert_eq!(editor.buffer().unwrap().id(), buffer);
        if policy == "ask" {
            wait(editor, |e| {
                e.buffer().unwrap().external_change_notice().is_some()
            });
            assert_eq!(text(editor), "saved changed source\n");
            editor.execute(Command::RevertBuffer);
        }
        wait(editor, |e| text(e) == base);
        assert!(!editor.buffer().unwrap().is_modified());
        assert!(!repository.git(["stash", "list"]).is_empty());
    }
}

// Refresh follows the full selected object through changed ordinal positions;
// a second list change after destructive review cannot drop a replacement.
#[test]
fn native_stash_positions() {
    let repository = Repository::new("sha256");
    let mut owned = editor(&repository, "modal", "", true);
    for message in ["older", "newer"] {
        fs::write(repository.root.join("other"), message).unwrap();
        repository.git([
            "stash",
            "push",
            "--include-untracked",
            &format!("--message={message}"),
        ]);
    }
    let older = String::from_utf8(repository.git(["rev-parse", "stash@{1}"])).unwrap();
    let editor = &mut owned.0;
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    editor.execute(Command::Git(GitCommand::StashList));
    wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    assert!(selected(editor).contains(older.trim()));
    fs::write(repository.root.join("other"), "third").unwrap();
    repository.git(["stash", "push", "--include-untracked", "--message=third"]);
    plain(editor, 'g');
    wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
    assert!(selected(editor).contains(older.trim()));
    plain(editor, 'k');
    wait(editor, |e| {
        e.echo().contains("Confirm stash drop exact stash entry")
    });
    fs::write(repository.root.join("other"), "fourth").unwrap();
    repository.git(["stash", "push", "--include-untracked", "--message=fourth"]);
    let before = repository.git(["stash", "list", "--format=%H"]);
    assert_eq!(before.iter().filter(|b| **b == b'\n').count(), 4);
    plain(editor, 'y');
    wait(editor, |e| {
        e.echo().contains("Stash request failed or became stale")
    });
    assert_eq!(repository.git(["stash", "list", "--format=%H"]), before);
    assert!(!repository.root.join("other").exists());
}

// A real native pop conflict retains the exact stash and exposes Git's actual
// index sides through the existing conflict owner without inventing a sequence.
#[test]
fn native_stash_conflict() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let path = repository.root.join("source.txt");
    fs::write(&path, "STASHED_SIDE\n").unwrap();
    repository.git(["stash", "push", "--message=conflicted"]);
    let object = repository.git(["rev-parse", "refs/stash"]);
    fs::write(&path, "CURRENT_SIDE\n").unwrap();
    repository.commit();
    let editor = &mut owned.0;
    editor.execute(Command::RevertBuffer);
    wait(editor, |e| text(e) == "CURRENT_SIDE\n");
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    editor.execute(Command::Git(GitCommand::StashList));
    wait(editor, |e| e.echo().contains("Git Stashes refreshed"));
    plain(editor, 'p');
    wait(editor, |e| {
        e.echo().contains("Confirm stash apply exact object, then")
    });
    plain(editor, 'y');
    wait(editor, |e| {
        e.echo().contains("Stash effects require inspection")
    });
    assert_eq!(repository.git(["rev-parse", "refs/stash"]), object);
    let index = repository.git(["ls-files", "--unmerged", "-z"]);
    assert!(!index.is_empty());
    assert!(!repository.root.join(".git/MERGE_HEAD").exists());
    editor.execute(Command::Git(GitCommand::Conflicts));
    wait(editor, |e| {
        e.echo().contains("Git-owned sequence state checked")
    });
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |e| {
        e.echo().contains("Checked conflict sides retained")
    });
    let mut captured = String::new();
    for _ in 0..4 {
        captured.push_str(&render(editor).join("\n"));
        key(editor, KeyCode::PageDown, KeyModifiers::NONE);
    }
    assert!(captured.contains("CURRENT_SIDE"), "{captured}");
    assert!(captured.contains("STASHED_SIDE"), "{captured}");
    assert_eq!(repository.git(["ls-files", "--unmerged", "-z"]), index);
}

// A native PTY rooted in the target remains alive and owned when worktree
// removal is refused; only the fixture's Editor drop shuts down that process.
#[test]
fn native_process_owner() {
    let repository = Repository::new("sha1");
    let script = repository.fixture.write("terminal.sh", "pwd > \"$1\"\nprintf 'WORKTREE_READY\\n'\nwhile IFS= read -r line; do printf 'GOT:%s\\n' \"$line\"; done\n");
    let cwd = repository.fixture.0.join("terminal-cwd");
    let config = format!(
        "[terminal]\nprogram='/bin/sh'\nargs=[{},{}]",
        serde_json::to_string(&script).unwrap(),
        serde_json::to_string(&cwd).unwrap()
    );
    let mut owned = editor(&repository, "non-modal", &config, true);
    let path = super::worktrees::linked(&repository, "linked");
    let editor = &mut owned.0;
    editor.load(&path.join("source.txt")).unwrap();
    editor.execute(Command::Terminal(TerminalCommand::Open));
    wait(editor, |e| e.terminal_focused());
    let terminal = editor.terminals().ids().next().unwrap();
    wait(editor, |_| cwd.is_file());
    assert_eq!(
        fs::read_to_string(&cwd).unwrap(),
        format!("{}\n", path.display())
    );
    wait(editor, |e| {
        e.terminals().get(terminal).unwrap().owns_process()
    });
    editor.execute(Command::Terminal(TerminalCommand::Toggle));
    // Retire the visiting file so the live PTY alone retains the linked target.
    editor.execute(Command::Buffer(BufferCommand::Kill));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    editor.load(&repository.root.join("source.txt")).unwrap();
    entry(editor);
    wait(editor, |e| e.echo().contains("Git Status refreshed"));
    editor.execute(Command::Git(GitCommand::WorktreeList));
    wait(editor, |e| e.echo().contains("Git Worktrees refreshed"));
    key(editor, KeyCode::Down, KeyModifiers::NONE);
    plain(editor, 'k');
    wait(editor, |e| {
        e.echo().contains("protected by a retained native process")
    });
    assert!(path.join("source.txt").exists());
    assert!(editor.terminals().get(terminal).unwrap().owns_process());
}
