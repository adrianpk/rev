use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Mode;
use rev::command::{
    Command, History, MarkdownCommand, Motion, RegionCommand, Split, WindowCommand,
};
use rev::coordinates::{Coordinates, Scalar};
use rev::window::Area;

use super::*;

fn toggle(session: &mut Session) {
    session.execute(Command::Markdown(MarkdownCommand::Pretty));
}

fn reset(session: &mut Session) {
    session.execute(Command::Markdown(MarkdownCommand::ResetPresentation));
}

fn selector(session: &mut Session, name: &str, colon: bool) {
    session.handle(Event::Key(if colon {
        KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE)
    } else {
        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT)
    }));
    session.handle(Event::Paste(name.into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Both profile selectors admit readonly presentation and reset the false startup
// value; Text suspension refuses controls without losing their view provenance.
#[test]
fn controls_and_admission() {
    for (profile, colon) in [("non-modal", false), ("modal", false), ("modal", true)] {
        let fixture = Fixture::new();
        let config = format!("editor.interaction_profile='{profile}'\nmarkdown.pretty=false");
        let path = fixture.write("config.toml", &config);
        let startup = prepare(&path);
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers
            .visit(&fixture.write("notes.md", "## Title\n"))
            .unwrap();
        buffers.get_mut(id).unwrap().set_readonly(true);
        let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
        let mut session = Session::new(buffers, view, SIZE).unwrap();
        settle(&mut session);
        let point = session.view().point;
        let state = session.buffer().unwrap().state();
        for (name, choice) in [
            ("markdown-pretty-mode", Some(true)),
            ("markdown-pretty-mode", Some(false)),
            ("markdown-reset-presentation", None),
        ] {
            selector(&mut session, name, colon);
            assert_eq!(session.view().markdown_presentation_override(), choice);
            assert_eq!(session.view().pretty_markdown(), choice.unwrap_or(false));
            assert_eq!(session.view().point, point);
            assert_eq!(session.buffer().unwrap().state(), state);
        }
        toggle(&mut session);
        session.execute(Command::Mode(Mode::Text));
        let view = session.view().clone();
        toggle(&mut session);
        reset(&mut session);
        assert_eq!(session.echo(), "Native Markdown mode is not active");
        assert_eq!(session.view(), &view);
        assert!(
            session.frame().unwrap().rows()[0]
                .text()
                .contains("## Title")
        );
        session.execute(Command::Mode(Mode::Markdown));
        settle(&mut session);
        assert!(
            session.frame().unwrap().rows()[0]
                .text()
                .contains(" ○ Title")
        );
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(std::fs::read_to_string(path).unwrap(), config);
    }
}

// Splits inherit an effective choice by value, then keep independent overrides
// across mode/buffer switches, canonical source edits and undo/redo publication.
#[test]
fn split_and_source_lifecycle() {
    let (fixture, mut session) = editor("syntax.enabled=false", "notes.md", "## Title\n");
    settle(&mut session);
    toggle(&mut session);
    session.execute(Command::Split(Split::Right));
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, view)| view.markdown_presentation_override() == Some(false))
    );
    toggle(&mut session);
    let selected = session.windows().selected_id();
    session
        .load(&fixture.write("plain.txt", "## Plain\n"))
        .unwrap();
    settle(&mut session);
    assert!(
        session.frame().unwrap().rows()[0]
            .text()
            .contains("## Plain")
    );
    session.load(&fixture.0.join("notes.md")).unwrap();
    settle(&mut session);
    session.execute(Command::Motion(Motion::BufferStart));
    session.insert('#');
    settle(&mut session);
    for (id, view) in session.windows().iter() {
        assert_eq!(view.pretty_markdown(), id == selected);
        let frame = Frame::project(session.buffer().unwrap(), view, SIZE, "").unwrap();
        assert!(frame.rows()[0].text().contains(if id == selected {
            "  ✸ Title"
        } else {
            "### Title"
        }));
        assert!(frame.rows()[0].syntax().is_empty());
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), "## Title\n");
    session.execute(Command::History(History::Redo));
    settle(&mut session);
    assert!(session.view().pretty_markdown());
    reset(&mut session);
    assert_eq!(session.view().markdown_presentation_override(), None);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.view().markdown_presentation_override(), Some(false));
}

// Narrow wrapping, partial prefixes, wide/combining source and every prefix
// scalar share unchanged canonical hit/cursor maps in pretty and source views.
#[test]
fn source_geometry() {
    let source = "###### Title 界e\u{301} with words to wrap and clip\n";
    for wrapped in [true, false] {
        for pretty in [true, false] {
            let config = format!("editor.soft_wrap={wrapped}\nmarkdown.pretty={pretty}");
            let (_fixture, mut session) = editor(&config, "notes.md", source);
            settle(&mut session);
            let buffer = session.buffer().unwrap();
            let mut view = session.view().clone();
            for width in [10, 12, 23, 40] {
                geometry_width(buffer, &mut view, width, wrapped);
            }
            assert_eq!(text(buffer), source);
            assert!(buffer.history().is_empty());
        }
    }
}

