use super::*;
use rev::command::{BufferCommand, CompletionCommand, Motion};
use rev::coordinates::Scalar;
use rev::macros::Wait;

fn named(session: &mut Session, selector: Event, name: &str) {
    feed(
        session,
        [selector, Event::Paste(name.into()), key(KeyCode::Enter)],
    );
}

// Command selectors retain the resolved identity and typed count once. Modal
// colon and M-x converge on the native handler without replaying query text.
#[test]
fn named_prefixes() {
    for (config, selector, modal, expected) in [
        ("", meta('x'), false, "abXcdXef"),
        (
            "[editor]\ninteraction_profile = 'modal'\n",
            character(':'),
            true,
            "abXcdXef",
        ),
    ] {
        let mut session = editor(config, "abcdef");
        session.handle(key(KeyCode::F(3)));
        session.handle(meta('2'));
        named(&mut session, selector, "forward-char");
        assert_eq!(session.view().point.position, Scalar(2));
        if modal {
            session.handle(character('i'));
        }
        session.handle(character('X'));
        if modal {
            session.handle(key(KeyCode::Esc));
        }
        session.handle(key(KeyCode::F(4)));
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), expected);
        assert_eq!(session.macro_stop(), None);
    }
}

// A goto argument is captured only when its native owner accepts it. Rejected
// drafts and prompt editing do not become source text or consume another count.
#[test]
fn accepted_line() {
    let mut session = editor("", "one\ntwo\nthree\n");
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "goto-line");
    feed(
        &mut session,
        [Event::Paste("wrong".into()), key(KeyCode::Enter)],
    );
    feed(
        &mut session,
        [
            control('a'),
            control('d'),
            control('d'),
            control('d'),
            control('d'),
            control('d'),
            character('2'),
            key(KeyCode::Enter),
            character('>'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    session.execute(Command::Motion(Motion::BufferStart));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "one\n>>two\nthree\n");
    assert_eq!(session.macro_stop(), None);
}

// Buffer candidate navigation is resolved to an accepted literal name. The
// definition returns to its source and preserves buffer-local native history.
#[test]
fn buffer_names() {
    for inventory in [false, true] {
        let mut buffers = Buffers::new().unwrap();
        let first = buffers.create("source").unwrap();
        let second = buffers.create("target").unwrap();
        let first_name = buffers.get(first).unwrap().name().to_owned();
        let second_name = buffers.get(second).unwrap().name().to_owned();
        let view = View::new(buffers.get(first).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 16,
            },
        )
        .unwrap();
        session.handle(key(KeyCode::F(3)));
        for name in [&second_name, &first_name] {
            if inventory {
                feed(
                    &mut session,
                    [
                        control('x'),
                        control('b'),
                        key(if name == &second_name {
                            KeyCode::Down
                        } else {
                            KeyCode::Up
                        }),
                        key(KeyCode::Enter),
                    ],
                );
            } else {
                feed(
                    &mut session,
                    [
                        control('x'),
                        character('b'),
                        Event::Paste(name.clone()),
                        key(KeyCode::Tab),
                        key(KeyCode::Enter),
                    ],
                );
            }
            if name == &second_name {
                session.handle(character('X'));
            }
        }
        session.handle(key(KeyCode::F(4)));
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), "source");
        session.execute(Command::Buffer(BufferCommand::Switch));
        feed(
            &mut session,
            [Event::Paste(second_name), key(KeyCode::Enter)],
        );
        assert_eq!(text(&session), "XXtarget");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "Xtarget");
    }
}

// File acceptance reuses native ingress and returns to the original source.
// Replays visit the literal accepted path instead of a completion row ordinal.
#[test]
fn file_paths() {
    let fixture = fixture::Fixture::new();
    let first = fixture.write("first.txt", "source");
    let second = fixture.write("second.txt", "target");
    let mut session = editor("", "");
    session.load(&first).unwrap();
    session.handle(key(KeyCode::F(3)));
    for path in [&second, &first] {
        feed(
            &mut session,
            [
                control('x'),
                control('f'),
                Event::Paste(path.to_str().unwrap().into()),
                key(KeyCode::Enter),
            ],
        );
        if path == &second {
            session.handle(character('X'));
        }
    }
    session.handle(key(KeyCode::F(4)));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "source");
    session.load(&second).unwrap();
    assert_eq!(text(&session), "XXtarget");
    assert_eq!(std::fs::read_to_string(second).unwrap(), "target");
}

