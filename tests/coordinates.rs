#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::coordinates::{Byte, Cell, Coordinates, Error, LineByte, Scalar};
use unicode_segmentation::UnicodeSegmentation;

// Non-newline controls must remain visible text within their logical line.
#[test]
fn control_lines() {
    let mut buffers = Buffers::new().unwrap();
    for (name, text) in [
        ("vertical tab", "a\u{b}b"),
        ("form feed", "a\u{c}b"),
        ("carriage return", "a\rb"),
    ] {
        let id = buffers.create(text).unwrap();
        assert_eq!(buffers.get(id).unwrap().len_lines(), 1, "{name}");
    }
}

// Parsing bytes and terminal cells must round-trip the same qualified boundaries.
#[test]
fn coordinate_units() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}").unwrap();
    let buffer = buffers.get(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    assert_eq!(coordinates.stamp(), buffer.stamp());
    let cases = [
        (
            "start",
            0,
            0,
            LineByte { row: 0, column: 0 },
            Some(Cell { row: 0, column: 0 }),
        ),
        (
            "wide start",
            1,
            1,
            LineByte { row: 0, column: 1 },
            Some(Cell { row: 0, column: 1 }),
        ),
        (
            "crlf start",
            2,
            4,
            LineByte { row: 0, column: 4 },
            Some(Cell { row: 0, column: 3 }),
        ),
        ("crlf interior", 3, 5, LineByte { row: 0, column: 5 }, None),
        (
            "second line",
            4,
            6,
            LineByte { row: 1, column: 0 },
            Some(Cell { row: 1, column: 0 }),
        ),
        (
            "combining interior",
            5,
            7,
            LineByte { row: 1, column: 1 },
            None,
        ),
        (
            "end",
            6,
            9,
            LineByte { row: 1, column: 3 },
            Some(Cell { row: 1, column: 1 }),
        ),
    ];
    for (name, scalar, byte, line, cell) in cases {
        let point = Scalar(scalar);
        assert_eq!(coordinates.to_byte(point).unwrap(), Byte(byte), "{name}");
        assert_eq!(coordinates.from_byte(Byte(byte)).unwrap(), point, "{name}");
        assert_eq!(coordinates.to_line(point).unwrap(), line, "{name}");
        assert_eq!(coordinates.from_line(line).unwrap(), point, "{name}");
        match cell {
            Some(cell) => {
                assert_eq!(coordinates.to_cell(point).unwrap(), cell, "{name}");
                assert_eq!(coordinates.from_cell(cell).unwrap(), point, "{name}");
            }
            None => assert_eq!(
                coordinates.to_cell(point).unwrap_err(),
                Error::Cluster(point),
                "{name}"
            ),
        }
    }
}

// Motion must never stop inside emoji, combining sequences, flags, or CRLF.
#[test]
fn grapheme_motion() {
    let mut buffers = Buffers::new().unwrap();
    let cases: [(&str, &str, &[usize]); 6] = [
        ("empty", "", &[0]),
        ("ascii", "ab", &[0, 1, 2]),
        ("combining", "e\u{301}", &[0, 2]),
        ("emoji modifier", "👩🏽‍💻", &[0, 4]),
        ("flag", "🇵🇱", &[0, 2]),
        ("mixed", "a👩‍💻e\u{301}🇵🇱\r\n界", &[0, 1, 4, 6, 8, 10, 11]),
    ];
    for (name, text, boundaries) in cases {
        let id = buffers.create(text).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        for point in 0..=text.chars().count() {
            let next = boundaries
                .iter()
                .copied()
                .find(|end| *end > point)
                .unwrap_or(point);
            let previous = boundaries
                .iter()
                .copied()
                .rfind(|start| *start < point)
                .unwrap_or(point);
            assert_eq!(
                coordinates.is_boundary(Scalar(point)).unwrap(),
                boundaries.contains(&point),
                "{name}:{point}"
            );
            assert_eq!(
                coordinates.next(Scalar(point)).unwrap(),
                Scalar(next),
                "{name}:{point}"
            );
            assert_eq!(
                coordinates.previous(Scalar(point)).unwrap(),
                Scalar(previous),
                "{name}:{point}"
            );
        }
    }
}

