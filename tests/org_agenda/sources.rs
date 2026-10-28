use super::*;
use rev::org::agenda::SourceVersion;
use std::fs;
// The actual key and named command share selection. Empty configuration prompts
// without discovery, and an unsaved eligible buffer can be explicitly selected.
#[test]
fn agenda_source_selection() {
    let (f, mut s) = editor("", "source.org", "* TODO Disk\r\n");
    settle(&mut s);
    s.handle(Event::Paste("* TODO Unsaved\n".into()));
    let before = text(&s);
    let stamp = s.buffer().unwrap().stamp();
    let history = s.buffer().unwrap().history().len();
    s.handle(control('c'));
    s.handle(key(KeyCode::Char('a')));
    assert!(s.agenda_sources().is_none());
    assert!(s.echo().contains("Select current"));
    s.handle(control('g'));
    assert!(s.agenda_sources().is_none());
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    let result = s.agenda_sources().unwrap();
    assert!(result.complete(), "{}", result.feedback());
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].text().to_string(), before);
    assert_eq!(result.documents[0].version, SourceVersion::Buffer(stamp));
    assert_eq!(text(&s), before);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(
        fs::read_to_string(f.0.join("source.org")).unwrap(),
        "* TODO Disk\r\n"
    );
    assert_eq!(s.remaining_input(Instant::now()), None);
}
// A configured relative root is resolved at its declaring layer. Unsaved text
// in a hidden open source wins over disk while a failed sibling stays visible.
#[test]
fn agenda_configured_sources() {
    let f = Fixture::new();
    let path = f.write("sources/task.org", "* Old");
    f.write(
        "sources/config.toml",
        "org.agenda.sources=['task.org','absent.org']",
    );
    let config = f.write("config.toml", "include=['sources/config.toml']");
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: b.span(0..b.len_chars()).unwrap(),
        text: "* TODO Dirty".into(),
    };
    let prepared = b.prepare(&[edit], &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 10,
        },
    )
    .unwrap();
    action(&mut s, OrgCommand::Agenda);
    let result = s.agenda_sources().unwrap();
    assert!(!result.complete());
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].text().to_string(), "* TODO Dirty");
    assert_eq!(result.issues.len(), 1);
    assert!(s.echo().contains("incomplete"));
    assert!(s.echo().contains("absent.org"));
    assert_eq!(text(&s), "");
    assert_eq!(fs::read_to_string(path).unwrap(), "* Old");
}
fn many_sources() -> (Fixture, Session) {
    let f = Fixture::new();
    for n in 0..256 {
        f.write(
            &format!("sources/{n}.org"),
            &format!("* TODO Task {n}\n{}", "body\n".repeat(64)),
        );
    }
    let config = f.write("config.toml", "org.agenda.sources=['sources']");
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 10,
        },
    )
    .unwrap();
    (f, s)
}
// A real 256-file request uses one owned worker. Editing its initiating buffer
// neither waits for IO nor invalidates unrelated sources, and idle adds no poll.
#[test]
fn agenda_source_responsiveness() {
    let (_f, mut s) = many_sources();
    let started = Instant::now();
    s.execute(Command::Org(OrgCommand::Agenda));
    s.handle(Event::Paste("Still editable".into()));
    s.execute(Command::Motion(rev::command::Motion::BackwardChar));
    assert_eq!(text(&s), "Still editable");
    settle(&mut s);
    let result = s.agenda_sources().unwrap();
    assert!(result.complete(), "{}", result.feedback());
    assert_eq!(result.documents.len(), 256);
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    eprintln!(
        "256-source discovery with native editing: {:?}, {} bytes",
        started.elapsed(),
        result.bytes
    );
    for _ in 0..5 {
        s.frame().unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
    }
    assert_eq!(text(&s), "Still editable");
}
// C-g retires result authority immediately, keeps ownership until the worker is
// reaped, and never substitutes a successful empty result for cancellation.
#[test]
fn agenda_source_cancel() {
    let (_f, mut s) = many_sources();
    s.execute(Command::Org(OrgCommand::Agenda));
    s.handle(control('g'));
    assert!(s.echo().contains("cancelled"), "{}", s.echo());
    assert!(s.echo().contains("incomplete"));
    s.handle(Event::Paste("usable".into()));
    settle(&mut s);
    assert!(s.agenda_sources().is_none());
    assert_eq!(text(&s), "usable");
    assert_eq!(s.remaining_input(Instant::now()), None);
    action(&mut s, OrgCommand::Agenda);
    assert!(s.agenda_sources().unwrap().complete());
}
// Publication rechecks every current-buffer stamp rather than the initiating
// cursor. A changed source is removed with a refresh diagnostic, other files stay.
#[test]
fn agenda_source_stale() {
    let (f, mut s) = editor(
        "org.agenda.sources=['source.org','other.org']",
        "source.org",
        "* Old",
    );
    f.write("other.org", "* Other");
    settle(&mut s);
    s.execute(Command::Org(OrgCommand::Agenda));
    s.handle(Event::Paste("dirty".into()));
    settle(&mut s);
    let result = s.agenda_sources().unwrap();
    assert!(!result.complete());
    assert_eq!(result.documents.len(), 1);
    assert_eq!(result.documents[0].text().to_string(), "* Other");
    assert!(result.feedback().contains("changed"));
    action(&mut s, OrgCommand::Agenda);
    let result = s.agenda_sources().unwrap();
    assert!(result.complete());
    assert!(
        result
            .documents
            .iter()
            .any(|d| d.text() == text(&s).as_str())
    );
}
