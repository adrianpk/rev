use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{
    Command, GutterCommand as G, Motion, RectangleCommand as R, RegionCommand, Scroll, Split,
    TreeCommand,
};
use rev::coordinates::Scalar;
use rev::panel::{Composition, Panel};
use rev::session::Session;
use rev::syntax::Style;
use rev::view::{Size, View};

use super::{Buffers, config_fixture::Fixture, surface};

const SOURCE: &str = "\tfn target() { let text = \"界e\u{301}\"; }\r\n\tfn second() {}\r\n";

pub(super) fn editor(fixture: &Fixture, side: bool, visible: bool) -> Session {
    let path = fixture.write("main.rs", SOURCE);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.bookmark_markers = false;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 61,
            height: 13,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Below));
    session.select_view(session.windows().order()[1]).unwrap();
    session.execute(Command::Split(Split::Right));
    session.select_view(session.windows().order()[2]).unwrap();
    session.execute(Command::Gutter(G::Relative));
    if side {
        session.execute(Command::Gutter(G::ToggleSide));
    }
    if !visible {
        session.execute(Command::Gutter(G::Toggle));
    }
    session.execute(Command::Tree(TreeCommand::Toggle));
    session.execute(Command::KeyboardQuit);
    session
}

fn text_panel(session: &Session, width: u16) -> Panel {
    let composition = Composition::workspace(
        session.windows(),
        session.tree(),
        Size { width, height: 13 },
        0,
    )
    .unwrap();
    let panel = composition
        .panels()
        .iter()
        .find(|(id, _)| *id == session.windows().selected_id())
        .unwrap()
        .1;
    let geometry = session
        .view()
        .gutter_geometry(session.buffer().unwrap(), panel.content())
        .unwrap();
    Panel::new(geometry.text(), false, true)
}

// Fixed physical expectations cover both gutter sides, hiding, tree coexistence
// and horizontally clipped narrow panes. Search/region/syntax stay on text cells
// and cannot style a number field, shared border or status row.
#[test]
fn overlay_origins() {
    for (name, width, right, visible, origin, left) in [
        ("left", 61, false, true, 47, 0),
        ("right", 61, true, true, 41, 0),
        ("hidden", 61, false, false, 41, 0),
        ("narrow left", 19, false, true, 16, 11),
        ("narrow right", 19, true, true, 10, 11),
    ] {
        overlay_case(name, width, right, visible, (origin, left));
    }
}