// Interior cells select the complete tab/wide cluster; scalar boundaries stay distinct.
#[test]
fn display_cells() {
    let mut buffers = Buffers::new().unwrap();
    let cases = [
        (
            "mixed",
            "a\t界e\u{301}👩‍💻",
            &[(0, 0), (1, 1), (2, 4), (3, 6), (5, 7), (8, 9)][..],
        ),
        ("tab stops", "\t\t", &[(0, 0), (1, 4), (2, 8)][..]),
        ("leading mark", "\u{301}x", &[(0, 0), (1, 1), (2, 2)][..]),
    ];
    for (name, text, boundaries) in cases {
        let id = buffers.create(text).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        for &(scalar, column) in boundaries {
            let cell = Cell { row: 0, column };
            assert_eq!(coordinates.to_cell(Scalar(scalar)).unwrap(), cell, "{name}");
            assert_eq!(
                coordinates.from_cell(cell).unwrap(),
                Scalar(scalar),
                "{name}"
            );
        }
    }
    let id = buffers.create("a\t界e\u{301}👩‍💻").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    let expected = [0, 1, 1, 1, 2, 2, 3, 5, 5, 8];
    for (column, scalar) in expected.into_iter().enumerate() {
        assert_eq!(
            coordinates.from_cell(Cell { row: 0, column }).unwrap(),
            Scalar(scalar)
        );
    }
    assert_eq!(
        coordinates
            .from_cell(Cell {
                row: 0,
                column: usize::MAX
            })
            .unwrap(),
        Scalar(8)
    );
}

// Document controls must become visible text, never terminal control sequences.
#[test]
fn safe_controls() {
    let mut buffers = Buffers::new().unwrap();
    let cases = [
        ("nul", "\0", "^@"),
        ("escape", "\x1b", "^["),
        ("carriage return", "\r", "^M"),
        ("delete", "\u{7f}", "^?"),
        ("next line", "\u{85}", "<85>"),
        ("csi", "\u{9b}", "<9B>"),
        ("unit separator", "\u{1f}", "^_"),
        ("vertical tab", "\u{b}", "^K"),
        ("form feed", "\u{c}", "^L"),
    ];
    for (name, control, expected) in cases {
        let id = buffers.create(&format!("a{control}b")).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(glyphs.len(), 3, "{name}");
        assert_eq!(glyphs[1].text, expected, "{name}");
        let rendered: String = glyphs.iter().map(|glyph| glyph.text.as_ref()).collect();
        assert!(!rendered.chars().any(char::is_control), "{name}");
        assert_eq!(
            coordinates.to_cell(Scalar(3)).unwrap(),
            Cell {
                row: 0,
                column: expected.len() + 2
            },
            "{name}"
        );
    }
    let id = buffers.create("\u{301}").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    let glyph = coordinates.glyphs(0).unwrap().next().unwrap().unwrap();
    assert_eq!(glyph.text, "◌\u{301}");
    assert_eq!(glyph.columns, 0..1);
}

