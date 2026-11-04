use super::*;
use rev::command::{LinearCommand, Motion, RegionCommand};
fn command(editor: &mut Editor, terminal: TerminalCommand) {
    editor.session.execute(Command::Terminal(terminal));
}
fn copy_word(editor: &mut Editor, expected: &str) {
    editor.session.execute(Command::Region(RegionCommand::Set));
    editor.session.execute(Command::Motion(Motion::ForwardWord));
    editor.session.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(editor.session.killed_text().unwrap().to_string(), expected);
    editor
        .session
        .execute(Command::Motion(Motion::BackwardWord));
}

// OSC reports travel across real IO reads/scrolling. Navigation uses captured
// boundaries, never prompt-looking text or later output from the live child.
#[test]
fn prompts() {
    if isolated("panel::evidence::prompts", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script = r#"stty raw -echo
printf '\033]133;A\007FIRST\r\n'
i=0; while [ $i -lt 40 ]; do printf 'noise%s\r\n' "$i"; i=$((i+1)); done
printf '\033]133;Afoo\007FAKE\r\n\033]133;A\033\\SECOND\r\nREADY'
while [ ! -f burst ]; do sleep 0.01; done
printf '\r\n\033]133;A\007THIRD\r\nCONTINUES'
while :; do sleep 60; done
"#;
        let mut editor = Editor::with_script(profile, 24, "", script);
        let source = editor.session.buffer().unwrap().stamp();
        command(&mut editor, TerminalCommand::Open);
        editor.wait(|s| text(s).contains("READY"));
        command(&mut editor, TerminalCommand::PreviousPrompt);
        assert!(editor.session.terminal_copy_view().is_some());
        copy_word(&mut editor, "SECOND");
        command(&mut editor, TerminalCommand::PreviousPrompt);
        copy_word(&mut editor, "FIRST");
        command(&mut editor, TerminalCommand::NextPrompt);
        copy_word(&mut editor, "SECOND");
        let point = editor.session.terminal_copy_view().unwrap().point;
        std::fs::write(editor.fixture.0.join("source/burst"), "ready").unwrap();
        editor.wait(|s| text(s).contains("CONTINUES"));
        command(&mut editor, TerminalCommand::NextPrompt);
        assert!(
            editor.session.echo().contains("No retained OSC 133"),
            "{}",
            editor.session.echo()
        );
        assert_eq!(editor.session.terminal_copy_view().unwrap().point, point);
        command(&mut editor, TerminalCommand::CopyModeDone);
        command(&mut editor, TerminalCommand::PreviousPrompt);
        copy_word(&mut editor, "THIRD");
        assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    }
}

// Only a positive, labelled, retained compiler location can visit a regular
// source file. Malformed reports and missing files cannot guess a source jump.
#[test]
fn error_locations() {
    if isolated("panel::evidence::error_locations", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        let script = r#"stty raw -echo
printf 'failure.rs:0:1: error: invalid\r\nfailure.rs:2:3: error: genuine\r\nREADY'
while :; do sleep 60; done
"#;
        let mut editor = Editor::with_script(profile, 24, "", script);
        let target = editor.fixture.write("source/failure.rs", "one\nTARGET\n");
        command(&mut editor, TerminalCommand::Open);
        editor.wait(|s| text(s).contains("READY"));
        command(&mut editor, TerminalCommand::NextError);
        assert!(
            editor.session.echo().contains("location opened"),
            "{}",
            editor.session.echo()
        );
        assert!(!editor.session.terminal_focused());
        assert_eq!(
            editor.session.buffer().unwrap().file().unwrap().path(),
            target
        );
        assert_eq!(editor.session.windows().selected().point.position.0, 6);
        assert!(editor.session.buffer().unwrap().history().is_empty());
        assert_eq!(editor.session.terminals().ids().count(), 1);
    }
    let script = r#"stty raw -echo
printf 'fake $ prompt\r\nmissing.rs:0:3: error: invalid\r\nmissing.rs:9: error: absent\r\nREADY'
while :; do sleep 60; done
"#;
    let mut editor = Editor::with_script("non-modal", 24, "", script);
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::PreviousPrompt);
    assert!(editor.session.echo().contains("No retained OSC 133"));
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    command(&mut editor, TerminalCommand::CopyModeDone);
    command(&mut editor, TerminalCommand::NextError);
    assert!(
        editor.session.echo().contains("file is unavailable"),
        "{}",
        editor.session.echo()
    );
    assert!(editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(!editor.fixture.0.join("source/missing.rs").exists());
    for location in [
        "document.org:99:1: error: row",
        "document.org:1:999: error: column",
    ] {
        let script =
            format!("stty raw -echo\nprintf '{location}\\r\\nREADY'\nwhile :; do sleep 60; done\n");
        let mut invalid = Editor::with_script("modal", 24, "", &script);
        let source = invalid.session.buffer().unwrap().stamp();
        let point = invalid.session.windows().selected().point;
        command(&mut invalid, TerminalCommand::Open);
        invalid.wait(|s| text(s).contains("READY"));
        command(&mut invalid, TerminalCommand::NextError);
        assert!(!invalid.session.echo().contains("location opened"));
        assert!(invalid.session.terminal_focused());
        assert_eq!(invalid.session.buffer().unwrap().stamp(), source);
        assert_eq!(invalid.session.windows().selected().point, point);
    }
}

