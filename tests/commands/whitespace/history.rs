use rev::buffer::{Buffer, Stamp, State};
use rev::command::{Command, History, RegionCommand, Split, Whitespace};
use rev::coordinates::Coordinates;
use rev::session::Session;

use super::policy::configured;
use super::{named, select, text};

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    bytes: Vec<u8>,
    stamp: Stamp,
    state: State,
    dirty: bool,
    readonly: bool,
    history: usize,
    redo: usize,
    mark: Option<rev::coordinates::Scalar>,
    active: bool,
}

fn bytes(buffer: &Buffer) -> Vec<u8> {
    let mut bytes = Vec::new();
    rev::file::write(buffer, &mut bytes).unwrap();
    bytes
}

fn snapshot(session: &Session) -> Snapshot {
    let buffer = session.buffer().unwrap();
    Snapshot {
        bytes: bytes(buffer),
        stamp: buffer.stamp(),
        state: buffer.state(),
        dirty: buffer.is_modified(),
        readonly: buffer.is_readonly(),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        mark: buffer.mark(),
        active: buffer.mark_active(),
    }
}

// Rejection and no-op paths preserve exact text/revision, saved state, history,
// mark and view. An empty region is a no-op even in a read-only buffer.
#[test]
fn admission_noops() {
    for (name, operation, source, readonly, region, message) in [
        (
            "missing tabify",
            Whitespace::Tabify,
            "    X",
            false,
            None,
            "Region is not active",
        ),
        (
            "missing untabify",
            Whitespace::Untabify,
            "\tX",
            false,
            None,
            "Region is not active",
        ),
        (
            "empty",
            Whitespace::Tabify,
            "    X",
            false,
            Some((0, 0)),
            "No whitespace changed",
        ),
        (
            "empty readonly",
            Whitespace::Untabify,
            "\tX",
            true,
            Some((0, 0)),
            "No whitespace changed",
        ),
        (
            "minimal tabs",
            Whitespace::Tabify,
            "\tX",
            false,
            Some((0, 1)),
            "No whitespace changed",
        ),
        (
            "spaces",
            Whitespace::Untabify,
            "    X",
            false,
            Some((0, 4)),
            "No whitespace changed",
        ),
        (
            "one cell",
            Whitespace::Tabify,
            " X",
            false,
            Some((0, 1)),
            "No whitespace changed",
        ),
        (
            "readonly tabify",
            Whitespace::Tabify,
            "    X",
            true,
            Some((0, 4)),
            "buffer is read-only",
        ),
        (
            "readonly untabify",
            Whitespace::Untabify,
            "\tX",
            true,
            Some((0, 1)),
            "buffer is read-only",
        ),
    ] {
        let mut session = configured(source, 4, readonly);
        if let Some((start, end)) = region {
            select(&mut session, start, end);
        }
        let before = snapshot(&session);
        let view = session.view().clone();
        named(&mut session, operation.name());
        assert_eq!(session.echo(), message, "{name}");
        assert_eq!(snapshot(&session), before, "{name}");
        assert_eq!(*session.view(), view, "{name}");
    }
}

// Disjoint runs publish one history group. Undo from another shared view
// restores the initiating endpoint and clean identity; redo restores conversion.
#[test]
fn compound_history() {
    for operation in [Whitespace::Tabify, Whitespace::Untabify] {
        for backward in [false, true] {
            history_case(operation, backward);
        }
    }
}

fn history_case(operation: Whitespace, backward: bool) {
    let (source, converted) = match operation {
        Whitespace::Tabify => ("x   \r\n    end", "x\t\r\n\tend"),
        Whitespace::Untabify => ("x\t\r\n\tend", "x   \r\n    end"),
    };
    let mut session = configured(source, 4, false);
    let first = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(rev::command::Motion::BufferEnd));
    let peer_cell = session.view().cell(session.buffer().unwrap()).unwrap();
    session.select_view(first).unwrap();
    select(&mut session, 0, source.chars().count());
    if backward {
        session.execute(Command::Region(RegionCommand::Exchange));
    }
    let before = snapshot(&session);
    let initial_point = session.view().point.position;
    named(&mut session, operation.name());
    assert_eq!(text(&session), converted);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    assert_runs(session.buffer().unwrap());
    assert_eq!(
        session
            .windows()
            .get(peer)
            .unwrap()
            .cell(session.buffer().unwrap())
            .unwrap(),
        peer_cell
    );
    let final_point = session.view().point.position;
    assert_eq!(
        final_point.0,
        if backward {
            0
        } else {
            converted.chars().count()
        }
    );
    session.select_view(peer).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(bytes(session.buffer().unwrap()), before.bytes);
    assert_eq!(session.buffer().unwrap().state(), before.state);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.buffer().unwrap().mark(), before.mark);
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        initial_point
    );
    assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), converted);
    assert_eq!(
        session.windows().get(first).unwrap().point.position,
        final_point
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    assert!(
        session
            .windows()
            .iter()
            .all(|(_, view)| view.cell(session.buffer().unwrap()).is_ok())
    );
}

