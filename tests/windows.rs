#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use rev::buffer::Buffers;
use rev::command::{Command, Split, WindowCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};
use rev::view::{Size, View};
use rev::window::Layout;
use unicode_width::UnicodeWidthStr;

fn editor(text: &str, size: Size) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(text).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(text.chars().count().min(2));
    view.column = Some(5);
    Session::new(buffers, view, size).unwrap()
}

fn size(width: u16, height: u16) -> Size {
    Size { width, height }
}

fn split(session: &mut Session, direction: Split) {
    assert_eq!(session.execute(Command::Split(direction)), Action::Redraw);
    assert_eq!(session.echo(), "");
}

fn bounded(session: &mut Session, size: Size) {
    let frame = session.frame().unwrap();
    let expected = if size.width == 0 {
        0
    } else {
        usize::from(size.height)
    };
    assert_eq!(frame.rows().len(), expected);
    for row in frame.rows() {
        assert_eq!(row.text().width(), usize::from(size.width));
        assert!(row.row() < size.height);
        assert!(!row.text().contains('\x1b'));
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < size.width && cursor.row < size.height);
    }
}

// Each split allocates one distinct view, copies its complete initial navigation
// state, shares the canonical buffer and retains the original selected identity.
#[test]
fn split_state() {
    for direction in [Split::Below, Split::Right] {
        let dimensions = size(31, 13);
        let mut session = editor("abcdef\nsecond", dimensions);
        let original = session.windows().selected_id();
        let before = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        split(&mut session, direction);
        assert_eq!(session.windows().selected_id(), original);
        assert_eq!(session.windows().iter().len(), 2);
        for (_, view) in session.windows().iter() {
            assert_eq!(*view, before);
            assert_eq!(view.point.stamp, stamp);
        }
        let ids = session
            .windows()
            .iter()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 2);
        bounded(&mut session, dimensions);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Below/right in either order produces the requested nested geometry, with
// separate status rows, a right divider and one reserved global echo row.
#[test]
fn mixed_layout() {
    for first in [Split::Below, Split::Right] {
        let mut session = editor("abc", size(31, 13));
        let original = session.windows().selected_id();
        split(&mut session, first);
        let second = if first == Split::Below {
            Split::Right
        } else {
            Split::Below
        };
        split(&mut session, second);
        assert_eq!(session.windows().selected_id(), original);
        let Layout::Split {
            direction,
            first: child,
            ..
        } = session.windows().layout()
        else {
            panic!("root split");
        };
        assert_eq!(*direction, first);
        assert!(matches!(**child, Layout::Split { .. }));
        let geometry = session.windows().geometry(size(31, 13)).unwrap();
        let actual = geometry
            .panes()
            .iter()
            .map(|(_, a)| (a.column(), a.row(), a.width(), a.height()))
            .collect::<Vec<_>>();
        let expected = if first == Split::Below {
            vec![(0, 0, 15, 6), (16, 0, 15, 6), (0, 6, 31, 6)]
        } else {
            vec![(0, 0, 15, 6), (0, 6, 15, 6), (16, 0, 15, 12)]
        };
        assert_eq!(actual, expected);
        assert_eq!(geometry.borders().len(), 1);
        assert_eq!(session.windows().iter().len(), 3);
        bounded(&mut session, size(31, 13));
    }
}

// A normal pane requires three text columns and text+status rows. Admission
// includes dividers/echo and rejects before changing layout, focus or view state.
#[test]
fn fit_admission() {
    let cases = [
        ("below exact", 3, 5, Split::Below, true),
        ("below short", 3, 4, Split::Below, false),
        ("right exact", 7, 3, Split::Right, true),
        ("right narrow", 6, 3, Split::Right, false),
        ("narrow text", 2, 8, Split::Below, false),
        ("short text", 20, 2, Split::Right, false),
        ("zero", 0, 0, Split::Below, false),
        ("maximum", u16::MAX, u16::MAX, Split::Right, true),
    ];
    for (name, width, height, direction, fits) in cases {
        let mut session = editor("text", size(width, height));
        let original = session.windows().selected_id();
        let layout = session.windows().layout().clone();
        let before = session.view().clone();
        session.execute(Command::Split(direction));
        assert_eq!(session.windows().selected_id(), original, "{name}");
        assert_eq!(*session.view(), before, "{name}");
        if fits {
            assert_eq!(session.windows().iter().len(), 2, "{name}");
        } else {
            assert_eq!(session.windows().iter().len(), 1, "{name}");
            assert_eq!(*session.windows().layout(), layout, "{name}");
            assert!(session.echo().starts_with("Split does not fit"));
        }
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

fn other(session: &Session) -> rev::window::ViewId {
    session
        .windows()
        .iter()
        .map(|(id, _)| id)
        .find(|id| *id != session.windows().selected_id())
        .unwrap()
}

// Moving and scrolling one selected pane cannot move its sibling. Native view
// selection makes both owners independently navigable with their own pane size.
#[test]
fn independent_navigation() {
    use rev::command::{Motion, Scroll};
    let text = ("0123456789\t界".repeat(12) + "\n").repeat(60);
    for direction in [Split::Below, Split::Right] {
        let dimensions = size(31, 13);
        let mut session = editor(&text, dimensions);
        let original = session.windows().selected_id();
        split(&mut session, direction);
        bounded(&mut session, dimensions);
        let sibling = other(&session);
        let sibling_before = session.windows().get(sibling).unwrap().clone();
        session.execute(Command::Motion(Motion::BufferEnd));
        session.execute(Command::Scroll(Scroll::PreviousLine));
        session.execute(Command::Motion(Motion::LineEnd));
        bounded(&mut session, dimensions);
        let moved = session.view().clone();
        assert!(moved.top > 0 && moved.left > 0);
        assert_eq!(*session.windows().get(sibling).unwrap(), sibling_before);
        session.select_view(sibling).unwrap();
        session.execute(Command::Scroll(Scroll::NextLine));
        bounded(&mut session, dimensions);
        assert_ne!(session.view().point, moved.point);
        assert_eq!(*session.windows().get(original).unwrap(), moved);
        assert!(session.buffer().unwrap().history().is_empty());
        session.select_view(original).unwrap();
        assert_eq!(*session.view(), moved);
    }
}

// An edit initiated by a non-first view publishes one canonical text revision
// and valid mapped points for both views. Undo/redo can be invoked from either
// view without stale stamps or a copied/independent document.
#[test]
fn shared_publication() {
    use rev::command::{History, Motion};
    let dimensions = size(31, 13);
    let mut session = editor("first\nsecond\nlast", dimensions);
    split(&mut session, Split::Right);
    let original = session.windows().selected_id();
    let sibling = other(&session);
    let owner = session.buffer().unwrap().id();
    let before = session.buffer().unwrap().stamp();
    let original_point = session.view().point.position;
    session.select_view(sibling).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.insert('界');
    assert_eq!(session.buffer().unwrap().id(), owner);
    assert_eq!(
        session.buffer().unwrap().stamp().revision,
        before.revision + 1
    );
    assert_eq!(
        session.windows().get(original).unwrap().point.position,
        original_point
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    bounded(&mut session, dimensions);
    session.select_view(original).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(
        session.buffer().unwrap().len_chars(),
        "first\nsecond\nlast".chars().count()
    );
    assert!(!session.buffer().unwrap().is_modified());
    bounded(&mut session, dimensions);
    session.select_view(sibling).unwrap();
    session.execute(Command::History(History::Redo));
    assert!(session.buffer().unwrap().is_modified());
    let stamp = session.buffer().unwrap().stamp();
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, stamp);
        view.cell(session.buffer().unwrap()).unwrap();
    }
    bounded(&mut session, dimensions);
}

// Wide/combining/tab glyphs clip at each pane seam rather than byte offsets.
// Only the selected pane supplies the status marker and translated hardware cursor.
#[test]
fn composed_cells() {
    use rev::display::Position;
    let dimensions = size(17, 9);
    let mut session = editor("a\t界e\u{301}👩\u{200d}💻 tail\nsecond", dimensions);
    split(&mut session, Split::Below);
    split(&mut session, Split::Right);
    let geometry = session.windows().geometry(dimensions).unwrap();
    let right = geometry.panes()[1].0;
    bounded(&mut session, dimensions);
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].text(), "a   界e\u{301} |a   界e\u{301} ");
    assert!(frame.rows()[3].text().starts_with("> "));
    assert_eq!(&frame.rows()[3].text()[8..11], "|  ");
    assert!(frame.rows()[7].text().starts_with("  "));
    assert_eq!(frame.cursor(), Some(Position { column: 4, row: 0 }));
    session.select_view(right).unwrap();
    let frame = session.frame().unwrap();
    assert!(frame.rows()[3].text().starts_with("  "));
    assert_eq!(&frame.rows()[3].text()[8..11], "|> ");
    assert_eq!(frame.cursor(), Some(Position { column: 13, row: 0 }));
}

