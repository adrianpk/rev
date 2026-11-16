use super::{Editor, advance};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::command::{Command, History, Motion};
use std::time::{Duration, Instant};

// Ordinary accepted keys and source edits use the same finite owner in
// both profiles. A source edit into ordinary body text cancels the active pair.
#[test]
fn accepted_input() {
    for profile in ["non-modal", "modal"] {
        accepted_profile(profile);
    }
}

fn accepted_profile(profile: &str) {
    let mut e = Editor::profile(profile, 1000, "(x)", 0);
    if profile == "modal" {
        e.native
            .session
            .execute(Command::Modal(rev::command::ModalCommand::Insert));
    }
    let started = Instant::now();
    e.native.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Right,
        KeyModifiers::NONE,
    )));
    assert!(started.elapsed() < Duration::from_millis(250));
    assert_eq!(e.sources(), [0, 2], "{profile}");
    assert_eq!(e.remaining(), Some(Duration::from_millis(1000)));
    e.native.session.handle(Event::Paste("body".into()));
    assert!(e.sources().is_empty());
    e.native.session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(e.sources(), [0, 6]);
    assert_eq!(e.remaining(), Some(Duration::from_millis(1000)));
}

// Typed and pasted closing endpoints qualify the newly published source in
// both native profiles, while source editing keeps its ordinary single undo.
#[test]
fn endpoint_edits() {
    for (profile, paste) in [
        ("non-modal", false),
        ("non-modal", true),
        ("modal", false),
        ("modal", true),
    ] {
        let mut e = Editor::profile(profile, 250, "(", 1);
        if profile == "modal" {
            e.native
                .session
                .execute(Command::Modal(rev::command::ModalCommand::Insert));
        }
        let event = if paste {
            Event::Paste(")".into())
        } else {
            Event::Key(KeyEvent::new(KeyCode::Char(')'), KeyModifiers::NONE))
        };
        e.native.session.handle(event);
        assert_eq!(e.sources(), [0, 1], "{profile}: paste={paste}");
        assert_eq!(e.remaining(), Some(Duration::from_millis(250)));
        assert_eq!(e.native.session.view().point.position.0, 2);
        let buffer = e.native.session.buffer().unwrap();
        assert_eq!(
            buffer
                .slice(&buffer.span(0..2).unwrap())
                .unwrap()
                .to_string(),
            "()"
        );
        e.native.session.execute(Command::History(History::Undo));
        let buffer = e.native.session.buffer().unwrap();
        assert_eq!(buffer.len_chars(), 1);
        assert_eq!(
            buffer
                .slice(&buffer.span(0..1).unwrap())
                .unwrap()
                .to_string(),
            "("
        );
    }
}

// Failed or clamped motions, release metadata and ignored events cannot replace
// a live expiry. Moving to EOF once is accepted; repeating that motion is a no-op.
#[test]
fn no_op_and_release() {
    let mut e = Editor::new(250, "(x)", 0);
    e.native.session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(e.sources(), [0, 2]);
    advance(100);
    e.native.session.execute(Command::Motion(Motion::BufferEnd));
    e.native.session.execute(Command::ForwardPair);
    e.native.session.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Left,
        KeyModifiers::NONE,
        KeyEventKind::Release,
    )));
    e.native.session.handle(Event::FocusGained);
    assert_eq!(e.remaining(), Some(Duration::from_millis(150)));
    assert_eq!(e.sources(), [0, 2]);
    e.native.unchanged("(x)", 3);
}

// Unfinished/mismatched/uncertain candidates release qualification and expiry.
// Deep/large pending work expires before a late result could authorize a mark.
#[test]
fn missing_and_late_results() {
    for source in [
        "(unfinished".to_string(),
        "([)]".into(),
        "(\"unfinished".into(),
        format!("({})", "word ".repeat(10000)),
    ] {
        let mut e = Editor::new(50, &source, 1);
        e.native
            .session
            .execute(Command::Motion(Motion::BufferStart));
        e.native.session.frame().unwrap();
        advance(50);
        assert!(e.sources().is_empty());
        assert_eq!(e.remaining(), None);
        for _ in 0..3 {
            assert!(e.sources().is_empty());
        }
        e.native.unchanged(&source, 0);
    }
}

// A style-only command does not move caret/scroll or trigger a pulse. Steady
// depth/match/off and returning to pulse each have no pulse-owned wakeup.
#[test]
fn style_controls() {
    let mut e = Editor::new(250, "(x)", 0);
    for style in ["depth", "match", "off", "pulse"] {
        e.native.choose(style);
        e.frame();
        assert_eq!(e.remaining(), None, "{style}");
        e.native.unchanged("(x)", 0);
    }
    assert!(e.sources().is_empty());
}

// Pulse uses match attributes while retaining a qualified quote's string
// foreground. Only endpoints receive underline, and no blink attribute is sent.
#[test]
fn quote_foreground() {
    let native = crate::rendering::Editor::new(
        "pairs.highlight='pulse'\nfiles.associations.'*.case'='rust'",
        "\"body\"",
        0,
    );
    let mut e = Editor::attach(native);
    e.jump();
    let frame = e.frame();
    let cells = crate::surface::draw(&frame);
    for column in [0, 5] {
        assert_eq!(
            cells[&(0, column)].foreground,
            Some(e.native.palette.syntax.string)
        );
        assert!(cells[&(0, column)].underline);
    }
    for column in 1..5 {
        assert!(!cells[&(0, column)].underline);
    }
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(!text.contains("\x1b[5m") && !text.contains("\x1b[6m"));
    e.native.unchanged("\"body\"", 5);
}

// Isolate crossterm's cached environment policy. Pulse remains a finite
// underline without RGB or terminal blink, and expiry restores ordinary cells.
#[test]
fn no_color() {
    if std::env::var_os("REV_PULSE_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "pulse::input::no_color", "--nocapture"])
            .env("REV_PULSE_COLOR_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let mut e = Editor::new(50, "(x)", 0);
    e.jump();
    let mut bytes = Vec::new();
    e.frame().write(&mut bytes).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains("38;2") && !text.contains("48;2") && !text.contains("\x1b[5m"));
    let cells = crate::surface::decode(bytes);
    assert!(cells[&(0, 0)].underline && cells[&(0, 2)].underline);
    advance(50);
    let mut bytes = Vec::new();
    e.frame().write(&mut bytes).unwrap();
    let cells = crate::surface::decode(bytes);
    assert!(!cells[&(0, 0)].underline && !cells[&(0, 2)].underline);
    assert_eq!(e.remaining(), None);
}
