use super::*;
use rev::{
    command::{Split, WindowCommand},
    org::time::{Date, ZoneEnvironment, ZoneSelection, ZonedOperation},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use unicode_width::UnicodeWidthStr;
static CAPTURES: AtomicUsize = AtomicUsize::new(0);
fn advancing(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    let n = CAPTURES.fetch_add(1, Ordering::SeqCst);
    ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709303400 + n as u64 * 86400)),
    )
}
// Painting and date movement retain the original operation. Only refresh reads
// a new clock/source snapshot; stale canonical locators cannot visit old bytes.
#[test]
fn calendar_dirty_refresh() {
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.span='day'",
        "notes.org",
        "* Original\n<2024-03-01 Fri 09:00>\n",
    );
    let mut s = session.with_org_clock(advancing);
    settle(&mut s);
    let id = s.buffer().unwrap().id();
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    let fresh = "# unsaved\n* Fresh\n<2024-03-02 Sat 10:00>\n";
    let end = s.buffer().unwrap().len_chars();
    s.replace(0..end, fresh).unwrap();
    settle(&mut s);
    assert!(super::calendar_native::visible(&mut s, "stale: g refresh"));
    super::calendar_native::selected(&mut s, "Original");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.calendar_agenda().is_some());
    assert!(s.echo().contains("changed"), "{}", s.echo());
    action(&mut s, OrgCommand::AgendaNext);
    action(&mut s, OrgCommand::AgendaToday);
    for _ in 0..3 {
        s.frame().unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
    }
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    let history = s.buffer().unwrap().history().len();
    let view = s.view().clone();
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 2);
    let calendar = s.calendar_agenda().unwrap();
    assert_eq!(calendar.query.today, Date::new(2024, 3, 2).unwrap());
    assert_eq!(calendar.query.start, calendar.query.today);
    assert!(calendar.rows.iter().any(|r| r.title.as_ref() == "Fresh"));
    assert!(!super::calendar_native::visible(&mut s, "stale: g refresh"));
    assert_eq!(s.view(), &view);
    assert_eq!(text(&s), fresh);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    super::calendar_native::selected(&mut s, "Fresh");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.calendar_agenda().is_none());
    assert_eq!(s.buffer().unwrap().id(), id);
    assert_eq!(s.view().point.position.0, "# unsaved\n".len());
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(
        std::fs::read_to_string(f.0.join("notes.org")).unwrap(),
        "* Original\n<2024-03-01 Fri 09:00>\n"
    );
    assert_eq!(s.remaining_input(Instant::now()), None);
}
// The projection owns one pane, not its shared canonical buffer. Unicode cell
// clipping, peer editing/cursor, source history and owner deletion stay native.
#[test]
fn calendar_shared_panes() {
    let source = "\u{feff}* Meeting 界e\u{301}\r\n<2024-03-01 Fri 09:00>\r\n";
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.span='day'",
        "notes.org",
        source,
    );
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = *s.windows().order().iter().find(|v| **v != owner).unwrap();
    let peer_view = s.windows().get(peer).unwrap().clone();
    let before = text(&s);
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    for (width, height) in [(120, 12), (25, 9), (8, 4), (1, 1), (100, 12)] {
        s.handle(Event::Resize(width, height));
        let frame = s.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width));
        }
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height);
        }
    }
    assert_eq!(s.windows().get(peer).unwrap().point, peer_view.point);
    s.handle(Event::Paste("blocked".into()));
    s.execute(Command::Newline);
    assert_eq!(text(&s), before);
    assert!(s.buffer().unwrap().history().is_empty());
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    argument(&mut s, "other-window");
    assert_eq!(s.windows().selected_id(), peer);
    assert!(s.frame().unwrap().cursor().is_some());
    s.handle(Event::Paste("# peer\r\n".into()));
    settle(&mut s);
    assert_eq!(text(&s), format!("# peer\r\n{before}"));
    let history = s.buffer().unwrap().history().len();
    s.execute(Command::Window(WindowCommand::Other));
    assert_eq!(s.windows().selected_id(), owner);
    assert!(s.frame().unwrap().cursor().is_none());
    let source_view = s.view().clone();
    s.handle(key(KeyCode::Char('q')));
    assert!(s.calendar_agenda().is_none());
    assert_eq!(s.view(), &source_view);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    argument(&mut s, "delete-window");
    assert!(s.calendar_agenda().is_none());
    assert_eq!(s.windows().selected_id(), peer);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(s.remaining_input(Instant::now()), None);
}
