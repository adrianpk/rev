use rev::buffer::edit::Point;
use rev::config::prepare;

use super::*;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, Motion, RegionCommand, Split, TreeCommand};
use rev::config::Palette;
use rev::panel::Composition;
use rev::session::Session;

fn session() -> (config_fixture::Fixture, Session, Palette) {
    let fixture = config_fixture::Fixture::new();
    let roles = [
        "background",
        "foreground",
        "cursor",
        "selection",
        "selection_text",
        "search",
        "search_text",
        "status",
        "status_text",
        "status_inactive",
        "border",
        "border_active",
        "tree",
        "tree_selected",
        "tree_selected_text",
        "message",
        "warning",
        "error",
    ];
    let config = roles
        .iter()
        .enumerate()
        .map(|(index, role)| {
            let channel = index + 1;
            format!("palette.{role} = '#{channel:02x}{channel:02x}{channel:02x}'\n")
        })
        .collect::<String>();
    let startup = prepare(&fixture.write("config.toml", &config));
    assert!(startup.diagnostic.is_none());
    let palette = startup.snapshot.settings().palette.clone();
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers
        .visit(&fixture.write("main.rs", "fn alpha() {}\n// note\n"))
        .unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    (fixture, session, palette)
}

fn colors(cell: &surface::Cell, foreground: rev::config::Rgb, background: rev::config::Rgb) {
    assert_eq!(
        (cell.foreground, cell.background),
        (Some(foreground), Some(background)),
        "{cell:?}"
    );
}

fn key(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

// Actual cell colors survive mixed status/document rows and shared junctions.
// Focusing the tree recolors only its own selection/status and adjacent border.
#[test]
fn panel_colors() {
    let (_fixture, mut session, palette) = session();
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Split(Split::Below));
    let frame = session.frame().unwrap();
    let cells = surface::draw(&frame);
    let composition = Composition::workspace(
        session.windows(),
        session.tree(),
        Size {
            width: 80,
            height: 12,
        },
        0,
    )
    .unwrap();
    for (_, panel) in composition.panels() {
        let status = panel.status().unwrap();
        let background = if panel.active() {
            palette.status
        } else {
            palette.status_inactive
        };
        colors(
            &cells[&(usize::from(status.row()), usize::from(status.column()))],
            palette.status_text,
            background,
        );
    }
    let borders: Vec<_> = cells
        .values()
        .filter(|cell| "│─┤├┬┴┼".contains(cell.character))
        .map(|cell| cell.foreground)
        .collect();
    assert!(borders.contains(&Some(palette.border_active)));
    assert!(borders.contains(&Some(palette.border)));
    colors(&cells[&(0, 2)], palette.foreground, palette.background);
    session.execute(Command::Tree(TreeCommand::Focus));
    let frame = session.frame().unwrap();
    let cells = surface::draw(&frame);
    let cursor = frame.cursor().unwrap();
    colors(
        &cells[&(usize::from(cursor.row), 0)],
        palette.tree_selected_text,
        palette.tree_selected,
    );
    colors(
        &cells[&(usize::from(cursor.row) + 1, 0)],
        palette.foreground,
        palette.tree,
    );
    colors(&cells[&(10, 0)], palette.status_text, palette.status);
    colors(
        &cells[&(10, 25)],
        palette.status_text,
        palette.status_inactive,
    );
    assert_eq!(session.buffer().unwrap().history().len(), 0);
    assert!(!session.buffer().unwrap().is_modified());
}

// Search owns text/background over syntax; a simultaneous region wins while
// search underlining remains. Later cells and status/prompt cannot inherit it.
#[test]
fn search_colors() {
    let (_fixture, mut session, palette) = session();
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("alpha".into()));
    let cells = surface::draw(&session.frame().unwrap());
    colors(&cells[&(0, 3)], palette.search_text, palette.search);
    assert!(cells[&(0, 3)].underline);
    colors(
        &cells[&(0, 8)],
        palette.syntax.punctuation,
        palette.background,
    );
    assert!(!cells[&(0, 8)].underline);
    key(&mut session, KeyCode::Enter);
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("alpha".into()));
    let frame = session.frame().unwrap();
    let cells = surface::draw(&frame);
    colors(&cells[&(0, 3)], palette.selection_text, palette.selection);
    assert!(cells[&(0, 3)].underline);
    colors(&cells[&(10, 0)], palette.status_text, palette.status);
    colors(&cells[&(11, 0)], palette.message, palette.background);
    assert!(!cells[&(10, 0)].underline && !cells[&(11, 0)].underline);
    assert_eq!(session.buffer().unwrap().history().len(), 0);
}

