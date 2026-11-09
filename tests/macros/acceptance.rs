use super::*;
use rev::command::MacroName;

// A multi-megabyte document keeps the same bounded host turns, cancellation,
// partial history and idle ownership as ordinary replay, in either profile.
#[test]
fn large_cancellation() {
    let source = "alpha beta gamma\n".repeat(131_072);
    for modal in [false, true] {
        let config = format!(
            "completion.automatic=false\neditor.interaction_profile='{}'\n",
            if modal { "modal" } else { "non-modal" }
        );
        let mut s = editor(&config, &source);
        support::settle(&mut s).unwrap();
        record(
            &mut s,
            if modal {
                vec![
                    character('i'),
                    Event::Paste("X".repeat(1024)),
                    key(KeyCode::Esc),
                ]
            } else {
                vec![Event::Paste("X".repeat(1024))]
            },
        );
        support::settle(&mut s).unwrap();
        let recorded = s.buffer().unwrap().len_bytes();
        feed(&mut s, [meta('0'), key(KeyCode::F(4))]);
        let turn = Instant::now();
        s.expire_input(Instant::now());
        let turn_elapsed = turn.elapsed();
        let partial = s.buffer().unwrap().len_bytes();
        assert!(partial > recorded && partial <= recorded + 256 * 1024);
        let cancel = Instant::now();
        s.handle(control('g'));
        let cancel_elapsed = cancel.elapsed();
        assert_eq!(s.macro_status(), Status::Idle);
        assert_eq!(s.macro_stop().unwrap().reason, StopReason::Canceled);
        assert_eq!(s.buffer().unwrap().len_bytes(), partial);
        assert!(cancel_elapsed < Duration::from_secs(5));
        support::settle(&mut s).unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
        assert_eq!(
            s.expire_input(Instant::now() + Duration::from_secs(60)),
            rev::session::Action::Wait
        );
        s.execute(Command::History(History::Undo));
        assert_eq!(s.buffer().unwrap().len_bytes(), partial - 1024);
        s.execute(Command::History(History::Redo));
        assert_eq!(s.buffer().unwrap().len_bytes(), partial);
        println!(
            "MEASURE macros profile={modal} source_bytes={} partial_bytes={partial} host_turn_ms={} cancel_us={}",
            source.len(),
            turn_elapsed.as_millis(),
            cancel_elapsed.as_micros()
        );
    }
}

// One native journey crosses independent ring/named/bound targets, explicit
// persistence and checked drafts. Deletion and editing cannot silently save or
// retarget the immutable snapshot restored by another session.
#[test]
fn snapshot_lifecycle() {
    let fixture = fixture::Fixture::new();
    let roots = rev::config::Roots {
        config: fixture.0.join("config"),
        data: fixture.0.join("data"),
        state: fixture.0.join("state"),
        cache: fixture.0.join("cache"),
    };
    let mut s = editor("macros.ring_limit=1\n", "").with_macro_library(&roots);
    record(&mut s, [character('A')]);
    feed(
        &mut s,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste("kept".into()),
            key(KeyCode::Enter),
        ],
    );
    feed(
        &mut s,
        [control('x'), control('k'), character('b'), character('8')],
    );
    s.execute(Command::Macro(MacroCommand::Save));
    feed(
        &mut s,
        [
            Event::Paste("kept".into()),
            key(KeyCode::Enter),
            character('y'),
        ],
    );
    let library = roots.state.join("macros/library.toml");
    let saved = std::fs::read(&library).unwrap();
    record(&mut s, [character('B')]);
    s.execute(Command::Macro(MacroCommand::List));
    s.handle(character('D'));
    s.handle(character('y'));
    s.handle(character('q'));
    feed(
        &mut s,
        [
            control('x'),
            control('k'),
            character('e'),
            Event::Paste("kept".into()),
            key(KeyCode::Enter),
        ],
    );
    let draft = text(&s).replace("key = \"A\"", "key = \"C\"");
    let length = s.buffer().unwrap().len_chars();
    s.replace(0..length, &draft).unwrap();
    feed(&mut s, [control('c'), control('c')]);
    feed(&mut s, [control('x'), control('k'), character('8')]);
    settle(&mut s);
    assert_eq!(text(&s), "ABC");
    assert_eq!(std::fs::read(&library).unwrap(), saved);
    s.execute(Command::Macro(MacroCommand::Load));
    s.execute(Command::MacroNamed(MacroName::new("kept").unwrap()));
    settle(&mut s);
    assert_eq!(text(&s), "ABCC");
    let mut restarted = editor("", "").with_macro_library(&roots);
    assert_eq!(text(&restarted), "");
    restarted.execute(Command::MacroNamed(MacroName::new("kept").unwrap()));
    settle(&mut restarted);
    assert_eq!(text(&restarted), "A");
    assert_eq!(std::fs::read(&library).unwrap(), saved);
}

// The beginner lesson leaves the next line ready, replays once per line and
// undoes only the last replay through the same native source/history owners.
#[test]
fn tutorial() {
    let mut s = editor("completion.automatic=false\n", "red\ngreen\nblue\n");
    record(
        &mut s,
        [
            control('a'),
            character('-'),
            character(' '),
            control('n'),
            control('a'),
        ],
    );
    assert_eq!(text(&s), "- red\ngreen\nblue\n");
    for _ in 0..2 {
        s.handle(key(KeyCode::F(4)));
        settle(&mut s);
    }
    assert_eq!(s.macro_status(), Status::Idle);
    assert!(s.macro_stop().is_none(), "{:?}", s.macro_stop());
    assert_eq!(text(&s), "- red\n- green\n- blue\n");
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "- red\n- green\nblue\n");
}
