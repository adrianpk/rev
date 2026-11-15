use super::*;

// Explicit wrapping ignores automatic-editing preferences and accepts either
// delimiter in its cancellable native prompt, always using opener placement.
#[test]
fn wrapping() {
    for (config, selection) in [
        ("pairs.enabled=false", ")"),
        ("pairs.wrap_selection=false", "["),
        ("pairs.quotes=false", "\""),
    ] {
        let mut e = selected(config, "plain", "alpha", true);
        named(&mut e, "pair-wrap");
        choose(&mut e, selection);
        let expected = match selection {
            ")" => "(|alpha)",
            "[" => "[|alpha]",
            _ => "\"|alpha\"",
        };
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    }
    let mut e = selected("pairs.enabled=false", "plain", "alpha", false);
    e.session
        .handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    e.session.handle(character('3'));
    e.session.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    e.session.handle(Event::Paste("pair-wrap".into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    choose(&mut e, "]");
    e.expect("[[[|alpha]]]");
}

// Pending prompt cancellation or invalid input preserves the exact source,
// cursor and active region. No default delimiter is guessed on empty acceptance.
#[test]
fn prompt() {
    for choice in ["", "<", "()"] {
        let mut e = selected("", "plain", "alpha", false);
        let point = e.session.view().point;
        named(&mut e, "pair-wrap");
        choose(&mut e, choice);
        assert_eq!(e.text(), "alpha");
        assert_eq!(e.session.view().point, point);
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
    let mut e = selected("", "plain", "alpha", true);
    let point = e.session.view().point;
    let mark = e.session.buffer().unwrap().mark();
    named(&mut e, "pair-wrap");
    e.session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    settle(&mut e.session);
    assert_eq!(e.session.view().point, point);
    assert_eq!(e.session.buffer().unwrap().mark(), mark);
    assert!(e.session.buffer().unwrap().mark_active());
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// Enclosing proof chooses the innermost pair, including point on either side.
// Explicit unwrap works with automatic editing off and maps source points natively.
#[test]
fn unwrapping() {
    for (source, expected) in [
        ("(|[alpha])", "(|alpha)"),
        ("([al|pha])", "(al|pha)"),
        ("([alpha|])", "(alpha|)"),
        ("\"al|pha\"", "al|pha"),
    ] {
        let mut e = Editor::new("pairs.enabled=false", "plain", source);
        named(&mut e, "pair-unwrap");
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().history().len(), 1);
        e.session.execute(Command::History(History::Undo));
        settle(&mut e.session);
        e.expect(source);
        e.session.execute(Command::History(History::Redo));
        settle(&mut e.session);
        e.expect(expected);
    }
}

// Refusal cannot turn an escape sequence into different literal text or remove
// unproved delimiters. Each request leaves source/revision/point/history unchanged.
#[test]
fn refusal() {
    for (language, source) in [
        ("plain", "\"a\\|nb\""),
        ("rust", "r#\"al|pha\"#"),
        ("plain", "([|)]"),
        ("plain", "al|pha"),
        ("plain", "(|)\u{301}"),
    ] {
        let mut e = Editor::new("", language, source);
        let point = e.session.view().point;
        named(&mut e, "pair-unwrap");
        e.expect(source);
        assert_eq!(e.session.view().point, point);
        assert!(e.session.buffer().unwrap().history().is_empty());
        assert!(!e.session.echo().is_empty());
    }
}

// The frozen command set plus three editing and seven navigation additions
// stays exact. The two editing helpers remain described and factory-unbound.
#[test]
fn inventory() {
    use std::collections::BTreeSet;
    let frozen: serde_json::Value = serde_json::from_str(include_str!(
        "../../../ops/default/report/rev-factory-shortcuts/catalog.json"
    ))
    .unwrap();
    let baseline: BTreeSet<_> = frozen["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    let added = BTreeSet::from([
        "auto-pair-mode",
        "pair-wrap",
        "pair-unwrap",
        "matching-delimiter",
        "forward-pair",
        "backward-pair",
        "up-pair",
        "down-pair",
        "forward-sexp",
        "backward-sexp",
    ]);
    let live: BTreeSet<_> = Command::ALL.iter().map(Command::name).collect();
    assert_eq!(baseline.len(), 707);
    assert_eq!(live.len(), 717);
    assert!(baseline.is_disjoint(&added));
    assert_eq!(baseline, live.difference(&added).copied().collect());
    for (name, description) in [
        (
            "pair-wrap",
            "Choose an admitted delimiter and wrap the active linear region in one undo step.",
        ),
        (
            "pair-unwrap",
            "Remove the innermost confirmed enclosing pair while preserving its contents.",
        ),
    ] {
        let command = Command::named(name).unwrap();
        assert_eq!(command.description(), Some(description));
        for context in rev::input::keymap::Context::ALL {
            assert!(!context.defaults().iter().any(|(_, bound)| *bound == name));
        }
    }
}

// Explicit unwrap maps the active mark through both removals. Undo restores
// the original active orientation, and redo restores the exact mapped endpoints.
#[test]
fn marked_unwrap() {
    for backward in [false, true] {
        let mut e = Editor::new("", "plain", "|(alpha)");
        if backward {
            e.session.execute(Command::Motion(Motion::BufferEnd));
            e.session.execute(Command::Motion(Motion::BackwardChar));
        }
        e.session.execute(Command::Region(RegionCommand::Set));
        if backward {
            e.session.execute(Command::Motion(Motion::BufferStart));
        } else {
            e.session.execute(Command::Motion(Motion::BufferEnd));
            e.session.execute(Command::Motion(Motion::BackwardChar));
        }
        let point = e.session.view().point.position;
        let mark = e.session.buffer().unwrap().mark();
        named(&mut e, "pair-unwrap");
        let expected = if backward { "|alpha" } else { "alpha|" };
        e.expect(expected);
        let mapped_mark = Some(rev::coordinates::Scalar(if backward { 5 } else { 0 }));
        assert_eq!(e.session.buffer().unwrap().mark(), mapped_mark);
        assert!(!e.session.buffer().unwrap().mark_active());
        e.session.execute(Command::History(History::Undo));
        settle(&mut e.session);
        assert_eq!(e.text(), "(alpha)");
        assert_eq!(e.session.view().point.position, point);
        assert_eq!(e.session.buffer().unwrap().mark(), mark);
        assert!(e.session.buffer().unwrap().mark_active());
        e.session.execute(Command::History(History::Redo));
        settle(&mut e.session);
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().mark(), mapped_mark);
        assert!(!e.session.buffer().unwrap().mark_active());
    }
}