// Closing a buffer already shown by several leaves repairs every live reference
// while preserving the split tree and selected view identity.
#[test]
fn replaced_references() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::BufferCommand;
    let dimensions = size(31, 13);
    let mut session = editor("original", dimensions);
    split(&mut session, Split::Below);
    split(&mut session, Split::Right);
    let closed = session.buffer().unwrap().id();
    let selected = session.windows().selected_id();
    let layout = session.windows().layout().clone();
    session.execute(Command::Buffer(BufferCommand::Kill));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(session.echo(), "Buffer closed");
    assert_ne!(session.buffer().unwrap().id(), closed);
    assert_eq!(session.windows().selected_id(), selected);
    assert_eq!(*session.windows().layout(), layout);
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, session.buffer().unwrap().stamp());
    }
    session.insert('!');
    bounded(&mut session, dimensions);
    for (_, view) in session.windows().iter() {
        view.cell(session.buffer().unwrap()).unwrap();
    }
}

// Insufficient resized geometry suspends projection without deleting the tree.
// Further split attempts remain rejected until the full layout fits again.
#[test]
fn suspended_layout() {
    use crossterm::event::Event;
    let dimensions = size(31, 13);
    let mut session = editor("original", dimensions);
    split(&mut session, Split::Below);
    split(&mut session, Split::Right);
    bounded(&mut session, dimensions);
    let layout = session.windows().layout().clone();
    let selected = session.windows().selected_id();
    let views = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.clone()))
        .collect::<Vec<_>>();
    for dimensions in [size(0, 0), size(1, 1), size(3, 3)] {
        session.handle(Event::Resize(dimensions.width, dimensions.height));
        bounded(&mut session, dimensions);
        assert!(session.frame().unwrap().cursor().is_none());
        session.execute(Command::Split(Split::Right));
        assert!(session.echo().starts_with("Split does not fit"));
        assert_eq!(*session.windows().layout(), layout);
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            views
        );
    }
    session.handle(Event::Resize(dimensions.width, dimensions.height));
    bounded(&mut session, dimensions);
    assert!(session.frame().unwrap().cursor().is_some());
    assert_eq!(*session.windows().layout(), layout);
}