fn viewer_complete(editor: &mut Editor) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.session.expire_input(Instant::now());
        if editor.session.echo().contains("viewer completed") {
            return;
        }
        assert!(Instant::now() < deadline, "{}", editor.session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
}
// Both named activation and Control-click retain the observed URI even after
// live output replaces the grid. Rejected URIs/absent spans start no viewer.
#[test]
fn retained_links() {
    if isolated("panel::evidence::retained_links", false) {
        return;
    }
    retained_link_case();
    link_capacity_case();
}
fn retained_link_case() {
    let uri = "https://example.invalid/$((1+2))?q=a%20b";
    let script = format!(
        "stty raw -echo\nprintf '%b' '\\033]8;;{uri}\\033\\\\CLICK\\033]8;;\\033\\\\\\r\\n\\033]8;;javascript:bad\\033\\\\REJECT\\033]8;;\\033\\\\READY'\nwhile [ ! -f burst ]; do sleep 0.01; done\nprintf '\\033[2J\\033[H\\033]8;;https://new.invalid\\033\\\\CHANGED\\033]8;;\\033\\\\'\nwhile :; do sleep 60; done\n"
    );
    let mut editor = Editor::with_script(
        "non-modal",
        24,
        "[org.tools]\nviewer=['/bin/sh','../viewer.sh','{target}']",
        &script,
    );
    editor
        .fixture
        .write("viewer.sh", "printf '%s' \"$1\" >called\n");
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::OpenLink);
    assert!(editor.session.echo().contains("No terminal hyperlink"));
    command(&mut editor, TerminalCommand::CopyMode);
    editor.session.execute(Command::Motion(Motion::BufferStart));
    std::fs::write(editor.fixture.0.join("source/burst"), "ready").unwrap();
    editor.wait(|s| text(s).contains("CHANGED"));
    command(&mut editor, TerminalCommand::OpenLink);
    viewer_complete(&mut editor);
    assert_eq!(
        std::fs::read_to_string(editor.fixture.0.join("source/called")).unwrap(),
        uri
    );
    std::fs::remove_file(editor.fixture.0.join("source/called")).unwrap();
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    editor.session.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 0,
        row: 12,
        modifiers: KeyModifiers::CONTROL,
    }));
    viewer_complete(&mut editor);
    assert_eq!(
        std::fs::read_to_string(editor.fixture.0.join("source/called")).unwrap(),
        uri
    );
    editor.session.execute(Command::Motion(Motion::BufferStart));
    editor
        .session
        .execute(Command::Scroll(rev::command::Scroll::NextLine));
    command(&mut editor, TerminalCommand::OpenLink);
    assert!(editor.session.echo().contains("not admitted"));
    editor.session.execute(Command::Motion(Motion::BufferEnd));
    command(&mut editor, TerminalCommand::OpenLink);
    assert!(editor.session.echo().contains("No retained hyperlink"));
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
}

// Copy link capacity exhaustion declines only missing evidence. An earlier
// admitted target remains usable and the live cell still carries the late URI.
fn link_capacity_case() {
    let prefix = format!("https://example.invalid/{}", "x".repeat(1024));
    let payload: String = (0..40)
        .map(|index| format!("\\033]8;;{prefix}/{index}\\033\\\\X\\033]8;;\\033\\\\ "))
        .collect();
    let script =
        format!("stty raw -echo\nprintf '%b' '{payload}READY'\nwhile :; do sleep 60; done\n");
    let mut editor = Editor::with_script(
        "modal",
        24,
        "[org.tools]\nviewer=['/bin/sh','../viewer.sh','{target}']",
        &script,
    );
    editor
        .fixture
        .write("viewer.sh", "printf '%s' \"$1\" >called\n");
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    let source = editor.session.buffer().unwrap().stamp();
    let session = editor
        .session
        .terminals()
        .get(editor.session.terminals().ids().next().unwrap())
        .unwrap();
    let point = alacritty_terminal::index::Point::new(
        alacritty_terminal::index::Line(0),
        alacritty_terminal::index::Column(78),
    );
    assert_eq!(
        session.screen().grid()[point].hyperlink().unwrap().uri(),
        format!("{prefix}/39")
    );
    command(&mut editor, TerminalCommand::CopyMode);
    editor.session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..78 {
        editor.session.execute(Command::Motion(Motion::ForwardChar));
    }
    command(&mut editor, TerminalCommand::OpenLink);
    assert!(
        editor.session.echo().contains("No retained hyperlink"),
        "{}",
        editor.session.echo()
    );
    assert!(!editor.fixture.0.join("source/called").exists());
    editor.session.execute(Command::Motion(Motion::BufferStart));
    command(&mut editor, TerminalCommand::OpenLink);
    viewer_complete(&mut editor);
    assert_eq!(
        std::fs::read_to_string(editor.fixture.0.join("source/called")).unwrap(),
        format!("{prefix}/0")
    );
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
}
