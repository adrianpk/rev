use super::*;
use rev::command::OrgCommand;
const SOURCE: &str = "* Parent\n:PROPERTIES:\n:EFFORT: 3:00\n:END:\nCLOCK: [2024-01-01 Mon 23:00]--[2024-01-02 Tue 01:00] => 2:00\n** Child\nCLOCK: [2024-01-02 Tue 00:30]--[2024-01-02 Tue 01:30] => 1:00\nCLOCK: [2024-01-02 Tue 10:00]\nCLOCK: broken\n* Other\nCLOCK: [2024-01-02 Tue 12:00]--[2024-01-02 Tue 12:15] => 0:15\n";
fn query(session: &mut Session, draft: &str) {
    settle(session);
    visibility::action(session, OrgCommand::ClockDisplay);
    assert!(session.echo().contains("C-g cancels"), "{}", session.echo());
    properties::argument(session, draft);
}
fn feedback(session: &mut Session, expected: &str) {
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains(expected)),
        "missing {expected:?}: {:?}",
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .map(|r| r.text())
            .collect::<Vec<_>>()
    );
}
// Real commands use unsaved canonical source, preserve text/view/history and
// distinguish task/subtree/file totals while clipping midnight-crossing records.
#[test]
fn clock_scopes() {
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'\n", "clocks.org", SOURCE);
    for (name, draft, expected) in [
        (
            "task",
            "task",
            "actual 2:00; effort 3:00; open 0; unresolved 0; overlaps 0",
        ),
        (
            "subtree",
            "subtree",
            "actual 3:00; effort 3:00; open 1; unresolved 1; overlaps 1",
        ),
        (
            "file",
            "file",
            "actual 3:15; effort 3:00; open 1; unresolved 1; overlaps 1",
        ),
        (
            "day",
            "subtree 2024-01-02 2024-01-03",
            "actual 2:00; effort 3:00; open 1; unresolved 1; overlaps 1",
        ),
    ] {
        let view = session.view().clone();
        query(&mut session, draft);
        feedback(&mut session, expected);
        assert_eq!(text(&session), SOURCE, "{name}");
        assert_eq!(session.view(), &view, "{name}");
    }
}
// Cancelled/stale/invalid requests and literal CLOCK text never authorize edits.
#[test]
fn clock_report_boundaries() {
    let source = "* Task\n#+begin_example\nCLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 1:00\n#+end_example\n";
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'\n", "literal.org", source);
    query(&mut session, "file");
    feedback(&mut session, "actual 0:00");
    query(&mut session, "file 2024-02-01 2024-01-01");
    feedback(&mut session, "start before end");
    visibility::action(&mut session, OrgCommand::ClockDisplay);
    session.handle(control('g'));
    assert_eq!(text(&session), source);
}
fn at(session: &mut Session, needle: &str) {
    let row = text(session)
        .lines()
        .position(|s| s.contains(needle))
        .unwrap()
        + 1;
    headings::goto(session, row);
}
// Dynamic reports are opaque observations until explicitly updated; only the
// selected body changes, with exact CRLF/Unicode/unknown blocks and one undo.
#[test]
fn clock_table_publication() {
    let source = "* Task é\r\nCLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 11:00] => 1:00\r\n#+BEGIN: unknown :keep yes\r\nCLOCK: [2024-01-01 Mon 10:00]--[2024-01-01 Mon 12:00] => 2:00\r\n#+END:\r\n#+bEgIn: clocktable :scope file :block untilnow :maxlevel 1\r\nold | report\r\n#+eNd:\r\nlast";
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'", "report.org", source);
    settle(&mut session);
    let stamp = session.buffer().unwrap().stamp();
    session.frame().unwrap();
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    at(&mut session, "#+bEgIn:");
    visibility::action(&mut session, OrgCommand::UpdateDynamicBlock);
    assert!(session.echo().contains("updated"), "{}", session.echo());
    let updated = text(&session);
    assert!(updated.contains("| *Total time* | *1:00* |"), "{updated}");
    assert!(updated.starts_with(source.split("old | report").next().unwrap()));
    assert!(updated.ends_with("#+eNd:\r\nlast"));
    assert!(!updated.replace("\r\n", "").contains('\n'));
    let revision = session.buffer().unwrap().stamp();
    at(&mut session, "#+bEgIn:");
    visibility::action(&mut session, OrgCommand::CtrlCCtrlC);
    assert_eq!(
        session.buffer().unwrap().stamp(),
        revision,
        "idempotent refresh"
    );
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(rev::command::History::Redo));
    assert_eq!(text(&session), updated);
}
// Unknown/malformed blocks and unsupported options retain source and history;
// a plain blank-line insertion preserves the existing absence of a final newline.
#[test]
fn clock_table_rejections() {
    for (name, block) in [
        ("unknown", "#+BEGIN: unknown\nkeep\n#+END:"),
        (
            "unsupported",
            "#+BEGIN: clocktable :formatter arbitrary\nkeep\n#+END:",
        ),
        ("incomplete", "#+BEGIN: clocktable\nkeep"),
        (
            "duplicate",
            "#+BEGIN: clocktable :scope file :scope subtree\nkeep\n#+END:",
        ),
    ] {
        let source = format!("* Task\n{block}");
        let (_fixture, mut session) = editor("[org.time]\nzone='UTC'", "reject.org", &source);
        settle(&mut session);
        at(&mut session, "#+BEGIN:");
        let stamp = session.buffer().unwrap().stamp();
        visibility::action(&mut session, OrgCommand::UpdateDynamicBlock);
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
    }
    let (_fixture, mut session) = editor("[org.time]\nzone='UTC'", "insert.org", "* Task\n");
    settle(&mut session);
    headings::goto(&mut session, 2);
    visibility::action(&mut session, OrgCommand::ClockReport);
    assert!(text(&session).ends_with("#+END:"), "{}", session.echo());
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&session), "* Task\n");
}
