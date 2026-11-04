use crate::{
    status::presentation::{click, editor, entry, key, plain, render, selected, text, wait},
    support::Repository,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::{
    command::{Command, Motion, Split, TerminalCommand},
    session::Session,
};
use std::{
    ffi::OsString,
    fs,
    os::unix::ffi::OsStringExt,
    time::{Duration, Instant},
};

fn baseline(repository: &Repository) -> Vec<String> {
    let lines: Vec<_> = (1..=48).map(|n| format!("line {n:02}\n")).collect();
    fs::write(repository.root.join("source.txt"), lines.concat()).unwrap();
    repository.commit();
    lines
}
fn layers(repository: &Repository, mut lines: Vec<String>) -> String {
    lines[3] = "INDEX CONTENT\n".into();
    fs::write(repository.root.join("source.txt"), lines.concat()).unwrap();
    repository.git(["add", "source.txt"]);
    lines[3] = "WORKTREE CONTENT\n".into();
    lines[39] = "DISTANT WORKTREE\n".into();
    let content = lines.concat();
    fs::write(repository.root.join("source.txt"), &content).unwrap();
    content
}
fn command(editor: &mut Session, name: &str) {
    plain(editor, ':');
    for c in name.chars() {
        plain(editor, c);
    }
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
}
fn compared(editor: &mut Session) {
    wait(editor, |s| {
        s.echo().contains("comparison retained as read-only")
    });
}
fn path(editor: &mut Session, group: &str, name: &str) {
    let rows = render(editor);
    let start = rows
        .iter()
        .position(|s| s.contains(&format!("{group} (")))
        .unwrap();
    let row = rows
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, s)| s.contains(name))
        .unwrap()
        .0;
    click(editor, row as u16);
}
fn tab(editor: &mut Session) {
    key(editor, KeyCode::Tab, KeyModifiers::NONE);
}
fn enter(editor: &mut Session) {
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
}

// Both native profiles inspect distinct layers and semantic hunks through real
// keys. Read-only detail and canonical visits preserve modified source owners.
#[test]
fn layers_and_visits() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let lines = baseline(&repository);
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&repository.root.join("source.txt")).unwrap();
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let before = text(editor);
        let point = editor.view().point;
        let buffer = editor.buffer().unwrap().id();
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        let content = layers(&repository, lines);
        let index = repository.git(["rev-parse", ":source.txt"]);
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        path(editor, "Unstaged", "source.txt");
        tab(editor);
        compared(editor);
        let rows = render(editor);
        assert_eq!(rows.iter().filter(|s| s.contains("@@")).count(), 2);
        assert!(
            !rows.iter().any(|s| s.contains("+WORKTREE CONTENT")),
            "hunk body starts folded"
        );
        plain(editor, 'n');
        assert!(selected(editor).contains("@@"));
        tab(editor);
        assert!(render(editor).join("\n").contains("+WORKTREE CONTENT"));
        command(editor, "git-diff");
        let detail = render(editor).join("\n");
        assert!(detail.contains("Git Diff: Unstaged"), "{detail}");
        assert!(detail.contains("-INDEX CONTENT"));
        assert!(detail.contains("+WORKTREE CONTENT"));
        editor.insert('!');
        editor.handle(Event::Paste("cannot edit detail".into()));
        editor.execute(Command::SaveBuffer);
        assert_eq!(text(editor), before);
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(editor.echo().contains("owned work retained"));
        plain(editor, '$');
        assert!(render(editor).join("\n").contains("Diff comparison"));
        enter(editor);
        assert!(render(editor).join("\n").contains("Git Process:"));
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(render(editor).join("\n").contains("Git Diff: Unstaged"));
        plain(editor, 'q');
        assert!(editor.git_visible());
        editor.execute(Command::Motion(Motion::BufferEnd));
        assert!(selected(editor).contains("source.txt"));
        command(editor, "git-diff");
        compared(editor);
        let staged = render(editor).join("\n");
        assert!(staged.contains("Git Diff: Staged"), "{staged}");
        assert!(staged.contains("+INDEX CONTENT"));
        assert!(!staged.contains("+WORKTREE CONTENT"));
        enter(editor);
        assert!(!editor.git_visible(), "{}", editor.echo());
        assert_eq!(editor.buffer().unwrap().id(), buffer);
        assert!(editor.buffer().unwrap().is_modified());
        assert_eq!(text(editor), before);
        assert_eq!(editor.view().point, point);
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
        assert_eq!(
            fs::read_to_string(repository.root.join("source.txt")).unwrap(),
            content
        );
        assert_eq!(repository.git(["rev-parse", ":source.txt"]), index);
    }
}

// A refreshed exact hunk keeps selection, but changed patch bytes fall back to
// the same path. Old comparison and resize receipts never redirect mouse hits.
#[test]
fn refreshed_hunk() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let lines = baseline(&repository);
        let content = layers(&repository, lines);
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&repository.root.join("source.txt")).unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        path(editor, "Unstaged", "source.txt");
        tab(editor);
        compared(editor);
        let rows = render(editor);
        let row = rows.iter().rposition(|s| s.contains("@@")).unwrap() as u16;
        click(editor, row);
        let hunk = selected(editor);
        plain(editor, 'g');
        compared(editor);
        click(editor, row);
        assert!(editor.echo().contains("redraw before selecting"));
        assert_eq!(selected(editor), hunk);
        editor.handle(Event::Resize(36, 12));
        click(editor, row.min(8));
        assert!(editor.echo().contains("redraw before selecting"));
        let resized = render(editor);
        assert_eq!(resized.len(), 12);
        assert!(
            resized
                .iter()
                .all(|s| unicode_width::UnicodeWidthStr::width(s.as_str()) == 36)
        );
        editor.handle(Event::Resize(80, 24));
        render(editor);
        fs::write(
            repository.root.join("source.txt"),
            content.replace("DISTANT WORKTREE", "CHANGED DISTANT"),
        )
        .unwrap();
        plain(editor, 'g');
        compared(editor);
        let selected = selected(editor);
        assert!(selected.contains("source.txt"), "{selected}");
        assert!(!selected.contains("@@"));
        for (width, height) in [(0, 0), (0, 4), (1, 1), (1, 2)] {
            editor.handle(Event::Resize(width, height));
            assert!(editor.frame().is_ok());
        }
    }
}

