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

// Shell completion closes either placement, restores source and frees capacity.
// A background exit must keep a pending buffer query usable and up to date.
#[test]
fn automatic_exit() {
    if isolated("panel::lifecycle::automatic_exit", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        for (placement, background) in [
            (TerminalCommand::Open, false),
            (TerminalCommand::NewBuffer, false),
            (TerminalCommand::Open, true),
            (TerminalCommand::NewBuffer, true),
        ] {
            automatic_exit_case(profile, placement, background);
        }
    }
}
fn automatic_exit_case(profile: &str, placement: TerminalCommand, background: bool) {
    let script = r#"stty raw -echo
printf READY
while [ ! -f "leave-$$" ]; do sleep 0.01; done
printf '\r\nFINAL_OUTPUT'
exit 7
"#;
    let mut editor = Editor::with_script(profile, 24, "max_sessions=1", script);
    editor.session.insert('!');
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, placement);
    editor.wait(|s| text(s).contains("READY"));
    let first = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let pid = editor
        .session
        .terminals()
        .get(first)
        .unwrap()
        .process_id()
        .unwrap();
    if background {
        cancel(&mut editor);
        editor
            .session
            .execute(Command::Buffer(BufferCommand::Switch));
        editor.session.handle(Event::Paste("term".into()));
    }
    editor
        .fixture
        .write(&format!("source/leave-{pid}"), "ready");
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.session.terminals().get(first).is_some() {
        editor.session.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "{}", editor.session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(editor.session.terminal_buffer().is_none());
    assert!(!editor.session.terminal_focused());
    assert!(editor.session.terminal_copy_view().is_none());
    if background {
        let frame = editor.session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .any(|r| r.text().contains("Switch to buffer: term"))
        );
        assert!(!frame.rows().iter().any(|r| r.text().trim() == "terminal"));
        editor.session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        for _ in 0..4 {
            editor
                .session
                .handle(key(KeyCode::Backspace, KeyModifiers::NONE));
        }
        editor.session.handle(Event::Paste("doc".into()));
        editor.session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        editor
            .session
            .handle(key(KeyCode::Enter, KeyModifiers::NONE));
        assert!(!editor.session.echo().contains("No buffer"));
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    }
    command(&mut editor, TerminalCommand::Open);
    assert_ne!(
        editor.session.terminal_buffer().unwrap().terminal(),
        Some(first)
    );
    editor.wait(|s| text(s).contains("READY"));
    assert_eq!(editor.session.terminals().ids().count(), 1);
}

