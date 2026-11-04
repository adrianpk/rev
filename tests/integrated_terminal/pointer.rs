use super::*;
use crossterm::event::{MouseButton as Button, MouseEvent, MouseEventKind as Kind};

fn mouse(kind: Kind, column: u16, row: u16, modifiers: KeyModifiers) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers,
    })
}

fn recorder(modes: &str, bytes: usize) -> String {
    format!(
        "stty raw -echo\nprintf '{modes}READY'\ndd bs=1 count={bytes} of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n"
    )
}

// Real child-enabled protocols receive complete native pointer encodings;
// Shift and stronger host prompts retain local authority without leaking bytes.
#[test]
fn child_mouse() {
    if isolated("panel::pointer::child_mouse", false) {
        return;
    }
    for profile in ["non-modal", "modal"] {
        for motion in [false, true] {
            child_mouse_case(profile, motion);
        }
    }
}

fn child_mouse_case(profile: &str, motion: bool) {
    let mut expected = b"\x1b[<0;5;3M\x1b[<32;8;4M\x1b[<0;10;1m\x1b[<64;5;3M\x1b[<73;5;3M\x1b[<66;5;3M\x1b[<67;5;3M\x1b[<18;5;3M\x1b[<18;5;3m".to_vec();
    if motion {
        expected.extend_from_slice(b"\x1b[<35;5;3M");
    }
    let modes = if motion {
        "\\033[?1003h\\033[?1006h"
    } else {
        "\\033[?1002h\\033[?1006h"
    };
    let mut editor = Editor::with_script(profile, 24, "", &recorder(modes, expected.len()));
    let source = editor.session.buffer().unwrap().stamp();
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| text(s).contains("READY"));
    for kind in [
        Kind::Down(Button::Left),
        Kind::Drag(Button::Left),
        Kind::Up(Button::Left),
    ] {
        editor
            .session
            .handle(mouse(kind, 4, 14, KeyModifiers::SHIFT));
    }
    let mut frame = Vec::new();
    editor.session.frame().unwrap().write(&mut frame).unwrap();
    assert!(String::from_utf8(frame).unwrap().contains("\x1b[7m"));
    editor
        .session
        .handle(mouse(Kind::ScrollUp, 4, 14, KeyModifiers::SHIFT));
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    editor
        .session
        .handle(mouse(Kind::Down(Button::Left), 4, 14, KeyModifiers::NONE));
    editor
        .session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    for (kind, column, row, modifiers) in [
        (Kind::Down(Button::Left), 4, 14, KeyModifiers::NONE),
        (Kind::Drag(Button::Left), 7, 15, KeyModifiers::NONE),
        (Kind::Up(Button::Left), 9, 1, KeyModifiers::NONE),
        (Kind::ScrollUp, 4, 14, KeyModifiers::NONE),
        (Kind::ScrollDown, 4, 14, KeyModifiers::ALT),
        (Kind::ScrollLeft, 4, 14, KeyModifiers::NONE),
        (Kind::ScrollRight, 4, 14, KeyModifiers::NONE),
        (Kind::Down(Button::Right), 4, 14, KeyModifiers::CONTROL),
        (Kind::Up(Button::Right), 4, 14, KeyModifiers::CONTROL),
        (Kind::Moved, 4, 14, KeyModifiers::NONE),
    ] {
        editor.session.handle(mouse(kind, column, row, modifiers));
    }
    editor.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
        expected
    );
    assert_eq!(editor.session.buffer().unwrap().stamp(), source);
    assert!(editor.session.buffer().unwrap().history().is_empty());
}

// Classic reporting cannot represent column 251. UTF-8 reporting can; neither
// may emit a truncated coordinate or spill a pointer operation into source text.
#[test]
fn legacy_mouse() {
    if isolated("panel::pointer::legacy_mouse", false) {
        return;
    }
    for utf8 in [false, true] {
        let modes = if utf8 {
            "\\033[?1000h\\033[?1005h"
        } else {
            "\\033[?1000h"
        };
        let expected = if utf8 {
            b"\x1b[M \xc4\x9b#\x1b[M#\xc4\x9b#\x1b[M %#".as_slice()
        } else {
            b"\x1b[M %#".as_slice()
        };
        let mut editor = Editor::with_script("non-modal", 24, "", &recorder(modes, expected.len()));
        editor
            .session
            .execute(Command::Terminal(TerminalCommand::Open));
        editor.wait(|s| text(s).contains("READY"));
        editor.session.handle(Event::Resize(300, 24));
        for kind in [Kind::Down(Button::Left), Kind::Up(Button::Left)] {
            editor
                .session
                .handle(mouse(kind, 250, 14, KeyModifiers::NONE));
        }
        editor
            .session
            .handle(mouse(Kind::Down(Button::Left), 4, 14, KeyModifiers::NONE));
        editor.wait(|s| text(s).contains("RECEIVED"));
        assert_eq!(
            std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
            expected
        );
    }
}

