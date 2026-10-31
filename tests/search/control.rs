use super::*;
use rev::command::Scroll;

// Unicode full folding can expand either source or query scalars. Results use
// original positions and consume complete expansions; uppercase restores exactness.
#[test]
fn smart_ranges() {
    for (name, source, needle, expected) in [
        ("Latin", "É", "é", Some(0..1)),
        ("uppercase", "éÉ", "É", Some(1..2)),
        ("Greek", "ς", "σ", Some(0..1)),
        ("Cherokee", "\u{13a0}", "\u{ab70}", Some(0..1)),
        ("Deseret", "\u{10400}", "\u{10428}", Some(0..1)),
        ("default non-Turkic", "ı", "i", None),
        ("source expansion", "Straße", "strasse", Some(0..6)),
        ("source offsets", "界 Straße é", "strasse", Some(2..8)),
        ("query expansion", "SS", "ß", Some(0..2)),
        ("capital sharp s", "ẞ", "ss", Some(0..1)),
        ("dotted I", "İ", "i\u{307}", Some(0..1)),
        ("partial expansion", "ß", "s", None),
        ("suffix expansion", "ßi", "si", None),
        ("exact uppercase", "Straße", "STRASSE", None),
    ] {
        let session = editor(source, 0);
        let buffer = session.buffer().unwrap();
        let pattern = Pattern::new(needle).unwrap();
        let forward = pattern.find(buffer, Scalar(0)).unwrap();
        assert_eq!(
            forward.as_ref().map(|span| span.range.clone()),
            expected,
            "{name}"
        );
        let backward = pattern.rfind(buffer, Scalar(buffer.len_chars())).unwrap();
        assert_eq!(backward, forward, "{name}");
    }
}

// C-r starts before point, repeats to an earlier occurrence and switches with
// C-s without reselecting the same match. Acceptance leaves the chosen point.
#[test]
fn backward_flow() {
    let mut session = editor("CAT cat cat", 11);
    let stamp = session.buffer().unwrap().stamp();
    control(&mut session, 'r');
    query(&mut session, "cat");
    assert_eq!(session.search_match().unwrap().range, 8..11);
    assert_eq!(session.view().point.position, Scalar(8));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .last()
            .unwrap()
            .text()
            .contains("I-search backward: cat")
    );
    control(&mut session, 'r');
    assert_eq!(session.search_match().unwrap().range, 4..7);
    control(&mut session, 's');
    assert_eq!(session.search_match().unwrap().range, 8..11);
    query(&mut session, "x");
    assert_eq!(session.echo(), "No match");
    assert_eq!(session.view().point.position, Scalar(11));
    key(&mut session, KeyCode::Backspace, KeyModifiers::NONE);
    control(&mut session, 'r');
    assert_eq!(session.search_match().unwrap().range, 4..7);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(session.view().point.position, Scalar(4));
    assert!(session.search_match().is_none());
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(text(&session), "CAT cat cat");
}

