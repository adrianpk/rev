use super::{Editor, advance, now};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, Motion, Split, WindowCommand};

// Source publication and native body motion retire point/source captures. A
// programmatic edit cannot turn itself into an accepted-input pulse trigger.
#[test]
fn source_and_point() {
    let mut e = Editor::editable(250, "(body)", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 5]);
    e.native.session.replace(2..2, "x").unwrap();
    assert_eq!(e.remaining(), None);
    assert!(e.sources().is_empty());
    e.native
        .session
        .execute(Command::Motion(Motion::BufferStart));
    assert_eq!(e.sources(), [0, 6]);
    e.native
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    assert_eq!(e.sources(), [0, 6]);
    e.native
        .session
        .execute(Command::Motion(Motion::ForwardChar));
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    assert_eq!(e.native.session.buffer().unwrap().history().len(), 1);
}

// Major-mode selection retires lexical qualification without changing the source
// stamp. Its previous pulse cannot retain a deadline or be re-admitted by paint.
#[test]
fn qualification_retirement() {
    let mut e = Editor::new(250, "(x)", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    let stamp = e.native.session.buffer().unwrap().stamp();
    e.native
        .session
        .execute(Command::Mode(rev::buffer::Mode::Text));
    assert_eq!(e.native.session.buffer().unwrap().stamp(), stamp);
    assert_eq!(e.remaining(), None);
    assert!(e.sources().is_empty());
    e.native.unchanged("(x)", 2);
}

// Off, style replacement, C-g and terminal focus loss all retire the mark and
// its own wakeup. Returning focus or selecting pulse cannot resurrect it.
#[test]
fn cancellation() {
    for cancel in ["off", "match", "quit", "focus"] {
        let mut e = Editor::new(250, "(x)", 0);
        e.jump();
        assert_eq!(e.sources(), [0, 2]);
        match cancel {
            "off" | "match" => e.native.choose(cancel),
            "quit" => {
                e.native.session.execute(Command::KeyboardQuit);
            }
            _ => {
                e.native.session.handle(Event::FocusLost);
            }
        }
        e.native.choose("pulse");
        e.native.session.handle(Event::FocusGained);
        assert!(e.sources().is_empty(), "{cancel}");
        assert_eq!(e.remaining(), None, "{cancel}");
        advance(1000);
        assert!(e.sources().is_empty());
        e.native.unchanged("(x)", 2);
    }
}

// A source prompt cancels its prior generation. Canceling that prompt is a
// focus return, not an accepted source motion that could start another mark.
#[test]
fn prompt_focus() {
    let mut e = Editor::new(250, "(x)", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    e.native.session.execute(Command::FindFile);
    e.native.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    e.native.unchanged("(x)", 2);
}

// Sibling source views inherit defaults without copying a pulse generation.
// Switching away and back, or closing a pulsed view, cannot revive its deadline.
#[test]
fn view_changes() {
    let mut e = Editor::new(250, "(x)", 0);
    e.native.session.execute(Command::Split(Split::Right));
    let ids = e.native.session.windows().order();
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    e.native.session.select_view(ids[1]).unwrap();
    e.native.session.select_view(ids[0]).unwrap();
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    e.native
        .session
        .execute(Command::Window(WindowCommand::Delete));
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    assert!(e.native.session.buffer().unwrap().history().is_empty());
}

// Reload cancellation precedes asynchronous file IO and replaces source through
// its native owner. Late reload/qualification completion cannot recreate a mark.
#[test]
fn reload() {
    let mut e = Editor::new(250, "(x)", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    let file = e
        .native
        .session
        .buffer()
        .unwrap()
        .file()
        .unwrap()
        .path()
        .to_owned();
    std::fs::write(file, "[replacement]").unwrap();
    e.native.session.execute(Command::RevertBuffer);
    assert!(e.sources().is_empty());
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while e.native.session.buffer().unwrap().len_chars() != 13 {
        assert!(std::time::Instant::now() < end, "owned reload deadline");
        e.native.session.expire_input(now());
        e.frame();
        std::thread::yield_now();
    }
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
}

// C-g retires a cooperative foreground delimiter request before it can trigger
// a pulse. Deferred cancellation cannot publish the old target or a late mark.
#[test]
fn pending_motion_cancel() {
    let source = format!("({})", "word ".repeat(10000));
    let mut e = Editor::new(250, &source, 0);
    e.jump();
    assert_eq!(e.native.session.view().point.position.0, 0);
    e.native.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    e.native.unchanged(&source, 0);
}