// Repeated nesting is bounded by u16 geometry and minimum child dimensions;
// reaching that boundary refuses further changes before any registry publication.
#[test]
fn nested_limits() {
    use rev::window::Windows;
    let buffers = Buffers::new().unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut windows = Windows::new(view).unwrap();
    let dimensions = size(u16::MAX, u16::MAX);
    let mut accepted = 0;
    for index in 0..40 {
        let direction = if index % 2 == 0 {
            Split::Below
        } else {
            Split::Right
        };
        let layout = windows.layout().clone();
        if windows.split(direction, dimensions).is_err() {
            assert_eq!(*windows.layout(), layout);
            break;
        }
        accepted += 1;
    }
    assert!(accepted > 20 && accepted < 32);
    assert_eq!(windows.iter().len(), accepted + 1);
    let geometry = windows.geometry(dimensions).unwrap();
    assert_eq!(geometry.panes().len(), accepted + 1);
    for (_, area) in geometry.panes() {
        assert!(area.width() >= 3 && area.height() >= 2);
    }
}

// Invalid native view identities and an active prompt cannot steal focus or
// change layout. Canceling the prompt returns to the same selected view.
#[test]
fn view_admission() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    let dimensions = size(31, 13);
    let mut session = editor("original", dimensions);
    split(&mut session, Split::Below);
    let selected = session.windows().selected_id();
    let sibling = other(&session);
    let foreign = editor("foreign", dimensions).windows().selected_id();
    assert!(session.select_view(foreign).is_err());
    assert_eq!(session.windows().selected_id(), selected);
    session.execute(Command::FindFile);
    assert!(session.select_view(sibling).is_err());
    assert_eq!(session.windows().selected_id(), selected);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(session.windows().selected_id(), selected);
    session.select_view(sibling).unwrap();
    assert_eq!(session.windows().selected_id(), sibling);
}