// Real handler outcomes supply warning/error roles without parsing message
// wording. An ensuing prompt returns to the ordinary message color.
#[test]
fn message_colors() {
    let (fixture, mut session, palette) = session();
    let stamp = session.buffer().unwrap().stamp();
    session.insert('\u{1b}');
    let cells = surface::draw(&session.frame().unwrap());
    colors(&cells[&(11, 0)], palette.warning, palette.background);
    let invalid = fixture.write("invalid.txt", "placeholder");
    std::fs::write(&invalid, [0xff]).unwrap();
    assert!(session.load(&invalid).is_err());
    let cells = surface::draw(&session.frame().unwrap());
    colors(&cells[&(11, 0)], palette.error, palette.background);
    session.execute(Command::FindFile);
    let cells = surface::draw(&session.frame().unwrap());
    colors(&cells[&(11, 0)], palette.message, palette.background);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(!session.buffer().unwrap().is_modified());
}

// Control-looking source and messages are visible data; only typed palette RGB
// can produce color controls. A rendered frame never emits cursor OSC itself.
#[test]
fn safe_colors() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("\x1b]12;red\x07").unwrap();
    let buffer = buffers.get(id).unwrap();
    let frame = Frame::project(
        buffer,
        &View::new(buffer),
        Size {
            width: 30,
            height: 4,
        },
        "\x1b]112\x07",
    )
    .unwrap();
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let wire = String::from_utf8(output).unwrap();
    assert!(!wire.contains("\x1b]"));
    assert!(!frame.rows()[0].text().contains('\x1b'));
    let cells = surface::draw(&frame);
    let palette = Palette::default();
    colors(&cells[&(0, 0)], palette.foreground, palette.background);
    colors(&cells[&(3, 0)], palette.message, palette.background);
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        "\x1b]12;red\x07"
    );
}

// A selected glyph suppresses its capture foreground, including a selection
// starting/ending inside one token. Leaving the region restores the capture.
#[test]
fn selected_syntax() {
    crossterm::style::force_color_output(true);
    let fixture = config_fixture::Fixture::new();
    let path = fixture.write("main.rs", "fn alpha() {}\n");
    let startup = prepare(&fixture.write("config.toml", "palette.syntax.function = '#012345'"));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    for (name, start, end) in [("linear keyword", 1, 2), ("rectangle function", 5, 6)] {
        let mark = Point {
            stamp: buffer.stamp(),
            position: Scalar(start),
        };
        buffer.activate_mark(mark).unwrap();
        if name.starts_with("rectangle") {
            buffer.toggle_rectangle(mark).unwrap();
        }
        let mut view = View::new(buffer);
        view.point.position = Scalar(end);
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 30,
                height: 4,
            },
            "",
        )
        .unwrap()
        .with_palette(&startup.snapshot.settings().palette);
        assert_eq!(frame.rows()[0].selection(), Some(start as u16..end as u16));
        assert!(!frame.rows()[0].syntax().is_empty());
        let cells = surface::draw(&frame);
        let palette = &startup.snapshot.settings().palette;
        assert_eq!(
            cells[&(0, start)].foreground,
            Some(palette.selection_text),
            "{name}"
        );
        assert_eq!(
            cells[&(0, start)].background,
            Some(palette.selection),
            "{name}"
        );
        let after = if end == 6 {
            palette.syntax.function
        } else {
            palette.foreground
        };
        assert_eq!(cells[&(0, end)].foreground, Some(after), "{name}");
        assert_eq!(
            cells[&(0, end)].background,
            Some(palette.background),
            "{name}"
        );
        assert_eq!(buffer.stamp(), frame.stamp());
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}
