use crate::support::Repository;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use rev::{
    buffer::Buffers,
    command::{Command, GitCommand, Split, TerminalCommand, TreeCommand},
    session::Session,
    view::{Size, View},
};
use std::{
    fs,
    time::{Duration, Instant},
};

struct Editor(Session);
impl Drop for Editor {
    fn drop(&mut self) {
        self.0.shutdown_terminals().unwrap();
    }
}
fn editor(repository: &Repository, profile: &str, extra: &str, file: bool) -> Editor {
    let snapshot = repository.snapshot(&format!(
        "[editor]\ninteraction_profile='{profile}'\n{extra}"
    ));
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = if file {
        let path = repository.root.join("source.txt");
        fs::write(&path, "LOCAL_SOURCE_SENTINEL\n").unwrap();
        repository.commit();
        buffers.visit(&path).unwrap()
    } else {
        buffers.initial()
    };
    let view = View::with_settings(buffers.get(id).unwrap(), &snapshot);
    Editor(
        Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap(),
    )
}
fn key(editor: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    editor.handle(Event::Key(KeyEvent::new(code, modifiers)));
}
fn plain(editor: &mut Session, c: char) {
    key(editor, KeyCode::Char(c), KeyModifiers::NONE);
}
fn entry(editor: &mut Session) {
    key(editor, KeyCode::Char('x'), KeyModifiers::CONTROL);
    plain(editor, 'g');
}
fn wait(editor: &mut Session, ready: impl Fn(&Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        if ready(editor) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native Git deadline: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn render(editor: &mut Session) -> Vec<String> {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text().to_owned())
        .collect()
}
fn text(editor: &Session) -> String {
    let buffer = editor.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn selected(editor: &mut Session) -> String {
    render(editor)
        .into_iter()
        .find(|s| s.starts_with('>'))
        .unwrap_or_default()
}
fn click(editor: &mut Session, row: u16) {
    editor.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

// Both profiles use the native key route and full canvas, preserving unsaved
// source text, split identity, point and the original transient editing state.
#[test]
fn native_canvas() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        if profile == "modal" {
            plain(editor, 'i');
        }
        plain(editor, 'x');
        editor.execute(Command::Split(Split::Below));
        let before = text(editor);
        let point = editor.view().point;
        let views: Vec<_> = editor.windows().iter().map(|(id, _)| id).collect();
        let state = editor.editing_state();
        entry(editor);
        assert!(editor.git_visible(), "{profile}: {}", editor.echo());
        wait(editor, |s| s.git_observation().is_some());
        let frame = render(editor).join("\n");
        assert!(frame.contains("Working tree clean"), "{frame}");
        assert!(!frame.contains("LOCAL_SOURCE_SENTINEL"));
        editor.handle(Event::Paste("untrusted text".into()));
        editor.insert('!');
        editor.execute(Command::SaveBuffer);
        assert_eq!(text(editor), before);
        assert_eq!(
            fs::read_to_string(repository.root.join("source.txt")).unwrap(),
            "LOCAL_SOURCE_SENTINEL\n"
        );
        plain(editor, '?');
        assert!(render(editor).join("\n").contains("Git actions"));
        plain(editor, 'q');
        assert!(editor.git_visible(), "q dismisses the innermost menu");
        // Colon in Git navigation is the same native command interface under
        // both startup profiles; it does not enter source insertion or Ex.
        let generation = editor.git_observation().unwrap().generation;
        plain(editor, ':');
        let popup = editor.frame().unwrap();
        assert!(
            popup
                .rows()
                .iter()
                .any(|r| r.band() == rev::display::Band::Completion),
            "native command candidates must remain visible"
        );
        for c in "git-refresh".chars() {
            plain(editor, c);
        }
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        wait(editor, |s| {
            s.git_observation()
                .is_some_and(|o| o.generation != generation)
        });
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert_eq!(text(editor), before);
        assert_eq!(editor.view().point, point);
        assert_eq!(editor.editing_state(), state);
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

// A directory target needs no visiting file. Startup failure restores its tree
// and source identities rather than leaving a fake clean Status presentation.
#[test]
fn directory_and_failure() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", false);
    let editor = &mut owned.0;
    editor.load(&repository.root).unwrap();
    editor.execute(Command::Tree(TreeCommand::Focus));
    let root = editor.tree().root().unwrap().to_owned();
    let buffer = editor.buffer().unwrap().id();
    assert!(editor.tree().focused());
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    assert_eq!(
        editor.git_observation().unwrap().identity.worktree.as_ref(),
        Some(&root)
    );
    plain(editor, 'q');
    assert!(editor.tree().focused());
    assert_eq!(editor.tree().root(), Some(root.as_path()));
    fs::remove_file(&repository.program).unwrap();
    entry(editor);
    wait(editor, |s| !s.git_visible());
    assert!(
        editor.echo().contains("Git unavailable"),
        "{}",
        editor.echo()
    );
    assert!(editor.echo().contains("Start Git"));
    assert_eq!(editor.buffer().unwrap().id(), buffer);
    assert_eq!(editor.tree().root(), Some(root.as_path()));
    assert!(editor.tree().focused());
}

// Selection follows exact path identity on reordered refresh. Mouse hits from
// a prior generation reject before they can redirect to a newly inserted row.
#[test]
fn semantic_pointer() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    for name in ["omega", "zeta"] {
        fs::write(repository.root.join(name), "untracked").unwrap();
    }
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    let frame = render(editor);
    let row = frame.iter().position(|s| s.contains("zeta")).unwrap() as u16;
    click(editor, row);
    assert!(selected(editor).contains("zeta"));
    let generation = editor.git_observation().unwrap().generation;
    fs::write(repository.root.join("alpha"), "new").unwrap();
    editor.execute(Command::Git(GitCommand::Refresh));
    wait(editor, |s| {
        s.git_observation()
            .is_some_and(|o| o.generation != generation)
    });
    click(editor, row);
    assert!(
        editor.echo().contains("redraw before selecting"),
        "{}",
        editor.echo()
    );
    assert!(selected(editor).contains("zeta"));
    let frame = render(editor);
    let omega = frame.iter().position(|s| s.contains("omega")).unwrap() as u16;
    click(editor, omega);
    assert!(selected(editor).contains("omega"));
    key(editor, KeyCode::Tab, KeyModifiers::NONE);
    let frame = render(editor).join("\n");
    assert!(frame.contains("Untracked (3)"));
    assert!(!frame.contains("omega"));
    editor.handle(Event::Resize(12, 8));
    let frame = render(editor);
    assert_eq!(frame.len(), 8);
    assert!(
        frame
            .iter()
            .all(|s| unicode_width::UnicodeWidthStr::width(s.as_str()) == 12)
    );
    assert_eq!(text(editor), "LOCAL_SOURCE_SENTINEL\n");
}

// Hidden terminals retain the same live native resource. Git q never enters
// the child input stream; the restored terminal still handles a later command.
#[test]
fn terminal_retention() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let script = repository.fixture.write("terminal.sh", "printf 'TERM_READY\\n'\nwhile IFS= read -r line; do printf 'GOT:%s\\n' \"$line\"; done\n");
        let config = format!(
            "[terminal]\nprogram='/bin/sh'\nargs=[{}]",
            serde_json::to_string(&script).unwrap()
        );
        let mut owned = editor(&repository, profile, &config, true);
        let editor = &mut owned.0;
        editor.execute(Command::Terminal(TerminalCommand::Open));
        assert!(editor.terminal_focused());
        let id = editor.terminals().ids().next().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !render(editor).join("\n").contains("TERM_READY") {
            editor.expire_input(Instant::now());
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        assert!(editor.git_visible());
        assert!(!editor.terminal_focused());
        assert!(!render(editor).join("\n").contains("TERM_READY"));
        assert!(editor.terminals().get(id).unwrap().owns_process());
        plain(editor, 'q');
        assert!(editor.terminal_focused());
        for c in "ping".chars() {
            plain(editor, c);
        }
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        let deadline = Instant::now() + Duration::from_secs(5);
        while !render(editor).join("\n").contains("GOT:ping") {
            editor.expire_input(Instant::now());
            assert!(
                Instant::now() < deadline,
                "terminal not restored: {:?}",
                render(editor)
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!render(editor).join("\n").contains("GOT:qping"));
        assert!(editor.terminals().get(id).unwrap().owns_process());
    }
}

// A configured source chord keeps its original meaning. Git's own navigation
// overrides use the admitted startup map without editing the hidden source.
#[test]
fn configured_bindings() {
    let repository = Repository::new("sha1");
    let mut owned = editor(
        &repository,
        "non-modal",
        "[keys.editor]\n'C-x g'='forward-char'\n[keys.git]\n'M-j'='next-line'",
        true,
    );
    let editor = &mut owned.0;
    fs::write(repository.root.join("untracked"), "data").unwrap();
    let point = editor.view().point;
    entry(editor);
    assert!(!editor.git_visible());
    assert_ne!(editor.view().point, point);
    key(editor, KeyCode::Char('x'), KeyModifiers::ALT);
    for c in "git-status".chars() {
        plain(editor, c);
    }
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    wait(editor, |s| s.git_observation().is_some());
    key(editor, KeyCode::Char('j'), KeyModifiers::ALT);
    assert!(selected(editor).contains("untracked"));
    let invalid = repository
        .fixture
        .write("invalid.toml", "[keys.git]\n'g'='save-buffer'");
    let startup = rev::config::prepare(&invalid);
    assert!(startup.diagnostic.is_some());
    assert_eq!(
        startup.snapshot.settings(),
        &rev::config::Settings::default()
    );
}

// Display keeps exact layers and escaped path labels. Reached row admission
// stays visible; retained frame projections share the same aggregate budget.
#[test]
fn display_bounds() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    fs::write(repository.root.join("source.txt"), "index content\n").unwrap();
    repository.git(["add", "source.txt"]);
    fs::write(repository.root.join("source.txt"), "worktree content\n").unwrap();
    use std::os::unix::ffi::OsStringExt;
    let path = std::ffi::OsString::from_vec(b"raw\n\x1b\xff".to_vec());
    fs::write(repository.root.join(path), "data").unwrap();
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    let frame = render(editor).join("\n");
    assert!(frame.find("Untracked").unwrap() < frame.find("Unstaged").unwrap());
    assert!(frame.find("Unstaged").unwrap() < frame.find("Staged").unwrap());
    assert!(!frame.contains('\x1b'));
    assert!(frame.contains("raw\\n"));
    for index in 0..1024 {
        fs::write(repository.root.join(format!("path-{index:04}")), "data").unwrap();
    }
    let generation = editor.git_observation().unwrap().generation;
    editor.execute(Command::Git(GitCommand::Refresh));
    wait(editor, |s| {
        s.git_observation()
            .is_some_and(|o| o.generation != generation)
    });
    assert!(render(editor).join("\n").contains("TRUNCATED"));
    editor.handle(Event::Resize(80, 512));
    let retained = editor.frame().unwrap();
    assert_eq!(editor.frame().err(), Some(rev::view::Error::GitCapacity));
    assert!(editor.git_visible());
    drop(retained);
    assert!(editor.frame().is_ok());
    for size in [(0, 0), (0, 4), (1, 1), (1, 2)] {
        editor.handle(Event::Resize(size.0, size.1));
        assert!(editor.frame().is_ok());
    }
}
