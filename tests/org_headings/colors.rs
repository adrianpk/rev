use rev::config::Rgb;
use rev::syntax::Style;

use super::*;

const COLORS: [Rgb; 8] = [
    Rgb(0x7a, 0xa2, 0xf7),
    Rgb(0x9d, 0x7c, 0xd8),
    Rgb(0xc0, 0xca, 0xf5),
    Rgb(0x9b, 0xb9, 0xf9),
    Rgb(0xb5, 0x9c, 0xe1),
    Rgb(0xbc, 0xd0, 0xfb),
    Rgb(0xce, 0xbd, 0xeb),
    Rgb(0xe4, 0xec, 0xfd),
];

// Actual terminal cells cycle eight faces independently of six markers, in
// pretty and source presentation. Drawing never edits or marks the source.
#[test]
fn independent_cycles() {
    let source: String = (1..=25)
        .map(|level| format!("{} Heading {level}\n", "*".repeat(level)))
        .collect();
    for pretty in [true, false] {
        let (_, mut buffers, view) = setup(&format!("org.headings.pretty={pretty}"), &source);
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        observe(buffer, Limits::default());
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 96,
                height: 30,
            },
            "",
        )
        .unwrap();
        let cells = surface::draw(&frame);
        for row in 0..25 {
            let expected = Some(COLORS[row % COLORS.len()]);
            let title = if pretty { 9 } else { 9 + row };
            assert_eq!(cells[&(row, 7)].foreground, expected, "bullet {row}");
            assert_eq!(cells[&(row, title)].foreground, expected, "title {row}");
            if pretty {
                assert_eq!(
                    cells[&(row, 7)].character,
                    ['◉', '○', '✸', '✿', '◆', '▶'][row % 6]
                );
            }
        }
        assert_eq!(text(buffer), source);
        assert!(!buffer.is_modified());
        assert!(buffer.history().is_empty());
    }
}

// Wrapped and clipped pieces resolve the original heading depth; an override
// changes its matching face only, while ordinary body text keeps its own color.
#[test]
fn projected_colors() {
    let source = "********** Heading 界é with enough words for several wrapped rows\nbody\n";
    for wrapped in [true, false] {
        let config = format!("editor.soft_wrap={wrapped}\npalette.org.heading_2='#012345'");
        let (fixture, mut buffers, mut view) = setup(&config, source);
        let snapshot = prepare(&fixture.0.join("config.toml")).snapshot;
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        observe(buffer, Limits::default());
        for left in if wrapped { vec![0] } else { vec![0, 2, 10] } {
            view.left = left;
            let frame = Frame::project(
                buffer,
                &view,
                Size {
                    width: 32,
                    height: 12,
                },
                "",
            )
            .unwrap()
            .with_palette(&snapshot.settings().palette);
            let painted = heading_cells(&frame, Rgb(1, 35, 69));
            assert!(painted > 0, "wrapped={wrapped}, left={left}");
            assert_eq!(text(buffer), source);
        }
    }
}

fn heading_cells(frame: &Frame, expected: Rgb) -> usize {
    let cells = surface::draw(frame);
    let mut painted = 0;
    for row in frame.rows() {
        for paint in row
            .syntax()
            .iter()
            .filter(|p| p.style == Style::Org(rev::org::Role::Heading))
        {
            let row = usize::from(row.row());
            let columns =
                (row, usize::from(paint.columns.start))..(row, usize::from(paint.columns.end));
            for (_, cell) in cells.range(columns) {
                assert_eq!(cell.foreground, Some(expected));
                painted += 1;
            }
        }
    }
    painted
}

// Selection and search override a non-default heading face in real terminal
// cells, retaining their own foreground/background and canonical source text.
#[test]
fn overlay_colors() {
    use crossterm::event::Event;
    use rev::command::{Motion, RegionCommand};

    let source = "** Heading\n";
    let palette = rev::config::Palette::default();
    for selected in [true, false] {
        let (_fixture, mut session) = editor("", source);
        settle(&mut session);
        assert_eq!(
            surface::draw(&session.frame().unwrap())[&(0, 7)].foreground,
            Some(COLORS[1])
        );
        let (column, foreground, background) = if selected {
            session.execute(Command::Region(RegionCommand::Set));
            session.execute(Command::Motion(Motion::LineEnd));
            (7, palette.selection_text, palette.selection)
        } else {
            session.execute(Command::SearchForward);
            session.handle(Event::Paste("Heading".into()));
            (9, palette.search_text, palette.search)
        };
        let cells = surface::draw(&session.frame().unwrap());
        assert_eq!(cells[&(0, column)].foreground, Some(foreground));
        assert_eq!(cells[&(0, column)].background, Some(background));
        assert_eq!(text(session.buffer().unwrap()), source);
        assert!(!session.buffer().unwrap().is_modified());
    }
}
