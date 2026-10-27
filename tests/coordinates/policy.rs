use rev::buffer::Buffers;
use rev::config::{IndentStyle, prepare};
use rev::coordinates::{Cell, Coordinates, Scalar};
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