// Local history and selection stay with the targeted mirror, including a drag
// released outside its pane. The shared child, source and peer viewport survive.
#[test]
fn local_views() {
    if isolated("panel::pointer::local_views", false) {
        return;
    }
    let mut editor = Editor::new("modal", 24);
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::NewBuffer));
    editor.wait(|s| text(s).contains("READY"));
    let id = editor.session.buffer().unwrap().terminal().unwrap();
    editor
        .session
        .execute(Command::Split(rev::window::Split::Right));
    let views = editor.session.windows().order();
    editor.session.select_view(views[1]).unwrap();
    editor.send("scroll");
    editor.wait(|s| text(s).contains("LINE59"));
    let composition = rev::panel::Composition::new(
        editor.session.windows(),
        rev::window::Area::new(0, 0, 80, 23).unwrap(),
        0,
    )
    .unwrap();
    let area = composition
        .panels()
        .iter()
        .find(|(id, _)| *id == views[0])
        .unwrap()
        .1
        .content();
    let before = editor.session.frame().unwrap().rows()[area.row() as usize]
        .text()
        .to_owned();
    editor.session.handle(mouse(
        Kind::ScrollUp,
        area.column(),
        area.row(),
        KeyModifiers::NONE,
    ));
    let offset = editor
        .session
        .windows()
        .get(views[0])
        .unwrap()
        .terminal
        .offset;
    assert!(offset > 0);
    assert_eq!(
        editor
            .session
            .windows()
            .get(views[1])
            .unwrap()
            .terminal
            .offset,
        0
    );
    assert_ne!(
        editor.session.frame().unwrap().rows()[area.row() as usize].text(),
        before
    );
    editor.session.handle(mouse(
        Kind::Down(Button::Left),
        area.column(),
        area.row(),
        KeyModifiers::NONE,
    ));
    editor.session.handle(mouse(
        Kind::Drag(Button::Left),
        area.column() + 4,
        area.row(),
        KeyModifiers::NONE,
    ));
    editor.session.handle(mouse(
        Kind::Up(Button::Left),
        u16::MAX,
        0,
        KeyModifiers::NONE,
    ));
    let selection = editor
        .session
        .windows()
        .get(views[0])
        .unwrap()
        .terminal
        .selection
        .unwrap();
    assert_eq!(selection.anchor.column.0, 0);
    assert!(selection.end.column.0 < usize::from(area.width()));
    assert!(
        editor
            .session
            .windows()
            .get(views[1])
            .unwrap()
            .terminal
            .selection
            .is_none()
    );
    editor.session.handle(mouse(
        Kind::ScrollDown,
        area.column(),
        area.row(),
        KeyModifiers::NONE,
    ));
    assert_eq!(
        editor
            .session
            .windows()
            .get(views[0])
            .unwrap()
            .terminal
            .offset,
        0
    );
    assert_eq!(editor.session.terminals().ids().count(), 1);
    assert!(
        editor
            .session
            .terminals()
            .get(id)
            .unwrap()
            .process_id()
            .is_some()
    );
}

// Only Control-click can activate an OSC 8 URI. The existing configured viewer
// receives one literal argv target; an executable URI and a second active viewer
// are rejected, and no click bytes leak to the mouse-reporting child.
#[test]
fn links() {
    if isolated("panel::pointer::links", false) {
        return;
    }
    let uri = "https://example.invalid/$((1+2))?q=a%20b";
    let script = format!(
        "stty raw -echo\nprintf '\\033[?1000h\\033[?1006h\\033]8;;{uri}\\033\\\\CLICK\\033]8;;\\033\\\\\\r\\n\\033]8;;javascript:bad\\033\\\\REJECT\\033]8;;\\033\\\\READY'\ndd bs=1 count=1 of=input.bin 2>/dev/null\nprintf RECEIVED\nwhile :; do sleep 60; done\n"
    );
    let mut editor = Editor::with_script(
        "modal",
        24,
        "[org.tools]\nviewer=['/bin/sh','../viewer.sh','{target}']",
        &script,
    );
    editor.fixture.write(
        "viewer.sh",
        "while [ ! -f release ]; do sleep 0.01; done\nprintf '%s' \"$1\" >called\n",
    );
    editor
        .session
        .execute(Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| text(s).contains("READY"));
    assert!(!editor.fixture.0.join("source/called").exists());
    editor.session.handle(mouse(
        Kind::Down(Button::Left),
        0,
        13,
        KeyModifiers::CONTROL,
    ));
    assert!(
        editor.session.echo().contains("not admitted"),
        "{}",
        editor.session.echo()
    );
    editor.session.handle(mouse(
        Kind::Down(Button::Left),
        0,
        12,
        KeyModifiers::CONTROL,
    ));
    assert!(
        editor.session.echo().contains("viewer running"),
        "{}",
        editor.session.echo()
    );
    editor.session.handle(mouse(
        Kind::Down(Button::Left),
        0,
        12,
        KeyModifiers::CONTROL,
    ));
    assert!(editor.session.echo().contains("still owned"));
    std::fs::write(editor.fixture.0.join("source/release"), "ready").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !editor.session.echo().contains("viewer completed") {
        editor.session.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "{}", editor.session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        std::fs::read_to_string(editor.fixture.0.join("source/called")).unwrap(),
        uri
    );
    assert!(editor.session.terminal_focused());
    assert!(editor.session.killed_text().is_none());
    editor
        .session
        .handle(key(KeyCode::Char('x'), KeyModifiers::NONE));
    editor.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(editor.fixture.0.join("source/input.bin")).unwrap(),
        b"x"
    );
    assert!(editor.session.buffer().unwrap().history().is_empty());
}