// Invalid byte offsets and row aliases must fail before unchecked arithmetic or slicing.
#[test]
fn invalid_positions() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    for point in [7, usize::MAX] {
        assert_eq!(
            coordinates.to_byte(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
        assert_eq!(
            coordinates.next(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
        assert_eq!(
            coordinates.previous(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
    }
    for byte in [2, 3, 8, 10, usize::MAX] {
        assert_eq!(
            coordinates.from_byte(Byte(byte)),
            Err(Error::Byte(Byte(byte)))
        );
    }
    let cases = [
        ("interior byte", LineByte { row: 0, column: 2 }),
        ("next row alias", LineByte { row: 0, column: 6 }),
        ("interior mark", LineByte { row: 1, column: 2 }),
        ("past line", LineByte { row: 1, column: 4 }),
        ("past row", LineByte { row: 2, column: 0 }),
        (
            "huge row",
            LineByte {
                row: usize::MAX,
                column: 0,
            },
        ),
    ];
    for (name, point) in cases {
        assert_eq!(
            coordinates.from_line(point),
            Err(Error::Line(point)),
            "{name}"
        );
    }
    assert_eq!(
        coordinates.from_cell(Cell {
            row: usize::MAX,
            column: 0
        }),
        Err(Error::Line(LineByte {
            row: usize::MAX,
            column: 0
        }))
    );
}

// Flat segmentation is an oracle for context-fed segmentation at real rope seams.
#[test]
fn chunk_context() {
    let cases = [
        ("combining", format!("xa{}z", "\u{301}".repeat(2048))),
        (
            "emoji context",
            format!("x👩{}\u{200d}💻z", "\u{301}".repeat(1024)),
        ),
        ("regional parity", format!("x{}z", "\u{1f1f5}".repeat(769))),
        ("indic context", format!("x{}कz", "क्".repeat(700))),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text) in cases {
        let id = buffers.create(&text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        let mut bytes: Vec<usize> = text.grapheme_indices(true).map(|(byte, _)| byte).collect();
        bytes.push(text.len());
        let scalars: Vec<usize> = bytes
            .iter()
            .map(|byte| text[..*byte].chars().count())
            .collect();
        let span = buffer.span(0..buffer.len_chars()).unwrap();
        let mut offset = 0;
        let seams: Vec<usize> = buffer
            .chunks(&span)
            .unwrap()
            .map(|chunk| {
                offset += chunk.len();
                offset
            })
            .collect();
        assert!(
            seams.len() > 1,
            "{name}: fixture must contain multiple rope chunks"
        );
        assert!(
            seams.iter().any(|seam| !bytes.contains(seam)),
            "{name}: a grapheme must cross a rope seam"
        );
        for pair in scalars.windows(2) {
            assert_eq!(
                coordinates.next(Scalar(pair[0])).unwrap(),
                Scalar(pair[1]),
                "{name}: forward {}",
                pair[0]
            );
            assert_eq!(
                coordinates.previous(Scalar(pair[1])).unwrap(),
                Scalar(pair[0]),
                "{name}: backward {}",
                pair[1]
            );
        }
        for seam in seams {
            let scalar = coordinates.from_byte(Byte(seam)).unwrap();
            assert_eq!(
                coordinates.is_boundary(scalar).unwrap(),
                bytes.contains(&seam),
                "{name}: seam {seam}"
            );
        }
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let starts: Vec<usize> = glyphs.iter().map(|glyph| glyph.source.start.0).collect();
        assert_eq!(starts, scalars[..scalars.len() - 1], "{name}");
        let rendered: String = glyphs.iter().map(|glyph| glyph.text.as_ref()).collect();
        assert_eq!(rendered, text, "{name}");
    }
}

// Empty text has one logical line and valid BOF/EOF coordinates without glyphs.
#[test]
fn empty_coordinates() {
    let buffers = Buffers::new().unwrap();
    let coordinates = Coordinates::new(buffers.get(buffers.initial()).unwrap()).unwrap();
    let origin = LineByte { row: 0, column: 0 };
    assert_eq!(coordinates.to_byte(Scalar(0)).unwrap(), Byte(0));
    assert_eq!(coordinates.from_byte(Byte(0)).unwrap(), Scalar(0));
    assert_eq!(coordinates.to_line(Scalar(0)).unwrap(), origin);
    assert_eq!(coordinates.from_line(origin).unwrap(), Scalar(0));
    assert_eq!(
        coordinates.to_cell(Scalar(0)).unwrap(),
        Cell { row: 0, column: 0 }
    );
    assert_eq!(
        coordinates
            .from_cell(Cell {
                row: 0,
                column: 100
            })
            .unwrap(),
        Scalar(0)
    );
    let mut glyphs = coordinates.glyphs(0).unwrap();
    assert!(glyphs.next().is_none());
    assert!(glyphs.next().is_none());
}

// Word navigation consumes Unicode alphanumeric/underscore grapheme runs,
// skips punctuation and emoji, and is directionally consistent at run edges.
#[test]
fn word_motion() {
    let cases = [
        ("empty", "", 0, 0, 0),
        ("word start", "alpha beta", 0, 5, 0),
        ("inside word", "alpha beta", 2, 5, 0),
        ("word end", "alpha beta", 5, 10, 0),
        ("punctuation", "one...two", 4, 9, 0),
        ("underscore", "a_b 42", 0, 3, 0),
        ("unicode", "é界_١٢!z", 0, 5, 0),
        ("combining", "e\u{301}!界", 0, 2, 0),
        ("emoji", "👩\u{200d}💻 x", 0, 5, 0),
        ("CRLF", "a\r\nb", 1, 4, 0),
        ("trailing space", "one  ", 5, 5, 0),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, point, next, previous) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        assert_eq!(
            coordinates.forward_word(Scalar(point)).unwrap(),
            Scalar(next),
            "{name}"
        );
        assert_eq!(
            coordinates.backward_word(Scalar(point)).unwrap(),
            Scalar(previous),
            "{name}"
        );
        assert!(!buffer.is_modified(), "{name}");
    }
}

fn session(source: &str, position: usize, read_only: bool) -> rev::session::Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_read_only(read_only);
    let mut view = rev::view::View::new(buffer);
    view.point.position = Scalar(position);
    rev::session::Session::new(
        buffers,
        view,
        rev::display::Size {
            width: 8,
            height: 4,
        },
    )
    .unwrap()
}

fn key(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> crossterm::event::Event {
    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(code, modifiers))
}

// Real binding events move the qualified point over whole clusters and CRLF,
// then reveal it, without publishing text or creating history/dirty state.
#[test]
fn bound_motion() {
    use crossterm::event::{KeyCode, KeyModifiers as Mod};
    use rev::session::Action;
    let source = "e\u{301}👩\u{200d}💻\r\nlast";
    let mut session = session(source, 0, false);
    let stamp = session.buffer().unwrap().stamp();
    for (code, modifiers, position) in [
        (KeyCode::Char('f'), Mod::CONTROL, 2),
        (KeyCode::Right, Mod::NONE, 5),
        (KeyCode::Char('f'), Mod::CONTROL, 7),
        (KeyCode::Char('b'), Mod::CONTROL, 5),
        (KeyCode::Left, Mod::NONE, 2),
        (KeyCode::Home, Mod::NONE, 0),
        (KeyCode::End, Mod::NONE, 5),
        (KeyCode::Char('>'), Mod::ALT, 11),
        (KeyCode::Char('<'), Mod::ALT, 0),
    ] {
        assert_eq!(session.handle(key(code, modifiers)), Action::Redraw);
        assert_eq!(session.view().point.position, Scalar(position));
        assert_eq!(session.view().point.stamp, stamp);
        let frame = session.frame().unwrap();
        assert!(frame.cursor().is_some());
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.stamp(), stamp);
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        source
    );
}

fn execute(session: &mut rev::session::Session, name: &str, position: usize) {
    let command = rev::command::Command::named(name).expect("implemented motion command");
    assert_eq!(session.execute(command), rev::session::Action::Redraw);
    assert_eq!(session.view().point.position, Scalar(position), "{name}");
    assert!(session.frame().unwrap().cursor().is_some(), "{name}");
}

// Named dispatch and all eight Emacs bindings must reach the same handler,
// including on read-only text where point movement remains available.
#[test]
fn named_motion() {
    use crossterm::event::{KeyCode, KeyModifiers as Mod};
    let source = "first\r\nαe\u{301}👩\u{200d}💻! last\n";
    let cases = [
        ("forward-char", 'f', Mod::CONTROL, 13),
        ("backward-char", 'b', Mod::CONTROL, 8),
        ("move-beginning-of-line", 'a', Mod::CONTROL, 7),
        ("move-end-of-line", 'e', Mod::CONTROL, 19),
        ("beginning-of-buffer", '<', Mod::ALT, 0),
        ("end-of-buffer", '>', Mod::ALT, 20),
        ("forward-word", 'f', Mod::ALT, 19),
        ("backward-word", 'b', Mod::ALT, 7),
    ];
    for (name, character, modifiers, position) in cases {
        let mut named = session(source, 10, true);
        let mut bound = session(source, 10, true);
        execute(&mut named, name, position);
        assert_eq!(
            bound.handle(key(KeyCode::Char(character), modifiers)),
            rev::session::Action::Redraw,
            "{name}"
        );
        assert_eq!(bound.view().point.position, Scalar(position), "{name}");
        assert!(bound.echo().is_empty(), "{name}");
        assert!(!bound.buffer().unwrap().is_modified(), "{name}");
    }
}

// Repeated commands skip separator runs, preserve mixed-script graphemes,
// and clamp at either end even when the entire document is separators.
#[test]
fn word_edges() {
    let cases: [(&str, &str, &[usize], &[usize]); 3] = [
        ("empty", "", &[0, 0], &[0, 0]),
        ("separators", " \t...👩\u{200d}💻\r\n", &[10, 10], &[0, 0]),
        (
            "mixed",
            "ab...界_٢ e\u{301}👩\u{200d}💻!z\r\n",
            &[2, 8, 11, 16, 18, 18],
            &[15, 9, 5, 0, 0],
        ),
    ];
    for (name, source, forward, backward) in cases {
        let mut session = session(source, 0, false);
        for &point in forward {
            execute(&mut session, "forward-word", point);
        }
        for &point in backward {
            execute(&mut session, "backward-word", point);
        }
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Logical line destinations exclude the complete CRLF separator, retain lone
// CR as visible text, and distinguish an empty middle line from trailing EOF.
#[test]
fn line_motion() {
    let source = "α\r\n\r\nz\rx\n";
    let cases = [
        ("first start", 0, 0, 1),
        ("first end", 1, 0, 1),
        ("empty line", 3, 3, 3),
        ("last start", 5, 5, 8),
        ("visible CR", 6, 5, 8),
        ("last end", 8, 5, 8),
        ("trailing line", 9, 9, 9),
    ];
    for (name, point, start, end) in cases {
        let mut beginning = session(source, point, false);
        let mut ending = session(source, point, false);
        execute(&mut beginning, "move-beginning-of-line", start);
        execute(&mut ending, "move-end-of-line", end);
        assert!(!beginning.buffer().unwrap().is_modified(), "{name}");
        assert!(!ending.buffer().unwrap().is_modified(), "{name}");
    }
}

// Retained forward/backward cursors must cross actual rope seams, including
// long combining clusters and regional-indicator context, without flattening.
#[test]
fn chunk_words() {
    let first = format!("e{}", "\u{301}".repeat(1500));
    let second = "界_٢".repeat(700);
    for (name, separator) in [
        ("punctuation", ".".repeat(1600)),
        ("ZWJ emoji", "👩\u{200d}💻".repeat(400)),
        ("flags", "🇵🇱".repeat(600)),
    ] {
        let source = format!("{first}{separator}{second}.end");
        let middle = first.chars().count() + separator.chars().count();
        let last = middle + second.chars().count();
        let mut session = session(&source, 0, false);
        let buffer = session.buffer().unwrap();
        let text = buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap();
        assert!(text.chunks().count() > 2, "{name}");
        for point in [first.chars().count(), last, last + 4, last + 4] {
            execute(&mut session, "forward-word", point);
        }
        for point in [last + 1, middle, 0, 0] {
            execute(&mut session, "backward-word", point);
        }
    }
}

// Horizontal and logical-row reveal use document cells, with a valid visible
// cursor after word, line, character and whole-buffer destinations.
#[test]
fn motion_reveal() {
    let source = format!("a\t界{}\r\none\ntwo\n界\tend", "x".repeat(12));
    let mut session = session(&source, 0, false);
    let cases = [
        ("move-end-of-line", 15, 0, 11, (7, 0)),
        ("backward-char", 14, 0, 11, (6, 0)),
        ("move-end-of-line", 15, 0, 11, (7, 0)),
        ("forward-char", 17, 0, 0, (0, 1)),
        ("end-of-buffer", 30, 2, 0, (7, 1)),
        ("backward-word", 27, 2, 0, (4, 1)),
        ("beginning-of-buffer", 0, 0, 0, (0, 0)),
    ];
    for (name, point, top, left, cursor) in cases {
        execute(&mut session, name, point);
        assert_eq!(
            (session.view().top, session.view().left),
            (top, left),
            "{name}"
        );
        let position = session.frame().unwrap().cursor().unwrap();
        assert_eq!((position.column, position.row), cursor, "{name}");
    }
}

fn buffer_state(
    buffer: &rev::buffer::Buffer,
) -> (String, rev::buffer::Stamp, Option<Scalar>, usize) {
    let text = buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string();
    (text, buffer.stamp(), buffer.mark(), buffer.history().len())
}

// Moving through dirty, marked, read-only text must leave its existing edit
// record, revision, newline metadata and mark unchanged.
#[test]
fn motion_state() {
    use rev::buffer::edit::Replacement;
    use rev::display::Size;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "changed\r\nwords".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    buffer.set_read_only(true);
    let before = buffer_state(buffer);
    let view = rev::view::View::new(buffer);
    let mut session = rev::session::Session::new(
        buffers,
        view,
        Size {
            width: 8,
            height: 4,
        },
    )
    .unwrap();
    for (name, point) in [
        ("end-of-buffer", 14),
        ("backward-char", 13),
        ("move-beginning-of-line", 9),
        ("forward-char", 10),
        ("move-end-of-line", 14),
        ("backward-word", 9),
        ("forward-word", 14),
        ("beginning-of-buffer", 0),
    ] {
        execute(&mut session, name, point);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer_state(buffer), before);
    assert!(buffer.is_modified() && buffer.is_read_only());
    assert_eq!(buffer.newline(), "\r\n");
}

// Motion key repeats dispatch once while releases are ignored; modifier
// normalization must preserve reportable shifted Meta buffer destinations.
#[test]
fn motion_phases() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind as Kind, KeyModifiers as Mod};
    let cases = [
        ('f', Mod::CONTROL, 3),
        ('b', Mod::CONTROL, 1),
        ('a', Mod::CONTROL, 0),
        ('e', Mod::CONTROL, 5),
        ('f', Mod::ALT, 5),
        ('b', Mod::ALT, 0),
        ('<', Mod::ALT | Mod::SHIFT, 0),
        ('>', Mod::ALT | Mod::SHIFT, 5),
    ];
    for (character, modifiers, point) in cases {
        let mut session = session("ab cd", 2, false);
        let event = |kind| {
            Event::Key(KeyEvent::new_with_kind(
                KeyCode::Char(character),
                modifiers,
                kind,
            ))
        };
        assert_eq!(
            session.handle(event(Kind::Release)),
            rev::session::Action::Wait
        );
        assert_eq!(session.view().point.position, Scalar(2));
        assert_eq!(
            session.handle(event(Kind::Repeat)),
            rev::session::Action::Redraw
        );
        assert_eq!(session.view().point.position, Scalar(point));
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Word scans reject out-of-range scalars and grapheme/CRLF interiors rather
// than creating an ambiguous source position or silently repairing one.
#[test]
fn invalid_words() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}\r\nx").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    for (name, point, error) in [
        ("combining interior", 1, Error::Cluster(Scalar(1))),
        ("CRLF interior", 3, Error::Cluster(Scalar(3))),
        ("past EOF", 6, Error::Scalar(Scalar(6))),
        ("overflow", usize::MAX, Error::Scalar(Scalar(usize::MAX))),
    ] {
        assert_eq!(
            coordinates.forward_word(Scalar(point)).unwrap_err(),
            error,
            "{name}"
        );
        assert_eq!(
            coordinates.backward_word(Scalar(point)).unwrap_err(),
            error,
            "{name}"
        );
    }
}

// Every registered motion is safe on an empty document; repeated character
// commands also clamp at the real ends of a multi-scalar emoji cluster.
#[test]
fn motion_limits() {
    use rev::command::Command;
    let mut empty = session("", 0, false);
    for command in Command::ALL {
        if let Command::Motion(_) = command {
            execute(&mut empty, command.name(), 0);
        }
    }
    let mut emoji = session("👩\u{200d}💻", 0, false);
    for (name, point) in [
        ("backward-char", 0),
        ("forward-char", 3),
        ("forward-char", 3),
        ("backward-char", 0),
        ("backward-char", 0),
    ] {
        execute(&mut emoji, name, point);
    }
    assert!(emoji.buffer().unwrap().history().is_empty());
    assert!(empty.buffer().unwrap().history().is_empty());
}
