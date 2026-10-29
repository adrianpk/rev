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
// Mark-all has an exact finite boundary and preserves existing explicit marks
// when one extra distinct heading would exceed it. No source edit occurs there.
#[test]
fn action_selection_limit() {
    for (name, count, message) in [
        ("at limit", 256, "256 visible source targets marked"),
        ("over limit", 257, "existing marks retained"),
    ] {
        let source = (0..count)
            .map(|i| format!("* TODO Task {i}\n"))
            .collect::<String>();
        let (_f, mut s) = editor("org.agenda.sources=['tasks.org']", "tasks.org", &source);
        settle(&mut s);
        open(&mut s);
        s.handle(key(KeyCode::Char('m')));
        s.handle(key(KeyCode::Char('*')));
        assert!(s.echo().contains(message), "{name}: {}", s.echo());
        if count == 257 {
            bulk(&mut s, "priority", "A");
            assert_eq!(s.agenda_action_results().len(), 1, "{name}");
            assert!(text(&s).starts_with("* TODO [#A] Task 0\n"));
        } else {
            s.handle(key(KeyCode::Char('B')));
            assert!(s.echo().contains("256 explicit source targets"));
            s.handle(control('g'));
            assert_eq!(text(&s), source);
            assert!(s.buffer().unwrap().history().is_empty());
        }
    }
}
// Custom block duplicates identify one qualified source heading. A section
// header cannot borrow a nearby target, and bulk changes that heading only once.
#[test]
fn action_selection_identity() {
    let (_f, mut s) = editor(
        "org.agenda.sources=['tasks.org']\norg.agenda.views=[{key='d',title='Duplicates',blocks=[{kind='todo'},{kind='todo'}]}]",
        "tasks.org",
        "* TODO One\n",
    );
    settle(&mut s);
    action(&mut s, OrgCommand::AgendaCustom);
    argument(&mut s, "d");
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 2);
    s.handle(key(KeyCode::Up));
    action(&mut s, OrgCommand::PriorityUp);
    assert!(s.echo().contains("not a section"), "{}", s.echo());
    assert!(s.buffer().unwrap().history().is_empty());
    s.handle(key(KeyCode::Char('*')));
    assert!(s.echo().contains("1 visible source targets"));
    bulk(&mut s, "priority", "A");
    assert_eq!(s.agenda_action_results().len(), 1);
    assert_eq!(s.agenda_action_results()[0].status, ActionStatus::Changed);
    assert_eq!(text(&s), "* TODO [#A] One\n");
    assert_eq!(s.buffer().unwrap().history().len(), 1);
}
// Stale disk evidence rejects its own target while another explicitly selected
// canonical target succeeds. Neither the external write nor dirty source is saved.
#[test]
fn action_stale_source() {
    let (f, mut s) = editor(
        "org.agenda.sources=['one.org','other.org']",
        "one.org",
        "* TODO One\n",
    );
    let other = f.write("other.org", "* TODO Other\n");
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    std::fs::write(&other, "* TODO External replacement\n").unwrap();
    bulk(&mut s, "priority", "A");
    let results = s.agenda_action_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, ActionStatus::Changed);
    assert_eq!(results[1].status, ActionStatus::Rejected);
    assert!(results[1].message.contains("changed"), "{results:?}");
    assert_eq!(text(&s), "* TODO [#A] One\n");
    assert_eq!(
        std::fs::read_to_string(other).unwrap(),
        "* TODO External replacement\n"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("one.org")).unwrap(),
        "* TODO One\n"
    );
}
// Cancelling a later required note preserves the earlier completed transaction
// and reports the remaining target separately; it never claims multi-file rollback.
#[test]
fn action_partial_cancel() {
    let source = "#+TODO: TODO(t) | DONE(d@)\n* TODO One\n* TODO Two\n";
    let (_f, session) = editor("org.agenda.sources=['tasks.org']", "tasks.org", source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    bulk(&mut s, "todo", "DONE");
    assert!(s.echo().contains("note"));
    argument(&mut s, "first completed");
    settle(&mut s);
    assert!(s.echo().contains("note"));
    s.handle(control('g'));
    settle(&mut s);
    let results = s.agenda_action_results();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, ActionStatus::Changed);
    assert_eq!(results[1].status, ActionStatus::Cancelled);
    assert!(text(&s).contains("* DONE One\n"));
    assert!(text(&s).contains("* TODO Two\n"));
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.handle(key(KeyCode::Char('q')));
    action(&mut s, OrgCommand::AgendaUndo);
    settle(&mut s);
    assert_eq!(text(&s), source);
    assert_eq!(s.remaining_input(Instant::now()), None);
}
// Cancellation can arrive after the ordinary edit publishes but before action
// bookkeeping. The outcome must disclose that change and retain ordinary undo.
#[test]
fn action_cancel_after_publish() {
    let source = "* TODO One\n";
    let (_f, mut s) = editor("org.agenda.sources=['tasks.org']", "tasks.org", source);
    settle(&mut s);
    open(&mut s);
    action(&mut s, OrgCommand::Priority);
    argument(&mut s, "A");
    s.handle(control('g'));
    settle(&mut s);
    assert_eq!(s.agenda_action_results().len(), 1);
    assert_eq!(
        s.agenda_action_results()[0].status,
        ActionStatus::ChangedWithFailure
    );
    assert!(s.agenda_action_results()[0].message.contains("no rollback"));
    assert_eq!(text(&s), "* TODO [#A] One\n");
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, OrgCommand::AgendaUndo);
    assert_eq!(text(&s), source);
}
// The host draws only after Redraw. Zero-delay source steps must therefore be
// serviced by deadline expiry, without extra keypresses or forced test frames.
#[test]
fn action_event_progress() {
    let (_f, mut s) = editor(
        "org.agenda.sources=['tasks.org']",
        "tasks.org",
        "* TODO One\n* TODO Two\n",
    );
    settle(&mut s);
    open(&mut s);
    s.handle(key(KeyCode::Char('*')));
    s.handle(key(KeyCode::Char('B')));
    argument(&mut s, "priority");
    argument(&mut s, "A");
    s.frame().unwrap();
    let until = Instant::now() + std::time::Duration::from_secs(1);
    while s.agenda_action_results().is_empty() {
        assert!(
            Instant::now() < until,
            "native action did not schedule progress"
        );
        if s.expire_input(Instant::now()) == rev::session::Action::Redraw {
            s.frame().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(s.agenda_action_results().len(), 2);
    assert!(
        s.agenda_action_results()
            .iter()
            .all(|r| r.status == ActionStatus::Changed)
    );
    assert_eq!(text(&s), "* TODO [#A] One\n* TODO [#A] Two\n");
    settle(&mut s);
    assert_eq!(s.remaining_input(Instant::now()), None);
}
