use super::*;
use crate::status::presentation::{editor, entry, key, plain, render, text, wait};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{
    Command, GitCommand, History, LinearCommand, Motion, RectangleCommand, RegionCommand, Split,
    TerminalCommand,
};

// A live terminal panel/pane and modified source retain their owners through normal
// submission/cancellation. Composer bytes never enter the child terminal, and
// returning restores the same process, source text, point and split identities.
#[test]
fn terminal_source_retention() {
    for (profile, placement) in [
        ("non-modal", TerminalCommand::Open),
        ("modal", TerminalCommand::NewBuffer),
    ] {
        let repository = Repository::new("sha1");
        let script = repository.fixture.write("terminal.sh", "printf 'TERM_READY\\n'\nwhile IFS= read -r line; do printf 'GOT:%s\\n' \"$line\"; done\n");
        let config = format!(
            "[terminal]\nprogram='/bin/sh'\nargs=[{}]",
            serde_json::to_string(&script).unwrap()
        );
        let mut owned = editor(&repository, profile, &config, true);
        let editor = &mut owned.0;
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let source = text(editor);
        let point = editor.view().point;
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        fs::write(repository.root.join("source.txt"), "staged\n").unwrap();
        repository.git(["add", "source.txt"]);
        editor.execute(Command::Terminal(placement));
        let terminal = editor.terminals().ids().next().unwrap();
        wait(editor, |s| s.terminal_focused());
        let selected_buffer = editor.buffer().unwrap().id();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        plain(editor, 'c');
        editor.handle(Event::Paste("COMPOSER_ONLY_SENTINEL".into()));
        assert_eq!(text(editor), "COMPOSER_ONLY_SENTINEL");
        let _ = render(editor);
        assert!(!editor.terminal_focused());
        assert!(editor.terminals().get(terminal).unwrap().owns_process());
        if profile == "non-modal" {
            key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
            wait(editor, |s| s.echo().contains("Commit created and verified"));
        } else {
            key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        plain(editor, 'q');
        assert!(editor.terminal_focused());
        assert_eq!(editor.buffer().unwrap().id(), selected_buffer);
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
        for c in "ping".chars() {
            plain(editor, c);
        }
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        let mut surface = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !surface.contains("GOT:ping") {
            editor.expire_input(std::time::Instant::now());
            surface = render(editor).join("\n");
            assert!(std::time::Instant::now() < deadline, "{surface}");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(!surface.contains("GOT:COMPOSER_ONLY_SENTINEL"));
        assert!(editor.terminals().get(terminal).unwrap().owns_process());
        if placement == TerminalCommand::NewBuffer {
            editor.execute(Command::Terminal(TerminalCommand::MoveToPanel));
        }
        assert_eq!(text(editor), source);
        assert_eq!(editor.view().point, point);
    }
}

// Hook rejection retains native text and the actual diagnostic. Correcting the
// fixture hook allows one explicit fresh submission; no automatic retry occurs.
#[test]
fn failed_draft_recovery() {
    let repository = Repository::new("sha1");
    let path = repository.root.join("source");
    fs::write(&path, "base\n").unwrap();
    repository.commit();
    fs::write(&path, "staged\n").unwrap();
    repository.git(["add", "source"]);
    let head = repository.git(["rev-parse", "HEAD"]);
    let hook = super::effects::hook(
        &repository,
        "pre-commit",
        "#!/bin/sh\necho NATIVE_HOOK_SENTINEL >&2\nexit 1\n",
    );
    let mut owned = editor(&repository, "modal", "", false);
    let editor = &mut owned.0;
    editor.load(&path).unwrap();
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'c');
    editor.handle(Event::Paste("Retained λ\n\nNative body".into()));
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
    wait(editor, |s| s.echo().contains("draft retained; $"));
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    plain(editor, '$');
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    assert!(render(editor).join("\n").contains("NATIVE_HOOK_SENTINEL"));
    plain(editor, 'q');
    plain(editor, 'q');
    plain(editor, 'c');
    assert_eq!(text(editor), "Retained λ\n\nNative body");
    fs::write(hook, "#!/bin/sh\nexit 0\n").unwrap();
    key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
    wait(editor, |s| s.echo().contains("Commit created and verified"));
    assert_eq!(repository.git(["rev-list", "--count", "HEAD"]), b"2\n");
    assert_eq!(
        repository.git(["show", "--format=%B", "--no-patch", "HEAD"]),
        "Retained λ\n\nNative body\n".as_bytes()
    );
}

// The native byte cap uses UTF-8 bytes rather than scalar count. Rejected edits
// preserve revision/history and retained text. Inner M-x/search cancellation
// keeps the outer draft; ordinary deletion and undo remain native operations.
#[test]
fn bounded_native_editing() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    let oversized = "r".repeat(MAX_COMMIT_BYTES + 1);
    editor.replace(0..0, &oversized).unwrap();
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(RegionCommand::Set));
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Linear(LinearCommand::Copy));
    let source_len = editor.buffer().unwrap().len_chars();
    editor
        .replace(0..source_len, &format!("{oversized}\n{oversized}"))
        .unwrap();
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Rectangle(RectangleCommand::Mark));
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Rectangle(RectangleCommand::Copy));
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    plain(editor, 'c');
    let stamp = editor.buffer().unwrap().stamp();
    editor.execute(Command::Linear(LinearCommand::Yank));
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
    editor.execute(Command::Rectangle(RectangleCommand::Yank));
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
    assert!(editor.replace(0..0, &oversized).is_err());
    let message = "λ".repeat(MAX_COMMIT_BYTES / 2);
    editor.handle(Event::Paste(message.clone()));
    assert_eq!(text(editor).len(), MAX_COMMIT_BYTES);
    let stamp = editor.buffer().unwrap().stamp();
    plain(editor, 'c');
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
    editor.execute(Command::BackwardDeleteChar);
    assert_eq!(text(editor).len(), MAX_COMMIT_BYTES - 2);
    let stamp = editor.buffer().unwrap().stamp();
    plain(editor, '雪');
    assert_eq!(editor.buffer().unwrap().stamp(), stamp);
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(editor), message);
    editor.execute(Command::History(History::Redo));
    assert_eq!(text(editor).len(), MAX_COMMIT_BYTES - 2);
    let retained = text(editor);
    let draft = editor.buffer().unwrap().id();
    for command in [Command::ExecuteExtended, Command::SearchForward] {
        editor.execute(command);
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(!editor.git_visible());
        assert_eq!(editor.buffer().unwrap().id(), draft);
        assert_eq!(text(editor), retained);
    }
    editor.execute(Command::Split(Split::Right));
    assert_eq!(editor.buffer().unwrap().id(), draft);
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert!(editor.git_visible());
    plain(editor, 'c');
    assert_eq!(text(editor), retained);
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
}

// Both review cancellation aliases keep the same accepted draft, including its
// final-newline state. Full SHA-256 target IDs stay inspectable and plain text
// cannot mutate a reviewed message or implicitly accept the request.
#[test]
fn cancelled_review() {
    for cancel in ['q', 'g'] {
        let repository = Repository::new("sha256");
        let mut owned = editor(&repository, "modal", "", true);
        let editor = &mut owned.0;
        let head = repository.git(["rev-parse", "HEAD"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        editor.execute(Command::Git(GitCommand::CommitAmend));
        wait(editor, |s| s.echo().contains("edit message, then C-Enter"));
        editor.execute(Command::Motion(Motion::BufferEnd));
        editor.handle(Event::Paste("Exact review body".into()));
        let accepted = text(editor);
        key(editor, KeyCode::Enter, KeyModifiers::CONTROL);
        assert!(
            render(editor)
                .join("\n")
                .contains(std::str::from_utf8(&head).unwrap().trim())
        );
        plain(editor, 'x');
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        if cancel == 'q' {
            plain(editor, 'q');
        } else {
            key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        assert!(editor.git_visible());
        assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
        plain(editor, 'c');
        assert_eq!(text(editor), accepted);
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    }
}
