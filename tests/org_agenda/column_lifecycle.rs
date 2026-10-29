use super::*;
use rev::command::{History, Motion, RegionCommand, Split};
fn line(source: &str, needle: &str) -> usize {
    source[..source.find(needle).unwrap()].lines().count() + 1
}
// Inherited COLUMNS belongs to the selected subtree. Deep summaries use direct
// children, while maxlevel hides deep rows only after their values contributed.
#[test]
fn column_scope() {
    let source = "#+COLUMNS: %ITEM %EFFORT{:}\n* Project\n:PROPERTIES:\n:COLUMNS: %ITEM %SCORE{+}\n:SCORE: 99\n:END:\n#+BEGIN: columnview :id local :maxlevel 2\nold\n#+END:\n** First\n:PROPERTIES:\n:SCORE: 2\n:END:\n*** Leaf\n:PROPERTIES:\n:SCORE: 3\n:END:\n** Second\n:PROPERTIES:\n:SCORE: 4\n:END:\n* Outside\n:PROPERTIES:\n:SCORE: 100\n:END:\n";
    let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::Columns);
    let rows = &session.query_agenda().unwrap().rows;
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows.iter()
            .map(|r| r.fields["SCORE"].as_str())
            .collect::<Vec<_>>(),
        ["7", "3", "3", "4"]
    );
    session.handle(key(KeyCode::Char('q')));
    goto(&mut session, line(source, "#+BEGIN:"));
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    let expected = source.replacen(
        "old\n",
        "| ITEM | SCORE |\n|---+---|\n| Project | 7 |\n| First | 3 |\n| Second | 4 |\n",
        1,
    );
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}
// Structural keyword admission ignores declarations inside literal source.
// A real lowercase file declaration still selects actual named property values.
#[test]
fn column_declarations() {
    for (name, declaration, columns) in [
        (
            "literal only",
            "",
            vec!["ITEM", "EFFORT", "CLOCKSUM", "CLOCKOPEN", "CLOCKUNRESOLVED"],
        ),
        ("native", "#+columns: %ITEM %OWNER\n", vec!["ITEM", "OWNER"]),
    ] {
        let source = format!(
            "#+begin_example\n#+COLUMNS: %25ITEM\n#+end_example\n{declaration}* Task\n:PROPERTIES:\n:OWNER: Native\n:END:\n"
        );
        let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", &source);
        action(&mut session, OrgCommand::Columns);
        let result = session
            .query_agenda()
            .unwrap_or_else(|| panic!("{name}: {}", session.echo()));
        assert!(result.complete(), "{name}");
        assert_eq!(
            result.view.as_ref().unwrap().blocks[0].columns,
            columns,
            "{name}"
        );
        if name == "native" {
            assert_eq!(result.rows[0].fields["OWNER"], "Native");
        }
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}
// Numeric and duration totals use independent expected arithmetic. A configured
// hour changes duration interpretation, and the parent value is not double-counted.
#[test]
fn column_units() {
    let source = "#+COLUMNS: %ITEM %EFFORT{:} %SCORE{+}\n* Project\n:PROPERTIES:\n:EFFORT: 9h\n:SCORE: 99\n:END:\n** First\n:PROPERTIES:\n:EFFORT: 1h\n:SCORE: 1.5\n:END:\n** Second\n:PROPERTIES:\n:EFFORT: 30min\n:SCORE: -0.5\n:END:\n";
    let (_fixture, mut session) = editor(
        "org.time.zone='UTC'\norg.duration.units.h=90",
        "notes.org",
        source,
    );
    action(&mut session, OrgCommand::Columns);
    let result = session.query_agenda().unwrap();
    assert_eq!(result.rows[0].fields["EFFORT"], "2:00");
    assert_eq!(result.rows[0].fields["SCORE"], "1");
    assert_eq!(result.rows[1].fields["EFFORT"], "1:30");
    assert_eq!(result.rows[2].fields["EFFORT"], "0:30");
    assert_eq!(text(&session), source);
}
// Physical C-c C-c updates the recognized body in the real canonical file,
// retaining BOM/CRLF/EOF, peer points and an inactive mark through save/undo/redo.
#[test]
fn column_file_history() {
    let source = "\u{feff}#+BEGIN: columnview :id global :format \"%ITEM %EFFORT{:}\"\r\nold\r\n#+END:\r\n* Project 界\r\n:PROPERTIES:\r\n:EFFORT: 1h\r\n:END:\r\nEOF";
    let (fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", source);
    let original = text(&session);
    let owner = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    settle(&mut session);
    let peer = session
        .windows()
        .iter()
        .find(|(id, _)| *id != owner)
        .unwrap()
        .0;
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::KeyboardQuit);
    session.select_view(owner).unwrap();
    session.handle(control('c'));
    session.handle(control('c'));
    settle(&mut session);
    let expected = original.replacen(
        "old\r\n",
        "| ITEM | EFFORT |\r\n|---+---|\r\n| Project 界 | 1:00 |\r\n",
        1,
    );
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected.chars().count()
    );
    assert_eq!(
        session.buffer().unwrap().mark().unwrap().0,
        expected.chars().count()
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
        source
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
        format!("\u{feff}{expected}")
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), original);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        original.chars().count()
    );
    assert_eq!(
        session.buffer().unwrap().mark().unwrap().0,
        original.chars().count()
    );
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}
// Native insertion and property edits do not trigger an implicit report update.
// Explicit refresh then changes exactly one report body/history transaction.
#[test]
fn column_explicit_refresh() {
    let source = "* Task\n\n";
    let (_fixture, mut session) = editor("org.time.zone='UTC'", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::ColumnsInsertDblock);
    let inserted = "* Task\n#+BEGIN: columnview :id global\n| ITEM | EFFORT | CLOCKSUM | CLOCKOPEN | CLOCKUNRESOLVED |\n|---+---+---+---+---|\n| Task | 0:00 | 0:00 | 0 | 0 |\n#+END:\n";
    assert_eq!(text(&session), inserted, "{}", session.echo());
    goto(&mut session, 1);
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "30min");
    settle(&mut session);
    let before = text(&session);
    assert!(before.contains(":EFFORT: 30min"), "{}", session.echo());
    assert!(before.contains("| Task | 0:00 |"));
    session.execute(Command::SaveBuffer);
    settle(&mut session);
    assert_eq!(text(&session), before);
    goto(&mut session, line(&before, "#+BEGIN:"));
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    let expected = before.replacen("| Task | 0:00 |", "| Task | 0:30 |", 1);
    assert_eq!(text(&session), expected);
    assert_eq!(session.buffer().unwrap().history().len(), 3);
    action(&mut session, OrgCommand::UpdateDynamicBlock);
    assert_eq!(session.buffer().unwrap().history().len(), 3);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), before);
}
