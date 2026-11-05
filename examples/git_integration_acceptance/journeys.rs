use crate::{repository::Repository, support::*};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::{
    command::{Command, Motion, Split, TerminalCommand},
    session::Session,
};
use std::{fs, process::Command as Process};

pub(crate) fn run() -> Result<()> {
    for (profile, format) in [("non-modal", "sha1"), ("modal", "sha256")] {
        index_commit(profile, format)?;
        refs_transport(profile, format)?;
        conflict_sequence(profile, format)?;
        println!(
            "PASS combined native profile={profile}: hunks/index/commit, local refs/fetch/push, exact conflict sides/replacement/abort and retained live terminal"
        );
    }
    Ok(())
}

fn index_commit(profile: &str, format: &str) -> Result<()> {
    let repository = Repository::new(format);
    let base: Vec<_> = (1..=48).map(|n| format!("line {n:02}\n")).collect();
    fs::write(repository.root.join("source.txt"), base.concat())?;
    repository.commit();
    let mut changed = base.clone();
    changed[3] = "first change\n".into();
    changed[39] = "second change\n".into();
    fs::write(repository.root.join("source.txt"), changed.concat())?;
    let mut owned = editor(&repository, profile)?;
    let editor = &mut owned.0;
    editor.execute(Command::Split(Split::Below));
    let windows: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
    let point = editor.view().point;
    let source = text(editor);
    editor.execute(Command::Terminal(TerminalCommand::OpenHere));
    editor.execute(Command::Terminal(TerminalCommand::MoveToBuffer));
    wait(editor, |s| surface(s).contains("TERM_READY"));
    let terminal = editor.terminals().ids().next().unwrap();
    let buffer = editor.buffer()?.id();
    let pid = editor.terminals().get(terminal).unwrap().process_id();
    named(editor, "git-status");
    wait(editor, |s| s.git_observation().is_some());
    assert_eq!(
        editor.git_observation().unwrap().identity.worktree.as_ref(),
        Some(&repository.root)
    );
    plain(editor, 'n');
    key(editor, KeyCode::Tab, KeyModifiers::NONE);
    wait(editor, |s| {
        s.echo().contains("comparison retained as read-only")
    });
    plain(editor, 'n');
    plain(editor, 's');
    wait(editor, |s| s.echo().contains("Index updated and verified"));
    let mut first = base.clone();
    first[3] = changed[3].clone();
    assert_eq!(
        repository.git(["show", ":source.txt"]),
        first.concat().as_bytes()
    );
    assert_eq!(
        fs::read(repository.root.join("source.txt"))?,
        changed.concat().as_bytes()
    );
    named(editor, "git-quit");
    named(editor, "git-status");
    named(editor, "git-refresh");
    wait(editor, |s| s.echo().contains("Git Status refreshed"));
    editor.execute(Command::Motion(Motion::BufferStart));
    plain(editor, 'n');
    plain(editor, 's');
    wait(editor, |s| s.echo().contains("Index updated and verified"));
    assert_eq!(
        repository.git(["show", ":source.txt"]),
        changed.concat().as_bytes()
    );
    compose(editor)?;
    assert_eq!(
        repository.git(["log", "-1", "--format=%B"]),
        b"cc q NATIVE_COMMIT_ONLY\n"
    );
    assert_eq!(
        repository.git(["show", "HEAD:source.txt"]),
        changed.concat().as_bytes()
    );
    assert!(repository.git(["status", "--porcelain"]).is_empty());
    named(editor, "git-quit");
    assert!(editor.terminal_focused());
    assert_eq!(editor.buffer()?.id(), buffer);
    assert_eq!(editor.terminals().get(terminal).unwrap().process_id(), pid);
    editor.handle(Event::Paste("ping".into()));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |s| surface(s).contains("GOT:ping"));
    assert!(!surface(editor).contains("GOT:cc q NATIVE_COMMIT_ONLY"));
    editor.execute(Command::Terminal(TerminalCommand::MoveToPanel));
    assert_eq!(text(editor), source);
    assert_eq!(editor.view().point, point);
    assert_eq!(
        editor
            .windows()
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        windows
    );
    Ok(())
}

