#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod config_fixture;
#[path = "display/org_colors.rs"]
mod org_colors;
#[path = "display/policy.rs"]
mod policy;
#[path = "display/status.rs"]
mod status;

#[path = "display/palette.rs"]
mod palette;
#[path = "support/surface.rs"]
mod surface;

use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Band, Frame, Position, Size};
use rev::view::View;
use unicode_width::UnicodeWidthStr;

// The initial frame borrows the canonical empty buffer, reserves distinct
// document/status/echo rows, and places the active cursor at the first text cell after numbering.
#[test]
fn initial_frame() {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let view = View::new(buffer);
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 80,
            height: 24,
        },
        "C-x C-c: exit",
    )
    .unwrap();
    assert_eq!(frame.stamp(), buffer.stamp());
    assert_eq!(frame.cursor(), Some(Position { column: 7, row: 0 }));
    assert_eq!(frame.rows().len(), 24);
    for (index, row) in frame.rows()[..22].iter().enumerate() {
        let prefix = if index == 0 { "   1 │" } else { "     │" };
        assert_eq!(row.band(), Band::Document);
        assert!(row.text().strip_prefix(prefix).unwrap().trim().is_empty());
    }
    assert_eq!(frame.rows()[22].band(), Band::Status);
    assert!(
        frame.rows()[22]
            .text()
            .starts_with("> *scratch* -- RW -- L1 C1")
    );
    assert_eq!(frame.rows()[23].band(), Band::Echo);
    assert!(frame.rows()[22].text().ends_with("Text"));
    assert!(frame.rows()[23].text().starts_with("C-x C-c: exit"));
    for (index, row) in frame.rows().iter().enumerate() {
        assert_eq!(usize::from(row.row()), index);
        assert_eq!(row.text().width(), 80);
    }
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

// Factory wrapping reveals continuation rows; explicit no-wrap retains horizontal
// cell scrolling. Both modes preserve the source-qualified point and history.
#[test]
fn viewport_reveal() {
    let fixture = config_fixture::Fixture::new();
    for (wrap, top, left, first) in [(true, 2, 0, "a   "), (false, 1, 4, "nd  ")] {
        let startup = rev::config::prepare(
            &fixture.write("config.toml", &format!("editor.soft_wrap = {wrap}")),
        );
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers
            .create("first\nsecond\na\t界e\u{301}\nlast")
            .unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.bookmark_markers = false;
        view.point.position = Scalar(18);
        let size = Size {
            width: 4,
            height: 4,
        };
        view.reveal(buffer, size).unwrap();
        assert_eq!((view.top, view.left), (top, left));
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        assert_eq!(frame.rows()[0].text(), first);
        assert_eq!(frame.rows()[1].text(), "界e\u{301} ");
        assert_eq!(frame.cursor(), Some(Position { column: 3, row: 1 }));
        let mut independent = View::new(buffer);
        independent.bookmark_markers = false;
        independent.reveal(buffer, size).unwrap();
        assert_eq!((independent.top, independent.left), (0, 0));
        assert_eq!(buffer.stamp(), view.point.stamp);
    }
}

fn bounded(frame: &Frame, size: Size) {
    for row in frame.rows() {
        assert!(row.row() < size.height);
        assert_eq!(row.text().width(), usize::from(size.width));
        assert!(!row.text().chars().any(char::is_control));
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < size.width);
        assert!(cursor.row < size.text_rows());
    }
}

