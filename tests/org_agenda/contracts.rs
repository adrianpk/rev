use super::*;
use rev::org::{
    Cancellation, Limits, StructureScan,
    agenda::{ClockQuery, ClockRequest, ClockScan},
    time::Zone,
};
fn scan(buffer: &rev::buffer::Buffer, query: &str) -> Result<ClockScan, String> {
    let cancel = Cancellation::default();
    let structure = StructureScan::new(buffer, Limits::default())
        .unwrap()
        .run(&cancel);
    ClockScan::new(
        buffer,
        &structure,
        ClockRequest {
            point: rev::coordinates::Scalar(0),
            query: ClockQuery::parse(query)?,
            policy: &rev::config::Settings::default().org,
        },
        &cancel,
    )
}
// Derived rows keep exact canonical identity, actual interval totals, independent
// effort and incomplete/overlap diagnostics; they never become edit authority.
#[test]
fn source_provenance() {
    let mut buffers = Buffers::new().unwrap();
    let id=buffers.create("* Root | é\r\n:PROPERTIES:\r\n:EFFORT: 3:00\r\n:END:\r\nCLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 1:00\r\n** Child\r\nCLOCK: [2024-01-01 Mon 10:30]--[2024-01-01 Mon 11:30] => 1:00\r\nCLOCK: [2024-01-01 Mon 12:00]\r\nCLOCK: broken\r\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let result = scan(buffer, "file")
        .unwrap()
        .run(&Zone::utc(), &Cancellation::default())
        .unwrap();
    assert_eq!(result.source.stamp, stamp);
    assert_eq!(result.minutes, 120);
    assert_eq!(result.effort.unwrap().minutes(), 180.0);
    assert_eq!((result.open, result.unresolved, result.overlaps), (1, 1, 1));
    assert_eq!(result.rows[1].parent, Some(0));
    assert!(
        buffer
            .slice(&result.rows[1].source)
            .unwrap()
            .to_string()
            .starts_with("** Child")
    );
    let table = result.table(1, "\r\n").unwrap();
    assert!(table.contains("Root \\vert{} é | 2:00 | 3:00"), "{table}");
    assert!(!table.contains("Child"));
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
}
// Snapshot creation rejects stale structure. A later source edit cannot turn
// old derived spans into current source, even when snapshot calculation succeeds.
#[test]
fn stale_observations() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* Task\n").unwrap();
    let cancel = Cancellation::default();
    let structure = StructureScan::new(buffers.get(id).unwrap(), Limits::default())
        .unwrap()
        .run(&cancel);
    let snapshot = scan(buffers.get(id).unwrap(), "file").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "changed\n".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let result = snapshot.run(&Zone::utc(), &cancel).unwrap();
    assert!(buffers.get(id).unwrap().slice(&result.source).is_err());
    let policy = rev::config::Settings::default();
    assert!(
        ClockScan::new(
            buffers.get(id).unwrap(),
            &structure,
            ClockRequest {
                point: rev::coordinates::Scalar(0),
                query: ClockQuery::parse("file").unwrap(),
                policy: &policy.org
            },
            &cancel
        )
        .is_err()
    );
}
// Exact finite source/record boundaries reject whole results, never a plausible
// partial total. Canceled observations cannot publish derived output either.
#[test]
fn report_bounds() {
    let mut buffers = Buffers::new().unwrap();
    for (name, source, admit) in [
        (
            "records admitted",
            format!("* Task\n{}", "CLOCK: broken\n".repeat(4096)),
            true,
        ),
        (
            "records rejected",
            format!("* Task\n{}", "CLOCK: broken\n".repeat(4097)),
            false,
        ),
    ] {
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let result = scan(buffer, "file")
            .unwrap()
            .run(&Zone::utc(), &Cancellation::default());
        assert_eq!(result.is_ok(), admit, "{name}");
        if let Ok(result) = result {
            assert_eq!(result.unresolved, 4096, "{name}");
        }
    }
    let id = buffers.create("* Task\n").unwrap();
    let snapshot = scan(buffers.get(id).unwrap(), "file").unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(snapshot.run(&Zone::utc(), &cancel).is_err());
}
// Read-only display remains usable while source publication rejects before
// time acquisition; canceled/stale workers leave text, history and panes intact.
#[test]
fn report_publication_guards() {
    let source = "* Task\n#+BEGIN: clocktable :block untilnow\nmanual\n#+END:\n";
    let (mut buffers, id) = readonly_source(source);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    settle(&mut session);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    assert!(session.echo().contains("read-only"), "{}", session.echo());
    action(&mut session, OrgCommand::ClockDisplay);
    argument(&mut session, "file");
    settle(&mut session);
    assert!(session.echo().contains("actual 0:00"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'", "guard.org", source);
    settle(&mut session);
    goto(&mut session, 2);
    session.execute(Command::Org(OrgCommand::UpdateDynamicBlock));
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    session.execute(Command::Org(OrgCommand::UpdateDynamicBlock));
    session.execute(Command::Motion(rev::command::Motion::ForwardChar));
    settle(&mut session);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}
// All panes track one canonical report transaction. Undo restores their source
// positions and the complete manual body; later edits invalidate pending output.
#[test]
fn shared_report_history() {
    let source = "* Task\n#+BEGIN: clocktable :block untilnow\nmanual\n#+END:\nTail";
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'", "shared.org", source);
    settle(&mut session);
    goto(&mut session, 5);
    session.execute(Command::Split(rev::command::Split::Right));
    let peer = session.windows().order()[1];
    let before = session.windows().get(peer).unwrap().point;
    goto(&mut session, 2);
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    let updated = text(&session);
    assert!(updated.contains("| *Total time*"), "{}", session.echo());
    let point = session.windows().get(peer).unwrap().point;
    assert_eq!(point.stamp, session.buffer().unwrap().stamp());
    assert_eq!(
        point.position.0,
        updated.split("Tail").next().unwrap().chars().count()
    );
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        before.position
    );
    goto(&mut session, 2);
    settle(&mut session);
    session.execute(Command::Org(OrgCommand::UpdateDynamicBlock));
    session.replace(0..0, "foreign\n").unwrap();
    settle(&mut session);
    assert_eq!(text(&session), format!("foreign\n{source}"));
}

fn readonly_source(source: &str) -> (Buffers, rev::buffer::BufferId) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    buffers
        .get_mut(id)
        .unwrap()
        .select_mode(rev::buffer::Mode::Org)
        .unwrap();
    (buffers, id)
}
// Worst-case table escaping cannot cross the output budget, even though its
// source fits the input budget; oversized source rejects before snapshot copy.
#[test]
fn source_and_output_limits() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create(&format!("* {}\n", "|".repeat(200000)))
        .unwrap();
    let result = scan(buffers.get(id).unwrap(), "file")
        .unwrap()
        .run(&Zone::utc(), &Cancellation::default())
        .unwrap();
    assert!(result.table(2, "\n").unwrap_err().contains("1048576"));
    for (name, length, admit) in [
        ("exact bytes", 262144, true),
        ("excess bytes", 262145, false),
    ] {
        let prefix = "* Task\n";
        let mut source = prefix.to_owned();
        source.push_str(&("# ".to_owned() + &"x".repeat(125) + "\n").repeat(2047));
        source.push_str(&"#".repeat(length - source.len()));
        assert_eq!(source.len(), length);
        let id = buffers.create(&source).unwrap();
        assert_eq!(
            scan(buffers.get(id).unwrap(), "file").is_ok(),
            admit,
            "{name}"
        );
    }
}
