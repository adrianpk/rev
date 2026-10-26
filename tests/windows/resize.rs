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
