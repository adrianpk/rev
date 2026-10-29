use super::*;
const SOURCE: &str = "* TODO Small :work:\n:PROPERTIES:\n:EFFORT: 0:30\n:OWNER: Ada\n:END:\n<2024-03-01 Fri>\n* TODO Large :work:\n:PROPERTIES:\n:EFFORT: 2:00\n:OWNER: Grace\n:END:\n* TODO Home :home:\n";
const CONFIG: &str = "org.time.zone='UTC'\norg.agenda.sources=['tasks.org']\norg.agenda.views=[{key='w',title='Work',blocks=[{kind='todo',filter='work',columns=['OWNER'],sort=['effort-down']}]}]";
fn visible(s: &mut Session, value: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(value))
}
fn option(s: &mut Session, command: OrgCommand, draft: &str) {
    action(s, command);
    argument(s, draft);
    settle(s);
}
fn named(s: &mut Session, name: &str) {
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    argument(s, name);
    settle(s);
}
// Compiled and keyed controls share real rendering, canonical source locators,
// preset reset and retained snapshots; only refresh observes later source edits.
#[test]
fn custom_native_controls() {
    let (f, session) = editor(CONFIG, "tasks.org", SOURCE);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    let origin = s.view().clone();
    named(&mut s, "org-agenda-custom-view");
    assert!(s.frame().unwrap().cursor().is_some());
    argument(&mut s, "w");
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 2);
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "Large");
    assert!(visible(&mut s, "OWNER=Grace"));
    s.handle(key(KeyCode::Char('/')));
    argument(&mut s, "EFFORT<60");
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 1);
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "Small");
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 1);
    s.handle(key(KeyCode::Char('|')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 2);
    s.handle(key(KeyCode::Char('S')));
    argument(&mut s, "title-up");
    settle(&mut s);
    s.handle(key(KeyCode::Char('V')));
    argument(&mut s, "OWNER,EFFORT");
    settle(&mut s);
    assert!(visible(&mut s, "EFFORT=2:00"));
    option(&mut s, OrgCommand::AgendaFilter, "EFFORT=>bad");
    assert_eq!(s.query_agenda().unwrap().rows.len(), 2);
    s.handle(control('g'));
    assert!(s.query_agenda().is_some());
    s.handle(key(KeyCode::Char('q')));
    assert_eq!(s.view(), &origin);
    assert_eq!(text(&s), SOURCE);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(
        std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
        SOURCE
    );
}
// Custom views observe configured block roots relative to the declaring file,
// show empty blocks, support date movement and visit the same canonical buffer.
#[test]
fn custom_native_blocks() {
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.views=[{key='c',title='Calendar',blocks=[{kind='calendar',sources=['tasks.org'],span='day'},{kind='tags',query='missing'}]}]",
        "tasks.org",
        SOURCE,
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    option(&mut s, OrgCommand::AgendaCustom, "c");
    assert_eq!(s.query_agenda().unwrap().rows.len(), 1);
    assert!(visible(&mut s, "2:Tags(0)"));
    s.handle(key(KeyCode::Char('v')));
    s.handle(key(KeyCode::Char('m')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().context.days(), 31);
    s.handle(key(KeyCode::Char('f')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 0);
    s.handle(key(KeyCode::Char('.')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 1);
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.query_agenda().is_none());
    assert_eq!(
        s.buffer().unwrap().file().unwrap().path(),
        f.0.join("tasks.org")
    );
    assert_eq!(text(&s), SOURCE);
}
