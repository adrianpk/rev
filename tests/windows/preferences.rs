use rev::command::Scroll;
use rev::config::prepare;
use rev::coordinates::{Cell, Coordinates};
use rev::panel::Composition;

use super::*;

fn configured(text: &str, source: &str, extent: Size) -> Session {
    let fixture = config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", source));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(text).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(buffers, view, extent).unwrap()
}

// The same admitted ratio must drive window navigation and native composition,
// including nested axes. Resize cannot change split identities or canonical text.
#[test]
fn preferred_ratio() {
    let extent = size(81, 15);
    for (ratio, columns, rows) in [(0.1, 8, 2), (0.5, 40, 6), (0.9, 72, 11)] {
        let mut session = configured(
            "retained",
            &format!("windows.split_ratio = {ratio}"),
            extent,
        );
        split(&mut session, Split::Right);
        let selected = session.windows().selected_id();
        assert_eq!(
            session.windows().area(selected, extent).unwrap().width(),
            columns
        );
        split(&mut session, Split::Below);
        let geometry = session.windows().geometry(extent).unwrap();
        let composition =
            Composition::workspace(session.windows(), session.tree(), extent, 0).unwrap();
        let panel = composition
            .panels()
            .iter()
            .find(|(id, _)| *id == selected)
            .unwrap()
            .1;
        assert_eq!(
            (panel.outer().width(), panel.outer().height()),
            (columns, rows)
        );
        assert_eq!(
            geometry
                .panes()
                .iter()
                .find(|(id, _)| *id == selected)
                .unwrap()
                .1,
            panel.outer()
        );
        let order = session.windows().order();
        session.handle(crossterm::event::Event::Resize(0, 0));
        session.frame().unwrap();
        session.handle(crossterm::event::Event::Resize(extent.width, extent.height));
        assert_eq!(session.windows().order(), order);
        assert_eq!(session.windows().geometry(extent).unwrap(), geometry);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Overlap is global, uses the current content height and clamps to positive
// progress even above the pane height. Suspended pages retain their live point.
#[test]
fn page_overlap() {
    let text = (0..60)
        .map(|row| format!("line {row}\n"))
        .collect::<String>();
    for (overlap, distance) in [(0, 6), (2, 4), (5, 1), (1000, 1)] {
        let mut session = configured(
            &text,
            &format!("scroll.page_overlap = {overlap}"),
            size(24, 8),
        );
        for _ in 0..10 {
            session.execute(Command::Scroll(Scroll::NextLine));
        }
        session.execute(Command::Scroll(Scroll::PageForward));
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap().row,
            10 + distance
        );
        session.execute(Command::Scroll(Scroll::PageBackward));
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            Cell { row: 10, column: 0 }
        );
        let point = session.view().point;
        session.handle(crossterm::event::Event::Resize(2, 2));
        session.execute(Command::Scroll(Scroll::PageForward));
        assert_eq!(session.view().point, point);
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_cell(point.position)
            .unwrap();
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
