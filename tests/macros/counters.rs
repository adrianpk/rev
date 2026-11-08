use super::*;

fn command(session: &mut Session, key: char) {
    feed(session, [control('x'), control('k'), control(key)]);
}
fn value(session: &mut Session, key: char, value: &str) {
    command(session, key);
    feed(
        session,
        [Event::Paste(value.into()), super::key(KeyCode::Enter)],
    );
}

// F3 inside capture is one semantic insertion, with persistent current and
// previous values. Separate iterations remain independently undoable.
#[test]
fn insertion() {
    let mut session = editor("", "");
    record(
        &mut session,
        [key(KeyCode::F(3)), control('u'), key(KeyCode::F(3))],
    );
    assert_eq!(text(&session), "00");
    feed(&mut session, [meta('2'), key(KeyCode::F(4))]);
    settle(&mut session);
    assert_eq!(text(&session), "001122");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "0011");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "00");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "0033");
}

// Numeric increments, assignment, iteration reset and cross-macro restore
// retain their distinct prefix forms instead of treating C-u as integer four.
#[test]
fn operations() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    feed(&mut session, [meta('3')]);
    command(&mut session, 'c');
    feed(&mut session, [meta('2'), key(KeyCode::F(3))]);
    feed(&mut session, [control('u')]);
    command(&mut session, 'c');
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(4)));
    assert_eq!(text(&session), "30");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "3031");
    value(&mut session, 'c', "42");
    command(&mut session, 'i');
    session.handle(key(KeyCode::F(3)));
    feed(&mut session, [control('u')]);
    command(&mut session, 'a');
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(4)));
    assert!(text(&session).ends_with("4242"));
}

// Accepted literal prompt values are semantic operations. Replay does not
// reopen a counter or format prompt or depend on its presentation keys.
#[test]
fn prompt_capture() {
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    value(&mut session, 'c', "16");
    value(&mut session, 'a', "-2");
    value(&mut session, 'f', "[%#06x]%%");
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(4)));
    assert_eq!(text(&session), "[0x000e]%");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "[0x000e]%[0x000e]%");
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(session.macro_stop(), None);
}

// Formats are captured per definition; changing the outside default affects
// only later recordings, and an in-body format takes effect at that step.
#[test]
fn independent_formats() {
    let mut session = editor("", "");
    value(&mut session, 'f', "A%02d");
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(3)));
    value(&mut session, 'f', "B%d");
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(4)));
    assert_eq!(text(&session), "A00B1");
    value(&mut session, 'f', "C%d");
    record(&mut session, [key(KeyCode::F(3))]);
    session.execute(Command::Macro(MacroCommand::CallSecond));
    settle(&mut session);
    assert_eq!(text(&session), "A00B1C0A02B3");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "A00B1C0A02B3C1");
}

// Supported flags, radices, precision, UTF-8 literals and escaped percents
// produce bounded exact text, including signed-minimum decimal formatting.
#[test]
fn formats() {
    for (format, number, expected) in [
        ("%d", "-9223372036854775808", "-9223372036854775808"),
        ("%+05d", "12", "+0012"),
        ("%-5.3d", "12", "012  "),
        ("% d", "12", " 12"),
        ("%#o", "8", "010"),
        ("%#X", "31", "0X1F"),
        ("界%%%.0d", "0", "界%"),
        ("", "7", "7"),
    ] {
        let mut session = editor("", "");
        value(&mut session, 'f', format);
        session.handle(key(KeyCode::F(3)));
        value(&mut session, 'c', number);
        session.handle(key(KeyCode::F(3)));
        session.handle(key(KeyCode::F(4)));
        assert_eq!(text(&session), expected, "{format}");
    }
}

// Invalid formats leave both the outside default and a recording's current
// format intact. Width/precision and byte bounds reject before formatting.
#[test]
fn invalid_formats() {
    for format in [
        "%s".into(),
        "%d%d".into(),
        "%*d".into(),
        "%129d".into(),
        "%.129d".into(),
        "%".into(),
        "plain".into(),
        format!("{}%d", "界".repeat(85)),
    ] {
        let mut session = editor("", "");
        value(&mut session, 'f', "[%d]");
        value(&mut session, 'f', &format);
        session.handle(key(KeyCode::F(3)));
        value(&mut session, 'f', &format);
        session.handle(key(KeyCode::F(3)));
        session.handle(key(KeyCode::F(4)));
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), "[0][1]", "{format}");
        assert_eq!(session.macro_stop(), None);
    }
}

// Overflow refuses the insertion/update before effects, keeps the last valid
// definition and reports the exact automatic step rather than wrapping.
#[test]
fn overflow() {
    let mut session = editor("", "");
    record(&mut session, [key(KeyCode::F(3)), character('x')]);
    value(&mut session, 'c', "9223372036854775807");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "0x");
    let stop = session.macro_stop().unwrap();
    assert_eq!(stop.reason, StopReason::CounterOverflow);
    assert_eq!((stop.iteration, stop.step), (1, 1));
    value(&mut session, 'c', "-9223372036854775808");
    value(&mut session, 'a', "-1");
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::CounterOverflow
    );
    feed(&mut session, [control('u')]);
    command(&mut session, 'i');
    assert_eq!(text(&session), "0x0");
}

// Exact supported field/format ceilings remain usable, while invalid signed
// prompt values leave the previously accepted counter unchanged.
#[test]
fn bounds() {
    for (format, expected) in [
        ("%128d".to_owned(), format!("{}0", " ".repeat(127))),
        ("%.128d".to_owned(), "0".repeat(128)),
        (
            format!("{}%d", "a".repeat(254)),
            format!("{}0", "a".repeat(254)),
        ),
    ] {
        let mut session = editor("", "");
        value(&mut session, 'f', &format);
        record(&mut session, [key(KeyCode::F(3))]);
        assert_eq!(text(&session), expected);
    }
    let mut session = editor("", "");
    session.handle(key(KeyCode::F(3)));
    value(&mut session, 'c', "7");
    for invalid in ["9223372036854775808", "-9223372036854775809", "text"] {
        value(&mut session, 'c', invalid);
    }
    session.handle(key(KeyCode::F(3)));
    session.handle(key(KeyCode::F(4)));
    assert_eq!(text(&session), "7");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "77");
}

// Shared controls preserve native Normal, Insert and Replace admission: only
// Replace overwrites source, and replay keeps each recorded entry state.
#[test]
fn modal_controls() {
    for (entry, recorded, replayed) in [
        (None, "0abcd", "01abcd"),
        (Some('i'), "0abcd", "01abcd"),
        (Some('R'), "0bcd", "01cd"),
    ] {
        let mut session = editor("[editor]\ninteraction_profile = 'modal'\n", "abcd");
        if let Some(entry) = entry {
            session.handle(character(entry));
        }
        record(&mut session, [key(KeyCode::F(3))]);
        assert_eq!(text(&session), recorded, "{entry:?}");
        session.handle(key(KeyCode::F(4)));
        settle(&mut session);
        assert_eq!(text(&session), replayed, "{entry:?}");
        assert_eq!(session.macro_stop(), None, "{entry:?}");
    }
}
