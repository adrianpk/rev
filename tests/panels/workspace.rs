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
