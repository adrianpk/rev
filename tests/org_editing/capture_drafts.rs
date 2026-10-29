use super::properties::argument;
use super::timestamps::clock;
use super::*;
use rev::command::{History, Motion, OrgCommand as C, RegionCommand};

fn start(session: &mut Session, key: &str) {
    session.execute(Command::Org(C::Capture));
    argument(session, key);
    settle(session);
}
fn abort(session: &mut Session) {
    session.handle(control('c'));
    session.handle(control('k'));
    settle(session);
}
// Normal text editing can start capture without visiting its destination. The
// transient draft has ordinary editing/history and abort restores the origin.
#[test]
fn native_draft() {
    let (fixture, session) = editor(
        "org.capture.default_file='unwritten.org'",
        "origin.txt",
        "界e\u{301}\r\nlast",
    );
    let mut session = session.with_org_clock(clock);
    session.execute(Command::Motion(Motion::ForwardChar));
    let home = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.handle(control('c'));
    session.handle(key(KeyCode::Char('c')));
    argument(&mut session, "");
    settle(&mut session);
    let draft = session
        .capture_draft()
        .unwrap_or_else(|| panic!("{}", session.echo()));
    let draft_id = draft.id();
    assert!(draft.is_transient());
    assert_eq!(draft.mode(), "Capture");
    assert!(!draft.is_org());
    assert!(draft.file().is_none());
    assert!(text(&session).starts_with("* TODO \n  ["));
    assert_eq!(session.view().point.position.0, 7);
    assert!(session.capture_target().unwrap().target.missing.len() == 1);
    assert!(!fixture.0.join("unwritten.org").exists());
    session.handle(Event::Paste("Task 界".into()));
    let changed = text(&session);
    assert!(changed.starts_with("* TODO Task 界"));
    session.execute(Command::History(History::Undo));
    assert!(text(&session).starts_with("* TODO \n"));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), changed);
    for command in [
        Command::SaveBuffer,
        Command::WriteFile,
        Command::FindFile,
        Command::Mode(Mode::Org),
        Command::Quit,
    ] {
        session.execute(command);
        assert!(session.echo().contains("draft retained"));
        assert_eq!(session.buffer().unwrap().id(), draft_id);
        assert_eq!(text(&session), changed);
    }
    assert!(session.load(&fixture.0.join("unwritten.org")).is_err());
    abort(&mut session);
    assert!(session.capture_draft().is_none());
    assert_eq!(session.view().point, home.point);
    assert_eq!(session.view().top, home.top);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), "界e\u{301}\r\nlast");
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!fixture.0.join("unwritten.org").exists());
    assert_eq!(session.remaining_input(Instant::now()), None);
}
// Explicit destination entry follows the origin directory, creates no parents,
// and configured prompts preserve actual region data as literal draft text.
#[test]
fn selection_prompts() {
    let config = r#"[org.capture]
templates=[{key='n',description='Note',kind='plain',target={type='file'},template='界%?%i %^{Topic|default|one|two} %n %x'}]
[keys.org_capture]
'M-j'='newline'
"#;
    let (fixture, session) = editor(config, "origin.txt", "%(opaque)\r\n界");
    let mut session = session.with_org_clock(clock);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::BufferEnd));
    let home = session.view().point;
    start(&mut session, "n");
    assert!(session.echo().contains("literal local file"));
    argument(&mut session, "missing/destination.org");
    settle(&mut session);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Topic: default"))
    );
    session.handle(key(KeyCode::Tab));
    assert!(session.echo().contains("one | two"));
    argument(&mut session, "");
    argument(&mut session, "Ada");
    argument(&mut session, "clipboard");
    settle(&mut session);
    assert_eq!(text(&session), "界%(opaque)\r\n界界 default Ada clipboard");
    assert_eq!(session.view().point.position.0, 1);
    session.handle(meta('j'));
    assert!(text(&session).starts_with("界\r\n%(opaque)"));
    abort(&mut session);
    assert_eq!(session.view().point, home);
    assert_eq!(text(&session), "%(opaque)\r\n界");
    assert!(!fixture.0.join("missing").exists());
}
// Another view edits the canonical origin while a draft is retained. Ordinary
// checked publication maps the saved point and viewport before returning home.
#[test]
fn mapped_origin() {
    let (_fixture, mut session) = editor(
        "org.capture.default_file='target.org'",
        "origin.txt",
        "first\nsecond\nthird\n",
    );
    session.execute(Command::Split(Split::Right));
    let owner = session.windows().selected_id();
    let peer = session
        .windows()
        .iter()
        .find(|(id, _)| *id != owner)
        .unwrap()
        .0;
    session.execute(Command::Motion(Motion::BufferEnd));
    let before = session.view().point.position.0;
    start(&mut session, "t");
    let draft = session.buffer().unwrap().id();
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.handle(Event::Paste("prefix\n".into()));
    session.execute(Command::Window(WindowCommand::Delete));
    assert!(session.echo().contains("draft retained"));
    session.select_view(owner).unwrap();
    assert_eq!(session.buffer().unwrap().id(), draft);
    abort(&mut session);
    assert_eq!(session.windows().selected_id(), owner);
    assert_eq!(session.view().point.position.0, before + 7);
    assert_eq!(text(&session), "prefix\nfirst\nsecond\nthird\n");
    assert_eq!(
        session.windows().get(peer).unwrap().point.stamp,
        session.buffer().unwrap().stamp()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}
