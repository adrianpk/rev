use rev::buffer::Buffers;
use rev::config::{IndentStyle, prepare};
use rev::coordinates::{Byte, Cell, Coordinates, Error, Scalar};
use rev::file::Loaded;

use crate::config_fixture::Fixture;

// Literal tabs project to the buffer's admitted stops without changing text.
// Boundary and interior-cell mappings must agree with the same glyph spans.
#[test]
fn tab_stops() {
    let fixture = Fixture::new();
    for (width, first, second) in [(1, 2, 4), (2, 2, 4), (4, 4, 8), (8, 8, 16), (32, 32, 64)] {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.create("a\tb\tc").unwrap();
        let buffer = buffers.get(id).unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        let columns = [0, 1, first, first + 1, second, second + 1];
        for (point, column) in columns.into_iter().enumerate() {
            let cell = Cell { row: 0, column };
            assert_eq!(
                coordinates.to_cell(Scalar(point)).unwrap(),
                cell,
                "width {width}"
            );
            assert_eq!(
                coordinates.from_cell(cell).unwrap(),
                Scalar(point),
                "width {width}"
            );
        }
        assert_eq!(
            coordinates
                .from_cell(Cell {
                    row: 0,
                    column: first - 1
                })
                .unwrap(),
            Scalar(1)
        );
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        assert_eq!(glyphs[1].columns, 1..first);
        assert_eq!(glyphs[3].columns, first + 1..second);
        assert_eq!(glyphs[1].text, " ".repeat(first - 1));
        assert_eq!(
            buffer.slice(&buffer.span(0..5).unwrap()).unwrap(),
            "a\tb\tc"
        );
        assert!(!buffer.is_modified());
        assert!(buffer.history().is_empty());
    }
}

// The registry supplies one frozen snapshot for scratch, raw ingress, new text
// and visiting buffers. A Go table changes only its explicit editing leaves.
#[test]
fn buffer_defaults() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "editor.indent_style = 'tabs'\neditor.indent_width = 6\neditor.tab_width = 8\nlanguage.go.indent_width = 2\nlanguage.go.tab_width = 2");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let plain = [
        buffers.initial(),
        buffers.create("text").unwrap(),
        buffers
            .load(Loaded::read(b"raw".as_slice()).unwrap())
            .unwrap(),
    ];
    for id in plain {
        let policy = buffers.get(id).unwrap().policy();
        assert_eq!(
            (policy.indent_style, policy.indent_width, policy.tab_width),
            (IndentStyle::Tabs, 6, 8)
        );
    }
    let path = fixture.write("main.go", "a\tb");
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    assert_eq!(
        (buffer.policy().indent_width, buffer.policy().tab_width),
        (2, 2)
    );
    assert_eq!(buffer.policy().indent_style, IndentStyle::Tabs);
    assert_eq!(buffer.mode(), "Text");
    let cell = Coordinates::new(buffer)
        .unwrap()
        .to_cell(Scalar(2))
        .unwrap();
    assert_eq!(cell, Cell { row: 0, column: 2 });
}

// Tab display changes cannot change UTF-8/scalar units, grapheme boundaries or
// CRLF ownership. Wide, combining and ZWJ clusters still occupy whole cells.
#[test]
fn mixed_units() {
    let fixture = Fixture::new();
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let id = buffers.create("a\t界e\u{301}👩\u{200d}💻\r\nx\tz").unwrap();
        let buffer = buffers.get(id).unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        let points = [
            (0, 0, 0),
            (1, 1, 1),
            (2, 2, stop),
            (3, 5, stop + 2),
            (5, 8, stop + 3),
            (8, 19, stop + 5),
        ];
        for (scalar, byte, column) in points {
            assert_eq!(coordinates.to_byte(Scalar(scalar)).unwrap(), Byte(byte));
            assert_eq!(
                coordinates.to_cell(Scalar(scalar)).unwrap(),
                Cell { row: 0, column }
            );
        }
        assert_eq!(
            coordinates.to_cell(Scalar(10)).unwrap(),
            Cell { row: 1, column: 0 }
        );
        assert_eq!(
            coordinates.to_cell(Scalar(12)).unwrap(),
            Cell {
                row: 1,
                column: stop
            }
        );
        assert_eq!(coordinates.to_byte(Scalar(10)).unwrap(), Byte(21));
        assert_eq!(
            coordinates
                .from_cell(Cell {
                    row: 0,
                    column: stop + 1
                })
                .unwrap(),
            Scalar(2)
        );
        assert_eq!(
            coordinates
                .from_cell(Cell {
                    row: 0,
                    column: stop + 4
                })
                .unwrap(),
            Scalar(5)
        );
        for scalar in [4, 6, 7, 9] {
            assert_eq!(
                coordinates.to_cell(Scalar(scalar)),
                Err(Error::Cluster(Scalar(scalar)))
            );
        }
        assert_eq!(buffer.newline(), "\r\n");
    }
}

// A combining cluster crosses real rope chunks immediately before a tab.
// Its display width and following tab stop must not depend on chunk boundaries.
#[test]
fn policy_seams() {
    let fixture = Fixture::new();
    let cluster = format!("a{}", "\u{301}".repeat(2048));
    let text = format!("{cluster}\t界👩\u{200d}💻");
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let id = buffers.create(&text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let whole = buffer.span(0..buffer.len_chars()).unwrap();
        let first = buffer
            .chunks(&whole)
            .unwrap()
            .next()
            .unwrap()
            .chars()
            .count();
        assert!(first > 0 && first < cluster.chars().count());
        let coordinates = Coordinates::new(buffer).unwrap();
        assert!(!coordinates.is_boundary(Scalar(first)).unwrap());
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(glyphs.len(), 4);
        assert_eq!(glyphs[0].text, cluster);
        assert_eq!(glyphs[0].columns, 0..1);
        assert_eq!(glyphs[1].columns, 1..stop);
        assert_eq!(glyphs[3].columns, stop + 2..stop + 4);
        assert_eq!(
            coordinates.to_cell(Scalar(buffer.len_chars())).unwrap(),
            Cell {
                row: 0,
                column: stop + 4
            }
        );
    }
}

// A policy replacement cannot invalidate scalar edit candidates or consume redo.
// Undo and redo retain their exact text/saved-state semantics under the new width.
#[test]
fn policy_replay() {
    use rev::buffer::edit::{History, Replacement};

    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "editor.tab_width = 8");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\tb\r\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.apply_settings(&prepare(&root).snapshot);
    buffer.publish(prepared, &mut []).unwrap();
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert!(!buffer.is_modified());
    let stamp = buffer.stamp();
    let saved = buffer.state();
    buffer.apply_settings(&rev::config::Snapshot::default());
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.state(), saved);
    assert_eq!(buffer.redo_history().len(), 1);
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        "!a\tb\r\n"
    );
    assert!(buffer.is_modified());
    assert_eq!(buffer.policy().tab_width, 4);
}
