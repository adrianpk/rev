use super::*;

// The exact step ceiling admits a complete recording. A further source key
// refuses capture rather than publishing an over-limit partial definition.
#[test]
fn step_limit() {
    let mut session = editor("", "");
    record(&mut session, (0..4_096).map(|_| control('f')));
    session.handle(key(KeyCode::F(3)));
    feed(&mut session, (0..4_097).map(|_| control('f')));
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::CaptureLimit
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(session.macro_stop(), None);
    assert_eq!(text(&session), "");
}

// Escape-compatible Meta input is retained as one logical chord. A numeric
// argument inside capture remains attached to its own edit on every replay.
#[test]
fn normalized_prefixes() {
    let mut session = editor("[keys.editor]\n'M-f' = 'forward-char'\n", "abcd");
    record(
        &mut session,
        [key(KeyCode::Esc), character('f'), character('X')],
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "aXbXcd");

    let mut session = editor("", "");
    record(&mut session, [control('u'), character('3'), character('x')]);
    feed(&mut session, [meta('2'), key(KeyCode::F(4))]);
    settle(&mut session);
    assert_eq!(text(&session), "xxxxxxxxx");
}

// Recording publishes real Unicode edits immediately. Function-key controls
// are excluded; the same logical native input replays a positive count.
#[test]
fn native_text() {
    let mut session = editor("", "");
    record(
        &mut session,
        [Event::Paste("界e\u{301}".into()), key(KeyCode::Enter)],
    );
    assert_eq!(text(&session), "界e\u{301}\n");
    feed(
        &mut session,
        [control('u'), character('3'), key(KeyCode::F(4))],
    );
    settle(&mut session);
    assert_eq!(text(&session), "界e\u{301}\n".repeat(4));
    assert_eq!(session.macro_stop(), None);
    assert_eq!(session.macro_status(), Status::Idle);
}

// Ending a recording includes its first pass in a supplied C-x ) count;
// C-x e instead runs the finished definition once more.
#[test]
fn control_chords() {
    for (name, ending, expected) in [
        ("plain end", vec![control('x'), character(')')], "x"),
        (
            "counted end",
            vec![control('u'), character('4'), control('x'), character(')')],
            "xxxx",
        ),
        ("end and call", vec![control('x'), character('e')], "xx"),
    ] {
        let mut session = editor("", "");
        feed(&mut session, [control('x'), character('('), character('x')]);
        feed(&mut session, ending);
        settle(&mut session);
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.macro_status(), Status::Idle, "{name}");
    }
}

// Releases never enter a definition; admitted repeated text remains input.
// Resize/focus notifications do not become playback effects or stop capture.
#[test]
fn logical_phases() {
    let mut session = editor("", "");
    let event = |kind| {
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Char('A'),
            KeyModifiers::SHIFT,
            kind,
        ))
    };
    record(
        &mut session,
        [
            event(KeyEventKind::Press),
            event(KeyEventKind::Repeat),
            event(KeyEventKind::Release),
            Event::Resize(80, 16),
            Event::FocusLost,
            Event::FocusGained,
        ],
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "AAAA");
}

// Discarding capture never rolls back its real edits or replaces the last
// valid definition, including bounded-input refusal before insertion.
#[test]
fn preserve_previous() {
    for (name, abort, reason) in [
        (
            "cancel",
            vec![character('z'), control('g')],
            StopReason::Canceled,
        ),
        ("empty", vec![key(KeyCode::F(4))], StopReason::Empty),
        ("unsupported job", vec![meta('!')], StopReason::Unsupported),
        (
            "binary paste",
            vec![Event::Paste("bad\0text".into())],
            StopReason::Unsupported,
        ),
        (
            "oversized",
            vec![Event::Paste("z".repeat(256 * 1024))],
            StopReason::CaptureLimit,
        ),
    ] {
        let mut session = editor("", "");
        record(&mut session, [character('x')]);
        session.handle(key(KeyCode::F(3)));
        feed(&mut session, abort);
        assert_eq!(session.macro_status(), Status::Idle, "{name}");
        assert_eq!(session.macro_stop().unwrap().reason, reason, "{name}");
        let before = text(&session);
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), format!("{before}x"), "{name}");
    }
}

// User key overrides retain ownership; the native start command remains
// available without reserving its default physical function key.
#[test]
fn effective_bindings() {
    let mut session = editor("[keys.editor]\nF3 = 'delete-char'\n", "abc");
    session.handle(key(KeyCode::F(3)));
    settle(&mut session);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(text(&session), "bc");
    session.execute(Command::Macro(MacroCommand::Start));
    assert_eq!(session.macro_status(), Status::Recording);
}
