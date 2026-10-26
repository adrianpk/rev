use crossterm::event::Event;
use rev::command::{Command, Split};
use rev::view::Size;

use super::{bounded, editor, other, size, split};

// Unbalanced branches fit their exact recursive minima in either orientation.
// A resize recomputes rectangles without changing identities, text or history.
#[test]
fn branch_minima() {
    for (name, direction, dimensions) in [
        ("right", Split::Right, size(11, 3)),
        ("below", Split::Below, size(3, 7)),
    ] {
        for second in [false, true] {
            let mut session = editor("abc", size(31, 13));
            split(&mut session, direction);
            if second {
                session.select_view(other(&session)).unwrap();
            }
            split(&mut session, direction);
            let layout = session.windows().layout().clone();
            let ids = session.windows().order();
            session.handle(Event::Resize(dimensions.width, dimensions.height));
            let geometry = session.windows().geometry(dimensions).unwrap();
            assert_eq!(geometry.panes().len(), 3, "{name}");
            for (_, area) in geometry.panes() {
                assert_eq!(area.width(), 3, "{name}");
                assert_eq!(area.height(), 2, "{name}");
            }
            bounded(&mut session, dimensions);
            assert!(session.frame().unwrap().cursor().is_some(), "{name}");
            assert_eq!(*session.windows().layout(), layout);
            assert_eq!(session.windows().order(), ids);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// Tiny transient surfaces retain their input owner and clip their cursor to
// the echo/list rectangle, including zero width or height and one-cell screens.
#[test]
fn tiny_interactions() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    for mode in ["argument", "inventory", "decision"] {
        let mut session = editor("original", size(31, 13));
        split(&mut session, Split::Right);
        session.insert('!');
        super::interaction(&mut session, mode);
        let layout = session.windows().layout().clone();
        let point = session.view().point;
        let selected = session.windows().selected_id();
        let stamp = session.buffer().unwrap().stamp();
        for dimensions in tiny_sizes() {
            session.handle(Event::Resize(dimensions.width, dimensions.height));
            bounded(&mut session, dimensions);
            assert_eq!(session.view().point, point, "{mode}");
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(*session.windows().layout(), layout);
        }
        session.handle(Event::Resize(31, 13));
        bounded(&mut session, size(31, 13));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(super::buffer_text(&session), "or!iginal");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::Window(rev::command::WindowCommand::Other));
        assert_ne!(session.windows().selected_id(), selected);
    }
}

fn tiny_sizes() -> impl Iterator<Item = Size> {
    (0..4).flat_map(|width| (0..4).map(move |height| size(width, height)))
}

fn views(session: &rev::session::Session) -> Vec<(rev::window::ViewId, rev::view::View)> {
    session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.clone()))
        .collect()
}

fn overlaps(first: rev::window::Area, second: rev::window::Area) -> bool {
    let x = |area: rev::window::Area| {
        u32::from(area.column())..u32::from(area.column()) + u32::from(area.width())
    };
    let y = |area: rev::window::Area| {
        u32::from(area.row())..u32::from(area.row()) + u32::from(area.height())
    };
    x(first).start < x(second).end
        && x(second).start < x(first).end
        && y(first).start < y(second).end
        && y(second).start < y(first).end
}

fn geometry_bounds(session: &rev::session::Session, dimensions: Size) {
    let Some(geometry) = session.windows().geometry(dimensions) else {
        return;
    };
    let areas = geometry
        .panes()
        .iter()
        .map(|(_, area)| *area)
        .chain(geometry.borders().iter().copied())
        .collect::<Vec<_>>();
    let mut cells = 0u64;
    for (index, area) in areas.iter().enumerate() {
        assert!(u32::from(area.column()) + u32::from(area.width()) <= u32::from(dimensions.width));
        assert!(u32::from(area.row()) + u32::from(area.height()) < u32::from(dimensions.height));
        cells += u64::from(area.width()) * u64::from(area.height());
        for other in &areas[index + 1..] {
            assert!(!overlaps(*area, *other));
        }
    }
    assert_eq!(
        cells,
        u64::from(dimensions.width) * u64::from(dimensions.height - 1)
    );
    for (_, area) in geometry.panes() {
        assert!(area.width() >= 3 && area.height() >= 2);
    }
}