// Exact invalid-UTF-8 and pathspec-looking paths visit their real native file.
// A deleted current file remains inspection-only and cannot create a buffer.
#[test]
fn exact_current_sources() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        let exact = OsString::from_vec(b"raw\n\xff".to_vec());
        let current = repository.root.join(&exact);
        fs::write(&current, "exact source\n").unwrap();
        fs::write(repository.root.join(":(glob)**"), "different source\n").unwrap();
        editor.load(&repository.root).unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        path(editor, "Untracked", "raw\\n");
        command(editor, "git-diff");
        compared(editor);
        let detail = render(editor).join("\n");
        assert!(detail.contains("Baseline: Empty"));
        assert!(detail.contains("+exact source"));
        enter(editor);
        assert!(!editor.git_visible());
        assert_eq!(text(editor), "exact source\n");
        assert_eq!(editor.buffer().unwrap().file().unwrap().path(), current);
        let buffer = editor.buffer().unwrap().id();
        repository.commit();
        fs::remove_file(&current).unwrap();
        entry(editor);
        let generation = editor.git_observation().map(|o| o.generation);
        wait(editor, |s| {
            s.git_observation()
                .is_some_and(|o| Some(o.generation) != generation)
        });
        path(editor, "Unstaged", "raw\\n");
        enter(editor);
        assert!(editor.git_visible());
        assert!(editor.echo().contains("no current source file"));
        assert_eq!(editor.buffer().unwrap().id(), buffer);
        assert_eq!(text(editor), "exact source\n");
    }
}

// Diff q restores the retained terminal and split owners, and a later source
// visit changes focus through file ingress without stopping the terminal job.
#[test]
fn terminal_return_and_visit() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let lines = baseline(&repository);
        layers(&repository, lines);
        let script = repository.fixture.write(
            "terminal.sh",
            "printf 'READY\\n'\nwhile IFS= read -r line; do printf 'GOT:%s\\n' \"$line\"; done\n",
        );
        let config = format!(
            "[terminal]\nprogram='/bin/sh'\nargs=[{}]",
            serde_json::to_string(&script).unwrap()
        );
        let mut owned = editor(&repository, profile, &config, false);
        let editor = &mut owned.0;
        editor.load(&repository.root.join("source.txt")).unwrap();
        editor.execute(Command::Split(Split::Below));
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        editor.execute(Command::Terminal(TerminalCommand::Open));
        let id = editor.terminals().ids().next().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !render(editor).join("\n").contains("READY") {
            editor.expire_input(Instant::now());
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        path(editor, "Unstaged", "source.txt");
        command(editor, "git-diff");
        compared(editor);
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(editor.terminal_focused());
        assert!(editor.terminals().get(id).unwrap().owns_process());
        entry(editor);
        let generation = editor.git_observation().map(|o| o.generation);
        wait(editor, |s| {
            s.git_observation()
                .is_some_and(|o| Some(o.generation) != generation)
        });
        path(editor, "Unstaged", "source.txt");
        enter(editor);
        assert!(!editor.git_visible());
        assert!(!editor.terminal_focused());
        assert!(editor.terminals().get(id).unwrap().owns_process());
        assert_eq!(
            editor
                .windows()
                .iter()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            views
        );
        assert!(text(editor).contains("WORKTREE CONTENT"));
    }
}

// Failed replacement retains visibly stale detail. Leaving a refreshing diff
// is a navigation decision: completion updates Status without reopening it.
#[test]
fn failed_and_hidden_detail() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let lines = baseline(&repository);
        layers(&repository, lines);
        let mut owned = editor(&repository, profile, "", false);
        let editor = &mut owned.0;
        editor.load(&repository.root.join("source.txt")).unwrap();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        path(editor, "Unstaged", "source.txt");
        command(editor, "git-diff");
        compared(editor);
        let original = fs::read(&repository.program).unwrap();
        fs::write(&repository.program, "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$3\" = diff ]; then echo 'Fixture diff failure' >&2; exit 70; fi\nexec /usr/bin/git \"$@\"\n").unwrap();
        plain(editor, 'g');
        wait(editor, |s| s.echo().contains("comparison failed"));
        let retained = render(editor).join("\n");
        assert!(retained.contains("Git Diff: Unstaged"));
        assert!(retained.contains("STALE / unverified"));
        assert!(retained.contains("+WORKTREE CONTENT"));
        let marker = repository.fixture.0.join("comparing");
        let marker = format!("'{}'", marker.to_str().unwrap().replace('\'', "'\\''"));
        fs::write(&repository.program, format!("#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$3\" = diff ]; then touch {marker}; sleep 0.2; fi\nexec /usr/bin/git \"$@\"\n")).unwrap();
        plain(editor, 'g');
        wait(editor, |_| repository.fixture.0.join("comparing").exists());
        plain(editor, 'q');
        assert!(render(editor).join("\n").contains("Git Status"));
        compared(editor);
        let returned = render(editor).join("\n");
        assert!(!returned.contains("Git Diff:"), "{returned}");
        assert!(returned.contains("Git Status"));
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert!(text(editor).contains("WORKTREE CONTENT"));
        fs::write(&repository.program, original).unwrap();
    }
}
