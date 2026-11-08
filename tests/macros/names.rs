use super::*;
use rev::command::MacroName;

fn confirming(session: &mut Session) -> bool {
    session
        .frame()
        .unwrap()
        .cursor()
        .is_some_and(|position| position.row == 15)
}

fn name(session: &mut Session, value: &str) {
    feed(
        session,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste(value.into()),
            key(KeyCode::Enter),
        ],
    );
}
fn call(session: &mut Session, value: &str) {
    feed(
        session,
        [meta('x'), Event::Paste(value.into()), key(KeyCode::Enter)],
    );
    settle(session);
}
fn bind(session: &mut Session, events: impl IntoIterator<Item = Event>) {
    feed(session, [control('x'), control('k'), character('b')]);
    feed(session, events);
}

// Native key capture displays its prefix and ignores physical repeats. Its
// normalized chords remain data until a complete binding has been admitted.
#[test]
fn key_capture() {
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    name(&mut session, "alpha");
    bind(&mut session, []);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("Bind macro key:"))
    );
    for _ in 0..20 {
        session.handle(key(KeyCode::F(0)));
    }
    session.handle(control('x'));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("C-x"))
    );
    session.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Char('k'),
        KeyModifiers::CONTROL,
        KeyEventKind::Repeat,
    )));
    feed(&mut session, [control('k'), character('8')]);
    assert_eq!(text(&session), "X");
    feed(&mut session, [control('x'), control('k'), character('8')]);
    settle(&mut session);
    assert_eq!(text(&session), "XX");
}

// A physical key calling a named macro records its canonical name once. Later
// key replacement changes physical dispatch without retargeting that recording.
#[test]
fn bound_capture() {
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    name(&mut session, "part");
    bind(&mut session, [character('7')]);
    session.handle(key(KeyCode::F(3)));
    feed(&mut session, [control('x'), control('k'), character('7')]);
    settle(&mut session);
    session.handle(key(KeyCode::F(4)));
    name(&mut session, "whole");
    record(&mut session, [character('B')]);
    name(&mut session, "other");
    bind(&mut session, [character('7')]);
    session.handle(character('y'));
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, "").unwrap();
    call(&mut session, "whole");
    assert_eq!(text(&session), "X");
    feed(&mut session, [control('x'), control('k'), character('7')]);
    settle(&mut session);
    assert_eq!(text(&session), "XB");
}
fn resume(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    )));
    settle(session);
}

// Session names retain independent bodies and counters after ring eviction;
// native undo removes one complete named playback iteration.
#[test]
fn snapshots() {
    let mut session = editor("[macros]\nring_limit=1\n", "");
    record(&mut session, [character('A')]);
    name(&mut session, "alpha");
    record(&mut session, [character('B')]);
    name(&mut session, "bravo");
    call(&mut session, "alpha");
    assert_eq!(text(&session), "ABA");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "AB");
    call(&mut session, "bravo");
    assert_eq!(text(&session), "ABB");
    let mut session = editor("", "");
    record(&mut session, [key(KeyCode::F(3))]);
    name(&mut session, "count-item");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    call(&mut session, "count-item");
    call(&mut session, "count-item");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "01122");
}

// Both native selectors admit the same session identity and typed positive
// prefix. Modal Normal entry remains distinct from Insert entry.
#[test]
fn profiles() {
    for modal in [false, true] {
        let mut session = editor(
            if modal {
                "[editor]\ninteraction_profile='modal'\n"
            } else {
                ""
            },
            "",
        );
        let events = if modal {
            vec![character('i'), character('X'), key(KeyCode::Esc)]
        } else {
            vec![character('X')]
        };
        record(&mut session, events);
        name(&mut session, "add-item");
        feed(
            &mut session,
            [
                meta('3'),
                if modal { character(':') } else { meta('x') },
                Event::Paste("add-item".into()),
                key(KeyCode::Enter),
            ],
        );
        settle(&mut session);
        assert_eq!(text(&session), "XXXX", "modal={modal}");
        assert_eq!(session.macro_status(), Status::Idle);
    }
}

