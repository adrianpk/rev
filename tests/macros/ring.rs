use super::*;

fn ring(session: &mut Session, key: char) {
    feed(session, [control('x'), control('k'), control(key)]);
}

// Rotation, exchange and second-entry execution address the global ring rather
// than a per-buffer last recording. Repeat controls leave selection explicit.
#[test]
fn selection() {
    let mut session = editor("", "");
    record(&mut session, [character('a')]);
    record(&mut session, [character('b')]);
    ring(&mut session, 'l');
    settle(&mut session);
    assert_eq!(text(&session), "aba");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "abab");
    ring(&mut session, 'n');
    session.handle(control('k'));
    settle(&mut session);
    assert_eq!(text(&session), "ababa");
    session.handle(control('t'));
    session.handle(control('k'));
    settle(&mut session);
    assert_eq!(text(&session), "ababab");
    session.handle(control('d'));
    session.handle(control('k'));
    settle(&mut session);
    assert_eq!(text(&session), "abababa");
    ring(&mut session, 'p');
    session.handle(control('k'));
    settle(&mut session);
    assert_eq!(text(&session), "abababaa");
    session.execute(Command::GenerateNewBuffer);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "a");
}

// A bare repeat consumes only its temporary gesture. The first unrelated
// input is dispatched normally and cannot leave a hidden repeat armed.
#[test]
fn repeat_exit() {
    let mut session = editor("", "");
    record(&mut session, [character('x')]);
    feed(&mut session, [control('x'), character('e')]);
    settle(&mut session);
    session.handle(character('e'));
    settle(&mut session);
    assert_eq!(text(&session), "xxx");
    feed(&mut session, [character('z'), character('e')]);
    assert_eq!(text(&session), "xxxze");
    ring(&mut session, 'n');
    feed(&mut session, [control('g'), control('d')]);
    assert_eq!(session.macro_stop(), None);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "xxxzex");
}

// The configured ceiling evicts only the oldest definition. Deleting the
// final entry leaves no runnable body; invalid config never activates.
#[test]
fn bounds() {
    for limit in [1, 2, 8, 32] {
        let mut session = editor(&format!("[macros]\nring_limit = {limit}\n"), "");
        for index in 0..=limit {
            record(
                &mut session,
                [character(char::from_u32(65 + index).unwrap())],
            );
        }
        for _ in 0..limit {
            ring(&mut session, 'd');
        }
        let before = text(&session);
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), before, "limit {limit}");
        assert!(session.echo().contains("No keyboard macro"));
    }
    for value in ["0", "33", "1.5", "true"] {
        let fixture = fixture::Fixture::new();
        let prepared = rev::config::prepare(
            &fixture.write("config.toml", &format!("[macros]\nring_limit = {value}\n")),
        );
        assert!(prepared.diagnostic.is_some(), "{value}");
    }
}

// Append replaces the selected body, including the preliminary execution
// policy. Double plain C-u always skips it and does not create a second entry.
#[test]
fn append_variants() {
    for (name, config, prefixes, before) in [
        ("plain", "", vec![control('u')], "aa"),
        ("double", "", vec![control('u'), control('u')], "a"),
        (
            "triple",
            "",
            vec![control('u'), control('u'), control('u')],
            "aa",
        ),
        ("numeric", "", vec![meta('2')], "aa"),
        (
            "disabled",
            "[macros]\nexecute_before_append = false\n",
            vec![control('u')],
            "a",
        ),
    ] {
        let mut session = editor(config, "");
        record(&mut session, [character('a')]);
        feed(&mut session, prefixes);
        session.handle(key(KeyCode::F(3)));
        settle(&mut session);
        assert_eq!(session.macro_status(), Status::Recording, "{name}");
        assert_eq!(text(&session), before, "{name}");
        feed(&mut session, [character('b'), key(KeyCode::F(4))]);
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), format!("{before}bab"), "{name}");
        ring(&mut session, 'd');
        let before = text(&session);
        session.handle(key(KeyCode::F(4)));
        assert_eq!(text(&session), before, "{name}");
    }
}

// A failing preliminary execution keeps the old body and its real partial
// undo unit. Failed/empty append capture cannot publish a partial replacement.
#[test]
fn append_failure() {
    let mut session = editor("", "a");
    record(&mut session, [control('d')]);
    feed(&mut session, [control('u'), key(KeyCode::F(3))]);
    settle(&mut session);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
    session.replace(0..0, "z").unwrap();
    session.handle(control('a'));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "");
    let mut partial = editor("", "a");
    record(&mut partial, [character('x'), control('d')]);
    feed(&mut partial, [control('u'), key(KeyCode::F(3))]);
    settle(&mut partial);
    assert_eq!(partial.macro_status(), Status::Idle);
    assert_eq!(
        partial.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
    assert_eq!(text(&partial), "xx");
    partial.execute(Command::History(History::Undo));
    assert_eq!(text(&partial), "x");
    for (name, events) in [
        ("empty", vec![key(KeyCode::F(4))]),
        ("cancel", vec![character('z'), control('g')]),
        ("oversized", vec![Event::Paste("z".repeat(256 * 1024))]),
    ] {
        let mut session = editor("", "");
        record(&mut session, [character('a')]);
        feed(
            &mut session,
            [control('u'), control('u'), key(KeyCode::F(3))],
        );
        feed(&mut session, events);
        let before = text(&session);
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), format!("{before}a"), "{name}");
    }
}

// Includes merge these two leaves independently; the packaged sample has the
// same defaults and accepts every supplied command/key identity.
#[test]
fn configuration() {
    let fixture = fixture::Fixture::new();
    fixture.write(
        "base.toml",
        "[macros]\nring_limit = 2\nexecute_before_append = false\n",
    );
    let prepared = rev::config::prepare(&fixture.write(
        "config.toml",
        "include = ['base.toml']\n[macros]\nring_limit = 3\n",
    ));
    assert!(prepared.diagnostic.is_none());
    let settings = &prepared.snapshot.settings().macros;
    assert_eq!(settings.ring_limit, 3);
    assert!(!settings.execute_before_append);
    let prepared =
        rev::config::prepare(&fixture.write("sample.toml", rev::config::default_template()));
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    assert_eq!(
        prepared.snapshot.settings().macros,
        rev::config::Macros::default()
    );
}

// Polling an idle macro owner cannot close ordinary typing history. A real
// playback still separates its own iteration from adjacent manual input.
#[test]
fn idle_history() {
    let mut session = editor("", "");
    session.handle(character('a'));
    session.expire_input(Instant::now());
    session.handle(character('b'));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
    record(&mut session, [character('x')]);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    session.handle(character('z'));
    session.expire_input(Instant::now());
    session.handle(character('y'));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "xx");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "x");
}

// Nonpositive counts repeat forward until the native edit boundary signals
// failure. Saturated successful motion remains cancellable between batches.
#[test]
fn nonpositive() {
    for prefix in ['0', '-'] {
        let mut session = editor("", "abcd");
        record(&mut session, [control('d')]);
        feed(&mut session, [meta(prefix), key(KeyCode::F(4))]);
        settle(&mut session);
        assert_eq!(text(&session), "");
        assert_eq!(
            session.macro_stop().unwrap().reason,
            StopReason::NativeFailure
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "d");
    }
    let mut session = editor("", "");
    record(&mut session, [control('f')]);
    feed(&mut session, [meta('0'), key(KeyCode::F(4))]);
    session.expire_input(Instant::now());
    assert_eq!(session.macro_status(), Status::Playing);
    session.handle(control('g'));
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
    assert_eq!(session.remaining_input(Instant::now()), None);
}
