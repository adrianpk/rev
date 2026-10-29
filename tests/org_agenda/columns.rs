use super::*;
const SOURCE: &str = "#+COLUMNS: %ITEM %OWNER %EFFORT{:} %POINTS{+} %CLOCKSUM %CLOCKOPEN %CLOCKUNRESOLVED\n* Project\n:PROPERTIES:\n:OWNER: Team\n:EFFORT: 9h\n:POINTS: 99\n:END:\n:LOGBOOK:\nCLOCK: [2024-03-01 Fri 08:00]--[2024-03-01 Fri 08:30] => 0:30\n:END:\n** Build\n:PROPERTIES:\n:EFFORT: 1h\n:POINTS: 2\n:END:\nCLOCK: [2024-03-01 Fri 09:00]--[2024-03-01 Fri 09:45] => 0:45\n** Check\n:PROPERTIES:\n:EFFORT: 30min\n:POINTS: 3\n:END:\nCLOCK: [2024-03-01 Fri 10:00]--[2024-03-01 Fri 10:15] => 0:15\nCLOCK: [2024-03-01 Fri 11:00]\nCLOCK: broken\n";
// Source columns project the actual canonical hierarchy and interval records.
// Summed properties ignore a parent's own estimate; clocks include its interval.
#[test]
fn column_projection() {
    let (fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", SOURCE);
    let stamp = session.buffer().unwrap().stamp();
    let view = session.view().clone();
    action(&mut session, OrgCommand::Columns);
    let result = session
        .query_agenda()
        .unwrap_or_else(|| panic!("{}", session.echo()));
    assert!(!result.complete());
    assert!(
        result.issues[0]
            .message
            .contains("open 1; unresolved 1; overlaps 0")
    );
    assert_eq!(result.rows.len(), 3);
    assert_eq!(
        result.sources[0].version,
        rev::org::agenda::SourceVersion::Buffer(stamp)
    );
    let root = &result.rows[0];
    assert_eq!(root.byte, SOURCE.find("* Project").unwrap());
    for (key, expected) in [
        ("EFFORT", "1:30"),
        ("POINTS", "5"),
        ("CLOCKSUM", "1:30"),
        ("CLOCKOPEN", "1"),
        ("CLOCKUNRESOLVED", "1"),
    ] {
        assert_eq!(
            root.fields.get(key).map(String::as_str),
            Some(expected),
            "{key}"
        );
    }
    assert_eq!(text(&session), SOURCE);
    assert_eq!(session.view(), &view);
    assert!(session.buffer().unwrap().history().is_empty());
    session.execute(Command::Org(OrgCommand::AgendaColumns));
    assert!(session.echo().contains("source COLUMNS declaration"));
    assert_eq!(
        session.query_agenda().unwrap().rows[0].fields["POINTS"],
        "5"
    );
    session.handle(Event::Paste("unwanted".into()));
    assert_eq!(text(&session), SOURCE);
    session.handle(key(KeyCode::Char('g')));
    settle(&mut session);
    assert_eq!(
        session.query_agenda().unwrap().rows[0].fields["EFFORT"],
        "1:30"
    );
    session.handle(key(KeyCode::Enter));
    settle(&mut session);
    assert_eq!(
        session.view().point.position.0,
        SOURCE.find("* Project").unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
        SOURCE
    );
    assert!(session.buffer().unwrap().history().is_empty());
}
// An explicit native columnview update replaces its recognized body only and
// creates one ordinary undo unit; repeated identical updates add no history.
#[test]
fn column_dynamic_update() {
    let source = format!(
        "#+BEGIN: columnview :id global :format \"%ITEM %EFFORT{{:}} %CLOCKSUM\"\nold report\n#+END:\n{SOURCE}"
    );
    let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", &source);
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    let expected = source.replacen("old report\n", "| ITEM | EFFORT | CLOCKSUM |\n|---+---+---|\n| Project | 1:30 | 1:30 |\n| Build | 1:00 | 0:45 |\n| Check | 0:30 | 0:15 |\n# Clock records: open 1; unresolved 1; overlaps 0\n", 1);
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(rev::command::History::Redo));
    assert_eq!(text(&session), expected);
}