// Replacing a name needs a fresh native decision. Repeats, releases and pasted
// answers cannot consent, and a declined replacement keeps the old snapshot.
#[test]
fn replacement() {
    let mut session = editor("", "");
    record(&mut session, [character('A')]);
    name(&mut session, "alpha");
    record(&mut session, [character('B')]);
    name(&mut session, "alpha");
    assert!(confirming(&mut session));
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        session.handle(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
            kind,
        )));
    }
    session.handle(Event::Paste("y".into()));
    assert!(confirming(&mut session));
    session.handle(character('n'));
    call(&mut session, "alpha");
    assert_eq!(text(&session), "ABA");
    name(&mut session, "alpha");
    session.handle(character('y'));
    call(&mut session, "alpha");
    assert_eq!(text(&session), "ABAB");
}

// Syntax and compiled/prompt/AI collisions cannot enter the session namespace.
// An absent named identity stops explicitly without launching any provider.
#[test]
fn collisions() {
    let config = "[ai.commands.review]\nprompt='Review text.'\ninput='buffer'\ncontext='input'\noutput='explanation'\n";
    for invalid in [
        "",
        "Bad",
        "1bad",
        "bad_name",
        "newline",
        "minibuffer-accept",
        "ai-review",
    ] {
        let mut session = editor(config, "");
        record(&mut session, [character('X')]);
        name(&mut session, invalid);
        assert_eq!(text(&session), "X", "{invalid}");
        assert!(!confirming(&mut session), "{invalid}");
        if MacroName::new(invalid).is_ok() {
            session.execute(Command::MacroNamed(MacroName::new(invalid).unwrap()));
            assert_eq!(
                session.macro_stop().unwrap().reason,
                StopReason::MissingCommand
            );
        }
    }
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    session.execute(Command::Macro(MacroCommand::Name));
    session.replace(1..1, "external").unwrap();
    feed(
        &mut session,
        [Event::Paste("stale-name".into()), key(KeyCode::Enter)],
    );
    session.execute(Command::MacroNamed(MacroName::new("stale-name").unwrap()));
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::MissingCommand
    );
    assert_eq!(text(&session), "Xexternal");
}

// Single digits and uppercase letters expand to reserved slots in both
// profiles; binding an unnamed recording snapshots it independently as well.
#[test]
fn reserved_slots() {
    for modal in [false, true] {
        let mut session = editor(
            if modal {
                "[editor]\ninteraction_profile='modal'\n"
            } else {
                ""
            },
            "",
        );
        if modal {
            session.handle(character('i'));
        }
        record(&mut session, [character('X')]);
        name(&mut session, "alpha");
        bind(&mut session, [character('7')]);
        assert!(!confirming(&mut session));
        feed(&mut session, [control('x'), control('k'), character('7')]);
        settle(&mut session);
        assert_eq!(text(&session), "XX");
        record(&mut session, [character('Y')]);
        bind(&mut session, [character('Q')]);
        record(&mut session, [character('Z')]);
        feed(&mut session, [control('x'), control('k'), character('Q')]);
        settle(&mut session);
        assert_eq!(text(&session), "XXYZY");
    }
}

// Occupied source keys require consent and expose effective hints. The
// underlying configured mapping and prompt-owned keys remain intact.
#[test]
fn binding_consent() {
    let mut session = editor("[keys.editor]\n'C-f'='backward-char'\n", "");
    record(&mut session, [character('X')]);
    name(&mut session, "alpha");
    bind(&mut session, [control('f')]);
    assert!(confirming(&mut session));
    session.handle(character('n'));
    session.handle(control('f'));
    assert_eq!(session.view().point.position.0, 0);
    bind(&mut session, [control('f')]);
    session.handle(character('y'));
    feed(&mut session, [meta('x'), Event::Paste("alpha".into())]);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("C-f"))
    );
    session.handle(control('f'));
    assert_eq!(text(&session), "X");
    session.handle(control('g'));
    session.handle(control('f'));
    settle(&mut session);
    assert_eq!(text(&session), "XX");
}

