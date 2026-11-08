use super::*;
use rev::command::{Motion, RegionCommand, Scroll};
fn open(session: &mut Session) {
    session.execute(Command::Macro(MacroCommand::List));
}
fn command(session: &mut Session, c: MacroCommand) {
    session.execute(Command::Macro(c));
}
fn rows(session: &mut Session) -> Vec<String> {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text().to_owned())
        .collect()
}
fn prompt_value(session: &mut Session, value: &str) {
    session.handle(control('e'));
    for _ in 0..16 {
        session.handle(key(KeyCode::Backspace));
    }
    feed(session, [Event::Paste(value.into()), key(KeyCode::Enter)]);
}

// The native component is read-only in both profiles and owns its local keys;
// q restores the original source without interpreting menu input as editing.
#[test]
fn focus() {
    for modal in [false, true] {
        let mut session = editor(
            if modal {
                "[editor]\ninteraction_profile='modal'\n"
            } else {
                ""
            },
            "",
        );
        record(
            &mut session,
            if modal {
                vec![character('i'), character('A'), key(KeyCode::Esc)]
            } else {
                vec![character('A')]
            },
        );
        feed(
            &mut session,
            [
                if modal { character(':') } else { meta('x') },
                Event::Paste("list-keyboard-macros".into()),
                key(KeyCode::Enter),
            ],
        );
        assert!(
            rows(&mut session)
                .iter()
                .any(|s| s.contains("Keyboard Macros"))
        );
        session.handle(Event::Paste("protected".into()));
        session.handle(character('z'));
        feed(
            &mut session,
            [
                character(':'),
                Event::Paste("kmacro-menu-help".into()),
                key(KeyCode::Enter),
            ],
        );
        assert!(session.echo().contains("transpose"));
        session.handle(character('q'));
        assert_eq!(text(&session), "A");
    }
}

// Counter/format/position changes target stable rows. Name/copy ownership is
// independent, and invalid property values cannot partially publish metadata.
#[test]
fn properties() {
    let mut session = editor("", "");
    record(&mut session, [key(KeyCode::F(3))]);
    feed(
        &mut session,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste("alpha".into()),
            key(KeyCode::Enter),
        ],
    );
    record(&mut session, [character('B')]);
    open(&mut session);
    session.handle(key(KeyCode::Down));
    session.handle(character('c'));
    prompt_value(&mut session, "9");
    session.handle(character('f'));
    prompt_value(&mut session, "%04d");
    let projected = rows(&mut session);
    assert!(
        projected.iter().any(|s| s.contains("0009")),
        "{} {:?}",
        session.echo(),
        projected
    );
    session.handle(character('f'));
    prompt_value(&mut session, "%129d");
    assert!(session.echo().contains("128"));
    session.handle(character('#'));
    prompt_value(&mut session, "0");
    assert!(
        rows(&mut session)
            .iter()
            .any(|s| s.starts_with('>') && s.contains("0009"))
    );
    session.handle(character('q'));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "0B0009");
    session.execute(Command::MacroNamed(
        rev::command::MacroName::new("alpha").unwrap(),
    ));
    settle(&mut session);
    assert_eq!(text(&session), "0B00091");
}

// Marks and flags address identities across transposition and refresh. Direct
// deletion requires fresh confirmation and never removes the named snapshot.
#[test]
fn stable_rows() {
    let mut session = editor("", "");
    record(&mut session, [character('A')]);
    feed(
        &mut session,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste("alpha".into()),
            key(KeyCode::Enter),
        ],
    );
    record(&mut session, [character('B')]);
    record(&mut session, [character('C')]);
    open(&mut session);
    session.handle(character('d'));
    session.handle(character('m'));
    session.handle(character('g'));
    feed(&mut session, [control('x'), control('t')]);
    assert!(rows(&mut session).iter().any(|s| s.contains('D')));
    session.handle(character('x'));
    session.handle(character('D'));
    session.handle(character('n'));
    session.handle(character('U'));
    assert!(rows(&mut session).iter().any(|s| s.contains("2 rows")));
    session.handle(character('D'));
    session.handle(character('y'));
    assert!(rows(&mut session).iter().any(|s| s.contains("1 rows")));
    session.handle(character('q'));
    session.execute(Command::MacroNamed(
        rev::command::MacroName::new("alpha").unwrap(),
    ));
    settle(&mut session);
    assert_eq!(text(&session), "ABCA");
}

// Region row selection precedes marks. Multiple-copy consent is checked before
// all mutations, preserving source, names and the ring when canceled.
#[test]
fn region_copy() {
    let mut session = editor("[macros]\nring_limit=8\n", "");
    for c in ['A', 'B', 'C'] {
        record(&mut session, [character(c)]);
    }
    open(&mut session);
    session.handle(character('m'));
    command(&mut session, MacroCommand::MenuUnmarkAll);
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Scroll(Scroll::NextLine));
    session.handle(character('C'));
    session.handle(character('n'));
    assert!(rows(&mut session).iter().any(|s| s.contains("3 rows")));
    session.handle(character('C'));
    session.handle(character('y'));
    assert!(rows(&mut session).iter().any(|s| s.contains("5 rows")));
    session.handle(key(KeyCode::Backspace));
    session.handle(character('u'));
    session.handle(character('U'));
    session.handle(character('q'));
    assert_eq!(text(&session), "ABC");
}

// The definition editor can be entered from the menu and returns to the same
// read-only component. Column selection routes Enter through native properties.
#[test]
fn editor_return() {
    let mut session = editor("", "");
    record(&mut session, [character('A')]);
    open(&mut session);
    session.handle(character('e'));
    let value = text(&session).replace("key = \"A\"", "key = \"B\"");
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, &value).unwrap();
    feed(&mut session, [control('c'), control('c')]);
    assert!(
        rows(&mut session)
            .iter()
            .any(|s| s.contains("Keyboard Macros"))
    );
    session.handle(character('g'));
    session.handle(key(KeyCode::Tab));
    session.handle(key(KeyCode::Enter));
    prompt_value(&mut session, "renamed");
    session.handle(character('q'));
    session.execute(Command::MacroNamed(
        rev::command::MacroName::new("renamed").unwrap(),
    ));
    settle(&mut session);
    assert_eq!(text(&session), "AB");
}
