use super::*;
use rev::command::{BufferCommand, Motion};

fn open(session: &mut Session) {
    feed(session, [control('x'), control('k'), key(KeyCode::Enter)]);
    assert!(text(session).contains("[[macros]]"), "{}", session.echo());
}
fn apply(session: &mut Session) {
    feed(session, [control('c'), control('c')]);
}
fn rewrite(session: &mut Session, value: &str) {
    let len = session.buffer().unwrap().len_chars();
    session.replace(0..len, value).unwrap();
}
fn close(session: &mut Session) {
    session.execute(Command::Buffer(BufferCommand::Kill));
}
fn change(session: &mut Session, from: &str, to: &str) {
    let value = text(session).replace(from, to);
    rewrite(session, &value);
}
fn call(session: &mut Session) {
    session.handle(key(KeyCode::F(4)));
    settle(session);
}

// The draft owns native text/undo but no source effects. Applying replaces its
// pinned ring body, while an independent named snapshot retains the old body.
#[test]
fn apply_target() {
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
                control('x'),
                control('k'),
                character('n'),
                Event::Paste("alpha".into()),
                key(KeyCode::Enter),
            ],
        );
        open(&mut session);
        change(&mut session, "key = \"A\"", "key = \"B\"");
        apply(&mut session);
        assert_eq!(text(&session), "A", "{}", session.echo());
        call(&mut session);
        assert_eq!(text(&session), "AB");
        session.execute(Command::MacroNamed(
            rev::command::MacroName::new("alpha").unwrap(),
        ));
        settle(&mut session);
        assert_eq!(text(&session), "ABA");
    }
}

// Invalid, retargeted and oversized drafts stay editable. Native closing drops
// only the draft; replay continues to use the prior published macro.
#[test]
fn retained_errors() {
    for variant in ["syntax", "identity", "revision", "oversized"] {
        let mut session = editor("", "");
        record(&mut session, [character('A')]);
        open(&mut session);
        let original = text(&session);
        let modified = match variant {
            "syntax" => "not TOML".into(),
            "identity" => original.replace(
                "id = \"00000000000000000000000000000001\"",
                "id = \"11111111111111111111111111111111\"",
            ),
            "revision" => original.replace("revision = 1", "revision = 2"),
            "oversized" => "X".repeat(300 * 1024),
            _ => unreachable!(),
        };
        rewrite(&mut session, &modified);
        apply(&mut session);
        assert_eq!(text(&session), modified, "{variant}");
        close(&mut session);
        assert_eq!(text(&session), "A");
        call(&mut session);
        assert_eq!(text(&session), "AA");
    }
}

// Identity pins survive ring reorder but reject removal; cancellation cannot
// recreate the removed definition or silently redirect the draft to a row.
#[test]
fn stale_target() {
    let mut session = editor("", "");
    record(&mut session, [character('A')]);
    open(&mut session);
    session.execute(Command::Macro(MacroCommand::Delete));
    apply(&mut session);
    assert!(text(&session).contains("[[macros]]"));
    assert!(session.echo().contains("removed"), "{}", session.echo());
    close(&mut session);
    call(&mut session);
    assert_eq!(text(&session), "A");
}

// Key insertion reads native normalized chords as inert notation. Uppercase
// input is a literal key, independently of the binding prompt's slot expansion.
#[test]
fn key_helper() {
    let mut session = editor("", "");
    record(&mut session, [character('A')]);
    open(&mut session);
    session.execute(Command::Motion(Motion::BufferEnd));
    feed(&mut session, [control('c'), control('q'), character('Z')]);
    assert!(text(&session).contains("key = \"Z\""), "{}", text(&session));
    apply(&mut session);
    call(&mut session);
    assert_eq!(text(&session), "AAZ");
}

