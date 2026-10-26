use super::*;
use rev::command::History;

fn begin(session: &mut Session, needle: &str, replacement: &str) {
    session.execute(Command::QueryReplace);
    query(session, needle);
    key(session, KeyCode::Enter, KeyModifiers::NONE);
    query(session, replacement);
    key(session, KeyCode::Enter, KeyModifiers::NONE);
}

fn answer(session: &mut Session, value: char) {
    key(session, KeyCode::Char(value), KeyModifiers::NONE);
}

// The real query/argument/decision flow replaces only a confirmed occurrence;
// skip and quit retain source bytes, and undo reverses that one accepted change.
#[test]
fn confirmed_flow() {
    let mut session = editor("cat cat cat", 0);
    let stamp = session.buffer().unwrap().stamp();
    begin(&mut session, "cat", "界");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.frame().unwrap().rows()[0].search(), Some(0..3));
    answer(&mut session, 'y');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    answer(&mut session, 'n');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    answer(&mut session, 'q');
    assert_eq!(text(&session), "界 cat cat");
    assert_eq!(session.echo(), "Quit");
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.search().is_none())
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "cat cat cat");
    assert!(!session.buffer().unwrap().is_modified());
}

// Literal replacement text is never interpreted or searched again. Each answer
// publishes its own transaction, including when inserted text contains the query.
#[test]
fn literal_progress() {
    let mut session = editor("a a", 0);
    begin(&mut session, "a", "a$a");
    answer(&mut session, 'y');
    assert_eq!(text(&session), "a$a a");
    answer(&mut session, 'y');
    assert_eq!(text(&session), "a$a a$a");
    assert_eq!(session.echo(), "Query replace complete: 2 replaced");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a$a a");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a a");
}

// Empty queries reject before replacement input. An absent literal query retires
// both argument stages without moving point or publishing a history transaction.
#[test]
fn empty_absent() {
    let mut session = editor("source", 2);
    let origin = session.view().clone();
    session.execute(Command::QueryReplace);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(session.echo(), "Query must not be empty");
    assert_eq!(session.view(), &origin);
    begin(&mut session, "absent", "replacement");
    assert_eq!(session.echo(), "Query replace complete: 0 replaced");
    assert_eq!(session.view(), &origin);
    assert_eq!(text(&session), "source");
    assert!(session.buffer().unwrap().history().is_empty());
}