fn window(session: &mut Session, command: WindowCommand) {
    assert_eq!(session.execute(Command::Window(command)), Action::Redraw);
}

// Traversal follows layout leaves rather than allocation order, wraps exactly,
// and does not modify any view's independent point/scroll state.
#[test]
fn window_cycle() {
    for first in [Split::Below, Split::Right] {
        let mut session = editor("abcdef\nsecond", size(31, 13));
        let original = session.windows().selected_id();
        split(&mut session, first);
        let second = other(&session);
        split(
            &mut session,
            if first == Split::Below {
                Split::Right
            } else {
                Split::Below
            },
        );
        let third = session
            .windows()
            .iter()
            .map(|(id, _)| id)
            .find(|id| *id != original && *id != second)
            .unwrap();
        assert_eq!(session.windows().order(), vec![original, third, second]);
        let before = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>();
        for expected in [third, second, original, third] {
            window(&mut session, WindowCommand::Other);
            assert_eq!(session.windows().selected_id(), expected);
            assert_eq!(
                session
                    .windows()
                    .iter()
                    .map(|(id, view)| (id, view.clone()))
                    .collect::<Vec<_>>(),
                before
            );
        }
    }
}

// Removing either child reconnects its sibling and retains both surviving
// view states. The removed identity cannot remain selected or be selected again.
#[test]
fn delete_reconnect() {
    let mut session = editor("abcdef\nsecond", size(31, 13));
    let original = session.windows().selected_id();
    split(&mut session, Split::Below);
    let below = other(&session);
    split(&mut session, Split::Right);
    let right = session.windows().order()[1];
    let before = session.windows().get(right).unwrap().clone();
    let owner = session.buffer().unwrap().id();
    window(&mut session, WindowCommand::Delete);
    assert_eq!(session.windows().selected_id(), right);
    assert_eq!(session.windows().order(), vec![right, below]);
    assert_eq!(*session.view(), before);
    assert!(session.select_view(original).is_err());
    let Layout::Split {
        direction,
        first,
        second,
    } = session.windows().layout()
    else {
        panic!("remaining below split");
    };
    assert_eq!(*direction, Split::Below);
    assert_eq!(**first, Layout::Leaf(right));
    assert_eq!(**second, Layout::Leaf(below));
    session.select_view(below).unwrap();
    window(&mut session, WindowCommand::Delete);
    assert_eq!(session.windows().order(), vec![right]);
    assert_eq!(*session.windows().layout(), Layout::Leaf(right));
    assert_eq!(session.windows().selected_id(), right);
    assert_eq!(session.buffer().unwrap().id(), owner);
    assert!(session.buffer().unwrap().history().is_empty());
    bounded(&mut session, size(31, 13));
}

// Keeping only the selected window retains that exact identity and view state;
// it does not replace it with another buffer view or alter canonical history.
#[test]
fn keep_selected() {
    let mut session = editor("abcdef\nsecond", size(31, 13));
    split(&mut session, Split::Below);
    let below = other(&session);
    split(&mut session, Split::Right);
    session.select_view(below).unwrap();
    session.execute(Command::Motion(rev::command::Motion::LineEnd));
    let before = session.view().clone();
    let removed = session
        .windows()
        .order()
        .into_iter()
        .filter(|id| *id != below)
        .collect::<Vec<_>>();
    window(&mut session, WindowCommand::Keep);
    assert_eq!(session.windows().selected_id(), below);
    assert_eq!(session.windows().order(), vec![below]);
    assert_eq!(*session.view(), before);
    assert_eq!(*session.windows().layout(), Layout::Leaf(below));
    for id in removed {
        assert!(session.select_view(id).is_err());
    }
    assert!(session.buffer().unwrap().history().is_empty());
    bounded(&mut session, size(31, 13));
}