fn assert_runs(buffer: &Buffer) {
    let changes = buffer.history().back().unwrap().changes();
    assert_eq!(changes.len(), 2);
    for change in changes {
        assert!(
            change
                .removed()
                .chars()
                .all(|character| matches!(character, ' ' | '\t'))
        );
        assert!(
            change
                .inserted()
                .chars()
                .all(|character| matches!(character, ' ' | '\t'))
        );
    }
}

// Real visit/save ownership preserves BOM, mixed LF/CRLF and final-newline
// presence. Undo after save correctly returns to a modified earlier text state.
#[test]
fn source_fidelity() {
    for (source, converted) in [
        ("\u{feff}A\t\r\n\tZ", "\u{feff}A   \r\n    Z"),
        ("\u{feff}A\t\n\t界\r\n", "\u{feff}A   \n    界\r\n"),
        ("A\t\r\n\tZ\n", "A   \r\n    Z\n"),
        ("\u{feff}\t\u{301}X\r\n", "\u{feff}     \u{301}X\r\n"),
    ] {
        let fixture = crate::config_fixture::Fixture::new();
        let path = fixture.write("source.txt", source);
        let mut session = super::editor("");
        session.load(&path).unwrap();
        let count = session.buffer().unwrap().len_chars();
        select(&mut session, 0, count);
        named(&mut session, "untabify");
        assert_eq!(bytes(session.buffer().unwrap()), converted.as_bytes());
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        session.execute(Command::SaveBuffer);
        assert_eq!(std::fs::read(&path).unwrap(), converted.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Undo));
        assert_eq!(bytes(session.buffer().unwrap()), source.as_bytes());
        assert!(session.buffer().unwrap().is_modified());
        assert!(
            Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(session.view().point.position)
                .unwrap()
        );
    }
}

// Canceling M-x before invocation preserves text/history and the complete view;
// C-g still performs ordinary region deactivation while retaining its mark.
#[test]
fn canceled_command() {
    let mut session = configured("    X", 4, false);
    select(&mut session, 0, 4);
    let before = snapshot(&session);
    let view = session.view().clone();
    session.execute(Command::ExecuteExtended);
    session.handle(super::Event::Paste("tabify".into()));
    session.handle(super::Event::Key(super::KeyEvent::new(
        super::KeyCode::Char('g'),
        super::KeyModifiers::CONTROL,
    )));
    assert_eq!(bytes(session.buffer().unwrap()), before.bytes);
    assert_eq!(session.buffer().unwrap().stamp(), before.stamp);
    assert_eq!(session.buffer().unwrap().state(), before.state);
    assert_eq!(session.buffer().unwrap().history().len(), before.history);
    assert_eq!(session.buffer().unwrap().mark(), before.mark);
    assert!(!session.buffer().unwrap().mark_active());
    assert_eq!(*session.view(), view);
}

// The conversion owns exactly the selected buffer. Switching afterward exposes
// unchanged bytes, revision and saved/history state in an unrelated live buffer.
#[test]
fn other_buffer() {
    let mut buffers = rev::buffer::Buffers::new().unwrap();
    let current = buffers.create("    X").unwrap();
    let other = buffers.create("\tother").unwrap();
    let other_buffer = buffers.get(other).unwrap();
    let name = other_buffer.name().to_owned();
    let stamp = other_buffer.stamp();
    let state = other_buffer.state();
    let view = rev::view::View::new(buffers.get(current).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        rev::view::Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    select(&mut session, 0, 4);
    named(&mut session, "tabify");
    assert_eq!(text(&session), "\tX");
    session.execute(Command::Buffer(rev::command::BufferCommand::Switch));
    session.handle(super::Event::Paste(name));
    session.handle(super::Event::Key(super::KeyEvent::new(
        super::KeyCode::Enter,
        super::KeyModifiers::NONE,
    )));
    assert_eq!(text(&session), "\tother");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().state(), state);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}
