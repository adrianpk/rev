use super::*;
use rev::command::{LinearCommand, Motion, RegionCommand};

fn command(editor: &mut Editor, terminal: TerminalCommand) {
    editor.session.execute(Command::Terminal(terminal));
}
fn retain_word(editor: &mut Editor, skip: usize) {
    editor.session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..skip {
        editor.session.execute(Command::Motion(Motion::ForwardWord));
        editor.session.execute(Command::Motion(Motion::ForwardChar));
    }
    editor.session.execute(Command::Region(RegionCommand::Set));
    editor.session.execute(Command::Motion(Motion::ForwardWord));
    editor.session.execute(Command::Linear(LinearCommand::Copy));
}

// The native retained-text ring and real PTY agree on exact paste bytes in
// both profiles/modes. A rejected replacement never guesses child backspaces.
#[test]
fn retained_paste() {
    if isolated("panel::transfer::retained_paste", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        for bracketed in [false, true] {
            paste_case(profile, bracketed);
        }
    }
}
fn paste_case(profile: &str, bracketed: bool) {
    let framing = if bracketed { "\\033[?2004h" } else { "" };
    let payload = if bracketed {
        "\x1b[200~alpha\x1b[201~"
    } else {
        "alpha"
    };
    let pasted = if bracketed {
        "\x1b[200~界é\t\n\x1b[201~"
    } else {
        "界é\t\n"
    };
    let script = format!(
        "stty raw -echo\nprintf '{framing}alpha beta\\r\\nREADY'\ndd bs=1 count={} of=first.bin 2>/dev/null\nprintf FIRST\ndd bs=1 count={} of=second.bin 2>/dev/null\nprintf SECOND\nwhile :; do sleep 60; done\n",
        payload.len(),
        pasted.len()
    );
    let mut editor = Editor::with_script(profile, 24, "", &script);
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::CopyMode);
    retain_word(&mut editor, 0);
    retain_word(&mut editor, 1);
    assert_eq!(editor.session.killed_text().unwrap().to_string(), "beta");
    command(&mut editor, TerminalCommand::CopyModeDone);
    // Both events occur before an IO service boundary: only alpha reaches the
    // child. The second paste's framing replaces the first framing atomically.
    editor
        .session
        .handle(key(KeyCode::Char('y'), KeyModifiers::CONTROL));
    editor
        .session
        .handle(key(KeyCode::Char('y'), KeyModifiers::ALT));
    assert!(
        editor.session.echo().contains("pasted"),
        "{}",
        editor.session.echo()
    );
    editor.wait(|s| text(s).contains("FIRST"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/first.bin")).unwrap(),
        payload.as_bytes()
    );
    command(&mut editor, TerminalCommand::YankPop);
    assert!(
        editor.session.echo().contains("no longer wholly queued"),
        "{}",
        editor.session.echo()
    );
    // Escape cannot terminate a user paste or grant control-byte authority.
    editor.session.handle(Event::Paste("\x1b[201~bad".into()));
    assert!(editor.session.echo().contains("control characters"));
    editor.session.handle(Event::Paste("界é\t\n".into()));
    editor.wait(|s| text(s).contains("SECOND"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/second.bin")).unwrap(),
        pasted.as_bytes()
    );
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(editor.session.buffer().unwrap().history().is_empty());
    assert_eq!(editor.session.terminal_buffer().unwrap().len_bytes(), 0);
}

// Every finite send name reaches the same mode-aware VT path as live keys.
// Clear sends C-l; undo sends C-_; copy input cannot silently reach the child.
#[test]
fn explicit_controls() {
    if isolated("panel::transfer::explicit_controls", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        for application in [false, true] {
            controls_case(profile, application);
        }
    }
}
fn controls_case(profile: &str, application: bool) {
    let names = [
        "return",
        "tab",
        "escape",
        "backspace",
        "delete",
        "up",
        "down",
        "left",
        "right",
        "prior",
        "next",
        "space",
    ];
    let mut expected = if application {
        b"\r\n\t\x1b\x7f\x1b[3~\x1bOA\x1bOB\x1bOD\x1bOC\x1b[5~\x1b[6~ ".to_vec()
    } else {
        b"\r\t\x1b\x7f\x1b[3~\x1b[A\x1b[B\x1b[D\x1b[C\x1b[5~\x1b[6~ ".to_vec()
    };
    expected.extend_from_slice(b"\x0c\x1f\x19\x1by");
    let mode = if application {
        "\\033[?1h\\033[20h"
    } else {
        ""
    };
    let script = format!(
        "stty raw -echo\nprintf '{mode}READY'\ndd bs=1 count={} of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n",
        expected.len()
    );
    let mut editor = Editor::with_script(profile, 24, "", &script);
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::CopyMode);
    editor.named("terminal-send-space", false);
    assert!(editor.session.echo().contains("Exit terminal copy"));
    command(&mut editor, TerminalCommand::SendNextKey);
    assert!(editor.session.echo().contains("Exit terminal copy"));
    command(&mut editor, TerminalCommand::CopyModeDone);
    for name in names {
        editor.named(&format!("terminal-send-{name}"), false);
    }
    editor.named("terminal-clear", false);
    editor.named("terminal-undo", false);
    for modifiers in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
        command(&mut editor, TerminalCommand::SendNextKey);
        editor.session.handle(key(KeyCode::Char('y'), modifiers));
    }
    editor.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
        expected
    );
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
}

