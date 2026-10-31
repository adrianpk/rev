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
        (
            "#+TAGS: 界work\n* Item\u{2003}:界|",
            "",
            "界work:",
            "* Item\u{2003}:界work:",
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

// Literal blocks, ordinary prose, directive/property values and literal inline
// objects stay quiet even when general string/comment suppression is disabled.
#[test]
fn suppressed_contexts() {
    for document in [
        "ordinary TO| prose",
        "* Title ordinary TO| prose",
        "#+TITLE: TI|",
        ":OW|",
        "* One\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* Two\n:PROPERTIES:\n:OTHER: OW|\n:END:",
        "#+BEGIN_SRC rust\n* TO| item\n#+END_SRC",
        "#+BEGIN_EXAMPLE\n#+TI|\n#+END_EXAMPLE",
        "#+BEGIN_COMMENT\n[[fi|]]\n#+END_COMMENT",
        "#+BEGIN_EXPORT html\n[[fi|]]\n#+END_EXPORT",
        "#+BEGIN_SRC rust\n[[fi|",
        "=literal [[fi|]]=",
        "~literal [[fi|]]~",
        "# [[fi|]]",
        "[[file:notes.org][description [[fi|]]]]",
        "escaped \\[[fi|]]",
    ] {
        let (_fixture, mut session) = named_source(
            "source.org",
            document,
            "completion.in_comments = true\ncompletion.in_strings = true",
        );
        let original = text(&session);
        complete(&mut session);
        assert!(
            session.completion().is_none(),
            "{document}: {:?}",
            names(&session)
        );
        assert_eq!(text(&session), original);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Structural punctuation/heading spacing can open automatic completion without
// the normal prefix minimum. Ordinary prose keeps both popup and history quiet.
#[test]
fn automatic_org() {
    for (document, ch, expected) in [
        ("*|", ' ', "TODO"),
        ("#|", '+', "TITLE:"),
        ("[[|", '*', "Project"),
        ("* Project\n[|", '[', "file:"),
        ("#+TAGS: work\n* Item |", ':', "work:"),
        ("#+TAGS: work\n* Item :wo|", 'r', "work:"),
    ] {
        let document = if document == "[[|" {
            "* Project\n[[|"
        } else {
            document
        };
        let (_fixture, mut session) =
            named_source("source.org", document, "completion.delay_ms = 0");
        key(&mut session, KeyCode::Char(ch), KeyModifiers::NONE);
        settle(&mut session);
        assert!(
            names(&session).contains(&expected),
            "{document}: {:?}: {}",
            names(&session),
            session.echo()
        );
    }
    let (_fixture, mut session) =
        named_source("source.org", "ordinary TO|", "completion.delay_ms = 0");
    key(&mut session, KeyCode::Char('D'), KeyModifiers::NONE);
    settle(&mut session);
    assert!(session.completion().is_none());
}

// A source-edit draft uses its declared language through the same completion
// path. Returning publishes one source transaction; ordinary source undo restores it.
#[test]
fn source_draft() {
    let document = "#+BEGIN_SRC rust\nfn sample() {}\nfn main() { sam|; }\n#+END_SRC\n";
    let (_fixture, mut session) = named_source("source.org", document, "");
    let original = text(&session);
    session.execute(Command::Org(O::EditSpecial));
    settle(&mut session);
    assert!(session.source_edit_draft().is_some(), "{}", session.echo());
    assert_eq!(session.buffer().unwrap().mode(), "Org Src Rust");
    let point = text(&session).rfind("sam;").unwrap();
    at(&mut session, point);
    for _ in 0..3 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    complete(&mut session);
    choose(&mut session, "sample");
    key(&mut session, KeyCode::Tab, KeyModifiers::NONE);
    assert!(text(&session).contains("{ sample; }"));
    session.execute(Command::History(History::Undo));
    assert!(text(&session).contains("{ sam; }"));
    session.execute(Command::History(History::Redo));
    session.execute(Command::Org(O::EditSrcExit));
    settle(&mut session);
    assert!(session.source_edit_draft().is_none(), "{}", session.echo());
    assert_eq!(text(&session), original.replace("{ sam; }", "{ sample; }"));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), original);
}
fn at(session: &mut Session, position: usize) {
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..position {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
}

// Cancelled proposals and edits retire Org targets. Outside an active popup,
// Tab keeps its native visibility action without inserting a completion or tab.
#[test]
fn org_lifecycle() {
    let (_fixture, mut session) = named_source("source.org", "* TO| task\nbody\n", "");
    let original = text(&session);
    complete(&mut session);
    assert!(session.completion().is_some());
    key(&mut session, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(text(&session), original);
    key(&mut session, KeyCode::Tab, KeyModifiers::NONE);
    settle(&mut session);
    assert_eq!(text(&session), original);
    assert!(session.buffer().unwrap().history().is_empty());
    complete(&mut session);
    session.replace(0..0, "#+TITLE: changed\n").unwrap();
    let changed = text(&session);
    let history = session.buffer().unwrap().history().len();
    session.execute(Command::Completion(C::Accept));
    assert_eq!(text(&session), changed);
    assert_eq!(session.buffer().unwrap().history().len(), history);
}
