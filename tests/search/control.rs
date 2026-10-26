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
        ("source expansion", "Straße", "strasse", Some(0..6)),
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
    let mut session = editor(&source, 5);
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
    assert!(session.view().left > origin.left);
    control(&mut session, 'g');
    assert_eq!(session.view(), &origin);
    session.frame().unwrap();
    assert_eq!(session.view(), &origin);
    assert_eq!(session.windows().get(other).unwrap(), &other_view);
    assert!(session.search_match().is_none());
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}