// C-x b lists both placements, filters names, and selects without creating a
// replacement child or changing a terminal's panel/main-area placement.
#[test]
fn buffer_candidates() {
    if isolated("panel::lifecycle::buffer_candidates", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::new(profile, 24);
        let source = editor.session.buffer().unwrap().stamp();
        editor.open();
        let panel = editor.session.terminal_buffer().unwrap().terminal();
        command(&mut editor, TerminalCommand::NewBuffer);
        editor.wait(|s| text(s).contains("READY"));
        let main = editor.session.terminal_buffer().unwrap().terminal();
        let main_stamp = editor.session.buffer().unwrap().stamp();
        assert_ne!(panel, main);
        buffer_selector(&mut editor);
        let frame = editor.session.frame().unwrap();
        assert!(frame.rows().iter().any(|row| {
            row.band() == rev::display::Band::Completion && row.text().trim() == "document.org"
        }));
        editor.session.handle(Event::Paste("term".into()));
        for name in ["terminal", "terminal-2"] {
            let frame = editor.session.frame().unwrap();
            assert!(
                frame.rows().iter().any(|row| {
                    row.band() == rev::display::Band::Completion && row.text().trim() == name
                }),
                "missing visible candidate {name}"
            );
            editor
                .session
                .handle(key(KeyCode::Down, KeyModifiers::NONE));
        }
        editor.session.handle(key(KeyCode::Up, KeyModifiers::NONE));
        editor.session.handle(key(KeyCode::Tab, KeyModifiers::NONE));
        editor
            .session
            .handle(key(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(editor.session.terminal_buffer().unwrap().terminal(), panel);
        assert_eq!(editor.session.buffer().unwrap().stamp(), main_stamp);
        cancel(&mut editor);
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
        buffer_selector(&mut editor);
        editor.session.handle(Event::Paste("term".into()));
        editor
            .session
            .handle(key(KeyCode::Down, KeyModifiers::NONE));
        editor
            .session
            .handle(key(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(editor.session.buffer().unwrap().terminal(), main);
        assert_eq!(editor.session.terminals().ids().count(), 2);
    }
}

fn buffer_selector(editor: &mut Editor) {
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    editor
        .session
        .handle(key(KeyCode::Char('b'), KeyModifiers::NONE));
}

// Completion of a hidden panel child cannot steal focus from a different
// main-area terminal, and must release only the completed session.
#[test]
fn background_exit() {
    if isolated("panel::lifecycle::background_exit", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script = "printf READY\nwhile [ ! -f leave-$$ ]; do sleep 0.01; done\nexit 0\n";
        let mut editor = Editor::with_script(profile, 24, "", script);
        command(&mut editor, TerminalCommand::Open);
        editor.wait(|s| text(s).contains("READY"));
        let first = editor
            .session
            .terminal_buffer()
            .unwrap()
            .terminal()
            .unwrap();
        let pid = editor
            .session
            .terminals()
            .get(first)
            .unwrap()
            .process_id()
            .unwrap();
        command(&mut editor, TerminalCommand::NewBuffer);
        editor.wait(|s| text(s).contains("READY"));
        let active = editor.session.buffer().unwrap().id();
        editor
            .fixture
            .write(&format!("source/leave-{pid}"), "ready");
        let deadline = Instant::now() + Duration::from_secs(3);
        while editor.session.terminals().get(first).is_some() {
            editor.session.expire_input(Instant::now());
            assert!(Instant::now() < deadline, "{}", editor.session.echo());
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(editor.session.buffer().unwrap().id(), active);
        assert!(editor.session.terminal_focused());
        assert_eq!(editor.session.terminals().ids().count(), 1);
    }
}

// An unrelated source-save decision must remain visible and cancellable when
// a background shell exits. Closing that shell cannot authorize source loss.
#[test]
fn exit_question() {
    if isolated("panel::lifecycle::exit_question", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script = "printf READY\nwhile [ ! -f leave-$$ ]; do sleep 0.01; done\nexit 0\n";
        let mut editor = Editor::with_script(profile, 24, "", script);
        editor.session.insert('!');
        let source = editor.session.buffer().unwrap().stamp();
        command(&mut editor, TerminalCommand::Open);
        editor.wait(|s| text(s).contains("READY"));
        let id = editor
            .session
            .terminal_buffer()
            .unwrap()
            .terminal()
            .unwrap();
        let pid = editor
            .session
            .terminals()
            .get(id)
            .unwrap()
            .process_id()
            .unwrap();
        cancel(&mut editor);
        close_current(&mut editor);
        let question = editor.session.echo().to_owned();
        assert!(question.contains("y=save"));
        editor
            .fixture
            .write(&format!("source/leave-{pid}"), "ready");
        let deadline = Instant::now() + Duration::from_secs(3);
        while editor.session.terminals().get(id).is_some() {
            editor.session.expire_input(Instant::now());
            assert!(Instant::now() < deadline, "{}", editor.session.echo());
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            editor.session.echo().contains(&question),
            "{}",
            editor.session.echo()
        );
        cancel(&mut editor);
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    }
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
        assert_eq!(
            editor.session.echo(),
            "Source or owned Git work changed; quit canceled before termination",
            "{profile}"
        );
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

// Real retained hyperlink allocations reach the extras admission boundary.
// The failed terminal remains inspectable and consumes capacity until explicit
// closure; a second PTY and the exact unsaved source remain usable throughout.
#[test]
fn extras_failure() {
    if isolated("panel::lifecycle::extras_failure", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        extras_failure_case(profile);
    }
}
fn extras_failure_case(profile: &str) {
    use std::io::Write;
    let script = "stty raw -echo\nprintf READY\nwhile [ ! -f role-$$ ]; do sleep 0.01; done\nif [ \"$(cat role-$$)\" = flood ]; then cat extras; else dd bs=1 count=1 >/dev/null 2>&1; printf PEER_ACK; fi\nwhile :; do sleep 60; done\n";
    let mut editor =
        Editor::with_script(profile, 24, "scrollback_lines=64\nmax_sessions=2", script);
    editor.session.insert('!');
    let source = editor.session.buffer().unwrap().stamp();
    let path = editor.fixture.0.join("source/extras");
    let mut file = std::fs::File::create(path).unwrap();
    for index in 0..2300 {
        write!(
            file,
            "\x1b]8;;https://example.invalid/{index:04}/{}\x07x",
            "a".repeat(3970)
        )
        .unwrap();
    }
    drop(file);
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    let victim = editor.session.terminals().ids().next().unwrap();
    let victim_pid = editor
        .session
        .terminals()
        .get(victim)
        .unwrap()
        .process_id()
        .unwrap();
    let victim_name = editor.session.terminal_buffer().unwrap().name().to_owned();
    command(&mut editor, TerminalCommand::New);
    editor.wait(|s| text(s).contains("READY"));
    let peer = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let peer_pid = editor
        .session
        .terminals()
        .get(peer)
        .unwrap()
        .process_id()
        .unwrap();
    editor
        .fixture
        .write(&format!("source/role-{victim_pid}"), "flood");
    editor
        .fixture
        .write(&format!("source/role-{peer_pid}"), "peer");
    editor
        .session
        .handle(key(KeyCode::Char('p'), KeyModifiers::NONE));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.session.expire_input(Instant::now());
        let victim = editor.session.terminals().get(victim).unwrap();
        assert!(victim.storage().extra_bytes <= rev::term::EXTRA_BYTES);
        if matches!(victim.state(), State::Failed(_))
            && text(editor.session.terminals().get(peer).unwrap()).contains("PEER_ACK")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "extras/peer deadline: {:?}, {}",
            victim.state(),
            victim.storage().extra_bytes
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        editor
            .session
            .terminals()
            .get(victim)
            .unwrap()
            .diagnostic()
            .unwrap()
            .contains("storage limit")
    );
    assert_eq!(
        editor.session.terminals().get(peer).unwrap().process_id(),
        Some(peer_pid)
    );
    cancel(&mut editor);
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(editor.session.buffer().unwrap().is_modified());
    command(&mut editor, TerminalCommand::New);
    assert!(editor.session.echo().contains("capacity"));
    assert_eq!(editor.session.terminals().ids().count(), 2);
    switch(&mut editor, &victim_name);
    assert_eq!(
        editor.session.terminal_buffer().unwrap().terminal(),
        Some(victim)
    );
    assert!(
        editor
            .session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Failed:"))
    );
    close_current(&mut editor);
    if editor.session.echo().starts_with("Terminate") {
        answer(&mut editor, true);
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.session.terminals().get(victim).is_some() {
        editor.session.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "{}", editor.session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        editor.session.terminals().get(peer).unwrap().process_id(),
        Some(peer_pid)
    );
    command(&mut editor, TerminalCommand::New);
    assert_eq!(editor.session.terminals().ids().count(), 2);
}