fn compose(editor: &mut Session) -> Result<()> {
    plain(editor, 'c');
    editor.handle(Event::Paste("cc q NATIVE_COMMIT_ONLY".into()));
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    plain(editor, 'c');
    assert_eq!(text(editor), "cc q NATIVE_COMMIT_ONLY");
    accept(editor);
    wait(editor, |s| s.echo().contains("Commit created and verified"));
    Ok(())
}

fn refs_transport(profile: &str, format: &str) -> Result<()> {
    let repository = Repository::new(format);
    fs::write(repository.root.join("source.txt"), "base\n")?;
    repository.commit();
    let endpoint = repository.fixture.0.join("origin.git");
    repository.git([
        "clone",
        "--bare",
        repository.root.to_str().unwrap(),
        endpoint.to_str().unwrap(),
    ]);
    repository.git(["remote", "add", "origin", endpoint.to_str().unwrap()]);
    repository.git(["fetch", "origin"]);
    repository.git(["branch", "--set-upstream-to=origin/main", "main"]);
    let base = repository.git(["rev-parse", "HEAD"]);
    repository.commit();
    let tip = repository.git(["rev-parse", "HEAD"]);
    let mut owned = editor(&repository, profile)?;
    let editor = &mut owned.0;
    named(editor, "git-status");
    wait(editor, |s| s.git_observation().is_some());
    assert_eq!(
        editor.git_observation().unwrap().identity.worktree.as_ref(),
        Some(&repository.root)
    );
    plain(editor, 'b');
    wait(editor, |s| s.echo().contains("Refs and Status checked"));
    plain(editor, '*');
    editor.execute(Command::Motion(Motion::BufferEnd));
    plain(editor, 'M');
    plain(editor, 'f');
    wait(editor, |s| s.echo().contains("Fetch completed"));
    assert_eq!(repository.git(["rev-parse", "HEAD"]), tip);
    plain(editor, 'M');
    plain(editor, 'p');
    wait(editor, |s| s.echo().starts_with("Push refs/heads/main at"));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(
        repository.git(["--git-dir", endpoint.to_str().unwrap(), "rev-parse", "main"]),
        base
    );
    accept(editor);
    wait(editor, |s| s.echo().contains("Push verified"));
    assert_eq!(
        repository.git(["--git-dir", endpoint.to_str().unwrap(), "rev-parse", "main"]),
        tip
    );
    assert_eq!(repository.git(["show", "HEAD:source.txt"]), b"base\n");
    Ok(())
}

fn conflict_sequence(profile: &str, format: &str) -> Result<()> {
    let repository = Repository::new(format);
    fs::write(repository.root.join("source.txt"), "base\n")?;
    repository.commit();
    repository.git(["switch", "-c", "topic"]);
    fs::write(repository.root.join("source.txt"), "applied\n")?;
    repository.commit();
    repository.git(["switch", "main"]);
    fs::write(repository.root.join("source.txt"), "current\n")?;
    repository.commit();
    let head = repository.git(["rev-parse", "HEAD"]);
    let mut owned = editor(&repository, profile)?;
    let editor = &mut owned.0;
    assert!(
        !Process::new(&repository.program)
            .args(["merge", "--no-edit", "topic"])
            .current_dir(&repository.root)
            .output()?
            .status
            .success()
    );
    let index = repository.git(["ls-files", "--stage", "-z"]);
    named(editor, "git-status");
    wait(editor, |s| s.git_observation().is_some());
    assert_eq!(
        editor.git_observation().unwrap().identity.worktree.as_ref(),
        Some(&repository.root)
    );
    named(editor, "git-conflicts");
    wait(editor, |s| {
        s.echo().contains("Git-owned sequence state checked")
    });
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |s| {
        s.echo().contains("Checked conflict sides retained")
    });
    plain(editor, '3');
    wait(editor, |s| {
        s.echo().contains("Confirm replace result with Applied")
    });
    accept(editor);
    wait(editor, |s| s.echo().contains("Conflict action verified"));
    assert_eq!(fs::read(repository.root.join("source.txt"))?, b"applied\n");
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    named(editor, "git-operation-abort");
    wait(editor, |s| s.echo().contains("Confirm abort"));
    accept(editor);
    wait(editor, |s| s.echo().contains("sequence aborted"));
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(fs::read(repository.root.join("source.txt"))?, b"current\n");
    assert!(!repository.root.join(".git/MERGE_HEAD").exists());
    Ok(())
}