// Cell clipping never prints half a wide/emoji cluster or shifts following
// text. Tabs use logical stops even when their leading cells are offscreen.
#[test]
fn mixed_clipping() {
    let cases = [
        ("right wide", "ab界z", 0, 3, "ab "),
        ("left wide", "ab界z", 3, 3, " z "),
        ("whole wide", "ab界z", 2, 3, "界z"),
        ("both edges", "界界界", 1, 3, " 界"),
        ("tab fragment", "a\t界e\u{301}", 2, 4, "  界"),
        ("tab stops", "\t\tX", 3, 6, "     X"),
        ("combining", "ae\u{301}z", 1, 3, "e\u{301}z "),
        ("emoji edge", "a👩\u{200d}💻z", 2, 3, " z "),
        ("leading mark", "\u{301}x", 0, 3, "◌\u{301}x "),
        ("CRLF", "a\r\nb", 0, 3, "a  "),
    ];
    let fixture = config_fixture::Fixture::new();
    for wrap in [true, false] {
        let startup = rev::config::prepare(
            &fixture.write("config.toml", &format!("editor.soft_wrap = {wrap}")),
        );
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        for (name, source, left, width, unwrapped) in cases {
            let expected = if wrap {
                match name {
                    "right wide" | "left wide" | "whole wide" => "ab ",
                    "both edges" => "界 ",
                    "tab fragment" => "a   ",
                    "tab stops" => "      ",
                    "combining" => "ae\u{301}z",
                    "emoji edge" => "a👩\u{200d}💻",
                    _ => unwrapped,
                }
            } else {
                unwrapped
            };
            let id = buffers.create(source).unwrap();
            let buffer = buffers.get(id).unwrap();
            let mut view = View::new(buffer);
            view.bookmark_markers = false;
            view.left = left;
            let size = Size { width, height: 3 };
            let frame = Frame::project(buffer, &view, size, "").unwrap();
            assert_eq!(frame.rows()[0].text(), expected, "{name}, wrap={wrap}");
            bounded(&frame, size);
            assert_eq!(
                buffer
                    .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                    .unwrap()
                    .to_string(),
                source,
                "{name}"
            );
            assert!(!buffer.is_modified(), "{name}");
        }
    }
}

// Document and message controls use safe visible glyphs; the separate echo row
// can report an error without replacing the document or its mode indication.
#[test]
fn safe_echo() {
    let mut buffers = Buffers::new().unwrap();
    let source = "x\x1b[2J\u{9b}";
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    let size = Size {
        width: 40,
        height: 4,
    };
    let frame = Frame::project(buffer, &view, size, "Save failed: denied\x1b[2J\n").unwrap();
    assert!(frame.rows()[0].text().starts_with("   1 │x^[[2J<9B>"));
    assert!(frame.rows()[2].text().contains("-- RW -- L1 C1"));
    assert!(frame.rows()[2].text().ends_with("Text"));
    assert!(
        frame.rows()[3]
            .text()
            .starts_with("Save failed: denied^[[2J^J")
    );
    bounded(&frame, size);
    assert_eq!(buffer.len_bytes(), source.len());
    assert!(buffer.history().is_empty());
}

// Every tiny shape, including zero dimensions, is a valid suspended frame.
// Resizing presentation cannot rewrite text, point, or its revision stamp.
#[test]
fn tiny_geometry() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界e\u{301}\r\nlast").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut original = View::new(buffer);
    original.bookmark_markers = false;
    original.point.position = Scalar(buffer.len_chars());
    original.top = 1;
    original.left = 2;
    for (width, height) in (0..6).flat_map(|width| (0..6).map(move |height| (width, height))) {
        let size = Size { width, height };
        let mut view = original.clone();
        view.reveal(buffer, size).unwrap();
        let frame = Frame::project(buffer, &view, size, "error").unwrap();
        bounded(&frame, size);
        assert_eq!(
            frame.rows().len(),
            if width == 0 { 0 } else { usize::from(height) }
        );
        assert_eq!(view.point, original.point);
        if !size.usable() {
            assert_eq!(view, original);
            assert_eq!(frame.cursor(), None);
            assert!(frame.rows().iter().all(|row| row.band() != Band::Status));
        }
    }
    assert_eq!(buffer.stamp(), original.point.stamp);
    assert!(!buffer.is_modified());
}

