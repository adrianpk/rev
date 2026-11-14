use super::*;
use rev::buffer::edit::History;
use rev::command::OrgCommand;
use std::time::{Duration, Instant};

#[path = "org/guards.rs"]
mod guards;

pub(super) fn profiles() -> [Snapshot; 2] {
    [settings(""), settings(profile(Some(ModalCommand::Normal)))]
}

pub(super) fn org_editor(
    snapshot: &Snapshot,
    source: &str,
    point: usize,
    readonly: bool,
) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(readonly);
    let mut view = View::with_settings(buffers.get(id).unwrap(), snapshot);
    view.point.position = Scalar(point);
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    settle(&mut session);
    (fixture, session)
}

pub(super) fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Org work did not settle: {}",
            session.echo()
        );
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

pub(super) fn prompt(session: &mut Session, label: &str) -> bool {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .any(|row| row.text().contains(label))
}

pub(super) fn named(session: &mut Session, command: &str) {
    keys(session, "M-x");
    session.handle(Event::Paste(command.into()));
    keys(session, "RET");
    settle(session);
}

// The new effort chord cycles inherited choices in every Org editing state.
// Only the selected child's value changes, preserving CRLF, Unicode and undo.
#[test]
fn effort_cycles() {
    let snapshots = profiles();
    let source = "#+PROPERTY: EFFORT_ALL 0 0:10 1h\r\n* Parent\r\n** TODO 界\r\n:PROPERTIES:\r\n:EFFORT: 0:10\r\n:OWNER: Keep\r\n:END:\r\nbody";
    for state in STATES {
        let (fixture, mut session) = org_editor(
            &snapshots[usize::from(state.is_some())],
            source,
            source.find("** TODO").unwrap(),
            false,
        );
        enter(&mut session, state);
        let pane = session.windows().selected_id();
        for choice in ["1h", "0"] {
            keys(&mut session, "C-c C-x E");
            settle(&mut session);
            assert_eq!(
                text(&session),
                source.replace(":EFFORT: 0:10", &format!(":EFFORT: {choice}")),
                "{state:?}: {}",
                session.echo()
            );
        }
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        keys(&mut session, "C-_");
        assert!(text(&session).contains(":EFFORT: 1h\r\n"));
        keys(&mut session, "C-_");
        assert_eq!(text(&session), source);
        assert_eq!(session.windows().selected_id(), pane);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
            source
        );
    }
}

// Missing/invalid declarations and read-only authority keep native diagnostics
// and preserve the exact source revision instead of manufacturing a choice.
#[test]
fn effort_errors() {
    for snapshot in profiles() {
        for (name, source, readonly, diagnostic) in [
            ("absent choices", "* Task\n", false, "effort_all"),
            (
                "invalid choice",
                "#+PROPERTY: EFFORT_ALL broken\n* Task\n",
                false,
                "duration",
            ),
            (
                "read-only",
                "#+PROPERTY: EFFORT_ALL 1h\n* Task\n",
                true,
                "read-only",
            ),
        ] {
            let (_fixture, mut session) =
                org_editor(&snapshot, source, source.find('*').unwrap(), readonly);
            let stamp = session.buffer().unwrap().stamp();
            keys(&mut session, "C-c C-x E");
            settle(&mut session);
            assert!(
                session.echo().to_ascii_lowercase().contains(diagnostic),
                "{name}: {}",
                session.echo()
            );
            assert_eq!(text(&session), source);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// The replacement opens qualified property columns even for read-only source.
// It cannot insert a clocktable, change focus/source or create an undo group.
#[test]
fn column_view() {
    let snapshots = profiles();
    let source = "#+COLUMNS: %ITEM %OWNER %EFFORT{:}\n* Parent\n:PROPERTIES:\n:OWNER: Team\n:EFFORT: 9h\n:END:\n** Child\n:PROPERTIES:\n:EFFORT: 1h\n:END:\n";
    for state in STATES {
        let (fixture, mut session) =
            org_editor(&snapshots[usize::from(state.is_some())], source, 0, true);
        enter(&mut session, state);
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        let pane = session.windows().selected_id();
        keys(&mut session, "C-c C-x C-c");
        settle(&mut session);
        let result = session
            .query_agenda()
            .unwrap_or_else(|| panic!("{}", session.echo()));
        assert_eq!(result.rows.len(), 2, "{state:?}");
        assert_eq!(result.rows[0].fields["EFFORT"], "1:00");
        assert_eq!(result.rows[0].fields["OWNER"], "Team");
        assert_eq!(
            result.sources[0].version,
            rev::org::agenda::SourceVersion::Buffer(stamp)
        );
        session.handle(Event::Paste("blocked".into()));
        keys(&mut session, "q");
        settle(&mut session);
        assert!(session.query_agenda().is_none());
        assert_eq!(text(&session), source);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.view().point, point);
        assert_eq!(session.windows().selected_id(), pane);
        assert!(session.buffer().unwrap().history().is_empty());
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
            source
        );
    }
}

fn header_key(session: &mut Session, suffix: &str) {
    keys(session, "C-c C-v");
    if suffix == "C-j" {
        // Exercise the actual Control-J event, independently of parsed spelling.
        session.handle(Event::Key(KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::CONTROL,
        )));
    } else {
        keys(session, suffix);
    }
    settle(session);
}

// Plain j, raw Control-J and terminal Return reach one checked header owner.
// Cancellation retains source; acceptance edits one CRLF header and undoes once.
#[test]
fn header_aliases() {
    let snapshots = profiles();
    for state in STATES {
        for suffix in ["j", "C-j", "RET"] {
            header_case(&snapshots[usize::from(state.is_some())], state, suffix);
        }
    }
}

fn header_case(snapshot: &Snapshot, state: Option<ModalCommand>, suffix: &str) {
    let source = "#+BEGIN_SRC sh :results output\r\nprintf '界'\r\n#+END_SRC\r\n";
    let (fixture, mut session) = org_editor(snapshot, source, 0, false);
    enter(&mut session, state);
    let stamp = session.buffer().unwrap().stamp();
    keys(&mut session, "C-c C-v h");
    settle(&mut session);
    assert!(!prompt(&mut session, "Source header argument"));
    assert_eq!(text(&session), source);
    header_key(&mut session, suffix);
    assert!(
        prompt(&mut session, "Source header argument"),
        "{state:?}/{suffix}: {}",
        session.echo()
    );
    keys(&mut session, "C-g");
    settle(&mut session);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    enter(&mut session, state);
    header_key(&mut session, suffix);
    session.handle(Event::Paste(":exports both".into()));
    keys(&mut session, "RET");
    settle(&mut session);
    assert_eq!(
        text(&session),
        source.replace(":results output", ":results output :exports both")
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.source_edit_draft().is_none());
    keys(&mut session, "C-_");
    assert_eq!(text(&session), source);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
        source
    );
}

// Source admission rejects a read-only block and a non-block target before
// acquiring header input; neither route may publish source or history changes.
#[test]
fn header_refusal() {
    for snapshot in profiles() {
        for (source, readonly) in [
            ("#+BEGIN_SRC sh\ntrue\n#+END_SRC\n", true),
            ("* Task\n", false),
        ] {
            let (_fixture, mut session) = org_editor(&snapshot, source, 0, readonly);
            let stamp = session.buffer().unwrap().stamp();
            header_key(&mut session, "j");
            assert!(!prompt(&mut session, "Source header argument"));
            assert!(!session.echo().is_empty());
            assert_eq!(text(&session), source);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}
