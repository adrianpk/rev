use super::*;
use rev::term::State;

fn command(editor: &mut Editor, terminal: TerminalCommand) {
    editor.session.execute(Command::Terminal(terminal));
}
fn switch(editor: &mut Editor, name: &str) {
    editor
        .session
        .execute(Command::Buffer(BufferCommand::Switch));
    editor.session.handle(Event::Paste(name.into()));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
}
fn close_current(editor: &mut Editor) {
    editor.session.execute(Command::Buffer(BufferCommand::Kill));
    editor
        .session
        .handle(key(KeyCode::Enter, KeyModifiers::NONE));
}

// Detaching the final view retains the child. Actual exit keeps its final
// output/status, a new default leaves it inspectable, and only closure frees
// capacity. All observations cross the native buffer and real PTY owners.
#[test]
fn retained_exit() {
    if isolated("panel::lifecycle::retained_exit", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        retained_exit_case(profile);
    }
}
fn retained_exit_case(profile: &str) {
    let script = r#"stty raw -echo
printf READY
while [ ! -f "leave-$$" ]; do sleep 0.01; done
printf '\r\nFINAL_OUTPUT'
exit 7
"#;
    let mut editor = Editor::with_script(profile, 24, "max_sessions=2", script);
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::NewBuffer);
    editor.wait(|s| text(s).contains("READY"));
    let first = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let buffer = editor.session.terminal_buffer().unwrap().id();
    let pid = editor
        .session
        .terminals()
        .get(first)
        .unwrap()
        .process_id()
        .unwrap();
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert_eq!(
        *editor.session.terminals().get(first).unwrap().state(),
        State::Running
    );
    assert!(
        editor
            .session
            .terminals()
            .get(first)
            .unwrap()
            .owns_process()
    );
    editor
        .fixture
        .write(&format!("source/leave-{pid}"), "ready");
    editor.wait(|s| matches!(s.state(), State::Exited(_)));
    let exited = editor.session.terminals().get(first).unwrap();
    assert!(matches!(exited.state(),State::Exited(status) if status.code()==Some(7)));
    assert!(!exited.owns_process());
    assert_eq!(exited.process_id(), None);
    assert!(text(exited).contains("FINAL_OUTPUT"));
    assert!(!text(exited).contains("Exited (7)"));
    switch(&mut editor, "terminal");
    assert_eq!(editor.session.terminal_buffer().unwrap().id(), buffer);
    assert_eq!(editor.session.terminal_buffer().unwrap().len_bytes(), 0);
    assert!(
        editor
            .session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("Exited (7)"))
    );
    command(&mut editor, TerminalCommand::CopyMode);
    assert!(
        editor
            .session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("[COPY] Exited (7)"))
    );
    command(&mut editor, TerminalCommand::Open);
    let second = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    assert_ne!(first, second);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::New);
    assert!(
        editor.session.echo().contains("capacity"),
        "{}",
        editor.session.echo()
    );
    assert_eq!(editor.session.terminals().ids().count(), 2);
    assert!(text(editor.session.terminals().get(first).unwrap()).contains("FINAL_OUTPUT"));
    switch(&mut editor, "terminal");
    assert_eq!(editor.session.terminal_buffer().unwrap().id(), buffer);
    close_current(&mut editor);
    assert!(!editor.session.echo().starts_with("Terminate"));
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.session.terminals().get(first).is_some() {
        editor.session.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "{}", editor.session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(editor.session.terminals().ids().count(), 1);
    assert!(editor.session.terminal_copy_view().is_none());
    command(&mut editor, TerminalCommand::Open);
    assert_eq!(
        editor.session.terminal_buffer().unwrap().terminal(),
        Some(second)
    );
    command(&mut editor, TerminalCommand::New);
    assert_eq!(editor.session.terminals().ids().count(), 2);
}

fn answer(editor: &mut Editor, yes: bool) -> rev::session::Action {
    editor.session.handle(key(
        KeyCode::Char(if yes { 'y' } else { 'n' }),
        KeyModifiers::NONE,
    ))
}
fn cancel(editor: &mut Editor) {
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
}
fn begin_exit(editor: &mut Editor) {
    assert_eq!(
        editor.session.execute(Command::Quit),
        rev::session::Action::Redraw
    );
    assert!(
        editor.session.echo().contains("n=discard"),
        "{}",
        editor.session.echo()
    );
    answer(editor, false);
    assert!(
        editor
            .session
            .echo()
            .contains("terminate 2 terminal sessions"),
        "{}",
        editor.session.echo()
    );
}
fn two_live(profile: &str) -> Editor {
    let script = "trap '' HUP\nset -m\nsleep 60 & printf '%s' $! > job-$$\nstty raw -echo\nprintf READY\nwhile :; do read -r line; printf ACK; done\n";
    let mut editor = Editor::with_script(profile, 24, "", script);
    editor.session.insert('!');
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::New);
    editor.wait(|s| text(s).contains("READY"));
    cancel(&mut editor);
    editor
}
fn live_pids(editor: &Editor) -> Vec<u32> {
    editor
        .session
        .terminals()
        .ids()
        .map(|id| {
            editor
                .session
                .terminals()
                .get(id)
                .unwrap()
                .process_id()
                .unwrap()
        })
        .collect()
}
fn assert_live(editor: &Editor, pids: &[u32]) {
    assert_eq!(live_pids(editor), pids);
    for id in editor.session.terminals().ids() {
        assert_eq!(
            *editor.session.terminals().get(id).unwrap().state(),
            State::Running
        );
    }
}