// Native dimension limits and excessive scroll offsets cannot overflow row or
// cell arithmetic. A usable reveal repairs scroll without touching the point.
#[test]
fn extreme_geometry() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界\tend").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.top = usize::MAX;
    view.left = usize::MAX;
    for size in [
        Size {
            width: u16::MAX,
            height: 1,
        },
        Size {
            width: 1,
            height: u16::MAX,
        },
        Size {
            width: 3,
            height: 3,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        bounded(&frame, size);
        assert_eq!(frame.cursor(), None);
    }
    view.top = 0;
    let clipped = Frame::project(
        buffer,
        &view,
        Size {
            width: 3,
            height: 3,
        },
        "",
    )
    .unwrap();
    assert_eq!(clipped.rows()[0].text(), "界 ");
    assert_eq!(clipped.cursor(), Some(Position { column: 0, row: 0 }));
    view.reveal(
        buffer,
        Size {
            width: 3,
            height: 3,
        },
    )
    .unwrap();
    assert_eq!((view.top, view.left), (0, 0));
    assert_eq!(view.point.position, Scalar(0));
}

// Reprojection reads both current rope text and current metadata. A read-only
// change needs no text revision to update status, and an edit updates it once.
#[test]
fn fresh_projection() {
    use rev::buffer::edit::Replacement;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    let size = Size {
        width: 48,
        height: 4,
    };
    let before = Frame::project(buffer, &view, size, "").unwrap();
    let edits = [Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "a\t界e\u{301}".into(),
    }];
    let prepared = buffer
        .prepare(&edits, std::slice::from_ref(&view.point), None)
        .unwrap();
    buffer
        .publish(prepared, std::slice::from_mut(&mut view.point))
        .unwrap();
    let edited = Frame::project(buffer, &view, size, "Save failed: denied").unwrap();
    assert_ne!(before.stamp(), edited.stamp());
    assert_eq!(edited.stamp(), buffer.stamp());
    assert!(edited.rows()[0].text().starts_with("   1 │a   界e\u{301}"));
    assert!(
        edited.rows()[2]
            .text()
            .starts_with("> *scratch* ** RW -- L1 C8")
    );
    buffer.set_readonly(true);
    assert!(edited.rows()[2].text().ends_with("Text"));
    let readonly = Frame::project(buffer, &view, size, "Save failed: denied").unwrap();
    assert_eq!(readonly.stamp(), edited.stamp());
    assert!(
        readonly.rows()[2]
            .text()
            .starts_with("> *scratch* ** RO -- L1 C8")
    );
    assert_eq!(readonly.rows()[0].text(), edited.rows()[0].text());
    assert!(readonly.rows()[2].text().ends_with("Text"));
    assert!(readonly.rows()[3].text().starts_with("Save failed: denied"));
    assert_eq!(buffer.history().len(), 1);
    bounded(&readonly, size);
}

// Projection rejects foreign/stale points and invalid scalar/grapheme
// boundaries instead of displaying a cursor detached from its source revision.
#[test]
fn qualified_views() {
    use rev::buffer::{Error as Source, Stamp};
    use rev::coordinates::Error as Coordinate;
    use rev::view::Error;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}界").unwrap();
    let other = buffers.initial();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let cases = [
        (
            "foreign",
            Stamp {
                buffer: other,
                ..stamp
            },
            0,
            Error::Buffer(Source::Foreign {
                expected: id,
                actual: other,
            }),
        ),
        (
            "stale",
            Stamp {
                revision: 0,
                ..stamp
            },
            0,
            Error::Buffer(Source::Stale {
                expected: 1,
                actual: 0,
            }),
        ),
        (
            "outside",
            stamp,
            4,
            Error::Buffer(Source::Bounds {
                start: 4,
                end: 4,
                len: 3,
            }),
        ),
        (
            "inside cluster",
            stamp,
            1,
            Error::Coordinates(Coordinate::Cluster(Scalar(1))),
        ),
    ];
    for (name, stamp, position, expected) in cases {
        let mut view = View::new(buffer);
        view.bookmark_markers = false;
        view.point.stamp = stamp;
        view.point.position = Scalar(position);
        assert_eq!(
            Frame::project(
                buffer,
                &view,
                Size {
                    width: 8,
                    height: 3
                },
                ""
            )
            .err(),
            Some(expected),
            "{name}"
        );
    }
    assert!(!buffer.is_modified());
}

