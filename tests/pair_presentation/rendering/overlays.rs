use super::{Editor, surface};
use crossterm::event::Event;
use rev::command::{Command, Motion, RegionCommand};

// Region foreground/background and attributes remain authoritative over depth
// and active marks; incremental search owns its own foreground/background.
#[test]
fn selection_and_search() {
    let mut e = Editor::new("", "(x)", 0);
    e.session.execute(Command::Region(RegionCommand::Set));
    e.session.execute(Command::Motion(Motion::ForwardChar));
    let cells = surface::draw(&e.frame());
    assert_eq!(cells[&(0, 0)].foreground, Some(e.palette.selection_text));
    assert_eq!(cells[&(0, 0)].background, Some(e.palette.selection));
    assert!(!cells[&(0, 0)].underline);
    e.session.execute(Command::KeyboardQuit);
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::SearchForward);
    e.session.handle(Event::Paste("(".into()));
    let cells = surface::draw(&e.frame());
    assert_eq!(cells[&(0, 0)].foreground, Some(e.palette.search_text));
    assert_eq!(cells[&(0, 0)].background, Some(e.palette.search));
    assert!(cells[&(0, 0)].underline);
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// A confirmed offscreen mate offers source line/column only when echo is empty.
// Explicit feedback, errors and minibuffer prompts retain their native surface.
#[test]
fn offscreen_hint() {
    let source = format!("(\n{})\n", "body\n".repeat(20));
    let mut e = Editor::new("", &source, 0);
    e.session.execute(Command::Motion(Motion::BufferStart));
    let frame = e.frame();
    assert!(
        frame
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Matching delimiter: line 22, column 1")
    );
    assert_eq!(e.session.echo(), "");
    e.session.execute(Command::PairHighlightMode);
    assert!(
        e.frame()
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Pair highlight style: off")
    );
    e.session.execute(Command::PairHighlightMode);
    e.session.execute(Command::PairHighlightStyle);
    let frame = e.frame();
    assert!(
        frame
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Pair highlight style (depth/match/pulse/off):")
    );
    e.session.execute(Command::KeyboardQuit);
    e.session.handle(Event::Key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('!'),
        crossterm::event::KeyModifiers::NONE,
    )));
    assert!(
        e.frame()
            .rows()
            .last()
            .unwrap()
            .text()
            .contains("read-only")
    );
    e.unchanged(&source, 0);
}

// NO_COLOR remains process-isolated because crossterm caches environment policy.
// Active endpoints use underline without color escapes or invented characters.
#[test]
fn no_color() {
    if std::env::var_os("REV_PAIR_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "rendering::overlays::no_color", "--nocapture"])
            .env("REV_PAIR_COLOR_CHILD", "1")
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
    let mut e = Editor::new("", "(x)", 0);
    let frame = e.frame();
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let text = String::from_utf8(output.clone()).unwrap();
    assert!(!text.contains("38;2") && !text.contains("48;2"));
    let cells = surface::decode(output);
    assert!(cells[&(0, 0)].underline && cells[&(0, 2)].underline);
    assert!(!cells[&(0, 1)].underline);
    e.unchanged("(x)", 0);
}

// Expiry services pending qualification without requiring a frame observer.
// Completed steady styles keep no pair-owned deadline; repaint cannot restart work.
#[test]
fn bounded_idle() {
    for style in ["depth", "match", "off"] {
        let mut e = Editor::new(&format!("pairs.highlight='{style}'"), &"()".repeat(1500), 0);
        e.session.frame().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut turns = 0;
        while e
            .session
            .remaining_input(std::time::Instant::now())
            .is_some()
        {
            assert!(std::time::Instant::now() < deadline);
            e.session.expire_input(std::time::Instant::now());
            turns += 1;
        }
        assert!(style == "off" || turns > 0);
        e.session.frame().unwrap();
        assert!(
            e.session
                .remaining_input(std::time::Instant::now())
                .is_none()
        );
    }
}

// Real native history exists before painting. Style changes and repeated frames
// cannot consume the available redo transaction or publish an extra source edit.
#[test]
fn retained_history() {
    use rev::buffer::edit::{History, Replacement};
    let mut e = Editor::prepared("", "(x)", 0, |buffer, view| {
        buffer.set_readonly(false);
        let mut points = [view.point];
        let change = Replacement {
            source: buffer.span(2..2).unwrap(),
            text: "y".into(),
        };
        let prepared = buffer.prepare(&[change], &points, None).unwrap();
        buffer.publish(prepared, &mut points).unwrap();
        buffer.navigate(History::Undo, &mut points, None).unwrap();
        view.point = points[0];
        buffer.set_readonly(true);
    });
    let stamp = e.session.buffer().unwrap().stamp();
    for style in ["depth", "match", "off"] {
        e.choose(style);
        e.frame();
        let buffer = e.session.buffer().unwrap();
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.redo_history().len(), 1);
        e.unchanged("(x)", 0);
    }
}