// Frozen point alignment is admitted only against still-represented text.
// Clearing history preserves the live screen and never sends a control byte.
#[test]
fn copy_controls() {
    if isolated("panel::transfer::copy_controls", false) {
        return;
    }
    let script = "stty raw -echo\ni=0; while [ $i -lt 50 ]; do printf 'OLD%s\\r\\n' \"$i\"; i=$((i+1)); done\nprintf 'alpha beta READY'\nwhile [ ! -f burst ]; do sleep 0.01; done\nprintf CHANGED\ndd bs=1 count=1 of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n";
    let mut editor = Editor::with_script("non-modal", 24, "", script);
    let source = editor.session.buffer().unwrap().stamp();
    command(&mut editor, TerminalCommand::Open);
    editor.wait(|s| text(s).contains("READY"));
    command(&mut editor, TerminalCommand::CopyMode);
    let origin = editor.session.terminal_copy_view().unwrap().point;
    command(&mut editor, TerminalCommand::BeginningOfLine);
    assert!(editor.session.terminal_copy_view().unwrap().point.position < origin.position);
    command(&mut editor, TerminalCommand::EndOfLine);
    assert!(editor.session.terminal_copy_view().unwrap().point.position > origin.position);
    command(&mut editor, TerminalCommand::ResetCursorPoint);
    assert_eq!(editor.session.terminal_copy_view().unwrap().point, origin);
    std::fs::write(editor.fixture.0.join("source/burst"), "ready").unwrap();
    editor.wait(|s| text(s).contains("CHANGED"));
    command(&mut editor, TerminalCommand::ResetCursorPoint);
    assert!(editor.session.echo().contains("not represented"));
    assert_eq!(editor.session.terminal_copy_view().unwrap().point, origin);
    let id = editor.session.terminals().ids().next().unwrap();
    let before = text(editor.session.terminals().get(id).unwrap());
    use alacritty_terminal::grid::Dimensions;
    assert!(
        editor
            .session
            .terminals()
            .get(id)
            .unwrap()
            .screen()
            .grid()
            .history_size()
            > 0
    );
    editor.named("terminal-clear-scrollback", false);
    assert!(editor.session.terminal_copy_view().is_none());
    assert_eq!(
        editor
            .session
            .terminals()
            .get(id)
            .unwrap()
            .screen()
            .grid()
            .history_size(),
        0
    );
    assert_eq!(text(editor.session.terminals().get(id).unwrap()), before);
    editor
        .session
        .handle(key(KeyCode::Char('p'), KeyModifiers::NONE));
    editor.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
        b"p"
    );
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
}