#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    flushes: usize,
}

impl std::io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushes += 1;
        Ok(())
    }
}

// The actual output adapter writes admitted row moves with safe full-width
// payloads and one deliberate flush; zero geometry emits only cursor hiding.
#[test]
fn draw_boundary() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\x1b[2J").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    for size in [
        Size {
            width: 8,
            height: 3,
        },
        Size {
            width: 0,
            height: 0,
        },
        Size {
            width: 2,
            height: 2,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "\x1b[2J").unwrap();
        bounded(&frame, size);
        let mut output = Output::default();
        frame.write(&mut output).unwrap();
        assert_eq!(output.flushes, 1);
        let wire = String::from_utf8(output.bytes).unwrap();
        assert!(wire.starts_with("\x1b[?25l"));
        assert_eq!(wire.ends_with("\x1b[?25h"), frame.cursor().is_some());
        assert!(!wire.contains("\x1b[2J"));
        let plain = without_sgr(&wire);
        for row in frame.rows() {
            assert!(plain.contains(&format!("\x1b[{};1H{}", row.row() + 1, row.text())));
        }
        if size.width == 0 {
            assert_eq!(wire, "\x1b[?25l");
        }
    }
}

fn editor(source: &str, point: rev::coordinates::Cell, size: Size) -> rev::session::Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.point.position = rev::coordinates::Coordinates::new(buffer)
        .unwrap()
        .from_cell(point)
        .unwrap();
    rev::session::Session::new(buffers, view, size).unwrap()
}

fn navigate(session: &mut rev::session::Session, name: &str, point: rev::coordinates::Cell) {
    let command = rev::command::Command::named(name).expect("implemented navigation command");
    assert_eq!(session.execute(command), rev::session::Action::Redraw);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        point,
        "{name}"
    );
    assert!(session.echo().is_empty(), "{name}");
}