// Every small shape fits exactly when its recursive minima fit. Projected
// rectangles cover the frame except the echo, never overlap, and stay bounded.
#[test]
fn resize_grid() {
    let cases = [
        (Split::Right, Split::Right, 11, 3),
        (Split::Right, Split::Below, 7, 5),
        (Split::Below, Split::Right, 7, 5),
        (Split::Below, Split::Below, 3, 7),
    ];
    for (first, second, width, height) in cases {
        for branch in [false, true] {
            grid_case(first, second, branch, size(width, height));
        }
    }
}

fn grid_case(first: Split, second: Split, branch: bool, minimum: Size) {
    let mut session = editor("a\t界e\u{301}\nlast", size(31, 13));
    split(&mut session, first);
    if branch {
        session.select_view(other(&session)).unwrap();
    }
    split(&mut session, second);
    let layout = session.windows().layout().clone();
    let points = views(&session)
        .into_iter()
        .map(|(id, view)| (id, view.point))
        .collect::<Vec<_>>();
    for dimensions in (0..18).flat_map(|width| (0..11).map(move |height| size(width, height))) {
        let before = views(&session);
        session.handle(Event::Resize(dimensions.width, dimensions.height));
        let fits = dimensions.width >= minimum.width && dimensions.height >= minimum.height;
        assert_eq!(
            session.windows().geometry(dimensions).is_some(),
            fits,
            "{dimensions:?}"
        );
        geometry_bounds(&session, dimensions);
        bounded(&mut session, dimensions);
        if !fits {
            assert_eq!(views(&session), before);
        }
        assert_eq!(*session.windows().layout(), layout);
        assert_eq!(
            views(&session)
                .into_iter()
                .map(|(id, view)| (id, view.point))
                .collect::<Vec<_>>(),
            points
        );
    }
    assert_eq!(super::buffer_text(&session), "a\t界e\u{301}\nlast");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Native dimension endpoints are admitted without caps. Extreme axes render
// separately; a maximal two-axis report validates geometry without a huge raster.
#[test]
fn native_extremes() {
    let mut session = editor("a\t界e\u{301}\nlast", size(31, 13));
    split(&mut session, Split::Right);
    split(&mut session, Split::Below);
    let layout = session.windows().layout().clone();
    let points = views(&session)
        .into_iter()
        .map(|(id, view)| (id, view.point))
        .collect::<Vec<_>>();
    for dimensions in [
        size(0, u16::MAX),
        size(u16::MAX, 0),
        size(1, u16::MAX),
        size(u16::MAX, 1),
        size(7, u16::MAX),
        size(u16::MAX, 5),
    ] {
        session.handle(Event::Resize(dimensions.width, dimensions.height));
        geometry_bounds(&session, dimensions);
        bounded(&mut session, dimensions);
        assert_eq!(*session.windows().layout(), layout);
        assert_eq!(
            views(&session)
                .into_iter()
                .map(|(id, view)| (id, view.point))
                .collect::<Vec<_>>(),
            points
        );
    }
    geometry_bounds(&session, size(u16::MAX, u16::MAX));
    assert!(
        session
            .windows()
            .geometry(size(u16::MAX, u16::MAX))
            .is_some()
    );
    session.handle(Event::Resize(31, 13));
    bounded(&mut session, size(31, 13));
}

fn navigate_view(session: &mut rev::session::Session, id: rev::window::ViewId, row: usize) {
    use rev::command::{Motion, Scroll};
    session.select_view(id).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..row {
        session.execute(Command::Scroll(Scroll::NextLine));
    }
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::Scroll(Scroll::NextLine));
    session.execute(Command::Scroll(Scroll::PreviousLine));
    session.execute(Command::Scroll(Scroll::Recenter));
    session.frame().unwrap();
}

