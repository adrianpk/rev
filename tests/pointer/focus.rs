use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::coordinates::Scalar;
use rev::display::{Position, Size};
use rev::panel::Composition;
use rev::session::{Action, Session};
use rev::view::View;
use rev::window::Split;

fn session() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("alpha\nbeta\n").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

pub(super) fn press(position: Position) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: position.column,
        row: position.row,
        modifiers: KeyModifiers::NONE,
    })
}

fn composition(session: &Session) -> Composition {
    Composition::workspace(
        session.windows(),
        session.tree(),
        Size {
            width: 80,
            height: 12,
        },
        0,
    )
    .unwrap()
}

// Native primary hits select the composed leaf and map only its text content.
// Shared source/history and the inactive view's point remain unchanged.
#[test]
fn text_focus() {
    let mut session = session();
    let original = session.windows().selected_id();
    let before = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let layout = composition(&session);
    let panel = layout.panels()[1].1;
    let text = session
        .windows()
        .get(peer)
        .unwrap()
        .gutter_geometry(session.buffer().unwrap(), panel.content())
        .unwrap()
        .text();
    assert_eq!(
        session.handle(press(Position {
            column: text.column() + 2,
            row: 1
        })),
        Action::Redraw
    );
    assert_eq!(session.windows().selected_id(), peer);
    assert_eq!(session.view().point.position, Scalar(8));
    assert_eq!(
        session.windows().get(original).unwrap().point.position,
        Scalar(0)
    );
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        before
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        session.frame().unwrap().cursor(),
        Some(Position {
            column: text.column() + 2,
            row: 1
        })
    );
}

// Status and number cells may focus their owning pane without moving point.
// Shared separators, prompt and outside coordinates never select another view.
#[test]
fn decoration_focus() {
    let mut session = session();
    let original = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let layout = composition(&session);
    let panel = layout.panels()[1].1;
    for position in [
        Position {
            column: panel.outer().column(),
            row: 0,
        },
        Position {
            column: panel.outer().column(),
            row: panel.status().unwrap().row(),
        },
    ] {
        session.select_view(original).unwrap();
        assert_eq!(session.handle(press(position)), Action::Redraw);
        assert_eq!(session.windows().selected_id(), peer);
        assert_eq!(session.view().point.position, Scalar(0));
    }
    let border = layout.borders()[0];
    for position in [
        Position {
            column: border.column(),
            row: 0,
        },
        Position {
            column: 0,
            row: layout.prompt().row(),
        },
        Position {
            column: u16::MAX,
            row: u16::MAX,
        },
    ] {
        assert_eq!(session.handle(press(position)), Action::Wait);
        assert_eq!(session.windows().selected_id(), peer);
        assert_eq!(session.view().point.position, Scalar(0));
    }
}
