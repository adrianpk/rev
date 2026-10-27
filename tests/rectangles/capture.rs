use rev::config::prepare;

use super::*;
use crate::config_fixture::Fixture;

fn configured(width: usize, source: &str, a: Cell, b: Cell, readonly: bool) -> Session {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_cell(a).unwrap();
    let end = coordinates.from_cell(b).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: start,
        })
        .unwrap();
    buffer.set_readonly(readonly);
    let mut view = View::new(buffer);
    view.point.position = end;
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

// Copy uses the source policy at each admitted representative width. Short rows
// gain no right padding, CRLF remains source-only, and touched wide/combining/ZWJ
// clusters remain complete even when the right edge cuts through their cells.
#[test]
fn source_policy() {
    let anchor = "0".repeat(40);
    let source = format!("{anchor}\r\na\t界e\u{301}👩\u{200d}💻Z\r\nx\r\nzz\r\n\r\n{anchor}");
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let mut session = configured(width, &source, cell(0, 1), cell(5, stop + 4), true);
        let before = (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state(),
            session.view().point,
            session.buffer().unwrap().mark(),
        );
        copy(&mut session);
        let edge = "0".repeat(stop + 3);
        assert_eq!(
            session.rectangle().unwrap().rows(),
            [
                edge.clone(),
                format!("{}界e\u{301}👩\u{200d}💻", " ".repeat(stop - 1)),
                String::new(),
                "z".into(),
                String::new(),
                edge
            ]
        );
        assert_eq!(session.rectangle().unwrap().width(), stop + 3);
        assert_eq!(session.buffer().unwrap().policy().tab_width, width);
        assert_eq!(text(&session), source);
        assert_eq!(
            (
                session.buffer().unwrap().stamp(),
                session.buffer().unwrap().state(),
                session.view().point,
                session.buffer().unwrap().mark()
            ),
            before
        );
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(session.buffer().unwrap().newline(), "\r\n");
        let frame = session.frame().unwrap();
        assert_eq!(
            frame.rows()[1].selection(),
            Some(7..u16::try_from(stop + 11).unwrap())
        );
        assert!(frame.rows()[2].selection().is_none());
        assert!(frame.rows()[4].selection().is_none());
    }
}

// These original golden fixtures encode the accepted width-four observation:
// copy retains spaces and leaves a-tab-b/c-tab-d untouched; kill retains tabs
// and removes both rows in one publication. LF/CRLF separators survive replay.
#[test]
fn reference_rows() {
    for (separator, command) in [
        ("\n", RectangleCommand::Copy),
        ("\n", RectangleCommand::Kill),
        ("\r\n", RectangleCommand::Copy),
        ("\r\n", RectangleCommand::Kill),
    ] {
        reference_case(separator, command);
    }
}

fn reference_case(separator: &str, command: RectangleCommand) {
    use rev::command::History;

    let source = format!("a\tb{separator}c\td");
    let mut session = configured(4, &source, cell(0, 0), cell(1, 5), false);
    let before = session.view().point;
    let mark = session.buffer().unwrap().mark();
    let kill = command == RectangleCommand::Kill;
    session.execute(Command::Rectangle(command));
    let rows = if kill {
        ["a\tb", "c\td"]
    } else {
        ["a   b", "c   d"]
    };
    assert_eq!(session.rectangle().unwrap().rows(), rows);
    assert_eq!(session.rectangle().unwrap().width(), 5);
    assert_eq!(text(&session), if kill { separator } else { &source });
    assert_eq!(session.buffer().unwrap().history().len(), usize::from(kill));
    assert_eq!(
        session.buffer().unwrap().stamp().revision,
        before.stamp.revision + u64::from(kill)
    );
    assert_eq!(session.kill_ring().len(), 0);
    if kill {
        assert_eq!(
            session
                .buffer()
                .unwrap()
                .history()
                .back()
                .unwrap()
                .changes()
                .len(),
            2
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        assert_eq!(session.view().point.position, before.position);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), separator);
        assert_eq!(session.rectangle().unwrap().rows(), rows);
    } else {
        assert_eq!(session.view().point, before);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Kill uses the same configured source geometry as copy, but keeps complete
// tabs in retained rows. Short rows and whole Unicode expansion preserve their
// source bytes and all changed rows undo as one canonical history operation.
#[test]
fn policy_kill() {
    use rev::command::History;

    let anchor = "0".repeat(40);
    let source = format!("{anchor}\r\na\t界e\u{301}👩\u{200d}💻Z\r\nx\r\nzz\r\n\r\n{anchor}");
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let mut session = configured(width, &source, cell(0, 1), cell(5, stop + 4), false);
        let before = session.view().point;
        session.execute(Command::Rectangle(RectangleCommand::Kill));
        assert_eq!(session.echo(), "Rectangle killed");
        let edge = "0".repeat(stop + 3);
        assert_eq!(
            session.rectangle().unwrap().rows(),
            [
                edge.clone(),
                "\t界e\u{301}👩\u{200d}💻".into(),
                String::new(),
                "z".into(),
                String::new(),
                edge
            ]
        );
        let remaining = "0".repeat(37 - stop);
        assert_eq!(
            text(&session),
            format!("{remaining}\r\naZ\r\nx\r\nz\r\n\r\n{remaining}")
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(
            session
                .buffer()
                .unwrap()
                .history()
                .back()
                .unwrap()
                .changes()
                .len(),
            4
        );
        assert_eq!(
            session.buffer().unwrap().stamp().revision,
            before.stamp.revision + 1
        );
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            cell(0, 1)
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        assert_eq!(session.view().point.position, before.position);
        assert!(!session.buffer().unwrap().is_modified());
    }
}