// The sole editor cannot be deleted. Keep/other remain safe no-ops, preserving
// its identity, canonical buffer and independent navigation state.
#[test]
fn sole_window() {
    for command in [
        WindowCommand::Delete,
        WindowCommand::Keep,
        WindowCommand::Other,
    ] {
        let mut session = editor("original", size(31, 13));
        session.insert('!');
        let selected = session.windows().selected_id();
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        window(&mut session, command);
        assert_eq!(session.windows().order(), vec![selected]);
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(*session.windows().layout(), Layout::Leaf(selected));
        assert_eq!(*session.view(), view);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().state(), state);
        if command == WindowCommand::Delete {
            assert!(session.echo().contains("sole editing window"));
        }
        assert!(session.buffer().unwrap().is_modified());
        bounded(&mut session, size(31, 13));
    }
}

// Deleting a leaf opposite a nested sibling promotes the whole surviving
// subtree, including direction, leaf order and every independent view state.
#[test]
fn promoted_subtree() {
    for remove_first in [true, false] {
        let mut session = editor("original", size(31, 13));
        let original = session.windows().selected_id();
        split(
            &mut session,
            if remove_first {
                Split::Right
            } else {
                Split::Below
            },
        );
        let second = other(&session);
        if remove_first {
            session.select_view(second).unwrap();
        }
        split(
            &mut session,
            if remove_first {
                Split::Below
            } else {
                Split::Right
            },
        );
        let removed = if remove_first { original } else { second };
        let survivors = session
            .windows()
            .order()
            .into_iter()
            .filter(|id| *id != removed)
            .collect::<Vec<_>>();
        let before = session
            .windows()
            .iter()
            .filter(|(id, _)| *id != removed)
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>();
        session.select_view(removed).unwrap();
        window(&mut session, WindowCommand::Delete);
        assert_eq!(session.windows().order(), survivors);
        assert_eq!(session.windows().selected_id(), survivors[0]);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            before
        );
        let Layout::Split { direction, .. } = session.windows().layout() else {
            panic!("promoted split");
        };
        assert_eq!(
            *direction,
            if remove_first {
                Split::Below
            } else {
                Split::Right
            }
        );
        bounded(&mut session, size(31, 13));
    }
}

// Repeated split/cycle/delete collapses parent nodes without orphaning leaves
// or reusing removed identities. Registry and layout contain exactly the same IDs.
#[test]
fn repeated_deletion() {
    let dimensions = size(31, 13);
    let mut session = editor("original", dimensions);
    let mut allocated = session
        .windows()
        .order()
        .into_iter()
        .collect::<BTreeSet<_>>();
    for round in 0..12 {
        let first = if round % 2 == 0 {
            Split::Below
        } else {
            Split::Right
        };
        let second = if first == Split::Below {
            Split::Right
        } else {
            Split::Below
        };
        let original = session.windows().selected_id();
        split(&mut session, first);
        let sibling = other(&session);
        assert!(allocated.insert(sibling));
        split(&mut session, second);
        let middle = session.windows().order()[1];
        assert!(allocated.insert(middle));
        window(&mut session, WindowCommand::Other);
        assert_eq!(session.windows().selected_id(), middle);
        window(&mut session, WindowCommand::Delete);
        assert!(session.windows().get(middle).is_err());
        assert_eq!(session.windows().selected_id(), original);
        window(&mut session, WindowCommand::Other);
        assert_eq!(session.windows().selected_id(), sibling);
        window(&mut session, WindowCommand::Delete);
        assert!(session.windows().get(sibling).is_err());
        assert_eq!(session.windows().order(), vec![original]);
        let registered = session
            .windows()
            .iter()
            .map(|(id, _)| id)
            .collect::<BTreeSet<_>>();
        assert_eq!(registered, session.windows().order().into_iter().collect());
        bounded(&mut session, dimensions);
    }
    assert!(session.buffer().unwrap().history().is_empty());
}

