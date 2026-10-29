use super::*;
use rev::{command::History, org::agenda::ActionStatus};
fn open(s: &mut Session) {
    action(s, OrgCommand::TodoList);
    argument(s, "");
    settle(s);
}
fn visible(s: &mut Session, text: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(text))
}
// A checked unopened target becomes an ordinary dirty canonical buffer. Remote
// effort editing preserves the origin view, BOM/CRLF, disk bytes and source undo.
#[test]
fn source_action_disk() {
    let (f, session) = editor(
        "org.agenda.sources=['other.org']",
        "origin.org",
        "* Origin\n",
    );
    let source = "\u{feff}* TODO Target\r\nBody\r\n";
    let path = f.write("other.org", source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    let origin = s.view().clone();
    open(&mut s);
    action(&mut s, OrgCommand::SetEffort);
    assert!(s.frame().unwrap().cursor().is_some());
    argument(&mut s, "1:30");
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert_eq!(s.view(), &origin);
    assert_eq!(text(&s), "* Origin\n");
    assert!(s.query_agenda().is_some());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
    s.handle(key(KeyCode::Char('q')));
    s.load(&path).unwrap();
    settle(&mut s);
    assert!(text(&s).contains(":EFFORT: 1:30\r\n"));
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "* TODO Target\r\nBody\r\n");
    s.execute(Command::History(History::Redo));
    assert!(text(&s).contains(":EFFORT: 1:30\r\n"));
}
// Cached buffer evidence cannot authorize an edit after unrelated source text
// changes. Refresh and explicit selection admit the new canonical revision.
#[test]
fn source_action_stale() {
    let (_f, mut s) = editor(
        "org.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO Target\n",
    );
    settle(&mut s);
    open(&mut s);
    s.replace(0..0, "# user\n").unwrap();
    settle(&mut s);
    let before = text(&s);
    let history = s.buffer().unwrap().history().len();
    action(&mut s, OrgCommand::Priority);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Rejected);
    assert!(s.echo().contains("revision changed"), "{}", s.echo());
    assert_eq!(text(&s), before);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    action(&mut s, OrgCommand::Priority);
    argument(&mut s, "A");
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert!(text(&s).contains("[#A]"));
    assert!(visible(&mut s, "stale: g refresh"));
}
// A task note is still owned by the existing task transition. Cancelling the
// note changes neither text/history nor the agenda's original point and scroll.
#[test]
fn source_action_cancel() {
    let source = "#+TODO: TODO(t) | DONE(d@)\n* TODO Target\n";
    let (_f, session) = editor("org.agenda.sources=['tasks.org']", "tasks.org", source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    let origin = s.view().clone();
    open(&mut s);
    action(&mut s, OrgCommand::SetTodo);
    argument(&mut s, "DONE");
    settle(&mut s);
    assert!(s.echo().contains("note"), "{}", s.echo());
    s.handle(control('g'));
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Cancelled);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(s.view(), &origin);
}

// Source read-only admission is rechecked before any prompt or canonical edit;
// the agenda's own projection never grants write permission to its source.
#[test]
fn source_action_readonly() {
    let f = Fixture::new();
    let source = "* TODO Locked\n";
    let startup = prepare(&f.write("config.toml", "org.agenda.sources=['tasks.org']"));
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("tasks.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
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
    action(&mut s, OrgCommand::SetTags);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Rejected);
    assert!(s.echo().contains("read-only"));
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Planning and local-tag handlers edit the same admitted heading while the
// ordinary query remains a stale observation until an explicit refresh.
#[test]
fn source_action_metadata() {
    let (_f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO Target\n",
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s);
    for (command, draft, expected) in [
        (
            OrgCommand::Schedule,
            "2024-03-02",
            "SCHEDULED: <2024-03-02 Sat>",
        ),
        (
            OrgCommand::Deadline,
            "2024-03-03",
            "DEADLINE: <2024-03-03 Sun>",
        ),
        (OrgCommand::SetTags, ":work:", ":work:"),
    ] {
        action(&mut s, command);
        argument(&mut s, draft);
        settle(&mut s);
        assert_eq!(
            s.agenda_action_results()[0].status,
            ActionStatus::Changed,
            "{command:?}: {}",
            s.echo()
        );
        assert!(text(&s).contains(expected), "{}", text(&s));
        s.handle(key(KeyCode::Char('g')));
        settle(&mut s);
    }
    assert_eq!(s.buffer().unwrap().history().len(), 3);
}
// Closing a remote clock must prove the selected heading owns that clock;
// selecting another task cannot stop a session-global active interval.
#[test]
fn source_action_clock() {
    let (_f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO One\n* TODO Two\n",
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s);
    action(&mut s, OrgCommand::ClockIn);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    s.handle(key(KeyCode::Down));
    action(&mut s, OrgCommand::ClockOut);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Rejected);
    assert!(s.echo().contains("does not own"), "{}", s.echo());
    s.handle(key(KeyCode::Up));
    action(&mut s, OrgCommand::ClockOut);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert!(text(&s).contains(" =>  0:00"));
}
// The saved origin participates in the same transaction maps as live peers.
// Inserting priority text before the origin maps its point rather than restoring
// an obsolete scalar coordinate or moving another pane to the agenda target.
#[test]
fn source_action_shared_views() {
    use rev::command::Split;
    let (_f, mut s) = editor(
        "org.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO First\nbody\ntail\n",
    );
    settle(&mut s);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = *s.windows().order().iter().find(|v| **v != owner).unwrap();
    let peer_point = s.windows().get(peer).unwrap().point;
    goto(&mut s, 3);
    let origin = s.view().point.position;
    open(&mut s);
    action(&mut s, OrgCommand::PriorityUp);
    settle(&mut s);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert_eq!(s.view().point.position.0, origin.0 + 5);
    assert_eq!(
        s.windows().get(peer).unwrap().point.position,
        peer_point.position
    );
    assert!(s.query_agenda().is_some());
}
