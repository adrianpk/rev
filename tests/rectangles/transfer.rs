use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BufferCommand, History};
use rev::config::prepare;

use super::*;
use crate::config_fixture::Fixture;

fn transfer(source_width: usize, target_width: usize, capture: RectangleCommand) -> Session {
    let fixture = Fixture::new();
    let config =
        format!("editor.tab_width = {source_width}\nlanguage.go.tab_width = {target_width}");
    let root = fixture.write("config.toml", &config);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let target = fixture.write("target.go", "\tX\nq");
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let source = buffers.create("a\tb\nc\td").unwrap();
    buffers.visit(&target).unwrap();
    let buffer = buffers.get_mut(source).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(buffer.len_chars());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 8,
        },
    )
    .unwrap();
    session.execute(Command::Rectangle(capture));
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste("target.go".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(text(&session), "\tX\nq");
    session
}

// Cross-policy transfer keeps copied spaces or killed literal tabs unchanged.
// Only left gaps are padded; one undo restores the destination's clean identity.
#[test]
fn cross_width() {
    for (source, target) in [(4, 8), (8, 2), (2, 4), (32, 1)] {
        for capture in [RectangleCommand::Copy, RectangleCommand::Kill] {
            transfer_case(source, target, capture);
        }
    }
}

fn transfer_case(source: usize, target: usize, capture: RectangleCommand) {
    let mut session = transfer(source, target, capture);
    let rows = if capture == RectangleCommand::Kill {
        ["a\tb".into(), "c\td".into()]
    } else {
        [
            format!("a{}b", " ".repeat(source - 1)),
            format!("c{}d", " ".repeat(source - 1)),
        ]
    };
    assert_eq!(session.rectangle().unwrap().rows(), rows);
    assert_eq!(session.rectangle().unwrap().width(), source + 1);
    at(&mut session, cell(0, target));
    let before = session.view().point.position;
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    let expected = format!("\t{}X\nq{}{}", rows[0], " ".repeat(target - 1), rows[1]);
    assert_eq!(text(&session), expected);
    assert_eq!(session.buffer().unwrap().policy().tab_width, target);
    let right = if capture == RectangleCommand::Kill {
        let after_letter = target + 1;
        after_letter + target - after_letter % target + 1
    } else {
        target + source + 1
    };
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        cell(1, right)
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "\tX\nq");
    assert_eq!(session.view().point.position, before);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert_eq!(session.rectangle().unwrap().rows(), rows);
    assert_eq!(session.kill_ring().len(), 0);
}

// Shared destination points map through a cross-width literal-tab yank. Undo
// from the peer restores both live points and the destination's saved identity.
#[test]
fn shared_transfer() {
    use rev::command::{Motion, RegionCommand, Split};

    let mut session = transfer(4, 8, RectangleCommand::Kill);
    at(&mut session, cell(0, 8));
    session.execute(Command::Region(RegionCommand::Set));
    let origin = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    let before = session.view().point.position;
    session.select_view(origin).unwrap();
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    let expected = "\ta\tbX\nq       c\td";
    assert_eq!(text(&session), expected);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session.view().point.position,
        Scalar(expected.chars().count())
    );
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        session.view().point.position
    );
    session.select_view(peer).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "\tX\nq");
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        Scalar(1)
    );
    assert_eq!(session.view().point.position, before);
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(1)));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert!(session.buffer().unwrap().is_modified());
    session.frame().unwrap();
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, session.buffer().unwrap().stamp());
        assert!(
            Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(view.point.position)
                .unwrap()
        );
    }
}
