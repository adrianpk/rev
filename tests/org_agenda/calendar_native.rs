use super::*;
use rev::{
    command::Scroll,
    config::AgendaSpan,
    org::{
        agenda::{CalendarLine, SourceVersion},
        time::Date,
    },
};
pub(super) fn visible(s: &mut Session, value: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(value))
}
pub(super) fn selected(s: &mut Session, title: &str) {
    let calendar = s.calendar_agenda().unwrap();
    let count = calendar.lines.len();
    let index = calendar
        .lines
        .iter()
        .position(
            |line| matches!(line,CalendarLine::Entry(i) if calendar.rows[*i].title.contains(title)),
        )
        .expect("visible source entry");
    for _ in 0..count {
        s.execute(Command::Scroll(Scroll::PreviousLine));
    }
    for _ in 0..index {
        s.execute(Command::Scroll(Scroll::NextLine));
    }
}
// Real calendar controls reproject captured observations without document
// edits. Invalid/cancelled date drafts retain the calendar and source view.
#[test]
fn native_calendar_controls() {
    let source = "* TODO Meeting 界\r\n<2024-03-01 Fri 09:00-10:00>\r\n";
    let (f, s) = editor(
        "org.time.zone='UTC'\norg.agenda.span='day'",
        "source.org",
        source,
    );
    let mut s = s.with_org_clock(super::habits::clock);
    settle(&mut s);
    let origin = s.view().clone();
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    assert!(visible(&mut s, "Org agenda [complete]"));
    assert!(visible(&mut s, "Meeting 界"));
    let history = s.buffer().unwrap().history().len();
    s.handle(Event::Paste("leak".into()));
    s.execute(Command::Newline);
    assert_eq!(text(&s), source);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    action(&mut s, OrgCommand::AgendaNext);
    assert_eq!(
        s.calendar_agenda().unwrap().query.start,
        Date::new(2024, 3, 2).unwrap()
    );
    action(&mut s, OrgCommand::AgendaWeek);
    assert_eq!(s.calendar_agenda().unwrap().query.span, AgendaSpan::Week);
    action(&mut s, OrgCommand::AgendaToday);
    assert_eq!(
        s.calendar_agenda().unwrap().query.start,
        Date::new(2024, 2, 26).unwrap()
    );
    action(&mut s, OrgCommand::AgendaDate);
    argument(&mut s, "2024-02-30");
    assert!(s.calendar_agenda().is_some());
    s.handle(control('g'));
    assert!(s.calendar_agenda().is_some());
    action(&mut s, OrgCommand::AgendaDate);
    argument(&mut s, "2024-03-15");
    settle(&mut s);
    assert_eq!(
        s.calendar_agenda().unwrap().query.start,
        Date::new(2024, 3, 11).unwrap()
    );
    s.handle(key(KeyCode::Char('q')));
    assert!(s.calendar_agenda().is_none());
    assert_eq!(s.view(), &origin);
    assert_eq!(text(&s), source);
    assert_eq!(
        std::fs::read_to_string(f.0.join("source.org")).unwrap(),
        source
    );
}
// Disk source return rereads bounded bytes in the owned worker, proves text
// equality, and installs ordinary BOM/CRLF canonical ingress. Stale rows reject.
#[test]
fn native_calendar_disk_return() {
    let (f, s) = editor(
        "org.time.zone='UTC'\norg.agenda.span='day'\norg.agenda.sources=['other.org']",
        "source.org",
        "* Origin",
    );
    let path = f.write(
        "other.org",
        "\u{feff}* Meeting 界\r\n<2024-03-01 Fri 09:00>\r\n",
    );
    let before = std::fs::read(&path).unwrap();
    let mut s = s.with_org_clock(super::habits::clock);
    settle(&mut s);
    action(&mut s, OrgCommand::Agenda);
    assert!(matches!(
        s.calendar_agenda().unwrap().sources[0].version,
        SourceVersion::Disk(_)
    ));
    selected(&mut s, "Meeting");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.calendar_agenda().is_none(), "{}", s.echo());
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), path);
    assert!(s.buffer().unwrap().has_bom());
    assert!(!s.buffer().unwrap().is_modified());
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    // A new session leaves the target on disk, so this half exercises the
    // disk observation rather than open-buffer precedence.
    let (f, session) = editor(
        "org.time.zone='UTC'\norg.agenda.span='day'\norg.agenda.sources=['other.org']",
        "source.org",
        "* Origin",
    );
    let path = f.write("other.org", "* Meeting\n<2024-03-01 Fri>\n");
    let mut s = session.with_org_clock(super::habits::clock);
    settle(&mut s);
    action(&mut s, OrgCommand::Agenda);
    std::fs::write(&path, "* Changed\n<2024-03-01 Fri>\n").unwrap();
    selected(&mut s, "Meeting");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.calendar_agenda().is_some());
    assert!(s.echo().contains("changed"), "{}", s.echo());
    assert_eq!(text(&s), "* Origin");
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    selected(&mut s, "Changed");
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert!(s.calendar_agenda().is_none());
    assert_eq!(text(&s), "* Changed\n<2024-03-01 Fri>\n");
}
