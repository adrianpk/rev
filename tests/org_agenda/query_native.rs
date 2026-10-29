use super::*;
use rev::org::agenda::{QueryKind, SourceVersion};
fn visible(s: &mut Session, value: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(value))
}
fn open(s: &mut Session, command: OrgCommand, expression: &str) {
    action(s, command);
    argument(s, expression);
    settle(s);
}
const SOURCE: &str = "* TODO Parent :work:\n:PROPERTIES:\n:TEAM: red\n:EFFORT: 1:30\n:END:\nComputer\n equipment\n** TODO Child :urgent:\n* DONE Finished :work:\n";
// Native prompts compile before discovery; query changes retain source scope,
// cancelled/invalid drafts preserve the view, and the agenda cannot edit text.
#[test]
fn query_native_controls() {
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.sources=['tasks.org']\norg.properties.inherit=['TEAM','EFFORT']",
        "tasks.org",
        SOURCE,
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    let origin = s.view().clone();
    let stamp = s.buffer().unwrap().stamp();
    open(&mut s, OrgCommand::TodoList, "");
    assert_eq!(s.query_agenda().unwrap().rows.len(), 2);
    assert!(visible(&mut s, "Org TODO agenda [complete]"));
    s.handle(Event::Paste("blocked".into()));
    s.execute(Command::Newline);
    assert_eq!(text(&s), SOURCE);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    s.handle(key(KeyCode::Char('m')));
    assert!(s.frame().unwrap().cursor().is_some());
    argument(&mut s, "TEAM=>red");
    assert_eq!(s.query_agenda().unwrap().query.kind, QueryKind::Todo);
    s.handle(control('g'));
    assert!(s.query_agenda().is_some());
    s.handle(key(KeyCode::Char('m')));
    argument(&mut s, r#"work+urgent+TEAM="red"+EFFORT<*120"#);
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "Child");
    s.handle(key(KeyCode::Char('s')));
    argument(&mut s, "computer equipment");
    settle(&mut s);
    assert_eq!(s.query_agenda().unwrap().rows[0].title.as_ref(), "Parent");
    s.handle(key(KeyCode::Char('q')));
    assert!(s.query_agenda().is_none());
    assert_eq!(s.view(), &origin);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(text(&s), SOURCE);
    assert_eq!(
        std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
        SOURCE
    );
}
// Refresh observes actual dirty canonical bytes. A retained locator cannot
// visit after an edit, and refresh/return never autosaves or creates history.
#[test]
fn query_native_refresh() {
    let (f, session) = editor("org.time.zone='UTC'", "tasks.org", SOURCE);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s, OrgCommand::TodoList, "");
    argument(&mut s, "current");
    settle(&mut s);
    let id = s.buffer().unwrap().id();
    s.replace(0..0, "# unsaved\n").unwrap();
    settle(&mut s);
    let history = s.buffer().unwrap().history().len();
    assert!(visible(&mut s, "stale: g refresh"));
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.query_agenda().is_some());
    assert!(s.echo().contains("changed"), "{}", s.echo());
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert!(!visible(&mut s, "stale: g refresh"));
    assert_eq!(s.query_agenda().unwrap().rows[0].byte, "# unsaved\n".len());
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.query_agenda().is_none());
    assert_eq!(s.buffer().unwrap().id(), id);
    assert_eq!(s.view().point.position.0, "# unsaved\n".len());
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(
        std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
        SOURCE
    );
    assert_eq!(s.remaining_input(Instant::now()), None);
}
// Disk query return shares bounded reread and exact-text checks, preserves BOM
// and CRLF, and keeps an unavailable selected source visible as incomplete.
#[test]
fn query_native_disk() {
    let (_f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.sources=['other.org','missing.org']",
        "origin.org",
        "* Origin",
    );
    let f = _f;
    let path = f.write("other.org", "\u{feff}* TODO Disk 界\r\n");
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    open(&mut s, OrgCommand::TodoList, "");
    assert!(!s.query_agenda().unwrap().complete());
    assert!(matches!(
        s.query_agenda().unwrap().sources[0].version,
        SourceVersion::Disk(_)
    ));
    std::fs::write(&path, "\u{feff}* TODO Changed 界\r\n").unwrap();
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.query_agenda().is_some());
    assert!(s.echo().contains("changed"), "{}", s.echo());
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert_eq!(
        s.query_agenda().unwrap().rows[0].title.as_ref(),
        "Changed 界"
    );
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.query_agenda().is_none());
    assert!(s.buffer().unwrap().has_bom());
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), path);
    assert_eq!(text(&s), "* TODO Changed 界\r\n");
    assert!(!s.buffer().unwrap().is_modified());
}
// Native query projection clips Unicode cells only in its owner. Keyboard
// window commands reach an editable shared pane, and deletion retires ownership.
#[test]
fn query_native_panes() {
    use rev::command::Split;
    use unicode_width::UnicodeWidthStr;
    let (f, session) = editor(
        "org.agenda.sources=['tasks.org']",
        "tasks.org",
        "\u{feff}* TODO 界e\u{301}\r\n",
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = *s.windows().order().iter().find(|v| **v != owner).unwrap();
    let peer_view = s.windows().get(peer).unwrap().clone();
    open(&mut s, OrgCommand::TodoList, "");
    for (width, height) in [(100, 12), (25, 9), (1, 1), (100, 12)] {
        s.handle(Event::Resize(width, height));
        let frame = s.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width));
        }
        if let Some(c) = frame.cursor() {
            assert!(c.column < width && c.row < height);
        }
    }
    assert_eq!(s.windows().get(peer).unwrap().point, peer_view.point);
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    argument(&mut s, "other-window");
    assert_eq!(s.windows().selected_id(), peer);
    assert!(s.frame().unwrap().cursor().is_some());
    s.handle(Event::Paste("# peer\r\n".into()));
    settle(&mut s);
    assert_eq!(text(&s), "# peer\r\n* TODO 界e\u{301}\r\n");
    let history = s.buffer().unwrap().history().len();
    s.execute(Command::Window(rev::command::WindowCommand::Other));
    assert_eq!(s.windows().selected_id(), owner);
    assert!(s.frame().unwrap().cursor().is_none());
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    argument(&mut s, "delete-window");
    assert!(s.query_agenda().is_none());
    assert_eq!(s.windows().selected_id(), peer);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(
        std::fs::read_to_string(f.0.join("tasks.org")).unwrap(),
        "\u{feff}* TODO 界e\u{301}\r\n"
    );
}
// A real expensive query shares the existing owned job. Cancellation preserves
// input/source and reaps that slot without publishing a complete empty agenda.
#[test]
fn query_native_cancel() {
    let source = format!("* TODO Long entry\n{}", "a".repeat(1000000));
    let (f, session) = editor("org.agenda.sources=['tasks.org']", "origin.org", "* Origin");
    let path = f.write("tasks.org", &source);
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    action(&mut s, OrgCommand::SearchView);
    argument(&mut s, "+{(a+)+b}");
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
