use crossterm::event::Event;
use rev::command::{Command, RegionCommand};
use rev::config::prepare;
use rev::session::{Action, Session};

use super::{Buffers, Frame, Scalar, Size, View, bounded};
use crate::config_fixture::Fixture;

// Golden clipped rows cover partial tab stops, wide edges, combining marks and
// ZWJ clusters under each admitted representative width. Stored tabs stay literal.
#[test]
fn policy_clipping() {
    let fixture = Fixture::new();
    let cases = [
        (1, 0, 10, "a 界e\u{301}👩\u{200d}💻z  "),
        (2, 0, 10, "a 界e\u{301}👩\u{200d}💻z  "),
        (4, 0, 10, "a   界e\u{301}👩\u{200d}💻z"),
        (8, 0, 12, "a       界e\u{301} "),
        (32, 28, 10, "    界e\u{301}👩\u{200d}💻z"),
        (4, 2, 8, "  界e\u{301}👩\u{200d}💻z"),
        (8, 9, 6, " e\u{301}👩\u{200d}💻z "),
    ];
    for (tab_width, left, width, expected) in cases {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {tab_width}"));
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let source = "a\t界e\u{301}👩\u{200d}💻z";
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.left = left;
        let size = Size {
            width: width + 6,
            height: 4,
        };
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        assert_eq!(
            frame.rows()[0].text(),
            format!("   1 │{expected}"),
            "tab {tab_width}, left {left}"
        );
        bounded(&frame, size);
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap(),
            source
        );
    }
}

// A real incremental search and active region use the same buffer policy as
// document/cursor projection. Styling expands an interior combining endpoint
// without changing the scalar search match or the underlying text.
#[test]
fn marked_geometry() {
    let fixture = Fixture::new();
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let id = buffers.create("a\t界e\u{301}z").unwrap();
        let mut view = View::new(buffers.get(id).unwrap());
        view.point.position = Scalar(1);
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 46,
                height: 6,
            },
        )
        .unwrap();
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::SearchForward);
        assert_eq!(session.handle(Event::Paste("界e".into())), Action::Redraw);
        assert_eq!(session.search_match().unwrap().range, 2..4);
        assert_eq!(session.view().point.position, Scalar(5));
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows()[0].search(), Some(stop + 6..stop + 9));
        assert_eq!(frame.rows()[0].selection(), Some(7..stop + 9));
        assert_eq!(
            session
                .view()
                .cell(session.buffer().unwrap())
                .unwrap()
                .column,
            usize::from(stop + 3)
        );
        let cells = super::surface::draw(&frame);
        let palette = rev::config::Palette::default();
        let cell = cells[&(0, usize::from(stop + 6))];
        assert_eq!(cell.foreground, Some(palette.selection_text));
        assert_eq!(cell.background, Some(palette.selection));
        assert!(cell.underline);
        assert_eq!(
            cells[&(0, usize::from(stop + 9))].background,
            Some(palette.background)
        );
        assert!(!session.buffer().unwrap().is_modified());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
