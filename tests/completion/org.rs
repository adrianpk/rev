use super::*;
use rev::command::OrgCommand as O;
fn names(session: &Session) -> Vec<&str> {
    session
        .completion()
        .map(|completion| {
            completion
                .candidates()
                .iter()
                .map(|candidate| candidate.text())
                .collect()
        })
        .unwrap_or_default()
}
fn choose(session: &mut Session, expected: &str) {
    let index = names(session)
        .iter()
        .position(|name| *name == expected)
        .unwrap_or_else(|| {
            panic!(
                "missing {expected}: {:?}: {}",
                names(session),
                session.echo()
            )
        });
    for _ in 0..index {
        key(session, KeyCode::Down, KeyModifiers::NONE);
    }
}

// Native task workflows, observed tags/properties and finite directives/link
// types share the popup and checked edit owner, including missing delimiters.
#[test]
fn document_contexts() {
    for (document, config, expected, changed) in [
        ("* TO| item", "", "TODO", "* TODO item"),
        (
            "* PL| item",
            "org.todo.sequence = ['PLAN', 'DONE']",
            "PLAN",
            "* PLAN item",
        ),
        (
            "#+TODO: NEXT(n) WAIT(w) FINISHED(f)\n* NE| item",
            "",
            "NEXT",
            "* NEXT item",
        ),
        (
            "#+TAGS: work(w) home(h)\n* Item :wo|",
            "",
            "work:",
            "* Item :work:",
        ),
        ("* Other :work:\n* Item :wo|:", "", "work", "* Item :work:"),
        (
            "* One\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* Two\n:PROPERTIES:\n:OW|\n:END:\n",
            "",
            "OWNER:",
            ":OWNER:\n:END:",
        ),
        (
            "* One\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* Two\n:PROPERTIES:\n:OW|: Bob\n:END:\n",
            "",
            "OWNER",
            ":OWNER: Bob",
        ),
        ("#+TI|", "", "TITLE:", "#+TITLE:"),
        ("#+BEGIN_S|", "", "BEGIN_SRC", "#+BEGIN_SRC"),
        ("See [[fi|]]", "", "file:", "[[file:]]"),
        ("See [[fi|:notes.org]]", "", "file", "[[file:notes.org]]"),
    ] {
        let (_fixture, mut session) = named_source("source.org", document, config);
        let original = text(&session);
        complete(&mut session);
        choose(&mut session, expected);
        assert_eq!(text(&session), original);
        assert!(session.buffer().unwrap().history().is_empty());
        key(&mut session, KeyCode::Tab, KeyModifiers::NONE);
        assert!(
            text(&session).contains(changed),
            "{document}: {}",
            text(&session)
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), original);
    }
}

// Heading targets strip native TODO/priority/tag metadata and encode literal
// link bytes. The current unsaved document supplies targets, not other files.
#[test]
fn heading_targets() {
    let (fixture, mut session) = named_source(
        "source.org",
        "* TODO [#A] Project Alpha :work:\nSee [[*Pro|]]",
        "",
    );
    let original = text(&session);
    let other = fixture.write("other.org", "* Project Foreign\n");
    session.load(&other).unwrap();
    session.load(&fixture.0.join("source.org")).unwrap();
    at(
        &mut session,
        "* TODO [#A] Project Alpha :work:\nSee [[*Pro"
            .chars()
            .count(),
    );
    complete(&mut session);
    assert!(!names(&session).contains(&"Project%20Foreign"));
    assert!(
        session
            .completion()
            .unwrap()
            .candidates()
            .iter()
            .any(|candidate| candidate.label().starts_with("Project Alpha  heading"))
    );
    choose(&mut session, "Project%20Alpha");
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert!(text(&session).ends_with("[[*Project%20Alpha]]"));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), original);
    session.execute(Command::Motion(Motion::BufferEnd));
    session.handle(Event::Paste("\n* Unsaved Target\n".into()));
    assert!(
        !std::fs::read_to_string(fixture.0.join("source.org"))
            .unwrap()
            .contains("Unsaved Target")
    );
    session.execute(Command::Motion(Motion::BufferEnd));
    session.handle(Event::Paste("See [[*Uns".into()));
    complete(&mut session);
    assert!(names(&session).contains(&"Unsaved%20Target"));
}
