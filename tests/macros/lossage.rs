use super::*;
fn steps(session: &Session) -> Vec<toml::Value> {
    let doc: toml::Value = toml::from_str(&text(session)).unwrap();
    doc["macros"][0]["steps"].as_array().unwrap().clone()
}
fn apply(session: &mut Session) {
    feed(session, [control('c'), control('c')]);
}

// Recent input is memory-only, bounded to the most recent 300 logical inputs.
// Creating a definition draft neither replays nor changes source history.
#[test]
fn recent_limit() {
    let mut session = editor("", "");
    for _ in 0..305 {
        session.handle(character('a'));
    }
    feed(&mut session, [control('x'), control('k'), character('l')]);
    assert_eq!(steps(&session).len(), 300);
    apply(&mut session);
    assert_eq!(text(&session).len(), 305);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session).len(), 605);
}

// Accepted M-x input contributes one canonical invocation, never its selector
// query/navigation. Replaying contributes no expanded events to the history.
#[test]
fn semantic_input() {
    let mut session = editor("", "");
    feed(
        &mut session,
        [
            character('A'),
            character('B'),
            meta('x'),
            Event::Paste("forward-char".into()),
            key(KeyCode::Enter),
        ],
    );
    session.execute(Command::Macro(MacroCommand::Lossage));
    let values = steps(&session);
    assert_eq!(values.len(), 3);
    assert_eq!(values[2]["name"].as_str(), Some("forward-char"));
    apply(&mut session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "ABAB");
    session.execute(Command::Macro(MacroCommand::Lossage));
    assert_eq!(steps(&session).len(), 3);
}

// Definition text, editor helpers and rejected oversized pastes cannot enter
// recent source history or replace its earlier valid logical input.
#[test]
fn protected_drafts() {
    let mut session = editor("", "");
    session.handle(character('A'));
    session.execute(Command::Macro(MacroCommand::Lossage));
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, "bad").unwrap();
    session.execute(Command::Buffer(rev::command::BufferCommand::Kill));
    session.handle(Event::Paste("B".repeat(300 * 1024)));
    session.execute(Command::Macro(MacroCommand::Lossage));
    assert_eq!(steps(&session).len(), 1);
    assert_eq!(steps(&session)[0]["key"].as_str(), Some("A"));
}