// The source layer cannot claim cancellation or native prefix arguments;
// canceled key capture leaves the source and previously valid bindings alone.
#[test]
fn protected_keys() {
    for event in [
        control('g'),
        control(']'),
        control('u'),
        meta('7'),
        meta('-'),
    ] {
        let mut session = editor("", "");
        record(&mut session, [character('X')]);
        bind(&mut session, [event]);
        assert!(!confirming(&mut session));
        assert_eq!(text(&session), "X");
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
}

// Captured named calls retain their canonical name, resolve its current
// snapshot on admission and keep all nested effects in the outer undo unit.
#[test]
fn nested_calls() {
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    name(&mut session, "part");
    session.handle(key(KeyCode::F(3)));
    session.handle(character('Y'));
    call(&mut session, "part");
    feed(&mut session, [character('Z'), key(KeyCode::F(4))]);
    name(&mut session, "whole");
    record(&mut session, [character('B')]);
    name(&mut session, "part");
    session.handle(character('y'));
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, "").unwrap();
    call(&mut session, "whole");
    assert_eq!(text(&session), "YBZ");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
}

// Replacing a paused called name affects its next admission. The already
// admitted body remains pinned through its native manual editing continuation.
#[test]
fn pinned_call() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    session.handle(character('X'));
    feed(&mut session, [meta('4'), control('x'), character('q')]);
    resume(&mut session);
    feed(&mut session, [character('Y'), key(KeyCode::F(4))]);
    name(&mut session, "alpha");
    record(&mut session, [character('B')]);
    call(&mut session, "alpha");
    assert_eq!(session.macro_wait(), Some(rev::macros::Wait::Editing));
    name(&mut session, "alpha");
    session.handle(character('y'));
    resume(&mut session);
    assert_eq!(text(&session), "XYBXY");
    call(&mut session, "alpha");
    assert_eq!(text(&session), "XYBXYB");
}

// A session replacement can create a self-call, which must stop at admission
// before child effects. The finite depth admits sixteen pinned frames only.
#[test]
fn call_bounds() {
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    name(&mut session, "alpha");
    session.handle(key(KeyCode::F(3)));
    call(&mut session, "alpha");
    session.handle(key(KeyCode::F(4)));
    name(&mut session, "alpha");
    session.handle(character('y'));
    let before = text(&session);
    call(&mut session, "alpha");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::CallCycle);
    assert_eq!(text(&session), before);
    let mut session = editor("", "");
    record(&mut session, [character('X')]);
    name(&mut session, "chain-1");
    for index in 2..=17 {
        session.handle(key(KeyCode::F(3)));
        call(&mut session, &format!("chain-{}", index - 1));
        session.handle(key(KeyCode::F(4)));
        name(&mut session, &format!("chain-{index}"));
    }
    let before = text(&session);
    call(&mut session, "chain-16");
    assert_eq!(text(&session), before.clone() + "X");
    call(&mut session, "chain-17");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::CallDepth);
    assert_eq!(text(&session), before + "X");
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// One accepted user name contributes one ranking event. Expanded native calls
// do not add usage or selector history, and a fresh session has no named state.
#[test]
fn command_history() {
    let fixture = fixture::Fixture::new();
    let path = fixture.0.join("command-history.toml");
    let mut session = editor("", "").with_command_usage(rev::command::usage::History::open(&path));
    record(&mut session, [character('X')]);
    name(&mut session, "alpha");
    session.handle(key(KeyCode::F(3)));
    call(&mut session, "alpha");
    session.handle(key(KeyCode::F(4)));
    name(&mut session, "bravo");
    feed(&mut session, [meta('3')]);
    call(&mut session, "bravo");
    session.flush_command_usage().unwrap();
    let value: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        value["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect::<Vec<_>>(),
        ["alpha", "bravo"]
    );
    feed(&mut session, [meta('x'), Event::Paste("a".into())]);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("alpha"))
    );
    assert!(!fixture.0.join("macros").exists());
    let mut fresh = editor("", "");
    fresh.execute(Command::MacroNamed(MacroName::new("alpha").unwrap()));
    assert_eq!(
        fresh.macro_stop().unwrap().reason,
        StopReason::MissingCommand
    );
    assert_eq!(text(&fresh), "");
}
