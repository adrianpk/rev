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