// Search reveal changes both viewport axes. C-g must restore the entire saved
// view, including intended column and the next recenter alignment, in its pane.
#[test]
fn cancel_view() {
    let source = "abcdefghij\n".repeat(45) + &"x".repeat(120) + "TARGET";
    for wrap in [true, false] {
        let mut session = editor(&source, 5);
        if !wrap {
            session.execute(Command::VisualLineMode);
        }
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Scroll(Scroll::NextLine));
        session.execute(Command::Scroll(Scroll::Recenter));
        session.frame().unwrap();
        let origin = session.view().clone();
        let other = session.windows().order()[1];
        let other_view = session.windows().get(other).unwrap().clone();
        control(&mut session, 's');
        query(&mut session, "target");
        session.frame().unwrap();
        assert!(session.view().top > origin.top);
        assert_eq!(session.view().left > origin.left, !wrap);
        control(&mut session, 'g');
        assert_eq!(session.view(), &origin);
        session.frame().unwrap();
        assert_eq!(session.view(), &origin);
        assert_eq!(session.windows().get(other).unwrap(), &other_view);
        assert!(session.search_match().is_none());
        assert_eq!(text(&session), source);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// A backward scan crosses an actual chunk seam containing expanded Unicode,
// CRLF and emoji, retaining exact scalar ranges instead of folded-text indices.
#[test]
fn backward_seams() {
    let source = "ÉßİΣ\r\n👩\u{200d}💻x".repeat(1500);
    let session = editor(&source, 0);
    let buffer = session.buffer().unwrap();
    let whole = buffer.span(0..buffer.len_chars()).unwrap();
    let seam = buffer
        .chunks(&whole)
        .unwrap()
        .next()
        .unwrap()
        .chars()
        .count();
    let expected = buffer.span(seam - 14..seam + 14).unwrap();
    assert!(buffer.chunks(&expected).unwrap().count() >= 2);
    let query = buffer
        .slice(&expected)
        .unwrap()
        .to_string()
        .to_lowercase()
        .replace('ß', "ss");
    let pattern = Pattern::new(&query).unwrap();
    assert_eq!(
        pattern.rfind(buffer, Scalar(expected.range.end)).unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        pattern.find(buffer, Scalar(expected.range.start)).unwrap(),
        Some(expected)
    );
    assert_eq!(text(&session), source);
    assert!(buffer.history().is_empty());
}

// Reverse matching includes the boundary itself, excludes incomplete matches,
// and can handle the maximum query expanded threefold without rejecting it.
#[test]
fn reverse_bounds() {
    for (source, needle, end, expected) in [
        ("catcat", "cat", 0, None),
        ("catcat", "cat", 2, None),
        ("catcat", "cat", 3, Some(0..3)),
        ("catcat", "cat", 4, Some(0..3)),
        ("catcat", "cat", 6, Some(3..6)),
        ("ababa", "aba", 5, Some(2..5)),
        ("ababa", "aba", 4, Some(0..3)),
        ("a\r\nb", "\n", 3, Some(2..3)),
    ] {
        let session = editor(source, 0);
        let pattern = Pattern::new(needle).unwrap();
        let found = pattern
            .rfind(session.buffer().unwrap(), Scalar(end))
            .unwrap();
        assert_eq!(found.map(|span| span.range), expected, "{source}:{end}");
    }
    let session = editor(&"FFI".repeat(4096), 0);
    let buffer = session.buffer().unwrap();
    let pattern = Pattern::new(&"ﬃ".repeat(4096)).unwrap();
    assert_eq!(
        pattern.rfind(buffer, Scalar(12288)).unwrap().unwrap().range,
        0..12288
    );
    assert!(pattern.rfind(buffer, Scalar(12289)).is_err());
}

// Uppercase insertion changes the active policy immediately; deleting it
// recovers the same anchor, and a failed boundary can change direction safely.
#[test]
fn case_edits() {
    let mut session = editor("cat CAT cat", 11);
    control(&mut session, 'r');
    query(&mut session, "cat");
    control(&mut session, 'a');
    control(&mut session, 'd');
    query(&mut session, "C");
    assert!(session.search_match().is_none());
    assert_eq!(session.echo(), "No match");
    assert_eq!(session.view().point.position, Scalar(9));
    key(&mut session, KeyCode::Backspace, KeyModifiers::NONE);
    query(&mut session, "c");
    assert_eq!(session.search_match().unwrap().range, 8..11);
    control(&mut session, 'r');
    assert_eq!(session.search_match().unwrap().range, 4..7);
    control(&mut session, 'r');
    assert_eq!(session.search_match().unwrap().range, 0..3);
    control(&mut session, 'r');
    assert!(session.search_match().is_none());
    assert_eq!(session.view().point.position, Scalar(0));
    control(&mut session, 's');
    assert_eq!(session.search_match().unwrap().range, 0..3);
    assert_eq!(session.view().point.position, Scalar(3));
    assert_eq!(text(&session), "cat CAT cat");
}

// Empty, failed, repeated and reversed search owners all drop their saved view
// once canceled; a later ordinary command cannot revive the canceled result.
#[test]
fn cancel_states() {
    for (name, entry, events) in [
        ("empty", 's', vec![]),
        ("forward failure", 's', vec![Event::Paste("absent".into())]),
        ("backward failure", 'r', vec![Event::Paste("absent".into())]),
        (
            "repeat",
            's',
            vec![
                Event::Paste("cat".into()),
                Event::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
            ],
        ),
        (
            "reverse",
            's',
            vec![
                Event::Paste("cat".into()),
                Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL)),
            ],
        ),
    ] {
        let mut session = editor("zero cat cat", 5);
        session.frame().unwrap();
        let origin = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        control(&mut session, entry);
        for event in events {
            session.handle(event);
        }
        session.frame().unwrap();
        control(&mut session, 'g');
        assert_eq!(session.view(), &origin, "{name}");
        assert!(session.search_match().is_none(), "{name}");
        let buffer = session.buffer().unwrap();
        assert_eq!((buffer.stamp(), buffer.state()), (stamp, state), "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        key(&mut session, KeyCode::Char('!'), KeyModifiers::NONE);
        assert_eq!(text(&session), "zero !cat cat", "{name}");
    }
}

// Exact named invocation reaches the same backward handler as C-r. Accepting
// retires its origin, so a later C-g retains the accepted location.
#[test]
fn named_backward() {
    for named in [false, true] {
        let mut session = editor("cat CAT cat", 11);
        if named {
            session.execute(Command::ExecuteExtended);
            query(&mut session, "isearch-backward");
            key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        } else {
            control(&mut session, 'r');
        }
        query(&mut session, "cat");
        control(&mut session, 'r');
        assert_eq!(session.search_match().unwrap().range, 4..7);
        key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
        control(&mut session, 'g');
        assert_eq!(session.view().point.position, Scalar(4));
        assert!(session.search_match().is_none());
        assert_eq!(text(&session), "cat CAT cat");
    }
}
