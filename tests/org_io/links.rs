use super::*;
use rev::command::History;
fn editor(config: &str, source: &str, destination: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let startup = prepare(&f.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    f.write("destination.org", destination);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("source.org", source)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 14,
        },
    )
    .unwrap();
    (f, session)
}
fn line(session: &mut Session, value: &str) {
    session.execute(Command::GotoLine);
    argument(session, value);
}
// Internal target families resolve the actual source, reveal hidden locations,
// and return through revision-qualified points without changing text/history.
#[test]
fn links_internal() {
    for (name, target, body, expected) in [
        ("dedicated", "spot", "* Root\nbody <<spot>>\n", "<<spot>>"),
        (
            "heading",
            "*Target",
            "* TODO Target :work:\nbody\n",
            "* TODO Target",
        ),
        (
            "custom id",
            "#stable",
            "* Target\n:PROPERTIES:\n:CUSTOM_ID: stable\n:END:\n",
            "* Target",
        ),
        (
            "named element",
            "table",
            "#+NAME: table\n| a |\n",
            "#+NAME:",
        ),
    ] {
        let source = format!("[[{target}][Go]]\n{body}");
        let (fixture, mut session) = editor("", &source, "* Other\n");
        let stamp = session.buffer().unwrap().stamp();
        action(&mut session, C::OpenLink);
        assert!(
            session.echo().contains("Org link destination"),
            "{name}: {}",
            session.echo()
        );
        let at = session.view().point.position.0;
        assert!(
            text(&session)
                .chars()
                .skip(at)
                .collect::<String>()
                .starts_with(expected),
            "{name}: {at}"
        );
        action(&mut session, C::LinkBack);
        assert_eq!(session.view().point.position.0, 0, "{name}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
            source
        );
    }
}
// File, ID and attachment links use explicit bounded sources, checked metadata
// and percent-decoded paths. The source stays intact when changing documents.
#[test]
fn links_documents() {
    for (name, source, row, destination, expected) in [
        (
            "file",
            "[[file:destination.org::*Target]]\n",
            "1",
            "* Other\n* Target 界\n* Target\n",
            "* Target\n",
        ),
        (
            "id",
            "[[id:stable]]\n",
            "1",
            "* Target\n:PROPERTIES:\n:ID: stable\n:END:\n",
            "* Target\n",
        ),
        (
            "attachment",
            "* Origin\n:PROPERTIES:\n:DIR: .\n:END:\n[[attachment:destination.org::*Target]]\n",
            "5",
            "* Target\n",
            "* Target\n",
        ),
    ] {
        let (fixture, mut session) = editor(
            "org.agenda.sources=['destination.org']",
            source,
            destination,
        );
        line(&mut session, row);
        let origin = session.view().point;
        action(&mut session, C::OpenLink);
        assert!(
            session.echo().contains("Org link destination"),
            "{name}: {}",
            session.echo()
        );
        assert!(
            text(&session)
                .chars()
                .skip(session.view().point.position.0)
                .collect::<String>()
                .starts_with(expected),
            "{name}"
        );
        assert!(session.buffer().unwrap().history().is_empty());
        action(&mut session, C::LinkBack);
        assert_eq!(session.view().point, origin, "{name}");
        assert_eq!(text(&session), source);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
            destination
        );
    }
    let (fixture, mut session) = editor("", "[[file:two%20notes-%E7%95%8C.org::2]]\n", "* Other\n");
    fixture.write("two notes-界.org", "* First\n* Second\n");
    action(&mut session, C::OpenLink);
    assert!(
        session.echo().contains("Org link destination"),
        "{}",
        session.echo()
    );
    assert_eq!(session.view().point.position.0, 8);
}
// Stored links feed the ordinary argument prompt. Accepted insertion/editing
// is one undo group; descriptions and selected source remain literal data.
#[test]
fn links_store_insert() {
    let (_fixture, mut session) = editor("", "* Target\nbody\n", "* Other\n");
    action(&mut session, C::StoreLink);
    assert!(session.echo().contains("stored"), "{}", session.echo());
    line(&mut session, "2");
    action(&mut session, C::InsertLink);
    assert!(
        session.echo().contains("literal link"),
        "{}",
        session.echo()
    );
    argument(&mut session, "");
    argument(&mut session, "");
    assert!(
        text(&session).contains("::*Target][Target]]body"),
        "{}",
        text(&session)
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "* Target\nbody\n");
    line(&mut session, "2");
    action(&mut session, C::InsertLink);
    argument(&mut session, "#custom");
    argument(&mut session, "Description 界");
    assert!(text(&session).contains("[[#custom][Description 界]]body"));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "* Target\nbody\n");
}