// Consecutive vertical commands retain their intended display column rather
// than adopting the clamped end of an intervening short logical line.
#[test]
fn desired_column() {
    use rev::coordinates::Cell;
    let size = Size {
        width: 8,
        height: 4,
    };
    let mut session = editor("abcdef\nx\nabcdef", Cell { row: 0, column: 5 }, size);
    let stamp = session.buffer().unwrap().stamp();
    for row in [1, 2, 1, 0] {
        let command = if row < session.view().cell(session.buffer().unwrap()).unwrap().row {
            "previous-line"
        } else {
            "next-line"
        };
        let column = if row == 1 { 1 } else { 5 };
        navigate(&mut session, command, Cell { row, column });
        assert_eq!(session.view().column, Some(5));
        bounded(&session.frame().unwrap(), size);
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

// Repeated C-l command dispatch cycles the selected logical line through
// center, top and bottom while preserving point and document state.
#[test]
fn recenter_cycle() {
    use rev::coordinates::Cell;
    let size = Size {
        width: 8,
        height: 7,
    };
    let point = Cell { row: 10, column: 2 };
    let mut session = editor(&"line\n".repeat(20), point, size);
    let stamp = session.view().point;
    for top in [8, 10, 6, 8, 10, 6] {
        navigate(&mut session, "recenter-top-bottom", point);
        assert_eq!(session.view().top, top);
        assert_eq!(session.view().point, stamp);
        bounded(&session.frame().unwrap(), size);
    }
    navigate(&mut session, "next-line", Cell { row: 11, column: 2 });
    navigate(
        &mut session,
        "recenter-top-bottom",
        Cell { row: 11, column: 2 },
    );
    assert_eq!(session.view().top, 9);
    navigate(
        &mut session,
        "recenter-top-bottom",
        Cell { row: 11, column: 2 },
    );
    assert_eq!(session.view().top, 11);
    session.execute(rev::command::Command::KeyboardQuit);
    navigate(
        &mut session,
        "recenter-top-bottom",
        Cell { row: 11, column: 2 },
    );
    assert_eq!(session.view().top, 9);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A goal inside a tab or wide cluster snaps to its start without replacing
// the goal; later long lines recover it across both LF and CRLF separators.
#[test]
fn cell_goals() {
    use rev::coordinates::Cell;
    let source = "0123456789\r\na\t界e\u{301}👩\u{200d}💻\r\nx\n0123456789";
    let size = Size {
        width: 5,
        height: 4,
    };
    for (name, goal, snapped) in [
        ("tab", 2, 1),
        ("wide", 5, 4),
        ("emoji edge", 7, 7),
        ("emoji interior", 8, 7),
    ] {
        let mut wrapped = editor(
            source,
            Cell {
                row: 0,
                column: goal,
            },
            size,
        );
        navigate(
            &mut wrapped,
            "next-line",
            Cell {
                row: 0,
                column: if goal < 5 { goal + 5 } else { 10 },
            },
        );
        assert_eq!(wrapped.view().column, Some(goal % 5), "{name}");
        navigate(
            &mut wrapped,
            "previous-line",
            Cell {
                row: 0,
                column: goal,
            },
        );
        assert_eq!(wrapped.view().column, Some(goal % 5), "{name}");
        let mut session = editor(
            source,
            Cell {
                row: 0,
                column: goal,
            },
            size,
        );
        session.execute(rev::command::Command::VisualLineMode);
        for (command, row, column) in [
            ("next-line", 1, snapped),
            ("next-line", 2, 1),
            ("next-line", 3, goal),
            ("previous-line", 2, 1),
            ("previous-line", 1, snapped),
            ("previous-line", 0, goal),
        ] {
            navigate(&mut session, command, Cell { row, column });
            assert_eq!(session.view().column, Some(goal), "{name}");
            bounded(&session.frame().unwrap(), size);
        }
    }
}

// Pages advance the viewport and point together with two context rows, retain
// the goal through short destinations, and clamp at both document boundaries.
#[test]
fn page_navigation() {
    use rev::coordinates::Cell;
    let source = (0..15)
        .map(|row| match row {
            3 => "x",
            9 => "a\t界",
            _ => "0123456789",
        })
        .collect::<Vec<_>>()
        .join("\n");
    let size = Size {
        width: 6,
        height: 7,
    };
    let mut session = editor(&source, Cell { row: 0, column: 8 }, size);
    navigate(
        &mut session,
        "scroll-up-command",
        Cell { row: 2, column: 2 },
    );
    assert_eq!(session.view().column, Some(2));
    navigate(
        &mut session,
        "scroll-down-command",
        Cell { row: 0, column: 8 },
    );
    assert_eq!(session.view().column, Some(2));
    let mut session = editor(&source, Cell { row: 0, column: 8 }, size);
    session.execute(rev::command::Command::VisualLineMode);
    session.frame().unwrap();
    let cases = [
        ("scroll-up-command", 3, 1, 3),
        ("scroll-up-command", 6, 8, 4),
        ("scroll-up-command", 9, 6, 7),
        ("scroll-up-command", 12, 8, 10),
        ("scroll-up-command", 14, 8, 12),
        ("scroll-up-command", 14, 8, 12),
        ("scroll-down-command", 11, 8, 9),
        ("scroll-down-command", 8, 8, 6),
        ("scroll-down-command", 5, 8, 3),
        ("scroll-down-command", 2, 8, 0),
        ("scroll-down-command", 0, 8, 0),
        ("scroll-down-command", 0, 8, 0),
    ];
    for (command, row, column, top) in cases {
        navigate(&mut session, command, Cell { row, column });
        assert_eq!(session.view().top, top);
        assert_eq!(session.view().column, Some(8));
        navigate(&mut session, "recenter-top-bottom", Cell { row, column });
        assert_eq!(session.view().column, Some(8));
        bounded(&session.frame().unwrap(), size);
    }
}

// Nonvertical movement and insertion discard a prior goal, so later vertical
// movement uses the newly chosen cell instead of resurrecting an old column.
#[test]
fn navigation_reset() {
    use rev::coordinates::Cell;
    let size = Size {
        width: 8,
        height: 5,
    };
    for wrap in [true, false] {
        let mut session = editor(
            "0123456789\nx\n0123456789",
            Cell { row: 0, column: 8 },
            size,
        );
        if !wrap {
            session.execute(rev::command::Command::VisualLineMode);
        }
        navigate(
            &mut session,
            "next-line",
            Cell {
                row: 1,
                column: if wrap { 0 } else { 1 },
            },
        );
        assert_eq!(session.view().column, Some(if wrap { 0 } else { 8 }));
        navigate(
            &mut session,
            "move-beginning-of-line",
            Cell { row: 1, column: 0 },
        );
        assert_eq!(session.view().column, None);
        navigate(&mut session, "next-line", Cell { row: 2, column: 0 });
        navigate(
            &mut session,
            "move-end-of-line",
            Cell {
                row: 2,
                column: if wrap { 7 } else { 10 },
            },
        );
        navigate(&mut session, "previous-line", Cell { row: 1, column: 1 });
        assert_eq!(session.view().column, Some(if wrap { 7 } else { 10 }));
        assert_eq!(session.insert('!'), rev::session::Action::Redraw);
        assert_eq!(session.view().column, None);
        navigate(
            &mut session,
            "previous-line",
            Cell {
                row: 0,
                column: if wrap { 10 } else { 2 },
            },
        );
        assert_eq!(session.view().column, Some(2));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}

// Suspended presentation keeps goals and scroll intact. A one-row pane still
// advances; usable wrap-width changes reseed goals, while no-wrap keeps columns.
#[test]
fn suspended_navigation() {
    use crossterm::event::Event;
    use rev::coordinates::Cell;
    for wrap in [true, false] {
        for (width, height) in [(0, 0), (1, 1), (2, 2), (3, 3)] {
            let size = Size { width, height };
            let mut session = editor(
                "abcdef\nx\nabcdef",
                Cell { row: 0, column: 5 },
                Size {
                    width: 8,
                    height: 7,
                },
            );
            if !wrap {
                session.execute(rev::command::Command::VisualLineMode);
            }
            navigate(&mut session, "next-line", Cell { row: 1, column: 1 });
            let origin = (session.view().top, session.view().left);
            session.handle(Event::Resize(width, height));
            let goal = if wrap && size.usable() { 1 } else { 5 };
            let point = if size.usable() {
                Cell {
                    row: 2,
                    column: goal,
                }
            } else {
                Cell { row: 1, column: 1 }
            };
            navigate(&mut session, "scroll-up-command", point);
            navigate(
                &mut session,
                "scroll-down-command",
                Cell { row: 1, column: 1 },
            );
            for _ in 0..4 {
                navigate(
                    &mut session,
                    "recenter-top-bottom",
                    Cell { row: 1, column: 1 },
                );
                assert_eq!(session.view().column, Some(goal));
                bounded(&session.frame().unwrap(), size);
            }
            if !size.usable() {
                assert_eq!((session.view().top, session.view().left), origin);
                assert!(session.frame().unwrap().cursor().is_none());
            }
            session.handle(Event::Resize(8, 7));
            navigate(
                &mut session,
                "next-line",
                Cell {
                    row: 2,
                    column: goal,
                },
            );
            assert_eq!(session.view().column, Some(goal));
            bounded(
                &session.frame().unwrap(),
                Size {
                    width: 8,
                    height: 7,
                },
            );
        }
    }
}

// Reported Emacs and arrow/Page bindings reach the same native handlers.
// Repeats move once; a release cannot move point or advance the recenter cycle.
#[test]
fn scroll_bindings() {
    use crossterm::event::{
        Event, KeyCode as Code, KeyEvent, KeyEventKind as Kind, KeyModifiers as Mod,
    };
    use rev::coordinates::Cell;
    let size = Size {
        width: 8,
        height: 7,
    };
    let cases = [
        (Code::Char('n'), Mod::CONTROL, 11, 7),
        (Code::Down, Mod::NONE, 11, 7),
        (Code::Char('p'), Mod::CONTROL, 9, 6),
        (Code::Up, Mod::NONE, 9, 6),
        (Code::Char('v'), Mod::CONTROL, 13, 9),
        (Code::PageDown, Mod::NONE, 13, 9),
        (Code::Char('v'), Mod::ALT, 7, 3),
        (Code::PageUp, Mod::NONE, 7, 3),
        (Code::Char('l'), Mod::CONTROL, 10, 8),
    ];
    for (code, modifiers, row, top) in cases {
        let mut session = editor(&"line\n".repeat(20), Cell { row: 10, column: 3 }, size);
        session.frame().unwrap();
        let before = session.view().clone();
        let event = |kind| Event::Key(KeyEvent::new_with_kind(code, modifiers, kind));
        assert_eq!(
            session.handle(event(Kind::Release)),
            rev::session::Action::Wait
        );
        assert_eq!(session.view(), &before);
        assert_eq!(
            session.handle(event(Kind::Repeat)),
            rev::session::Action::Redraw
        );
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            Cell { row, column: 3 }
        );
        assert_eq!(session.view().top, top);
        bounded(&session.frame().unwrap(), size);
    }
}

// Scrolling read-only dirty text preserves its stored mark, revision, newline
// metadata and existing reversible history rather than publishing an edit.
#[test]
fn scroll_state() {
    use rev::buffer::edit::Replacement;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let source = "a\t界e\u{301}\r\n".repeat(20);
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: source.clone(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    buffer.set_readonly(true);
    let stamp = buffer.stamp();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    let size = Size {
        width: 5,
        height: 7,
    };
    let mut session = rev::session::Session::new(buffers, view, size).unwrap();
    for name in [
        "next-line",
        "scroll-up-command",
        "recenter-top-bottom",
        "scroll-down-command",
        "previous-line",
    ] {
        let command = rev::command::Command::named(name).unwrap();
        assert_eq!(session.execute(command), rev::session::Action::Redraw);
        assert!(session.echo().is_empty());
        bounded(&session.frame().unwrap(), size);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.mark(), Some(Scalar(3)));
    assert_eq!(buffer.history().len(), 1);
    assert!(buffer.is_modified() && buffer.is_readonly());
    assert_eq!(buffer.newline(), "\r\n");
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        source
    );
}

// Empty buffers and the first/last logical lines admit repeated page, line and
// recenter commands without invented rows, empty edits or invalid coordinates.
#[test]
fn scroll_limits() {
    use rev::command::Command;
    use rev::coordinates::Cell;
    let size = Size {
        width: 8,
        height: 7,
    };
    let mut empty = editor("", Cell { row: 0, column: 0 }, size);
    for command in Command::ALL {
        if let Command::Scroll(_) = command {
            navigate(&mut empty, command.name(), Cell { row: 0, column: 0 });
        }
    }
    let mut session = editor("abcdef\nx\nabcdef", Cell { row: 2, column: 5 }, size);
    for (command, row, column) in [
        ("next-line", 2, 5),
        ("scroll-up-command", 2, 5),
        ("previous-line", 1, 1),
        ("scroll-down-command", 0, 5),
        ("previous-line", 0, 5),
        ("scroll-down-command", 0, 5),
    ] {
        navigate(&mut session, command, Cell { row, column });
        assert!(session.view().top < session.buffer().unwrap().len_lines());
        assert_eq!(session.view().column, Some(5));
        bounded(&session.frame().unwrap(), size);
    }
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

fn without_sgr(text: &str) -> String {
    let mut result = String::new();
    let mut tail = text;
    while let Some(start) = tail.find("\x1b[") {
        result.push_str(&tail[..start]);
        let rest = &tail[start + 2..];
        let end = rest.find(|c: char| ('@'..='~').contains(&c)).unwrap();
        if !rest[..=end].ends_with('m') {
            result.push_str(&tail[start..start + 3 + end]);
        }
        tail = &rest[end + 1..];
    }
    result.push_str(tail);
    result
}

#[path = "display/habits.rs"]
mod habits;
#[path = "display/org_timers.rs"]
mod org_timers;
#[path = "display/org_timestamps.rs"]
mod org_timestamps;
