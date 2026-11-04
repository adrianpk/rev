use super::*;
use crossterm::event::{KeyEventKind, KeyEventState};

// A raw real PTY records exact bytes, including controls otherwise claimed by
// editor maps. Host prompt/prefix cancellation must send no bytes to this child.
#[test]
fn live_keys() {
    if isolated("panel::interaction::live_keys", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        for application in [false, true] {
            live_keys_case(profile, application);
        }
    }
}

fn live_keys_case(profile: &str, application: bool) {
    let cases = key_cases(application);
    let mut expected: Vec<u8> = cases.iter().flat_map(|(_, bytes)| bytes.bytes()).collect();
    expected.extend_from_slice(b"\x07\x1bx\x18\x11");
    let modes = if application {
        "\\033[?1h\\033=\\033[20h"
    } else {
        ""
    };
    let script = format!(
        "stty raw -echo\nprintf '{modes}\\033[6 qREADY'\ndd bs=1 count={} of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n",
        expected.len()
    );
    let mut editor = Editor::with_script(profile, 24, "", &script);
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| text(s).contains("READY"));
    assert_eq!(
        editor.session.frame().unwrap().cursor_style(),
        Some(crossterm::cursor::SetCursorStyle::SteadyBar)
    );
    // A strong prompt owns Escape and C-g. C-g after C-x cancels that host
    // prefix, retaining the live terminal focus for the next native event.
    for cancel in [KeyCode::Esc, KeyCode::Char('g')] {
        editor
            .session
            .handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
        assert_eq!(editor.session.frame().unwrap().cursor_style(), None);
        let modifiers = if cancel == KeyCode::Esc {
            KeyModifiers::NONE
        } else {
            KeyModifiers::CONTROL
        };
        editor.session.handle(key(cancel, modifiers));
        editor
            .session
            .expire_input(Instant::now() + rev::input::META_TIMEOUT);
        assert!(editor.session.terminal_focused());
    }
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(editor.session.terminal_focused());
    for (event, _) in cases {
        editor.session.handle(event);
    }
    literal(&mut editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    literal(&mut editor, KeyCode::Char('x'), KeyModifiers::ALT);
    editor.named("terminal-send-next-key", false);
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    literal(&mut editor, KeyCode::Char('q'), KeyModifiers::CONTROL);
    editor.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
        expected
    );
    assert!(editor.session.terminal_focused());
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(editor.session.buffer().unwrap().history().is_empty());
}

fn literal(editor: &mut Editor, code: KeyCode, modifiers: KeyModifiers) {
    editor
        .session
        .handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
    let mut release = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    release.kind = KeyEventKind::Release;
    editor.session.handle(Event::Key(release));
    editor.session.handle(key(code, modifiers));
}

fn key_cases(application: bool) -> Vec<(Event, &'static str)> {
    let mut cases = vec![
        (key(KeyCode::Char(':'), KeyModifiers::NONE), ":"),
        (key(KeyCode::Char('é'), KeyModifiers::NONE), "é"),
        (key(KeyCode::Esc, KeyModifiers::NONE), "\x1b"),
        (
            key(KeyCode::Enter, KeyModifiers::NONE),
            if application { "\r\n" } else { "\r" },
        ),
        (key(KeyCode::Tab, KeyModifiers::NONE), "\t"),
        (key(KeyCode::BackTab, KeyModifiers::NONE), "\x1b[Z"),
        (key(KeyCode::Backspace, KeyModifiers::NONE), "\x7f"),
        (key(KeyCode::Backspace, KeyModifiers::CONTROL), "\x08"),
        (key(KeyCode::Delete, KeyModifiers::NONE), "\x1b[3~"),
        (key(KeyCode::PageUp, KeyModifiers::NONE), "\x1b[5~"),
        (key(KeyCode::PageDown, KeyModifiers::NONE), "\x1b[6~"),
        (key(KeyCode::Insert, KeyModifiers::NONE), "\x1b[2~"),
        (key(KeyCode::F(1), KeyModifiers::NONE), "\x1bOP"),
        (key(KeyCode::F(12), KeyModifiers::NONE), "\x1b[24~"),
        (key(KeyCode::F(2), KeyModifiers::CONTROL), "\x1b[1;5Q"),
        (
            key(KeyCode::Left, KeyModifiers::SHIFT | KeyModifiers::CONTROL),
            "\x1b[1;6D",
        ),
        (key(KeyCode::PageDown, KeyModifiers::ALT), "\x1b[6;3~"),
        (key(KeyCode::Char('b'), KeyModifiers::ALT), "\x1bb"),
        (key(KeyCode::Char('c'), KeyModifiers::CONTROL), "\x03"),
        (key(KeyCode::Char('w'), KeyModifiers::CONTROL), "\x17"),
        (key(KeyCode::Char('z'), KeyModifiers::CONTROL), "\x1a"),
        (key(KeyCode::Char(' '), KeyModifiers::CONTROL), "\x00"),
    ];
    for (code, normal, app) in [
        (KeyCode::Up, "\x1b[A", "\x1bOA"),
        (KeyCode::Down, "\x1b[B", "\x1bOB"),
        (KeyCode::Right, "\x1b[C", "\x1bOC"),
        (KeyCode::Left, "\x1b[D", "\x1bOD"),
        (KeyCode::Home, "\x1b[H", "\x1bOH"),
        (KeyCode::End, "\x1b[F", "\x1bOF"),
    ] {
        cases.push((
            key(code, KeyModifiers::NONE),
            if application { app } else { normal },
        ));
    }
    let mut keypad = KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE);
    keypad.state = KeyEventState::KEYPAD;
    cases.push((Event::Key(keypad), if application { "\x1bOr" } else { "2" }));
    cases
}
