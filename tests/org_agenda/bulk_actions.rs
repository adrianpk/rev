use super::*;
use rev::org::agenda::ActionStatus;
fn open(s: &mut Session) {
    action(s, OrgCommand::TodoList);
    argument(s, "");
    settle(s);
}
fn bulk(s: &mut Session, operation: &str, value: &str) {
    s.handle(key(KeyCode::Char('B')));
    argument(s, operation);
    argument(s, value);
    settle(s);
}
fn visible(s: &mut Session, needle: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(needle))
}
// Explicit marks drive one bulk operation. Later headings use mapped anchors
// after earlier insertions in the same source, and results identify every target.
#[test]
fn bulk_action_anchors() {
    let (f, mut s) = editor(
        "org.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO First\n* TODO Second\n",
    );
    settle(&mut s);
    let origin = s.view().clone();
    open(&mut s);
    s.handle(key(KeyCode::Char('m')));
    s.handle(key(KeyCode::Char('m')));
    assert!(visible(&mut s, "* TODO"));
    bulk(&mut s, "effort", "1:30");
    assert_eq!(s.agenda_action_results().len(), 2);
    assert!(
        s.agenda_action_results()
            .iter()
            .all(|r| r.status == ActionStatus::Changed),
        "{:?}",
        s.agenda_action_results()
    );
    assert_eq!(text(&s).matches(":EFFORT: 1:30").count(), 2);
    assert_eq!(s.buffer().unwrap().history().len(), 2);
    assert_eq!(s.view().point.position, origin.point.position);
    assert!(visible(&mut s, "Agenda actions: 2 targets"));
    assert!(visible(&mut s, "Changed"));
    assert_eq!(
        std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
        "* TODO First\n* TODO Second\n"
    );
    s.handle(key(KeyCode::Char('q')));
    assert!(s.query_agenda().is_some());
    action(&mut s, OrgCommand::AgendaUndo);
    settle(&mut s);
    assert_eq!(text(&s).matches(":EFFORT: 1:30").count(), 1);
    action(&mut s, OrgCommand::AgendaUndo);
    assert!(s.echo().contains("changed") || s.echo().contains("undo"));
}
// Source-file scopes and readonly admission remain per target. A failure in one
// source does not hide the successful, unsaved canonical change in another file.
#[test]
fn bulk_action_sources() {
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", "org.agenda.sources=['one.org','locked.org']"));
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("one.org", "* TODO One\n")).unwrap();
    let locked = buffers
        .visit(&f.write("locked.org", "* TODO Locked\n"))
        .unwrap();
    buffers.get_mut(locked).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    bulk(&mut s, "priority", "A");
    assert_eq!(s.agenda_action_results().len(), 2);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert_eq!(s.agenda_action_results()[1].status, ActionStatus::Rejected);
    assert!(s.agenda_action_results()[1].message.contains("read-only"));
    assert!(text(&s).contains("[#A]"));
    assert_eq!(
        std::fs::read_to_string(f.0.join("one.org")).unwrap(),
        "* TODO One\n"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("locked.org")).unwrap(),
        "* TODO Locked\n"
    );
    assert!(visible(&mut s, "Rejected"));
}
// A required source note remains a native prompt. Cancel reports every pending
// target and preserves the source/history even after the bulk value was entered.
#[test]
fn bulk_action_cancel() {
    let source = "#+TODO: TODO(t) | DONE(d@)\n* TODO One\n* TODO Two\n";
    let (_f, session) = editor("org.agenda.sources=['tasks.org']", "tasks.org", source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    bulk(&mut s, "todo", "DONE");
    assert!(s.echo().contains("note"), "{}", s.echo());
    s.handle(control('g'));
    settle(&mut s);
    assert_eq!(s.agenda_action_results().len(), 2);
    assert!(
        s.agenda_action_results()
            .iter()
            .all(|r| r.status == ActionStatus::Cancelled)
    );
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(s.remaining_input(Instant::now()), None);
}

// Bulk addition preserves each target's local tags and inherited owner. It
// cannot replace the first target's tags onto a later source heading.
#[test]
fn bulk_action_tags() {
    let source = "#+FILETAGS: :file:\n* TODO One :keep:\n* TODO Two :old:\n";
    let (_f, mut s) = editor("org.agenda.sources=['tasks.org']", "tasks.org", source);
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    bulk(&mut s, "+", "new");
    assert!(
        s.agenda_action_results()
            .iter()
            .all(|r| r.status == ActionStatus::Changed)
    );
    assert!(text(&s).contains("* TODO One :keep:new:"));
    assert!(text(&s).contains("* TODO Two :old:new:"));
    assert!(text(&s).starts_with("#+FILETAGS: :file:\n"));
}

// Multiple rows from an unopened source share one checked canonical snapshot.
// Reuse compares actual text as well as revision evidence before any mutation.
#[test]
fn bulk_action_disk() {
    let (f, session) = editor(
        "org.agenda.sources=['other.org']",
        "origin.org",
        "* Origin\n",
    );
    let source = "\u{feff}* TODO One 界\r\n* TODO Two\r\n";
    let path = f.write("other.org", source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    bulk(&mut s, "effort", "0:30");
    assert_eq!(s.agenda_action_results().len(), 2);
    assert!(
        s.agenda_action_results()
            .iter()
            .all(|r| r.status == ActionStatus::Changed),
        "{:?}",
        s.agenda_action_results()
    );
    assert_eq!(text(&s), "* Origin\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    s.handle(key(KeyCode::Char('q')));
    s.handle(key(KeyCode::Char('q')));
    s.load(&path).unwrap();
    settle(&mut s);
    assert_eq!(text(&s).matches(":EFFORT: 0:30\r\n").count(), 2);
    assert_eq!(s.buffer().unwrap().history().len(), 2);
}
