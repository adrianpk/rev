use super::*;
use rev::command::History;
use rev::input::{Focus, Input, Outcome, SearchAction};

// Query input is bounded and literal, including multiline/control-looking paste;
// held acceptance cannot close search or leak a second newline into editing.
#[test]
fn input_owner() {
    use crossterm::event::KeyEventKind::Repeat;
    let mut input = Input::default();
    input.search(rev::search::Direction::Forward).unwrap();
    assert_eq!(input.focus(), Focus::Search);
    assert_eq!(
        input.handle(Event::Paste("界\r\n\x18\x03".into())),
        Outcome::Changed
    );
    assert_eq!(input.prompt().unwrap().text(), "界\r\n\x18\x03");
    assert_eq!(
        input.handle(Event::Paste("x".repeat(4096))),
        Outcome::Message("Prompt input limit exceeded")
    );
    assert_eq!(input.prompt().unwrap().text(), "界\r\n\x18\x03");
    let held = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        Repeat,
    ));
    assert_eq!(input.handle(held.clone()), Outcome::Ignored);
    assert_eq!(input.focus(), Focus::Search);
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        ))),
        Outcome::Search(SearchAction::Accept)
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert_eq!(input.handle(held), Outcome::Ignored);
    assert!(input.prompt().is_none());
}

// KMP fallback must retain overlapping prefixes at a chunk seam and must not
// normalize Unicode. Smart case changes comparison without changing source ranges.
#[test]
fn literal_contract() {
    for (name, source, needle, start, expected) in [
        ("prefix fallback", "abababac", "ababac", 0, Some(2..8)),
        ("overlap start", "aaaaa", "aaa", 1, Some(1..4)),
        ("smart case", "É é", "é", 0, Some(0..1)),
        ("no normalization", "e\u{301}", "é", 0, None),
        ("no wrap", "target", "target", 6, None),
        ("empty", "abc", "", 0, None),
    ] {
        let session = editor(source, 0);
        let found = Pattern::new(needle)
            .unwrap()
            .find(session.buffer().unwrap(), Scalar(start))
            .unwrap();
        assert_eq!(found.map(|span| span.range), expected, "{name}");
    }
    let session = editor("abc", 0);
    assert!(
        Pattern::new("x")
            .unwrap()
            .find(session.buffer().unwrap(), Scalar(4))
            .is_err()
    );
    assert!(Pattern::new(&"界".repeat(4097)).is_err());
}

// Search can end inside combining/ZWJ clusters or a CRLF pair. Raw source ranges
// remain exact while point and visible underline expand to complete graphemes.
#[test]
fn grapheme_projection() {
    for (name, source, needle, raw, point, paint) in [
        ("base", "e\u{301}x", "e", 0..1, 2, Some(0..1)),
        ("accent", "e\u{301}x", "\u{301}", 1..2, 2, Some(0..1)),
        ("emoji", "👩\u{200d}💻x", "\u{200d}", 1..2, 3, Some(0..2)),
        ("CRLF", "a\r\nb", "\r", 1..2, 3, Some(1..80)),
    ] {
        let mut session = editor(source, 0);
        control(&mut session, 's');
        query(&mut session, needle);
        assert_eq!(session.search_match().unwrap().range, raw, "{name}");
        assert_eq!(session.view().point.position, Scalar(point), "{name}");
        assert_eq!(
            session.frame().unwrap().rows()[0].search(),
            paint.map(|range| range.start + 6..(range.end + 6).min(80)),
            "{name}"
        );
        assert_eq!(text(&session), source, "{name}");
    }
}

// A source-qualified result is unusable after publication. Redraw retires the
// stale owner, and cancellation cannot restore its old revision into live views.
#[test]
fn stale_result() {
    let mut session = editor("alpha beta", 0);
    control(&mut session, 's');
    query(&mut session, "beta");
    let stale = session.search_match().unwrap().clone();
    session.replace(0..0, "界").unwrap();
    let live = session.view().clone();
    assert!(session.buffer().unwrap().slice(&stale).is_err());
    assert!(session.search_match().is_none());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.search().is_none())
    );
    assert_eq!(session.echo(), "Search source changed");
    control(&mut session, 'g');
    assert_eq!(session.view().point, live.point);
    session.insert('!');
    assert_eq!(text(&session), "界!alpha beta");
}

