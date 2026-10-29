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

// Runtime filters use the retained unopened snapshot. Refresh alone reads later
// bytes; checked return rejects stale disk text and reset preserves its preset.
#[test]
fn custom_native_snapshot() {
    let (f, session) = editor(CONFIG, "origin.org", "* Origin\n");
    let path = f.write("tasks.org", SOURCE);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    option(&mut s, OrgCommand::AgendaCustom, "w");
    std::fs::write(
        &path,
        "* TODO New :work:\n:PROPERTIES:\n:EFFORT: 0:10\n:END:\n",
    )
    .unwrap();
    option(&mut s, OrgCommand::AgendaFilter, "EFFORT<60");
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "Small");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.echo().contains("changed"), "{}", s.echo());
    assert!(s.query_agenda().is_some());
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "New");
    s.handle(key(KeyCode::Char('|')));
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows.len(), 1);
    s.handle(key(KeyCode::Char('q')));
    assert_eq!(text(&s), "* Origin\n");
    option(&mut s, OrgCommand::AgendaCustom, "w");
    assert_eq!(
        s.query_agenda().unwrap().view.as_ref().unwrap().blocks[0].filter,
        "work"
    );
    assert_eq!(
        s.query_agenda().unwrap().view.as_ref().unwrap().blocks[0].columns,
        vec!["OWNER"]
    );
    assert!(s.buffer().unwrap().history().is_empty());
}
// Owned custom projections clip Unicode columns and leave peers editable.
// Deleting the owner removes its runtime view without changing canonical bytes.
#[test]
fn custom_native_panes() {
    use rev::command::{Split, WindowCommand};
    use unicode_width::UnicodeWidthStr;
    let (f, session) = editor(
        CONFIG,
        "tasks.org",
        "\u{feff}* TODO 界e\u{301} :work:\r\n:PROPERTIES:\r\n:OWNER: 界Ada\r\n:END:\r\n",
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    option(&mut s, OrgCommand::AgendaCustom, "w");
    for (width, height) in [(100, 12), (25, 9), (1, 1), (100, 12)] {
        s.handle(Event::Resize(width, height));
        let frame = s.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width));
        }
    }
    named(&mut s, "other-window");
    assert_ne!(s.windows().selected_id(), owner);
    assert!(s.frame().unwrap().cursor().is_some());
    s.handle(Event::Paste("# peer\r\n".into()));
    settle(&mut s);
    let history = s.buffer().unwrap().history().len();
    s.execute(Command::Window(WindowCommand::Other));
    assert!(visible(&mut s, "stale: g refresh"));
    named(&mut s, "delete-window");
    assert!(s.query_agenda().is_none());
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert!(
        std::fs::read_to_string(f.0.join("tasks.org"))
            .unwrap()
            .starts_with('\u{feff}')
    );
}
// The configured global selection and explicit block scopes remain distinct;
// missing files are visible failures, never complete empty filtered results.
#[test]
fn custom_native_scopes() {
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.sources=['tasks.org']\norg.agenda.views=[{key='s',title='Scoped',blocks=[{kind='todo'},{kind='todo',sources=['other.org']},{kind='todo',sources=['missing.org']}]}]",
        "tasks.org",
        "* TODO Default\n",
    );
    f.write("other.org", "* TODO Other\n");
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    option(&mut s, OrgCommand::AgendaCustom, "s");
    let result = s.query_agenda().unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].title.as_ref(), "Default");
    assert_eq!(result.rows[1].title.as_ref(), "Other");
    assert!(!result.complete());
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.path.as_ref().is_some_and(|p| p.ends_with("missing.org")))
    );
}

// Custom observation uses the same finite job slot; cancellation leaves its
// source and history intact and reaps the worker without idle polling.
#[test]
fn custom_native_cancel() {
    let source = format!("* TODO Long\n{}", "a".repeat(1000000));
    let (f, session) = editor(
        "org.agenda.sources=['tasks.org']\norg.agenda.views=[{key='c',title='Cancel',blocks=[{kind='search',query='+{(a+)+b}'}]}]",
        "origin.org",
        "* Origin",
    );
    let path = f.write("tasks.org", &source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    action(&mut s, OrgCommand::AgendaCustom);
    argument(&mut s, "c");
    s.handle(control('g'));
    settle(&mut s);
    assert!(s.query_agenda().is_none());
    assert_eq!(text(&s), "* Origin");
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert!(s.buffer().unwrap().history().is_empty());
    for _ in 0..3 {
        s.frame().unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
    }
}
