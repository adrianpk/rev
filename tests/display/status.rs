use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Frame, Size};
use rev::view::View;
use unicode_width::UnicodeWidthStr;

// The complete status keeps identity/state/access/position on the left and
// right-aligns language using only its own row, independently of document cells.
#[test]
fn complete_groups() {
    for (name, source, point, expected, mode) in [
        ("scratch", "", 0, "-- RW -- L1 C1", "Text"),
        ("Unicode", "\t界e\u{301}\r\n", 4, "-- RW -- L1 C8", "Text"),
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.point.position = Scalar(point);
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 80,
                height: 5,
            },
            "notice",
        )
        .unwrap();
        let expected = format!("> {} {expected}", buffer.name());
        let row = frame.rows()[3].text();
        assert!(row.starts_with(&expected), "{name}: {row:?}");
        assert!(row.ends_with(mode));
        assert_eq!(row.width(), 80);
        assert_eq!(
            row,
            format!(
                "{expected}{}{mode}",
                " ".repeat(80 - expected.width() - mode.len())
            )
        );
        assert!(frame.rows()[4].text().starts_with("notice"));
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}

fn rust_buffer(fixture: &super::config_fixture::Fixture) -> (Buffers, rev::buffer::BufferId) {
    let path = fixture.write("config.rs", "fn main() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().refresh_syntax();
    (buffers, id)
}

// Each threshold pins the accepted order: remove language, compact position,
// clip identity, then omit whole position/access tokens only when they cannot fit.
#[test]
fn narrow_order() {
    let fixture = super::config_fixture::Fixture::new();
    let (buffers, id) = rust_buffer(&fixture);
    let buffer = buffers.get(id).unwrap();
    let view = View::new(buffer);
    for (width, expected) in [
        (31, "> config.rs -- RW -- L1 C1 Rust"),
        (30, "> config.rs -- RW -- L1 C1"),
        (26, "> config.rs -- RW -- L1 C1"),
        (25, "> config.rs -- RW -- L1:1"),
        (24, "> config.r -- RW -- L1:1"),
        (16, "> -- RW -- L1:1"),
        (15, "> -- RW -- L1:1"),
        (14, "> config -- RW"),
        (9, "> c -- RW"),
        (8, "> -- RW"),
        (7, "> -- RW"),
        (6, ">-- RW"),
        (5, "-- RW"),
        (4, "> --"),
        (3, ">--"),
    ] {
        let frame = Frame::project(buffer, &view, Size { width, height: 4 }, "notice").unwrap();
        assert_eq!(frame.rows()[2].text().trim_end(), expected, "{width}");
        assert_eq!(frame.rows()[2].text().width(), usize::from(width));
    }
    for width in 0..3 {
        let frame = Frame::project(buffer, &view, Size { width, height: 4 }, "notice").unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.band() != rev::display::Band::Status)
        );
        assert!(frame.cursor().is_none());
    }
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// Dirty/write observations remain truthful in every combination. Status reads
// cannot publish edits or change the saved state; mode remains right-aligned.
#[test]
fn source_observations() {
    use rev::buffer::edit::Replacement;
    for (dirty, readonly, state, access) in [
        (false, false, "--", "RW"),
        (false, true, "--", "RO"),
        (true, false, "**", "RW"),
        (true, true, "**", "RO"),
    ] {
        let fixture = super::config_fixture::Fixture::new();
        let (mut buffers, id) = rust_buffer(&fixture);
        let buffer = buffers.get_mut(id).unwrap();
        let mut view = View::new(buffer);
        if dirty {
            let edit = Replacement {
                source: buffer.span(0..0).unwrap(),
                text: "!".into(),
            };
            let prepared = buffer
                .prepare(&[edit], std::slice::from_ref(&view.point), None)
                .unwrap();
            buffer
                .publish(prepared, std::slice::from_mut(&mut view.point))
                .unwrap();
            buffer.refresh_syntax();
        }
        buffer.set_readonly(readonly);
        let before = (buffer.stamp(), buffer.state(), buffer.history().len());
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 80,
                height: 5,
            },
            "save failed",
        )
        .unwrap();
        let column = if dirty { 2 } else { 1 };
        assert!(
            frame.rows()[3]
                .text()
                .starts_with(&format!("> config.rs {state} {access} -- L1 C{column}"))
        );
        assert!(frame.rows()[3].text().ends_with("Rust"));
        assert!(frame.rows()[4].text().starts_with("save failed"));
        assert_eq!(
            (buffer.stamp(), buffer.state(), buffer.history().len()),
            before
        );
        assert_eq!(buffer.is_modified(), dirty);
        assert_eq!(buffer.is_readonly(), readonly);
    }
}

// A wide/combining/control-bearing filename is measured and sanitized through
// the same glyph boundary as text, with complete language restoration on growth.
#[test]
fn unicode_label() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("界e\u{301}\u{1b}[31m.rs", "fn main() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.refresh_syntax();
    let view = View::new(buffer);
    for width in (3..=60).chain([80]) {
        let frame = Frame::project(buffer, &view, Size { width, height: 4 }, "").unwrap();
        let status = frame.rows()[2].text();
        assert_eq!(status.width(), usize::from(width));
        assert!(!status.contains('\u{1b}'));
        assert!(!status.contains('\u{fffd}'));
        if width == 80 {
            assert!(status.starts_with("> 界e\u{301}^[[31m.rs -- RW -- L1 C1"));
            assert!(status.ends_with("Rust"));
        }
    }
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// Shared views report their own logical points and active palette while sharing
// dirty facts. Gutter side/mode changes never offset status from its pane edge.
#[test]
fn shared_panes() {
    use rev::command::{Command, GutterCommand as G, Motion, Split};
    use rev::config::Palette;
    use rev::panel::Composition;
    use rev::session::Session;
    use rev::window::Area;

    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界\tab\r\nlast").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 120,
            height: 13,
        },
    )
    .unwrap();
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().next_view().unwrap();
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Gutter(G::Relative));
    session.execute(Command::Gutter(G::ToggleSide));
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    for selected in [original, peer] {
        session.select_view(selected).unwrap();
        let frame = session.frame().unwrap();
        let composition =
            Composition::new(session.windows(), Area::new(0, 0, 120, 13).unwrap(), 0).unwrap();
        let cells = super::surface::draw(&frame);
        for (id, pane) in composition.panels() {
            let area = pane.status().unwrap();
            let row = usize::from(area.row());
            let body: String = (area.column()..area.column() + area.width())
                .map(|col| cells[&(row, usize::from(col))].character)
                .collect();
            let position = if *id == original { "L1 C1" } else { "L2 C6" };
            assert!(body.contains(&format!("** RW -- {position}")));
            assert!(body.ends_with("Text"));
            assert_eq!(body.starts_with('>'), *id == selected);
            let color = if *id == selected {
                Palette::default().status
            } else {
                Palette::default().status_inactive
            };
            assert_eq!(
                cells[&(row, usize::from(area.column()))].background,
                Some(color)
            );
        }
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
    }
}