fn overlay_case(name: &str, width: u16, right: bool, visible: bool, expected: (u16, usize)) {
    let fixture = Fixture::new();
    let mut session = editor(&fixture, right, visible);
    session.execute(Command::VisualLineMode);
    session.handle(Event::Resize(width, 13));
    let peers: Vec<_> = session
        .windows()
        .iter()
        .filter(|(id, _)| *id != session.windows().selected_id())
        .map(|(id, view)| (id, view.point))
        .collect();
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("target".into()));
    let frame = session.frame().unwrap();
    let panel = text_panel(&session, width);
    let content = panel.content();
    assert_eq!((content.column(), session.view().left), expected, "{name}");
    let row = usize::from(content.row());
    let start = expected.0 + u16::try_from(7usize.saturating_sub(expected.1)).unwrap();
    let end = expected.0 + u16::try_from(13 - expected.1).unwrap();
    assert_eq!(frame.rows()[row].search(), Some(start..end), "{name}");
    let selection_start = expected.0 + u16::try_from(4usize.saturating_sub(expected.1)).unwrap();
    assert_eq!(
        frame.rows()[row].selection(),
        Some(selection_start..end),
        "{name}"
    );
    let cells = surface::draw(&frame);
    assert_eq!(
        cells[&(row, usize::from(start))].character,
        if expected.1 == 0 { 't' } else { 'e' }
    );
    assert!(cells[&(row, usize::from(start))].underline);
    check_styles(&frame, panel, expected.1);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(
        session.frame().unwrap().cursor(),
        panel.position(rev::display::Position {
            column: u16::try_from(13 - expected.1).unwrap(),
            row: 0
        })
    );
    assert_eq!(session.view().point.position, Scalar(10));
    assert_eq!(
        session
            .windows()
            .iter()
            .filter(|(id, _)| *id != session.windows().selected_id())
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>(),
        peers
    );
    rectangle(&mut session, width);
    assert_eq!(
        std::fs::read(fixture.0.join("main.rs")).unwrap(),
        SOURCE.as_bytes()
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

fn check_styles(frame: &rev::display::Frame, text: Panel, left: usize) {
    let content = text.content();
    let row = usize::from(content.row());
    for columns in [
        frame.rows()[row].selection().unwrap(),
        frame.rows()[row].search().unwrap(),
    ] {
        assert!(columns.start >= content.column());
        assert!(columns.end <= content.column() + content.width());
    }
    if left == 0 {
        assert!(
            frame.rows()[row]
                .syntax()
                .iter()
                .any(|paint| paint.style == Style::Keyword
                    && paint.columns == (content.column() + 4..content.column() + 6))
        );
    }
    assert!(frame.rows()[5].selection().is_none());
    assert!(frame.rows()[5].search().is_none());
    assert!(frame.rows()[5].syntax().is_empty());
    assert!(frame.rows()[11].selection().is_none());
    assert!(frame.rows()[11].search().is_none());
    assert!(
        text.position(rev::display::Position {
            column: content.width(),
            row: 0
        })
        .is_none()
    );
    assert!(
        text.position(rev::display::Position {
            column: 0,
            row: content.height()
        })
        .is_none()
    );
}

fn rectangle(session: &mut Session, width: u16) {
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Rectangle(R::Mark));
    session.execute(Command::Scroll(Scroll::NextLine));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Rectangle(R::Copy));
    assert_eq!(session.rectangle().unwrap().rows(), ["fn", "fn"]);
    assert_eq!(session.rectangle().unwrap().width(), 2);
    let frame = session.frame().unwrap();
    let panel = text_panel(session, width);
    let left = session.view().left;
    let start = u16::try_from(4usize.saturating_sub(left)).unwrap();
    let end = u16::try_from(6usize.saturating_sub(left)).unwrap();
    for local in 0..2 {
        let row = usize::from(panel.content().row() + local);
        assert_eq!(
            frame.rows()[row].selection(),
            panel.clip(local, start..end).map(|(_, columns)| columns)
        );
    }
    assert!(frame.rows().iter().all(|row| row.search().is_none()));
}

// The same nested/tree geometries wrap with factory policy. Search and region
// styles cover exactly the source text across fragments under either gutter
// side, hiding or narrow suppression; the terminal writer keeps them on text.
#[test]
fn wrapped_overlays() {
    for (width, right, visible) in [
        (61, false, true),
        (61, true, true),
        (61, false, false),
        (19, false, true),
        (19, true, true),
    ] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture, right, visible);
        session.handle(Event::Resize(width, 13));
        session.execute(Command::Motion(Motion::ForwardChar));
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::SearchForward);
        session.handle(Event::Paste("target".into()));
        let frame = session.frame().unwrap();
        let content = text_panel(&session, width).content();
        let cells = surface::draw(&frame);
        let mut searched = String::new();
        let mut selected = String::new();
        for (row, painted) in frame.rows().iter().enumerate() {
            for (span, text) in [
                (painted.search(), &mut searched),
                (painted.selection(), &mut selected),
            ] {
                if let Some(span) = span {
                    assert!(
                        row >= usize::from(content.row())
                            && row < usize::from(content.row() + content.height())
                    );
                    assert!(
                        span.start >= content.column()
                            && span.end <= content.column() + content.width()
                    );
                    for col in span {
                        text.push(cells[&(row, usize::from(col))].character);
                    }
                }
            }
        }
        assert_eq!(searched, "target", "{width}/{right}/{visible}");
        assert_eq!(selected, "fn target", "{width}/{right}/{visible}");
        assert_eq!(session.search_match().unwrap().range, 4..10);
        assert_eq!(session.view().point.position, Scalar(10));
        assert_eq!(session.view().left, 0);
        assert!(frame.cursor().is_some());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