// Whole-line selection projects only chosen step tables into the draft. The
// resulting body still requires a separate checked apply and is source-undoable.
#[test]
fn region_helper() {
    let mut session = editor("", "");
    record(
        &mut session,
        [character('A'), character('B'), character('C')],
    );
    open(&mut session);
    let draft = text(&session);
    let positions: Vec<_> = draft
        .match_indices("[[macros.steps]]")
        .map(|(i, _)| i)
        .collect();
    session.execute(Command::Motion(Motion::BufferStart));
    for _ in draft[..positions[1]].chars() {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Region(rev::command::RegionCommand::Set));
    for _ in draft[positions[1]..positions[2]].chars() {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    feed(&mut session, [control('c'), control('r')]);
    assert_eq!(
        text(&session).matches("[[macros.steps]]").count(),
        1,
        "{}",
        session.echo()
    );
    apply(&mut session);
    call(&mut session);
    assert_eq!(text(&session), "ABCB");
}

// Named and anonymous binding targets replace only their independent snapshot;
// the ring body and bindings of other identities remain usable after applying.
#[test]
fn selected_targets() {
    for named in [false, true] {
        let mut s = editor("", "");
        record(&mut s, [character('A')]);
        if named {
            feed(
                &mut s,
                [
                    control('x'),
                    control('k'),
                    character('n'),
                    Event::Paste("alpha".into()),
                    key(KeyCode::Enter),
                ],
            );
        }
        feed(
            &mut s,
            [control('x'), control('k'), character('b'), character('7')],
        );
        feed(
            &mut s,
            [
                control('x'),
                control('k'),
                character('e'),
                Event::Paste(if named { "alpha" } else { "C-x C-k 7" }.into()),
                key(KeyCode::Enter),
            ],
        );
        assert!(text(&s).contains("[[macros]]"), "{}", s.echo());
        change(&mut s, "key = \"A\"", "key = \"B\"");
        apply(&mut s);
        feed(&mut s, [control('x'), control('k'), character('7')]);
        settle(&mut s);
        assert_eq!(text(&s), "AB", "{}", s.echo());
        call(&mut s);
        assert_eq!(text(&s), "ABA");
    }
}

// The repeat variant publishes a normal checked draft and exposes the native
// temporary ring keys afterward; an unrelated key exits that temporary map.
#[test]
fn repeat_edit() {
    let mut s = editor("", "");
    record(&mut s, [character('A')]);
    record(&mut s, [character('B')]);
    feed(&mut s, [control('x'), control('k'), control('e')]);
    apply(&mut s);
    s.handle(control('n'));
    call(&mut s);
    assert_eq!(text(&s), "ABA");
}

// Editing an explicitly saved named snapshot changes only session data. Reload
// preserves that unsaved revision; restart still admits the old saved payload.
#[test]
fn saved_independence() {
    let fixture = fixture::Fixture::new();
    let roots = rev::config::Roots {
        config: fixture.0.join("config"),
        data: fixture.0.join("data"),
        state: fixture.0.join("state"),
        cache: fixture.0.join("cache"),
    };
    let mut s = editor("", "").with_macro_library(&roots);
    record(&mut s, [character('A')]);
    feed(
        &mut s,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste("alpha".into()),
            key(KeyCode::Enter),
        ],
    );
    s.execute(Command::Macro(MacroCommand::Save));
    feed(
        &mut s,
        [
            Event::Paste("alpha".into()),
            key(KeyCode::Enter),
            character('y'),
        ],
    );
    let path = roots.state.join("macros/library.toml");
    let before = std::fs::read(&path).unwrap();
    feed(
        &mut s,
        [
            control('x'),
            control('k'),
            character('e'),
            Event::Paste("alpha".into()),
            key(KeyCode::Enter),
        ],
    );
    change(&mut s, "key = \"A\"", "key = \"B\"");
    apply(&mut s);
    s.execute(Command::Macro(MacroCommand::Load));
    s.execute(Command::MacroNamed(
        rev::command::MacroName::new("alpha").unwrap(),
    ));
    settle(&mut s);
    assert_eq!(text(&s), "AB");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let mut restarted = editor("", "").with_macro_library(&roots);
    restarted.execute(Command::MacroNamed(
        rev::command::MacroName::new("alpha").unwrap(),
    ));
    settle(&mut restarted);
    assert_eq!(text(&restarted), "A");
}
