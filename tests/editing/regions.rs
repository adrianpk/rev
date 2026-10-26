use rev::command::{Command, Motion, RegionCommand};
use rev::coordinates::Scalar;
use rev::session::{Action, Session};

fn region(session: &mut Session, command: RegionCommand) {
    assert_eq!(session.execute(Command::Region(command)), Action::Redraw);
}

fn range(session: &Session) -> Option<std::ops::Range<usize>> {
    session
        .buffer()
        .unwrap()
        .region(session.view().point)
        .unwrap()
        .map(|span| span.range)
}

// Movement grows/shrinks a region in either direction. Exchange preserves the
// selected source text while swapping the stored mark and selected view point.
#[test]
fn endpoints() {
    let mut session = super::editor("ab界e\u{301}z", 2, false);
    let stamp = session.buffer().unwrap().stamp();
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(session.echo(), "Mark is not set");
    region(&mut session, RegionCommand::Set);
    assert_eq!(range(&session), Some(2..2));
    session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(range(&session), Some(2..6));
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(6)));
    assert_eq!(range(&session), Some(2..6));
    session.execute(Command::Motion(Motion::BufferStart));
    assert_eq!(range(&session), Some(0..6));
    session.execute(Command::KeyboardQuit);
    assert_eq!(range(&session), None);
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(6)));
    region(&mut session, RegionCommand::Exchange);
    assert_eq!(range(&session), Some(0..6));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Multi-row presentation highlights complete Unicode display cells and selected
// line breaks, leaving document text, status and echo free of escape sequences.
#[test]
fn multiline_surface() {
    let mut session = super::editor("a\t界e\u{301}\r\nlast", 1, false);
    region(&mut session, RegionCommand::Set);
    session.execute(Command::Motion(Motion::BufferEnd));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(1..48));
    assert_eq!(frame.rows()[1].selection(), Some(0..4));
    assert!(
        frame
            .rows()
            .iter()
            .skip(2)
            .all(|row| row.selection().is_none())
    );
    let mut wire = Vec::new();
    frame.write(&mut wire).unwrap();
    assert!(wire.windows(4).any(|part| part == b"\x1b[7m"));
    assert!(wire.windows(5).any(|part| part == b"\x1b[27m"));
    assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    assert!(!session.buffer().unwrap().is_modified());
}

// Successful text publication deactivates the transient region but still maps
// the left-affine stored mark; a read-only refusal preserves active selection.
#[test]
fn edit_deactivation() {
    for read_only in [false, true] {
        let mut session = super::editor("abcdef", 2, read_only);
        region(&mut session, RegionCommand::Set);
        session.execute(Command::Motion(Motion::BufferEnd));
        session.insert('X');
        assert_eq!(session.buffer().unwrap().mark_active(), read_only);
        assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(2)));
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(!read_only)
        );
    }
}
