use super::*;

fn overlaps(first: Area, second: Area) -> bool {
    let x =
        |area: Area| u32::from(area.column())..u32::from(area.column()) + u32::from(area.width());
    let y = |area: Area| u32::from(area.row())..u32::from(area.row()) + u32::from(area.height());
    x(first).start < x(second).end
        && x(second).start < x(first).end
        && y(first).start < y(second).end
        && y(second).start < y(first).end
}

fn coverage(composition: &Composition, outer: Area) {
    let areas = composition
        .panels()
        .iter()
        .map(|(_, panel)| panel.outer())
        .chain(composition.borders().iter().copied())
        .chain(composition.auxiliary().map(|panel| panel.outer()))
        .chain(std::iter::once(composition.prompt()))
        .collect::<Vec<_>>();
    let mut cells = 0u64;
    for (index, area) in areas.iter().copied().enumerate() {
        assert!(area.column() >= outer.column() && area.row() >= outer.row());
        assert!(
            u32::from(area.column()) + u32::from(area.width())
                <= u32::from(outer.column()) + u32::from(outer.width())
        );
        assert!(
            u32::from(area.row()) + u32::from(area.height())
                <= u32::from(outer.row()) + u32::from(outer.height())
        );
        cells += u64::from(area.width()) * u64::from(area.height());
        for other in &areas[index + 1..] {
            assert!(!overlaps(area, *other));
        }
    }
    assert_eq!(cells, u64::from(outer.width()) * u64::from(outer.height()));
}

// Every small terminal shape and completion reservation either fits the whole
// split tree or suspends it. Admitted panels/separators/auxiliary/prompt cover
// the surface exactly once, retaining leaf identities and selected activity.
#[test]
fn resize_grid() {
    for (first, second, width, height) in [
        (Split::Right, Split::Right, 11, 3),
        (Split::Right, Split::Below, 7, 6),
        (Split::Below, Split::Right, 7, 6),
        (Split::Below, Split::Below, 3, 9),
    ] {
        for peer in [false, true] {
            grid_case(first, second, peer, Size { width, height });
        }
    }
}

fn grid_case(first: Split, second: Split, peer: bool, minimum: Size) {
    let session = nested(first, second, peer);
    let layout = session.windows().layout().clone();
    let ids = session.windows().order();
    let views = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.clone()))
        .collect::<Vec<_>>();
    let dimensions = (0..16).flat_map(|width| {
        (0..12).flat_map(move |height| [0, 1, 8].map(|rows| (width, height, rows)))
    });
    for (width, height, rows) in dimensions {
        let outer = area(2, 3, width, height);
        let composition = Composition::new(session.windows(), outer, rows);
        assert_eq!(
            composition.is_some(),
            width >= minimum.width && height >= minimum.height,
            "{first:?}/{second:?} {width}x{height}"
        );
        if let Some(composition) = composition {
            coverage(&composition, outer);
            assert_eq!(
                composition
                    .panels()
                    .iter()
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>(),
                ids
            );
            assert_eq!(
                composition
                    .panels()
                    .iter()
                    .filter(|(_, panel)| panel.active())
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>(),
                [session.windows().selected_id()]
            );
        }
    }
    assert_eq!(session.windows().layout(), &layout);
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>(),
        views
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Native maxima use compact area arithmetic rather than a terminal-sized
// raster. Zero/tiny axes suspend without partial leaves or lost identities.
#[test]
fn native_geometry() {
    let session = nested(Split::Right, Split::Below, false);
    let ids = session.windows().order();
    for (width, height, fits) in [
        (0, u16::MAX, false),
        (u16::MAX, 0, false),
        (1, u16::MAX, false),
        (u16::MAX, 1, false),
        (7, u16::MAX, true),
        (u16::MAX, 6, true),
        (u16::MAX, u16::MAX, true),
    ] {
        let outer = area(0, 0, width, height);
        let composition = Composition::new(session.windows(), outer, u16::MAX);
        assert_eq!(composition.is_some(), fits);
        if let Some(composition) = composition {
            coverage(&composition, outer);
            assert_eq!(composition.panels().len(), ids.len());
            assert_eq!(composition.separator(position(u16::MAX, u16::MAX)), None);
        }
    }
    assert_eq!(session.windows().order(), ids);
}

// Reprojection follows live focus without replacing semantic layout, shared
// buffer points or history. Only activity metadata changes between projections.
#[test]
fn active_identity() {
    let mut session = nested(Split::Right, Split::Below, false);
    session.insert('!');
    let outer = area(0, 0, 15, 13);
    let first = Composition::new(session.windows(), outer, 2).unwrap();
    let peer = session.windows().order()[1];
    let points = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.point))
        .collect::<Vec<_>>();
    let before = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    session.select_view(peer).unwrap();
    let second = Composition::new(session.windows(), outer, 2).unwrap();
    assert_eq!(first.borders(), second.borders());
    for ((id, prior), (current, panel)) in first.panels().iter().zip(second.panels()) {
        assert_eq!(id, current);
        assert_eq!(prior.outer(), panel.outer());
        assert_eq!(prior.content(), panel.content());
        assert_eq!(panel.active(), *id == peer);
    }
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.point))
            .collect::<Vec<_>>(),
        points
    );
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        before
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
}
