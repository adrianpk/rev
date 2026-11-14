use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, History, LinearCommand, MarkdownCommand, Motion, RegionCommand};

use super::*;

const SOURCE: &str = "# Root\nordinary body.\n\n## Child\nchild body.\n\n- first\n  - second\n1. ordered\n\n### Deep\nlast body.\n";

fn at(session: &mut Session, row: usize, column: usize) {
    session.execute(Command::GotoLine);
    session.handle(Event::Paste((row + 1).to_string()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.execute(Command::Motion(Motion::LineStart));
    for _ in 0..column {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
}

fn pending(session: &Session) -> Frame {
    // No session.frame(): no replacement worker can start or publish here.
    Frame::project(session.buffer().unwrap(), session.view(), SIZE, "").unwrap()
}

fn samples(frame: &Frame, monochrome: bool) -> Vec<(usize, usize, surface::Cell)> {
    let cells = if monochrome {
        crossterm::style::force_color_output(false);
        let mut output = Vec::new();
        frame.write(&mut output).unwrap();
        crossterm::style::force_color_output(true);
        let ansi = std::str::from_utf8(&output).unwrap();
        assert!(!ansi.contains("[38;2;"));
        surface::decode(output)
    } else {
        surface::draw(frame)
    };
    let mut result = Vec::new();
    for (row, word) in [(0, "Root"), (3, "Child"), (10, "Deep")] {
        let byte = frame.rows()[row]
            .text()
            .find(word.chars().next().unwrap())
            .unwrap();
        let title = UnicodeWidthStr::width(&frame.rows()[row].text()[..byte]);
        result.push((row, title - 2, cells[&(row, title - 2)]));
        result.push((row, title, cells[&(row, title)]));
    }
    for row in [6, 7, 8] {
        let byte = frame.rows()[row].text().find(['•', '◦', '-', '1']).unwrap();
        let column = UnicodeWidthStr::width(&frame.rows()[row].text()[..byte]);
        result.push((row, column, cells[&(row, column)]));
    }
    result
}

// Each deliberately pending frame keeps three heading levels and neighboring
// nested/ordered markers under both pretty policies and all color treatments.
#[test]
fn continuity_frames() {
    for pretty in [true, false] {
        for colors in ["enabled", "disabled", "monochrome"] {
            check_frames(pretty, colors);
        }
    }
}

fn check_frames(pretty: bool, colors: &str) {
    let config = format!(
        "markdown.pretty={pretty}\nsyntax.enabled={}\npalette.markdown.heading_2='#123456'",
        colors != "disabled"
    );
    let (fixture, mut session) = editor(&config, "notes.md", SOURCE);
    let settings = prepare(&fixture.0.join("config.toml")).snapshot;
    let palette = &settings.settings().palette;
    settle(&mut session);
    let baseline = samples(
        &pending(&session).with_palette(palette),
        colors == "monochrome",
    );
    for (row, inserted) in [(4, "x"), (3, "界"), (4, "e\u{301}"), (3, "z"), (4, "q")] {
        at(&mut session, row, 4);
        session.handle(Event::Paste(inserted.into()));
        assert_eq!(
            samples(
                &pending(&session).with_palette(palette),
                colors == "monochrome"
            ),
            baseline,
            "{pretty}/{colors}/{row}"
        );
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
    let edited = text(session.buffer().unwrap());
    assert!(edited.contains('界'));
    assert!(edited.contains("e\u{301}"));
    assert_eq!(session.buffer().unwrap().history().len(), 5);
    settle(&mut session);
    assert_eq!(
        samples(
            &pending(&session).with_palette(palette),
            colors == "monochrome"
        ),
        baseline
    );
    assert_eq!(text(session.buffer().unwrap()), edited);
}

// Cancellation of carried evidence leaves literal source and no retry timer.
// An explicit re-enable requests fresh analysis at the same source revision.
#[test]
fn continuity_cancel() {
    let (_fixture, mut session) = editor("", "notes.md", SOURCE);
    settle(&mut session);
    at(&mut session, 4, 4);
    session.insert('z');
    assert!(pending(&session).rows()[3].text().contains('○'));
    session.execute(Command::KeyboardQuit);
    let frame = session.frame().unwrap();
    assert!(frame.rows()[3].text().contains("## Child"));
    assert!(frame.rows().iter().all(|row| row.syntax().is_empty()));
    assert_eq!(session.remaining_input(Instant::now()), None);
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::Markdown(MarkdownCommand::Pretty));
    session.execute(Command::Markdown(MarkdownCommand::Pretty));
    settle(&mut session);
    assert!(pending(&session).rows()[3].text().contains('○'));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// An explicit retry issued before canceled ownership is reaped must survive
// that worker's completion; no automatic retry or extra source edit is needed.
#[test]
fn continuity_retry() {
    let (_fixture, mut session) = editor("", "notes.md", SOURCE);
    settle(&mut session);
    at(&mut session, 4, 4);
    session.insert('z');
    session.frame().unwrap();
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    assert!(!pending(&session).rows()[3].text().contains('○'));
    session.execute(Command::Markdown(MarkdownCommand::Pretty));
    session.execute(Command::Markdown(MarkdownCommand::Pretty));
    settle(&mut session);
    assert!(pending(&session).rows()[3].text().contains('○'));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// Dirty lexical contexts are not resurrected by heading carry. A delimiter
// edit can retain heading roles while ordinary body inline faces disappear.
#[test]
fn continuity_lexical() {
    let source = SOURCE.replace("child body.", "child *body*.");
    let (_fixture, mut session) = editor("", "notes.md", &source);
    settle(&mut session);
    assert_eq!(
        style_at(&pending(&session), 4, "body"),
        Some(Style::Markdown(MarkdownRole::Emphasis))
    );
    at(&mut session, 3, 4);
    session.insert('*');
    assert!(pending(&session).rows()[3].text().contains('○'));
    assert!(pending(&session).rows()[4].syntax().is_empty());
    assert_eq!(
        style_at(&pending(&session), 3, "C"),
        Some(Style::Markdown(MarkdownRole::Heading(2)))
    );
}

// Carry cannot authorize a stale list edit. Source copying and BOM/CRLF save
// preserve the Unicode mutation, and native undo retires the display mapping.
#[test]
fn continuity_source() {
    let source = SOURCE.replace('\n', "\r\n");
    let (fixture, mut session) = editor("", "notes.md", &format!("\u{feff}{source}"));
    settle(&mut session);
    at(&mut session, 4, 4);
    session.handle(Event::Paste("界e\u{301}".into()));
    assert!(pending(&session).rows()[3].text().contains('○'));
    let edited = text(session.buffer().unwrap());
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(session.killed_text().unwrap().to_string(), "#");
    at(&mut session, 6, 3);
    session.execute(Command::Markdown(MarkdownCommand::IndentItem));
    assert_eq!(session.echo(), "Markdown structure is not current");
    assert_eq!(text(session.buffer().unwrap()), edited);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(fixture.0.join("notes.md")).unwrap(),
        format!("\u{feff}{edited}").as_bytes()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(session.buffer().unwrap()), source);
    assert!(!pending(&session).rows()[3].text().contains('○'));
}

// A native source edit must retire the whole carried presentation when its
// old row is a boundary, definition continuation, Setext title or literal form.
#[test]
fn continuity_refusals() {
    for (name, source, row, column) in [
        ("fence body", "```\nbody\n```\n\n## Child\n", 1, 2),
        ("fence opener", "```\nbody\n```\n\n## Child\n", 0, 0),
        ("definition", "[ref]: /a\n\n## Child\n", 0, 9),
        ("destination", "[ref]:\n /a\n\n## Child\n", 1, 2),
        ("Setext", "Title\n====\n\n## Child\n", 0, 2),
        ("code", "    body\n\n## Child\n", 0, 5),
        ("table", "| a | b |\n| - | - |\n\n## Child\n", 0, 3),
    ] {
        let (_fixture, mut session) = editor("", "notes.md", source);
        settle(&mut session);
        assert!(
            pending(&session)
                .rows()
                .iter()
                .any(|row| row.text().contains('○')),
            "base {name}"
        );
        at(&mut session, row, column);
        session.insert('z');
        let frame = pending(&session);
        assert!(
            !frame.rows().iter().any(|row| row.text().contains('○')),
            "{name}"
        );
        assert!(
            frame.rows().iter().all(|row| row.syntax().is_empty()),
            "{name}"
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
    }
}

// An offscreen opening/closing delimiter invalidates visible cross-line inline
// paint as a whole, while unrelated structural headings remain carried.
#[test]
fn continuity_viewport_context() {
    let source = "## Root\nordinary *open\ncontinued* tail\n\n### Deep\nbody\n";
    for (top, needle) in [(2, "*open"), (1, "* tail")] {
        let (_fixture, mut session) = editor("", "notes.md", source);
        settle(&mut session);
        let mut view = session.view().clone();
        view.top = top;
        let size = Size {
            width: 40,
            height: 3,
        };
        let before = Frame::project(session.buffer().unwrap(), &view, size, "").unwrap();
        assert!(
            before.rows()[0]
                .syntax()
                .iter()
                .any(|paint| paint.style == Style::Markdown(MarkdownRole::Emphasis))
        );
        let at = source.find(needle).unwrap();
        session.replace(at..at + 1, "").unwrap();
        view.point.stamp = session.buffer().unwrap().stamp();
        let current = Frame::project(session.buffer().unwrap(), &view, size, "").unwrap();
        assert!(current.rows()[0].syntax().is_empty());
        assert!(pending(&session).rows()[4].text().contains('✸'));
        settle(&mut session);
        assert!(
            Frame::project(session.buffer().unwrap(), &view, size, "")
                .unwrap()
                .rows()[0]
                .syntax()
                .is_empty()
        );
    }
}