fn buffer_text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Removing the last visible window never removes its dirty buffer. Both delete
// paths retain text/history and unsaved-quit protection, and switching recovers
// the same identity instead of allocating an independent replacement document.
#[test]
fn hidden_modified() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::BufferCommand;
    for action in [WindowCommand::Delete, WindowCommand::Keep] {
        let mut buffers = Buffers::new().unwrap();
        let active = buffers.create("active").unwrap();
        let hidden = buffers.create("hidden").unwrap();
        let name = buffers.get(hidden).unwrap().name().to_owned();
        let view = View::new(buffers.get(active).unwrap());
        let mut session = Session::new(buffers, view, size(31, 13)).unwrap();
        let original = session.windows().selected_id();
        split(&mut session, Split::Right);
        window(&mut session, WindowCommand::Other);
        session.execute(Command::Buffer(BufferCommand::Switch));
        for ch in name.chars() {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::NONE,
            )));
        }
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        session.insert('!');
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        if action == WindowCommand::Keep {
            session.select_view(original).unwrap();
        }
        window(&mut session, action);
        assert_eq!(session.windows().iter().len(), 1);
        assert_eq!(session.buffer().unwrap().id(), active);
        assert_eq!(session.execute(Command::Quit), Action::Redraw);
        assert!(session.echo().starts_with("Unsaved changes"));
        session.execute(Command::Buffer(BufferCommand::Switch));
        for ch in name.chars() {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::NONE,
            )));
        }
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(session.buffer().unwrap().id(), hidden);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().state(), state);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(buffer_text(&session), "!hidden");
        bounded(&mut session, size(31, 13));
    }
}

// Traversal/deletion use the logical editing tree even when geometry suspends.
// Restoring dimensions draws the surviving selected view without replacing it.
#[test]
fn hidden_layout_commands() {
    use crossterm::event::Event;
    let mut session = editor("original", size(31, 13));
    split(&mut session, Split::Below);
    split(&mut session, Split::Right);
    let order = session.windows().order();
    session.handle(Event::Resize(0, 0));
    window(&mut session, WindowCommand::Other);
    assert_eq!(session.windows().selected_id(), order[1]);
    window(&mut session, WindowCommand::Delete);
    assert_eq!(session.windows().selected_id(), order[0]);
    assert_eq!(session.windows().order(), vec![order[0], order[2]]);
    window(&mut session, WindowCommand::Keep);
    window(&mut session, WindowCommand::Delete);
    assert!(session.echo().contains("sole editing window"));
    assert_eq!(session.windows().order(), vec![order[0]]);
    session.handle(Event::Resize(31, 13));
    bounded(&mut session, size(31, 13));
    assert_eq!(session.windows().selected_id(), order[0]);
}

fn interaction(session: &mut Session, mode: &str) {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::BufferCommand;
    match mode {
        "argument" => {
            session.execute(Command::FindFile);
        }
        "inventory" => {
            session.execute(Command::Buffer(BufferCommand::List));
        }
        "decision" => {
            session.execute(Command::Buffer(BufferCommand::Kill));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )));
        }
        _ => panic!("unknown interaction"),
    }
}

// Prompt/list/decision ownership is separate from editing traversal. Native
// command dispatch cannot steal focus; cancellation enables normal cycling again.
#[test]
fn transient_focus() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    for mode in ["argument", "inventory", "decision"] {
        let mut session = editor("original", size(31, 13));
        split(&mut session, Split::Right);
        session.insert('!');
        let selected = session.windows().selected_id();
        let layout = session.windows().layout().clone();
        let stamp = session.buffer().unwrap().stamp();
        let before = session.view().clone();
        interaction(&mut session, mode);
        for command in [
            WindowCommand::Other,
            WindowCommand::Delete,
            WindowCommand::Keep,
        ] {
            window(&mut session, command);
            assert!(session.echo().contains("owns input focus"));
            assert_eq!(session.windows().selected_id(), selected);
            assert_eq!(*session.windows().layout(), layout);
            assert_eq!(*session.view(), before);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
        }
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
        )));
        window(&mut session, WindowCommand::Other);
        assert_ne!(session.windows().selected_id(), selected);
        bounded(&mut session, size(31, 13));
    }
}

// Changing editing focus ends ordinary typing continuity even when both views
// share the same rope and mapped position. Each view's typing is a whole undo step.
#[test]
fn cycle_typing() {
    use rev::command::History;
    let mut session = editor("abcdef", size(31, 13));
    split(&mut session, Split::Right);
    session.insert('x');
    window(&mut session, WindowCommand::Other);
    session.insert('y');
    assert_eq!(buffer_text(&session), "abxycdef");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(buffer_text(&session), "abxcdef");
    session.execute(Command::History(History::Undo));
    assert_eq!(buffer_text(&session), "abcdef");
    assert!(!session.buffer().unwrap().is_modified());
    bounded(&mut session, size(31, 13));
}