// A declined aggregate decision preserves both owned groups and dirty source.
// Acceptance submits cleanup once, never blocks the UI, and returns Exit only
// after every terminal receipt, buffer closure and actual leader reap completes.
#[test]
fn shutdown() {
    if isolated("panel::lifecycle::shutdown", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = two_live(profile);
        let source = editor.session.buffer().unwrap().stamp();
        let pids = live_pids(&editor);
        for decline in [false, true] {
            begin_exit(&mut editor);
            if decline {
                cancel(&mut editor);
            } else {
                answer(&mut editor, false);
            }
            assert_live(&editor, &pids);
            assert_eq!(editor.session.buffer().unwrap().stamp(), source);
            assert!(editor.session.buffer().unwrap().is_modified());
        }
        begin_exit(&mut editor);
        let start = Instant::now();
        assert_ne!(answer(&mut editor, true), rev::session::Action::Exit);
        assert!(
            start.elapsed() < Duration::from_millis(100),
            "UI close blocked"
        );
        assert!(editor.session.echo().contains("cleanup pending"));
        for id in editor.session.terminals().ids() {
            assert_eq!(
                *editor.session.terminals().get(id).unwrap().state(),
                State::Closing
            );
        }
        editor
            .session
            .handle(key(KeyCode::Char('z'), KeyModifiers::NONE));
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        loop {
            // Rendering may consume receipts, but must not consume the pending
            // native Exit action before the host event owner sees it.
            editor.session.frame().unwrap();
            if editor.session.expire_input(Instant::now()) == rev::session::Action::Exit {
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(3),
                "{}",
                editor.session.echo()
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(start.elapsed() >= Duration::from_millis(500), "HUP grace");
        assert_eq!(editor.session.terminals().ids().count(), 0);
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        assert!(editor.session.buffer().unwrap().is_modified());
        for pid in pids {
            assert!(
                !std::fs::exists(format!("/proc/{pid}")).unwrap(),
                "leader retained"
            );
            let job: u32 =
                std::fs::read_to_string(editor.fixture.0.join(format!("source/job-{pid}")))
                    .unwrap()
                    .parse()
                    .unwrap();
            assert!(
                super::super::resources::stopped(job),
                "owned background group survived"
            );
        }
    }
}

// Discard permission belongs to the exact revision. A new source publication
// before acceptance preserves every process; one during approved cleanup keeps
// the editor open and retains the new dirty source while that cleanup finishes.
#[test]
fn source_revision() {
    if isolated("panel::lifecycle::source_revision", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = two_live(profile);
        let pids = live_pids(&editor);
        begin_exit(&mut editor);
        editor.session.insert('X');
        let changed = editor.session.buffer().unwrap().stamp();
        assert_ne!(answer(&mut editor, true), rev::session::Action::Exit);
        assert!(editor.session.echo().contains("Source changed"));
        assert_live(&editor, &pids);
        assert_eq!(editor.session.buffer().unwrap().stamp(), changed);
        begin_exit(&mut editor);
        answer(&mut editor, true);
        editor.session.insert('Y');
        let latest = editor.session.buffer().unwrap().stamp();
        let start = Instant::now();
        while editor.session.terminals().ids().next().is_some() {
            assert_ne!(
                editor.session.expire_input(Instant::now()),
                rev::session::Action::Exit
            );
            assert!(start.elapsed() < Duration::from_secs(3));
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_ne!(
            editor.session.expire_input(Instant::now()),
            rev::session::Action::Exit
        );
        assert_eq!(editor.session.buffer().unwrap().stamp(), latest);
        assert!(editor.session.buffer().unwrap().is_modified());
    }
}

// Both native decline routes release the strong host prompt. Reopening and
// writing to the same live child proves C-g did not leave a stale close owner.
#[test]
fn declined_close() {
    if isolated("panel::lifecycle::declined_close", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::new(profile, 24);
        editor.open();
        let id = editor.session.terminals().ids().next().unwrap();
        let pid = editor.session.terminals().get(id).unwrap().process_id();
        for interrupt in [false, true] {
            close_current(&mut editor);
            assert!(editor.session.echo().contains("Terminate"));
            if interrupt {
                cancel(&mut editor);
            } else {
                answer(&mut editor, false);
            }
            command(&mut editor, TerminalCommand::Open);
            assert!(
                editor.session.terminal_focused(),
                "{}",
                editor.session.echo()
            );
            assert_eq!(
                editor.session.terminals().get(id).unwrap().process_id(),
                pid
            );
            editor.send("ping");
            editor.wait(|s| text(s).contains("PONG"));
        }
    }
}
