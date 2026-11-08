use super::*;
use rev::macros::Wait;

fn query(session: &mut Session, edit: bool) {
    if edit {
        session.handle(meta('4'));
    }
    feed(session, [control('x'), character('q')]);
}
fn resume(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    )));
}
fn definition(session: &mut Session) {
    session.handle(key(KeyCode::F(3)));
    session.handle(character('a'));
    query(session, false);
    feed(session, [character('b'), key(KeyCode::F(4))]);
}

// Every query answer is owned by the live prompt. Confirmation controls do
// not become source text or recorded answers, and waiting schedules no work.
#[test]
fn choices() {
    for (name, answer, expected) in [
        ("space", character(' '), "abab"),
        ("yes", character('y'), "abab"),
        ("backspace", key(KeyCode::Backspace), "aba"),
        ("no", character('n'), "aba"),
        ("return", key(KeyCode::Enter), "aba"),
        ("quit", character('q'), "aba"),
    ] {
        let mut session = editor("", "");
        definition(&mut session);
        assert_eq!(text(&session), "ab");
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(session.macro_wait(), Some(Wait::Query), "{name}");
        assert_eq!(session.remaining_input(Instant::now()), None, "{name}");
        assert_eq!(text(&session), "aba");
        session.handle(answer);
        settle(&mut session);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.macro_status(), Status::Idle, "{name}");
        assert_eq!(session.macro_stop(), None, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "ab", "{name}");
    }
}

// Skip closes only this partial iteration; stop closes all remaining ones.
// Completed and partial iterations retain separate native undo units.
#[test]
fn iterations() {
    let mut session = editor("", "");
    definition(&mut session);
    feed(&mut session, [meta('3'), key(KeyCode::F(4))]);
    settle(&mut session);
    session.handle(character('y'));
    settle(&mut session);
    assert_eq!(text(&session), "ababa");
    session.handle(character('n'));
    settle(&mut session);
    assert_eq!(text(&session), "ababaa");
    session.handle(character('q'));
    settle(&mut session);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ababa");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "abab");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ab");
}

// C-r suspends a query for ordinary editing. C-M-c returns to the same query;
// manual source effects stay in the outer iteration and are never replayed.
#[test]
fn query_edit() {
    let mut session = editor("", "");
    definition(&mut session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    session.handle(control('r'));
    assert_eq!(session.macro_wait(), Some(Wait::Editing));
    session.handle(character('M'));
    resume(&mut session);
    assert_eq!(session.macro_wait(), Some(Wait::Query));
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.handle(character('y'));
    settle(&mut session);
    assert_eq!(text(&session), "abaMb");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ab");
}

// Prefixed queries enter manual editing during recording and playback. Only
// the marker and automatic input survive capture, in both entry profiles.
#[test]
fn capture_pause() {
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
        session.handle(key(KeyCode::F(3)));
        session.handle(character('a'));
        query(&mut session, true);
        assert_eq!(session.macro_wait(), Some(Wait::Editing));
        session.handle(character('M'));
        resume(&mut session);
        feed(&mut session, [character('b'), key(KeyCode::F(4))]);
        assert_eq!(text(&session), "aMb");
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(session.macro_wait(), Some(Wait::Editing));
        assert_eq!(text(&session), "aMba");
        session.handle(character('N'));
        resume(&mut session);
        settle(&mut session);
        assert_eq!(text(&session), "aMbaNb");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "aMb");
    }
}

// Cancellation releases every suspended owner, retains real partial source
// effects and leaves the previously valid recording available after abort.
#[test]
fn abort() {
    for cancel in [control(']'), control('g')] {
        let mut session = editor("", "");
        record(&mut session, [character('Z')]);
        session.handle(key(KeyCode::F(3)));
        session.handle(character('a'));
        query(&mut session, true);
        feed(&mut session, [character('M'), cancel]);
        assert_eq!(session.macro_status(), Status::Idle);
        assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
        assert_eq!(session.remaining_input(Instant::now()), None);
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), "ZaMZ");
    }
    let mut session = editor("", "");
    definition(&mut session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    session.handle(control('g'));
    assert_eq!(text(&session), "aba");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ab");
}

// Confirmation repeats/releases and pasted answers cannot accept the query.
// A source mutation outside the native pause input invalidates its continuation.
#[test]
fn ownership() {
    let mut session = editor("", "");
    definition(&mut session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
        session.handle(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
            kind,
        )));
    }
    session.handle(Event::Paste("y".into()));
    assert_eq!(session.macro_wait(), Some(Wait::Query));
    session.replace(3..3, "external").unwrap();
    session.handle(character('y'));
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::OwnerChanged
    );
    assert_eq!(text(&session), "abaexternal");
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    query(&mut session, true);
    session.replace(0..0, "external").unwrap();
    resume(&mut session);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::OwnerChanged
    );
    assert_eq!(session.macro_status(), Status::Idle);
}

// Manual replay may itself pause, but cannot reset outer work limits or grow
// past four suspended frames. Overflow aborts all owned frames without a timer.
#[test]
fn nesting() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    query(&mut session, true);
    resume(&mut session);
    session.handle(key(KeyCode::F(4)));
    for _ in 0..4 {
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(session.macro_wait(), Some(Wait::Editing));
    }
    for remaining in (0..4).rev() {
        resume(&mut session);
        settle(&mut session);
        assert_eq!(
            session.macro_wait(),
            (remaining > 0).then_some(Wait::Editing)
        );
    }
    assert_eq!(session.macro_status(), Status::Idle);
    for _ in 0..4 {
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
    }
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::PauseLimit);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// The pause-local abort gesture appears through the real M-x hint resolver.
// Accepted naming still reaches the scoped native abort handler exactly once.
#[test]
fn pause_hints() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    query(&mut session, true);
    feed(
        &mut session,
        [meta('x'), Event::Paste("abort-recursive-edit".into())],
    );
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("C-]"))
    );
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
}

// Native counter edits during a pause update its suspended working state.
// The manual assignment is not added to the reusable automatic definition.
#[test]
fn counter_edit() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(3)));
    query(&mut session, false);
    feed(&mut session, [key(KeyCode::F(3)), key(KeyCode::F(4))]);
    assert_eq!(text(&session), "01");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    session.handle(control('r'));
    feed(
        &mut session,
        [meta('9'), control('x'), control('k'), control('c')],
    );
    resume(&mut session);
    session.handle(character('y'));
    settle(&mut session);
    assert_eq!(text(&session), "0129");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    session.handle(character('y'));
    settle(&mut session);
    assert_eq!(text(&session), "01291011");
}