// Repeated suspended redraws and rejected splits retain complete independent
// views and real undo/redo payloads, not just the current text or history counts.
#[test]
fn resize_roundtrip() {
    use rev::command::History;
    let text = ("0123456789\t界e\u{301}".repeat(16) + "\r\n").repeat(60);
    let mut session = super::marked_view(&text, 2, 0, 1);
    session.handle(Event::Resize(41, 15));
    session.insert('X');
    session.execute(Command::KeyboardQuit);
    session.insert('Y');
    session.execute(Command::History(History::Undo));
    split(&mut session, Split::Right);
    split(&mut session, Split::Below);
    for (index, id) in session.windows().order().into_iter().enumerate() {
        navigate_view(&mut session, id, (index + 1) * 10);
    }
    let before = views(&session);
    let layout = session.windows().layout().clone();
    let stamp = session.buffer().unwrap().stamp();
    let state = session.buffer().unwrap().state();
    let content = super::buffer_text(&session);
    for _ in 0..8 {
        for dimensions in tiny_sizes() {
            session.handle(Event::Resize(dimensions.width, dimensions.height));
            bounded(&mut session, dimensions);
            assert_eq!(views(&session), before);
        }
        session.execute(Command::Split(Split::Below));
        assert!(session.echo().starts_with("Split does not fit"));
        session.handle(Event::Resize(41, 15));
        bounded(&mut session, size(41, 15));
        assert_eq!(views(&session), before);
        assert_eq!(*session.windows().layout(), layout);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().state(), state);
        assert_eq!(super::buffer_text(&session), content);
        assert_eq!(
            session.buffer().unwrap().mark(),
            Some(rev::coordinates::Scalar(1))
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    }
    session.execute(Command::History(History::Redo));
    assert_eq!(
        session.buffer().unwrap().len_chars(),
        text.chars().count() + 2
    );
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    assert_eq!(super::buffer_text(&session), text);
    assert!(!session.buffer().unwrap().is_modified());
    bounded(&mut session, size(41, 15));
}

// Repeated successful split/delete/keep cycles must never reuse retired view
// identities or consume an existing redo branch, even across suspended frames.
#[test]
fn recreated_views() {
    for first in [Split::Below, Split::Right] {
        recreate(first);
    }
}

fn recreate(first: Split) {
    use rev::command::{History, Motion, WindowCommand};
    use rev::coordinates::Coordinates;
    use std::collections::BTreeSet;
    let source = "a\t界e\u{301}👩\u{200d}💻\r\nlast";
    let mut session = editor(source, size(61, 15));
    session.insert('X');
    session.execute(Command::KeyboardQuit);
    session.insert('Y');
    session.execute(Command::History(History::Undo));
    let anchor = session.windows().selected_id();
    let mut issued = BTreeSet::from([anchor]);
    let content = super::buffer_text(&session);
    let state = session.buffer().unwrap().state();
    for _ in 0..12 {
        split(&mut session, first);
        split(
            &mut session,
            if first == Split::Below {
                Split::Right
            } else {
                Split::Below
            },
        );
        let ids = session.windows().order();
        for id in ids.iter().copied().filter(|id| *id != anchor) {
            assert!(issued.insert(id), "a retired identity was reused");
            session.select_view(id).unwrap();
            session.execute(Command::Motion(Motion::BufferEnd));
        }
        session.select_view(anchor).unwrap();
        let points = views(&session)
            .into_iter()
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>();
        for dimensions in [size(0, 0), size(1, 1), size(7, 5), size(61, 15)] {
            session.handle(Event::Resize(dimensions.width, dimensions.height));
            bounded(&mut session, dimensions);
            assert_eq!(
                views(&session)
                    .into_iter()
                    .map(|(id, view)| (id, view.point))
                    .collect::<Vec<_>>(),
                points
            );
            let buffer = session.buffer().unwrap();
            let coordinates = Coordinates::new(buffer).unwrap();
            for (_, point) in &points {
                assert_eq!(point.stamp, buffer.stamp());
                assert!(coordinates.is_boundary(point.position).unwrap());
            }
        }
        let retired = ids.into_iter().find(|id| *id != anchor).unwrap();
        session.select_view(retired).unwrap();
        session.execute(Command::Window(WindowCommand::Delete));
        assert_eq!(session.windows().iter().len(), 2);
        assert!(session.windows().get(retired).is_err());
        session.select_view(anchor).unwrap();
        session.execute(Command::Window(WindowCommand::Keep));
        assert_eq!(session.windows().order(), [anchor]);
        assert_eq!(super::buffer_text(&session), content);
        assert_eq!(session.buffer().unwrap().state(), state);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    }
    session.execute(Command::History(History::Redo));
    assert_eq!(
        super::buffer_text(&session),
        "a\tXY界e\u{301}👩\u{200d}💻\r\nlast"
    );
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    assert_eq!(super::buffer_text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}