// Accepted literal searches retain their native continuation semantics. A
// failed replay is a typed stop before the following source transformation.
#[test]
fn search_literals() {
    let mut session = editor("", "hit hit\nhit hit\n");
    session.handle(key(KeyCode::F(3)));
    feed(
        &mut session,
        [
            control('s'),
            Event::Paste("hit".into()),
            control('s'),
            key(KeyCode::Enter),
            character('X'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    session.execute(Command::Motion(Motion::BufferStart));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "hit hitXX\nhit hit\n");
    session.execute(Command::Motion(Motion::BufferEnd));
    let before = text(&session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
}

// Consent is never retained as a replayable argument. A paused replacement has
// no automatic wake and each answer stays with its native owner and undo unit.
#[test]
fn fresh_consent() {
    let mut session = editor("", "cat cat");
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "query-replace");
    feed(
        &mut session,
        [
            Event::Paste("cat".into()),
            key(KeyCode::Enter),
            Event::Paste("dog".into()),
            key(KeyCode::Enter),
            character('y'),
            character('y'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    assert_eq!(text(&session), "dog dog");
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    session.execute(Command::Motion(Motion::BufferStart));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(session.macro_wait(), Some(Wait::Consent));
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(text(&session), "cat cat");
    session.handle(character('n'));
    settle(&mut session);
    assert_eq!(session.macro_wait(), Some(Wait::Consent));
    assert_eq!(text(&session), "cat cat");
    session.handle(character('y'));
    settle(&mut session);
    assert_eq!(text(&session), "cat dog");
    assert_eq!(session.macro_status(), Status::Idle);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "cat cat");
}

// A manual completion becomes a checked literal edit, independent of providers
// and popup order. Changed token text refuses replay without another edit.
#[test]
fn literal_completion() {
    let fixture = fixture::Fixture::new();
    let source = "fn config() {}\nfn cfg_value() {}\nfn main() { cfg }";
    let path = fixture.write("source.rs", source);
    let mut session = editor("", "");
    session.load(&path).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Motion(Motion::BackwardChar));
    session.execute(Command::Motion(Motion::BackwardChar));
    session.handle(key(KeyCode::F(3)));
    session.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::ALT)));
    let deadline = Instant::now() + Duration::from_secs(5);
    while session.completion().is_none() {
        assert!(Instant::now() < deadline, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(1));
    }
    session.handle(key(KeyCode::Down));
    session.handle(key(KeyCode::Enter));
    session.handle(key(KeyCode::F(4)));
    let completed = text(&session);
    assert_eq!(completed, source.replace("{ cfg }", "{ config }"));
    session.execute(Command::History(History::Undo));
    session.execute(Command::Completion(CompletionCommand::Mode));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), completed);
    session.execute(Command::History(History::Undo));
    let point = session.view().point.position.0;
    session.replace(point - 3..point, "bad").unwrap();
    let before = text(&session);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), before);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
}

// An accepted asynchronous command is rejected before launching a process or
// acquiring its prompt; selector activity never replaces the previous macro.
#[test]
fn unsupported_named_job() {
    let mut session = editor("", "");
    record(&mut session, [character('x')]);
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "shell-command");
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::Unsupported
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "xx");
}

// Only user acceptance contributes usage. Repeated expanded commands and their
// source effects do not create extra ranking or persisted history events.
#[test]
fn single_usage_event() {
    let fixture = fixture::Fixture::new();
    let path = fixture.0.join("command-history.toml");
    let history = rev::command::usage::History::open(&path);
    let mut session = editor("", "").with_command_usage(history);
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "newline");
    session.handle(key(KeyCode::F(4)));
    feed(&mut session, [meta('3'), key(KeyCode::F(4))]);
    settle(&mut session);
    assert_eq!(text(&session), "\n\n\n\n");
    session.flush_command_usage().unwrap();
    let stored: toml::Value = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        stored["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| name.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["newline"]
    );
}

// Overwrite consent comes from the live destination on every invocation. The
// recorded yes cannot authorize replacing later external contents.
#[test]
fn save_consent() {
    let fixture = fixture::Fixture::new();
    let first = fixture.write("first.txt", "source");
    let second = fixture.write("second.txt", "old target");
    let mut session = editor("", "");
    session.load(&first).unwrap();
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "write-file");
    feed(
        &mut session,
        [
            Event::Paste(second.to_str().unwrap().into()),
            key(KeyCode::Enter),
            character('y'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    assert_eq!(std::fs::read_to_string(&second).unwrap(), "source");
    named(&mut session, meta('x'), "write-file");
    feed(
        &mut session,
        [
            Event::Paste(first.to_str().unwrap().into()),
            key(KeyCode::Enter),
            character('y'),
        ],
    );
    std::fs::write(&second, "fresh target").unwrap();
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(session.macro_wait(), Some(Wait::Consent));
    assert_eq!(std::fs::read_to_string(&second).unwrap(), "fresh target");
    session.handle(character('n'));
    settle(&mut session);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(std::fs::read_to_string(second).unwrap(), "fresh target");
}

// Canceling a consent pause closes the partial iteration, retains its source
// edits and allows one native undo without replaying a saved answer.
#[test]
fn cancel_consent() {
    let mut session = editor("", "cat cat");
    session.handle(key(KeyCode::F(3)));
    named(&mut session, meta('x'), "query-replace");
    feed(
        &mut session,
        [
            Event::Paste("cat".into()),
            key(KeyCode::Enter),
            Event::Paste("dog".into()),
            key(KeyCode::Enter),
            character('n'),
            character('n'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    session.execute(Command::Motion(Motion::BufferStart));
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(session.macro_wait(), Some(Wait::Consent));
    session.handle(character('y'));
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(text(&session), "dog cat");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "cat cat");
}
