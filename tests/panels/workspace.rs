use rev::command::TreeCommand;
use rev::display::Frame;
use rev::window::Windows;

use super::*;

// Tree, editing leaves, completion content and prompt share one complete cell
// partition. Completion uses surplus rows without changing tree focus or any
// document state; its boundary joins both tree and nested editor separators.
#[test]
fn auxiliary_workspace() {
    let mut owner = editor();
    owner.execute(Command::Tree(TreeCommand::Toggle));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}\r\nlast").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut windows = Windows::new(view).unwrap();
    let size = Size {
        width: 61,
        height: 13,
    };
    windows.split(Split::Right, size).unwrap();
    windows.split(Split::Below, size).unwrap();
    let before = (
        buffers.get(id).unwrap().stamp(),
        buffers.get(id).unwrap().state(),
    );
    let composition = Composition::workspace(&windows, owner.tree(), size, 2).unwrap();
    super::geometry::coverage(&composition, area(0, 0, 61, 13));
    assert_eq!(
        composition.editing_size(),
        Size {
            width: 40,
            height: 10
        }
    );
    assert_eq!(
        composition.auxiliary().unwrap().content(),
        area(0, 10, 61, 2)
    );
    assert_eq!(
        composition.tree().unwrap().status(),
        Some(area(0, 8, 20, 1))
    );
    assert_eq!(composition.prompt(), area(0, 12, 61, 1));
    assert_eq!(composition.separator(position(20, 9)), Some('┴'));
    assert_eq!(composition.separator(position(40, 9)), Some('┴'));
    let frame =
        Frame::compose_workspace(&buffers, &windows, owner.tree(), &composition, "ready").unwrap();
    assert!(frame.rows()[10].text().trim().is_empty());
    assert!(frame.rows()[11].text().trim().is_empty());
    assert!(frame.rows()[12].text().starts_with("ready"));
    assert_eq!(frame.cursor(), Some(position(0, 0)));
    assert!(owner.tree().focused());
    assert_eq!(
        (
            buffers.get(id).unwrap().stamp(),
            buffers.get(id).unwrap().state()
        ),
        before
    );
    assert!(buffers.get(id).unwrap().history().is_empty());
}

// Maximal auxiliary requests clamp after the full editing minimum. Tiny frames
// keep a prompt and retained tree presentation without partially admitting a
// split subtree or moving focus into a hidden auxiliary panel.
#[test]
fn auxiliary_limits() {
    let mut owner = editor();
    owner.execute(Command::Tree(TreeCommand::Toggle));
    let session = nested(Split::Right, Split::Below, false);
    for (height, rows, panes) in [
        (3, 0, 0),
        (5, 0, 0),
        (6, 0, 3),
        (7, 0, 3),
        (8, 1, 3),
        (13, 6, 3),
    ] {
        let size = Size { width: 61, height };
        let composition =
            Composition::workspace(session.windows(), owner.tree(), size, u16::MAX).unwrap();
        assert_eq!(composition.panels().len(), panes);
        assert_eq!(
            composition
                .auxiliary()
                .map_or(0, |panel| panel.content().height()),
            rows
        );
        assert_eq!(composition.prompt(), area(0, height - 1, 61, 1));
        assert!(composition.tree().unwrap().active());
    }
}

// Native workspace projection keeps whole split subtrees and complete bounds
// across tree hiding thresholds and maximal auxiliary requests. Maximal axes
// inspect compact geometry without allocating a terminal-sized raster.
#[test]
fn workspace_grid() {
    let mut owner = editor();
    owner.execute(Command::Tree(TreeCommand::Toggle));
    let session = nested(Split::Right, Split::Below, false);
    let ids = session.windows().order();
    let dimensions = [0, 1, 7, 31, 32, 48, 61, 72, u16::MAX]
        .into_iter()
        .flat_map(|width| {
            [0, 1, 2, 3, 5, 6, 8, 13, u16::MAX]
                .into_iter()
                .flat_map(move |height| {
                    [0, 1, 8, u16::MAX].map(|rows| (Size { width, height }, rows))
                })
        });
    for (size, rows) in dimensions {
        let composition = Composition::workspace(session.windows(), owner.tree(), size, rows);
        assert_eq!(composition.is_some(), size.width > 0 && size.height > 0);
        if let Some(composition) = composition {
            workspace_bounds(&composition, size);
        }
    }
    assert_eq!(session.windows().order(), ids);
    assert!(owner.tree().focused());
    assert!(session.buffer().unwrap().history().is_empty());
}

fn workspace_bounds(composition: &Composition, size: Size) {
    assert_eq!(composition.size(), size);
    assert_eq!(
        composition.prompt(),
        area(0, size.height - 1, size.width, 1)
    );
    assert_eq!(
        composition.tree().is_some(),
        size.width >= 32 && size.height >= 3
    );
    assert!(composition.panels().len() == 3 || composition.panels().is_empty());
    if !composition.panels().is_empty() {
        super::geometry::coverage(composition, area(0, 0, size.width, size.height));
    }
}

// Command queries precede candidates while preserving the outer extent and
// complete editing minima, including native maximum auxiliary requests.
#[test]
fn command_region() {
    let session = editor();
    for (height, rows) in [(3, 0), (4, 0), (5, 1), (8, 4), (u16::MAX, u16::MAX - 4)] {
        let size = Size { width: 61, height };
        let composition =
            Composition::completion(session.windows(), session.tree(), size, u16::MAX).unwrap();
        assert_eq!(composition.size(), size);
        assert_eq!(
            composition
                .auxiliary()
                .map_or(0, |panel| panel.content().height()),
            rows
        );
        assert_eq!(
            composition.panels()[0].1.content().height(),
            if rows == 0 { height - 2 } else { 1 }
        );
        if let Some(panel) = composition.auxiliary() {
            assert_eq!(composition.prompt().row() + 1, panel.content().row());
            assert_eq!(panel.content().row() + panel.content().height(), height);
        }
    }
}
