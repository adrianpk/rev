use super::org::{named, org_editor, profiles, prompt, settle};
use super::*;
use rev::buffer::Stamp;
use rev::command::OrgCommand;
use rev::config::AgendaSpan;
use rev::org::agenda::{ActionStatus, QueryKind, SourceVersion};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};

#[path = "agenda/guards.rs"]
mod guards;
#[path = "agenda/habits.rs"]
mod habits;

const SOURCE: &str = "* TODO Zebra :work:\r\n<2024-03-01 Fri 09:00>\r\n* TODO Alpha 界 :home:\r\n<2024-03-02 Sat 10:00>\r\n";

fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709303400)),
    )
}

fn argument(session: &mut Session, draft: &str) {
    session.handle(Event::Paste(draft.into()));
    keys(session, "RET");
    settle(session);
}

fn open(session: &mut Session, command: OrgCommand) {
    session.execute(Command::Org(command));
    settle(session);
    argument(session, "current");
}

fn todo(session: &mut Session) {
    session.execute(Command::Org(OrgCommand::TodoList));
    settle(session);
    argument(session, "");
    argument(session, "current");
    assert_eq!(session.query_agenda().unwrap().rows.len(), 2);
}

fn unchanged(fixture: &Fixture, session: &Session, source: &str, stamp: Stamp) {
    assert_eq!(text(session), source);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
        source
    );
}

// Daily d and retained v d reproject the captured week in both profiles.
// Neither opens graph-date input nor edits source, history or the saved view.
#[test]
fn daily_view() {
    for snapshot in profiles() {
        let (fixture, session) = org_editor(&snapshot, SOURCE, 0, true);
        let mut session = session.with_org_clock(clock);
        let origin = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        open(&mut session, OrgCommand::Agenda);
        for sequence in ["d", "v d"] {
            keys(&mut session, "v w");
            settle(&mut session);
            assert_eq!(
                session.calendar_agenda().unwrap().query.span,
                AgendaSpan::Week
            );
            keys(&mut session, sequence);
            settle(&mut session);
            assert_eq!(
                session.calendar_agenda().unwrap().query.span,
                AgendaSpan::Day
            );
            assert!(!prompt(&mut session, "Habit agenda date"));
        }
        keys(&mut session, "q");
        assert!(session.calendar_agenda().is_none());
        assert_eq!(session.view(), &origin);
        unchanged(&fixture, &session, SOURCE, stamp);
    }
}

// Relocated queries retain their typed owner and captured qualified source.
// Retired leaves and cancelled prefixes cannot select a query or edit text.
#[test]
fn query_views() {
    for snapshot in profiles() {
        let (fixture, session) = org_editor(&snapshot, SOURCE, 0, false);
        let mut session = session.with_org_clock(clock);
        let stamp = session.buffer().unwrap().stamp();
        todo(&mut session);
        for retired in ["M", "s", "A", "G", "S"] {
            keys(&mut session, retired);
            assert_eq!(session.query_agenda().unwrap().query.kind, QueryKind::Todo);
            assert_eq!(session.query_agenda().unwrap().rows.len(), 2);
            assert!(session.frame().unwrap().cursor().is_none());
        }
        keys(&mut session, "v C-g");
        assert!(session.query_agenda().is_none());
        unchanged(&fixture, &session, SOURCE, stamp);
        todo(&mut session);
        for (sequence, label, draft, kind, title) in [
            (
                "v M",
                "Tag/property match",
                "work",
                QueryKind::Tags,
                "Zebra",
            ),
            (
                "v /",
                "Search [phrase",
                "Alpha",
                QueryKind::Search,
                "Alpha 界",
            ),
        ] {
            keys(&mut session, sequence);
            assert!(prompt(&mut session, label));
            let before = session.query_agenda().unwrap().query.kind;
            keys(&mut session, "C-g");
            assert_eq!(session.query_agenda().unwrap().query.kind, before);
            keys(&mut session, sequence);
            argument(&mut session, draft);
            let result = session.query_agenda().unwrap();
            assert_eq!(result.query.kind, kind);
            assert_eq!(result.rows.len(), 1);
            assert_eq!(result.rows[0].title.as_ref(), title);
            assert_eq!(result.sources[0].version, SourceVersion::Buffer(stamp));
        }
        keys(&mut session, "q");
        unchanged(&fixture, &session, SOURCE, stamp);
    }
}

// Sort is a view-only prompt; cancellation and invalid drafts retain its rows.
// The new family orders actual results and leaves the existing span route usable.
#[test]
fn sort_view() {
    for snapshot in profiles() {
        let (fixture, session) = org_editor(&snapshot, SOURCE, 0, false);
        let mut session = session.with_org_clock(clock);
        let stamp = session.buffer().unwrap().stamp();
        todo(&mut session);
        keys(&mut session, "v S");
        assert!(prompt(&mut session, "Agenda sort keys"));
        keys(&mut session, "C-g");
        assert_eq!(
            session.query_agenda().unwrap().rows[0].title.as_ref(),
            "Zebra"
        );
        keys(&mut session, "v S");
        argument(&mut session, "unknown");
        assert!(session.echo().contains("Unknown native agenda sort key"));
        keys(&mut session, "C-g");
        keys(&mut session, "v S");
        argument(&mut session, "title-up");
        let result = session.query_agenda().unwrap();
        assert_eq!(result.rows[0].title.as_ref(), "Alpha 界");
        assert_eq!(result.rows[1].title.as_ref(), "Zebra");
        assert!(prompt(&mut session, "v S sort"));
        keys(&mut session, "v s");
        assert!(prompt(&mut session, "Agenda span"));
        argument(&mut session, "month");
        assert_eq!(
            session.query_agenda().unwrap().context.span,
            AgendaSpan::Month
        );
        keys(&mut session, "q");
        unchanged(&fixture, &session, SOURCE, stamp);
    }
}

// The relocated search selects a different heading from the underlying point.
// Effort edits that qualified row once, leaves disk untouched, and undoes once.
#[test]
fn selected_source() {
    for snapshot in profiles() {
        let (fixture, session) = org_editor(&snapshot, SOURCE, 0, false);
        let mut session = session.with_org_clock(clock);
        let point = session.view().point.position;
        todo(&mut session);
        keys(&mut session, "v /");
        argument(&mut session, "Alpha");
        let result = session.query_agenda().unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].byte, SOURCE.find("* TODO Alpha").unwrap());
        keys(&mut session, "e");
        settle(&mut session);
        argument(&mut session, "0:30");
        assert_eq!(
            session.agenda_action_results()[0].status,
            ActionStatus::Changed
        );
        assert_eq!(
            text(&session),
            SOURCE.replace(
                "* TODO Alpha 界 :home:\r\n",
                "* TODO Alpha 界 :home:\r\n:PROPERTIES:\r\n:EFFORT: 0:30\r\n:END:\r\n"
            )
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(session.view().point.position, point);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
            SOURCE
        );
        keys(&mut session, "C-_");
        settle(&mut session);
        assert_eq!(text(&session), SOURCE);
    }
}