fn geometry_width(buffer: &Buffer, view: &mut View, width: u16, wrapped: bool) {
    for left in if wrapped {
        vec![0]
    } else {
        vec![0, 2, 5, 6, 7]
    } {
        view.left = left;
        for scalar in 0..=buffer.len_chars() {
            view.point.position = Scalar(scalar);
            geometry_point(buffer, view, width);
        }
        let frame = Frame::project(buffer, view, Size { width, height: 20 }, "").unwrap();
        let visible = frame
            .rows()
            .iter()
            .flat_map(|row| row.text().chars())
            .collect::<String>();
        assert_eq!(
            visible.contains('▶'),
            view.pretty_markdown() && (wrapped || left <= 5 && left + usize::from(width - 7) > 5),
            "wrapped={wrapped}, width={width}, left={left}, text={visible:?}"
        );
    }
}

fn geometry_point(buffer: &Buffer, view: &View, width: u16) {
    let projected = Frame::project(buffer, view, Size { width, height: 20 }, "");
    let coordinates = Coordinates::new(buffer).unwrap();
    if matches!(
        coordinates.to_cell(view.point.position),
        Err(rev::coordinates::Error::Cluster(_))
    ) {
        assert!(matches!(
            projected,
            Err(rev::display::Error::Coordinates(
                rev::coordinates::Error::Cluster(_)
            ))
        ));
        return;
    }
    let frame = projected.unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .all(|row| row.text().width() == usize::from(width))
    );
    if let Some(cursor) = frame.cursor() {
        let area = Area::new(0, 0, width, 18).unwrap();
        let hit = view.hit(buffer, area, cursor).unwrap().unwrap();
        let cell = coordinates.to_cell(view.point.position).unwrap();
        assert_eq!(
            hit.position,
            coordinates.from_cell(cell).unwrap(),
            "{width}/{view:?}"
        );
    }
}

// Search colors override heading syntax; a selection crossing the substituted
// prefix has priority over search and copies canonical Markdown through yank.
#[test]
fn overlays_copy_and_save() {
    let source = "\u{feff}### Title 界e\u{301}\r\n";
    let (fixture, mut session) = editor("", "notes.md", source);
    settle(&mut session);
    let canonical = text(session.buffer().unwrap());
    let state = session.buffer().unwrap().state();
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("Title".into()));
    let palette = rev::config::Palette::default();
    let frame = session.frame().unwrap();
    let cells = surface::draw(&frame);
    assert_eq!(cells[&(0, 11)].foreground, Some(palette.search_text));
    assert_eq!(cells[&(0, 11)].background, Some(palette.search));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::LineEnd));
    let cells = surface::draw(&session.frame().unwrap());
    for column in [7, 9, 11] {
        assert_eq!(cells[&(0, column)].foreground, Some(palette.selection_text));
        assert_eq!(cells[&(0, column)].background, Some(palette.selection));
    }
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("Title".into()));
    let frame = session.frame().unwrap();
    assert!(frame.rows()[0].selection().unwrap().contains(&11));
    assert!(frame.rows()[0].search().unwrap().contains(&11));
    let cells = surface::draw(&frame);
    assert_eq!(cells[&(0, 11)].foreground, Some(palette.selection_text));
    assert_eq!(cells[&(0, 11)].background, Some(palette.selection));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::named("kill-ring-save").unwrap());
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::named("yank").unwrap());
    assert!(text(session.buffer().unwrap()).ends_with("### Title 界e\u{301}"));
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(text(session.buffer().unwrap()), canonical);
    assert_eq!(session.buffer().unwrap().state(), state);
    toggle(&mut session);
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.md")).unwrap(),
        source
    );
    assert!(session.buffer().unwrap().has_bom());
    assert!(!session.buffer().unwrap().is_modified());
}

// Monochrome retains symbols, reverse selection and underlined search;
// syntax-disabled admission still supplies pretty structural observations.
#[test]
fn monochrome_cues() {
    let (_fixture, mut session) = editor("syntax.enabled=false", "notes.md", "## Title\n");
    settle(&mut session);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::LineEnd));
    let frame = session.frame().unwrap();
    assert!(frame.rows()[0].syntax().is_empty());
    let cells = monochrome(&frame);
    assert_eq!(cells[&(0, 8)].character, '○');
    assert!(cells[&(0, 8)].reversed);
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("Title".into()));
    let cells = monochrome(&session.frame().unwrap());
    assert!(cells[&(0, 10)].underline);
    assert!(!cells[&(0, 10)].reversed);
    assert_eq!(text(session.buffer().unwrap()), "## Title\n");
}

fn monochrome(frame: &Frame) -> std::collections::BTreeMap<(usize, usize), surface::Cell> {
    crossterm::style::force_color_output(false);
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    crossterm::style::force_color_output(true);
    let ansi = std::str::from_utf8(&output).unwrap();
    assert!(!ansi.contains("[38;2;"));
    assert!(!ansi.contains("[48;2;"));
    surface::decode(output)
}

// Canceling pending work leaves no stale symbol or timer; explicit re-enabling
// retries that observation without changing the source revision or history.
#[test]
fn cancelled_observation() {
    let (_fixture, mut session) = editor("syntax.enabled=false", "notes.md", "## Title\n");
    let stamp = session.buffer().unwrap().stamp();
    session.frame().unwrap();
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    settle(&mut session);
    assert!(
        session.frame().unwrap().rows()[0]
            .text()
            .contains("## Title")
    );
    toggle(&mut session);
    toggle(&mut session);
    settle(&mut session);
    assert!(
        session.frame().unwrap().rows()[0]
            .text()
            .contains(" ○ Title")
    );
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.remaining_input(Instant::now()), None);
}
