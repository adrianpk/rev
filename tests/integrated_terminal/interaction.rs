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

// This controlled full-screen child speaks real VT through the PTY. It covers
// native cell/style projection and focus/resize transitions without borrowing
// an installed tool, its startup files, or its mutable repository state.
#[test]
fn full_screen() {
    if isolated("panel::interaction::full_screen", true) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        full_screen_case(profile);
    }
}

fn full_screen_case(profile: &str) {
    let script = r#"stty raw -echo
printf '\033[2J\033[HMAIN_READY'
dd bs=1 count=1 of=/dev/null 2>/dev/null
printf '\033[?1049h\033[2J\033[H\033]4;1;rgb:12/34/56\007\033[31;1;3;4mTOOL_READY\033[0m\033]52;c;Zm9yYmlkZGVu\007\033]0;FORBIDDEN_TITLE\007\033Punsupported;FORBIDDEN_EFFECT\033\\'
n=1
while [ $n -le 6 ]; do
 printf '\033[3;6H\033[%s q' "$n"
 dd bs=1 count=1 of=/dev/null 2>/dev/null
 n=$((n+1))
done
printf '\033[?25lHIDDEN'
dd bs=1 count=1 of=/dev/null 2>/dev/null
while [ ! -f wake ]; do sleep 0.01; done
printf '\033[?25h\033[4;1HBACKGROUND'
dd bs=1 count=1 of=/dev/null 2>/dev/null
stty size >size.txt
printf '\033[5;1HRESIZED'
dd bs=1 count=1 of=/dev/null 2>/dev/null
printf '\033[?1049l\033[HMAIN_RETURN'
while :; do sleep 60; done
"#;
    let mut editor = Editor::with_script(profile, 24, "", script);
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| text(s).contains("MAIN_READY"));
    let id = editor
        .session
        .terminal_buffer()
        .unwrap()
        .terminal()
        .unwrap();
    let pid = editor.session.terminals().get(id).unwrap().process_id();
    editor
        .session
        .handle(key(KeyCode::Char('a'), KeyModifiers::NONE));
    editor.wait(|s| text(s).contains("TOOL_READY"));
    use crossterm::cursor::SetCursorStyle::*;
    for (index, style) in [
        BlinkingBlock,
        SteadyBlock,
        BlinkingUnderScore,
        SteadyUnderScore,
        BlinkingBar,
        SteadyBar,
    ]
    .into_iter()
    .enumerate()
    {
        editor.wait(|s| {
            s.screen().cursor_style().shape
                == [
                    alacritty_terminal::vte::ansi::CursorShape::Block,
                    alacritty_terminal::vte::ansi::CursorShape::Underline,
                    alacritty_terminal::vte::ansi::CursorShape::Beam,
                ][index / 2]
                && s.screen().cursor_style().blinking == (index % 2 == 0)
        });
        let frame = editor.session.frame().unwrap();
        assert_eq!(frame.cursor_style(), Some(style));
        let cursor = frame.cursor().unwrap();
        assert_eq!(cursor.column, 5);
        assert_eq!(cursor.row, 14);
        let mut bytes = Vec::new();
        frame.write(&mut bytes).unwrap();
        let native = String::from_utf8(bytes).unwrap();
        assert!(native.contains("\x1b[38;2;18;52;86m"));
        for attribute in ["\x1b[1m", "\x1b[3m", "\x1b[4m"] {
            assert!(native.contains(attribute), "{attribute:?}");
        }
        for effect in [
            "FORBIDDEN_TITLE",
            "FORBIDDEN_EFFECT",
            "\x1b]52;",
            "\x1bPunsupported",
        ] {
            assert!(!native.contains(effect), "{effect}");
        }
        assert!(
            frame
                .rows()
                .iter()
                .any(|r| r.text().contains("source stays native"))
        );
        editor
            .session
            .handle(key(KeyCode::Char('n'), KeyModifiers::NONE));
    }
    editor.wait(|s| text(s).contains("HIDDEN"));
    assert_eq!(editor.session.frame().unwrap().cursor(), None);
    editor
        .session
        .handle(key(KeyCode::Char('b'), KeyModifiers::NONE));
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    std::fs::write(editor.fixture.0.join("source/wake"), "ready").unwrap();
    editor.wait(|s| text(s).contains("BACKGROUND"));
    assert!(!editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.session.handle(Event::Resize(100, 30));
    editor
        .session
        .handle(key(KeyCode::Char('r'), KeyModifiers::NONE));
    editor.wait(|s| text(s).contains("RESIZED"));
    assert_eq!(
        std::fs::read_to_string(editor.fixture.0.join("source/size.txt"))
            .unwrap()
            .trim(),
        "10 100"
    );
    editor
        .session
        .handle(key(KeyCode::Char('q'), KeyModifiers::NONE));
    editor.wait(|s| text(s).contains("MAIN_RETURN"));
    let child = editor.session.terminals().get(id).unwrap();
    assert!(
        !child
            .screen()
            .mode()
            .contains(alacritty_terminal::term::TermMode::ALT_SCREEN)
    );
    assert_eq!(child.process_id(), pid);
    assert!(editor.session.killed_text().is_none());
    assert_eq!(editor.session.terminals().ids().count(), 1);
    assert!(editor.session.terminal_focused());
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
}