// Editing an earlier query character recomputes a literal range; a first miss,
// empty query and failed continuation never move point outside the source.
#[test]
fn query_recovery() {
    let mut session = editor("cat cot", 0);
    control(&mut session, 's');
    query(&mut session, "zzz");
    assert_eq!(session.echo(), "No match");
    assert_eq!(session.view().point.position, Scalar(0));
    for _ in 0..3 {
        key(&mut session, KeyCode::Backspace, KeyModifiers::NONE);
    }
    assert_eq!(session.echo(), "");
    assert!(session.search_match().is_none());
    query(&mut session, "cat");
    control(&mut session, 'a');
    control(&mut session, 'f');
    control(&mut session, 'd');
    query(&mut session, "o");
    assert_eq!(session.search_match().unwrap().range, 4..7);
    control(&mut session, 's');
    assert_eq!(session.echo(), "No match");
    assert_eq!(session.view().point.position, Scalar(7));
    assert!(session.search_match().is_none());
    assert_eq!(text(&session), "cat cot");
}

// Match projection is clipped to the selected pane and survives complete
// suspension. Multiline query input remains visible control notation, not ANSI.
#[test]
fn multiline_resize() {
    let mut session = editor("before\na\t界\r\nend\nlast", 0);
    control(&mut session, 's');
    query(&mut session, "界\r\nend");
    let point = session.view().point;
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[1].search(), Some(10..80));
    assert_eq!(frame.rows()[2].search(), Some(6..9));
    for (width, height) in [(0, 0), (0, 8), (1, 1), (3, 3), (80, 10)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        let mut wire = Vec::new();
        frame.write(&mut wire).unwrap();
        assert_eq!(session.view().point, point);
        assert_eq!(session.search_match().unwrap().range, 9..15);
    }
    let frame = session.frame().unwrap();
    let visible_end = u16::try_from(3usize.saturating_sub(session.view().left)).unwrap();
    assert_eq!(frame.rows()[0].search(), Some(6..6 + visible_end));
    assert!(frame.rows().last().unwrap().text().contains("I-search:"));
    assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    assert_eq!(text(&session), "before\na\t界\r\nend\nlast");
}

// Binding and exact M-x invocation share search ownership and result semantics;
// accepting search preserves an existing redo future and the stored mark.
#[test]
fn command_history() {
    for named in [false, true] {
        let mut session = editor("cat", 0);
        session.insert('X');
        session.execute(Command::History(History::Undo));
        session.execute(Command::Region(RegionCommand::Set));
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        if named {
            session.execute(Command::ExecuteExtended);
            query(&mut session, "isearch-forward");
            key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        } else {
            control(&mut session, 's');
        }
        query(&mut session, "cat");
        assert_eq!(session.search_match().unwrap().range, 0..3);
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        let buffer = session.buffer().unwrap();
        assert_eq!((buffer.stamp(), buffer.state()), (stamp, state));
        assert_eq!(buffer.mark(), Some(Scalar(0)));
        assert!(buffer.history().is_empty());
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), "Xcat");
    }
}

// Larger multiline and long-line ropes repeatedly search their far end without
// source publication; a repeated-prefix miss exercises linear KMP fallback work.
#[test]
fn large_ropes() {
    for (name, body) in [
        ("many lines", "á界\trow\n".repeat(131_072)),
        ("long line", "a".repeat(1_048_576)),
    ] {
        let source = format!("{body}END界");
        let mut session = editor(&source, 0);
        let stamp = session.buffer().unwrap().stamp();
        control(&mut session, 's');
        for value in "END界".chars() {
            key(&mut session, KeyCode::Char(value), KeyModifiers::NONE);
        }
        assert_eq!(
            session.search_match().unwrap().range,
            body.chars().count()..source.chars().count(),
            "{name}"
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        let pattern = Pattern::new(&format!("{}b", "a".repeat(4095))).unwrap();
        assert!(
            pattern
                .find(session.buffer().unwrap(), Scalar(0))
                .unwrap()
                .is_none(),
            "{name}"
        );
        assert_eq!(text(&session), source, "{name}");
    }
}
