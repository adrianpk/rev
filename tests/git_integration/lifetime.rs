use super::*;
use rev::session::Action;
use std::path::PathBuf;

fn answer(editor: &mut Session, answer: char) -> Action {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(answer),
        KeyModifiers::NONE,
    )))
}
fn quoted(path: &std::path::Path) -> String {
    format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}
fn hold(repository: &Repository) -> PathBuf {
    let pid = repository.fixture.0.join("git-pid");
    let held = repository.fixture.write("hold", "held");
    fs::write(&repository.program, format!(
        "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$2\" = status ] && [ -f {} ]; then\n trap '' TERM\n echo $$ > {}\n while [ -f {} ]; do sleep 0.02; done\nfi\nexec /usr/bin/git \"$@\"\n",
        quoted(&held), quoted(&pid), quoted(&held)
    )).unwrap();
    pid
}
fn start_held(editor: &mut Session, pid_file: &std::path::Path) -> PathBuf {
    let _ = fs::remove_file(pid_file);
    editor.execute(Command::Git(GitCommand::Refresh));
    wait(editor, |_| pid_file.exists());
    let pid = fs::read_to_string(pid_file).unwrap();
    let process = PathBuf::from(format!("/proc/{}", pid.trim()));
    assert!(process.exists());
    process
}
fn wait_surface(editor: &mut Session, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        let surface = render(editor).join("\n");
        if surface.contains(needle) {
            return surface;
        }
        assert!(
            Instant::now() < deadline,
            "missing {needle}: {surface}\n{}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

// Delayed real Git proves view exit, inspection, menus and prompt cancellation
// preserve the same child. Only an explicit selected operation can be stopped;
// a completed record cannot redirect C-g to a replacement in the same context.
#[test]
fn selected_cancellation() {
    for profile in ["non-modal", "modal"] {
        let repository = Repository::new("sha1");
        let mut owned = editor(&repository, profile, "", true);
        let editor = &mut owned.0;
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        let pid_file = hold(&repository);
        let process = start_held(editor, &pid_file);
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert!(process.exists());
        editor.execute(Command::Git(GitCommand::ProcessCancel));
        assert!(editor.echo().contains("Select an owned operation"));
        assert!(process.exists());
        editor.execute(Command::Git(GitCommand::Processes));
        assert!(selected(editor).contains("Refreshing"));
        key(editor, KeyCode::Enter, KeyModifiers::NONE);
        assert!(render(editor).join("\n").contains("worker owns capture"));
        key(editor, KeyCode::Esc, KeyModifiers::NONE);
        editor.expire_input(Instant::now() + Duration::from_millis(150));
        assert!(
            process.exists(),
            "Escape is input dismissal, not cancellation authority"
        );
        key(editor, KeyCode::Char('x'), KeyModifiers::ALT);
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(
            process.exists(),
            "prompt cancellation must not stop its outer job"
        );
        plain(editor, 'q');
        plain(editor, 'q');
        assert!(!editor.git_visible());
        assert!(process.exists());
        editor.execute(Command::Git(GitCommand::Processes));
        plain(editor, '?');
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(process.exists(), "C-g dismisses the innermost menu");
        let started = Instant::now();
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        wait_surface(editor, "Cancelled");
        assert!(started.elapsed() < Duration::from_secs(3));
        assert!(!process.exists(), "the group leader must be reaped");
        let process = start_held(editor, &pid_file);
        key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(editor.echo().contains("no longer active"));
        assert!(
            process.exists(),
            "old selection must not retarget the replacement"
        );
        key(editor, KeyCode::Up, KeyModifiers::NONE);
        assert!(selected(editor).contains("Refreshing"));
        editor.execute(Command::Git(GitCommand::ProcessCancel));
        wait(editor, |_| !process.exists());
        wait_surface(editor, "Cancelled");
        assert_eq!(text(editor), "LOCAL_SOURCE_SENTINEL\n");
    }
}

// Git consent is captured before terminal shutdown. Both decline routes retain
// the active job and unsaved source; accepting both waits for bounded cleanup.
#[test]
fn guarded_exit() {
    for (profile, live_terminal) in [("non-modal", true), ("modal", true), ("non-modal", false)] {
        let repository = Repository::new("sha1");
        let script = repository.fixture.write(
            "terminal.sh",
            "while IFS= read -r line; do printf '%s\\n' \"$line\"; done\n",
        );
        let extra = format!(
            "[terminal]\nprogram='/bin/sh'\nargs=[{}]",
            serde_json::to_string(&script).unwrap()
        );
        let mut owned = editor(&repository, profile, &extra, true);
        let editor = &mut owned.0;
        editor.insert('!');
        let source = text(editor);
        if live_terminal {
            editor.execute(Command::Terminal(TerminalCommand::Open));
        }
        let terminal = editor.terminals().ids().next();
        entry(editor);
        wait(editor, |s| s.git_observation().is_some());
        let pid_file = hold(&repository);
        let process = start_held(editor, &pid_file);
        for cancel in [false, true] {
            assert_eq!(editor.execute(Command::Quit), Action::Redraw);
            assert!(editor.echo().contains("y=save"));
            answer(editor, 'n');
            assert!(editor.echo().contains("terminate Git Status observations"));
            key(editor, KeyCode::Enter, KeyModifiers::NONE);
            editor.handle(Event::Paste("y".into()));
            assert!(
                process.exists(),
                "Return/paste cannot grant termination consent"
            );
            if cancel {
                key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
            } else {
                answer(editor, 'n');
            }
            assert!(process.exists());
            if let Some(id) = terminal {
                assert!(editor.terminals().get(id).unwrap().owns_process());
            }
            assert_eq!(text(editor), source);
            assert!(editor.buffer().unwrap().is_modified());
        }
        if let Some(id) = terminal {
            editor.execute(Command::Quit);
            answer(editor, 'n');
            answer(editor, 'y');
            assert!(editor.echo().contains("terminal sessions"));
            assert!(process.exists(), "Git consent alone does not begin cleanup");
            answer(editor, 'n');
            assert!(process.exists());
            assert!(editor.terminals().get(id).unwrap().owns_process());
        }
        editor.execute(Command::Quit);
        answer(editor, 'n');
        answer(editor, 'y');
        if terminal.is_some() {
            answer(editor, 'y');
        }
        if !live_terminal {
            // Exit cancellation after consent cannot undo already requested
            // cleanup, but must prevent the coordinator from later exiting.
            key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
            let deadline = Instant::now() + Duration::from_secs(5);
            while process.exists() {
                assert_ne!(editor.expire_input(Instant::now()), Action::Exit);
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
            assert_ne!(editor.expire_input(Instant::now()), Action::Exit);
            assert!(editor.buffer().unwrap().is_modified());
            continue;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if editor.expire_input(Instant::now()) == Action::Exit {
                break;
            }
            assert!(Instant::now() < deadline, "exit cleanup: {}", editor.echo());
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!process.exists());
        assert!(editor.terminals().ids().next().is_none());
        assert_eq!(text(editor), source);
        assert_eq!(
            fs::read_to_string(repository.root.join("source.txt")).unwrap(),
            "LOCAL_SOURCE_SENTINEL\n"
        );
    }
}

// Failed stderr remains inspectable while confirmed Status survives. Raw VT,
// invalid UTF-8 and credential-bearing lines cannot become terminal controls or
// public process text, even when Git supplies those bytes itself.
#[test]
fn retained_diagnostics() {
    let repository = Repository::new("sha1");
    let mut owned = editor(&repository, "non-modal", "", true);
    let editor = &mut owned.0;
    entry(editor);
    wait(editor, |s| s.git_observation().is_some());
    let generation = editor.git_observation().unwrap().generation;
    fs::write(&repository.program, "#!/bin/sh\nif [ \"$2\" = status ]; then\n printf 'fatal diagnostic sentinel\\nfatal https://user:fixture-secret@example.invalid/repo\\nAuthorization: Bearer fixture_token\\n\\033[31mraw\\377\\n' >&2\n exit 7\nfi\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nexec /usr/bin/git \"$@\"\n").unwrap();
    editor.execute(Command::Git(GitCommand::Refresh));
    wait_surface(editor, "STALE");
    assert_eq!(editor.git_observation().unwrap().generation, generation);
    plain(editor, '$');
    assert!(selected(editor).contains("Failed"));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    editor.handle(Event::Resize(160, 40));
    let surface = render(editor).join("\n");
    assert!(surface.contains("exit status: 7"), "{surface}");
    assert!(surface.contains("fatal diagnostic sentinel"));
    assert!(surface.contains("diagnostic line redacted"));
    assert!(!surface.contains("fixture-secret"));
    assert!(!surface.contains("fixture_token"));
    assert!(!surface.contains('\x1b'));
    assert!(surface.contains("\\u{1b}[31mraw\\xFF"), "{surface}");
    plain(editor, 'q');
    plain(editor, 'q');
    assert!(render(editor).join("\n").contains("STALE"));
    assert_eq!(text(editor), "LOCAL_SOURCE_SENTINEL\n");
}
