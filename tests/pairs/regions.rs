#[path = "regions/helpers.rs"]
mod helpers;
#[path = "regions/lifecycle.rs"]
mod lifecycle;
#[path = "regions/qualification.rs"]
mod qualification;

use super::editor::*;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::command::{Command, History, Motion, RegionCommand};

fn selected(config: &str, language: &str, text: &str, backward: bool) -> Editor {
    let marked = if backward {
        format!("{text}|")
    } else {
        format!("|{text}")
    };
    let mut e = Editor::new(config, language, &marked);
    e.session.execute(Command::Region(RegionCommand::Set));
    e.session.execute(Command::Motion(if backward {
        Motion::BufferStart
    } else {
        Motion::BufferEnd
    }));
    settle(&mut e.session);
    e
}

fn named(e: &mut Editor, command: &str) {
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste(command.into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut e.session);
}

fn choose(e: &mut Editor, pair: &str) {
    e.session.handle(Event::Paste(pair.into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    settle(&mut e.session);
}

// Independent reference placement applies to either selection orientation.
// Openers enter the interior; closers finish after the generated closing side.
#[test]
fn placement() {
    for backward in [false, true] {
        for (c, expected) in [
            ('(', "(|alpha)"),
            (')', "(alpha)|"),
            ('[', "[|alpha]"),
            (']', "[alpha]|"),
            ('{', "{|alpha}"),
            ('}', "{alpha}|"),
            ('"', "\"|alpha\""),
        ] {
            let mut e = selected("", "plain", "alpha", backward);
            e.type_key(c);
            e.expect(expected);
            assert!(!e.session.buffer().unwrap().mark_active());
            assert_eq!(e.session.buffer().unwrap().history().len(), 1);
        }
    }
}

// Counted wrapping preserves original UTF-8 and CRLF bytes. Native history
// restores the initiating point, mark and activation in the original orientation.
#[test]
fn history() {
    for backward in [false, true] {
        for (c, expected) in [
            ('(', "(((|界e\u{301}\r\nend)))"),
            (')', "(((界e\u{301}\r\nend)))|"),
        ] {
            let mut e = selected("", "plain", "界e\u{301}\r\nend", backward);
            let point = e.session.view().point.position;
            let mark = e.session.buffer().unwrap().mark();
            e.counted("3", c);
            e.expect(expected);
            assert_eq!(e.session.buffer().unwrap().history().len(), 1);
            assert!(!e.session.buffer().unwrap().mark_active());
            e.session.execute(Command::History(History::Undo));
            settle(&mut e.session);
            assert_eq!(e.text(), "界e\u{301}\r\nend");
            assert_eq!(e.session.view().point.position, point);
            assert_eq!(e.session.buffer().unwrap().mark(), mark);
            assert!(e.session.buffer().unwrap().mark_active());
            e.session.execute(Command::History(History::Redo));
            settle(&mut e.session);
            e.expect(expected);
            assert!(!e.session.buffer().unwrap().mark_active());
        }
    }
}

// Automatic preferences affect typed wrapping only. Zero and negative prefixes
// keep their existing inert/refusal contract without touching selection/history.
#[test]
fn policy() {
    for config in [
        "pairs.enabled=false",
        "pairs.wrap_selection=false",
        "pairs.quotes=false",
    ] {
        let mut e = selected(config, "plain", "alpha", false);
        e.type_key('"');
        e.expect("alpha\"|");
    }
    for count in ["0", "-2", "257"] {
        let mut e = selected("", "plain", "alpha", false);
        let point = e.session.view().point;
        let mark = e.session.buffer().unwrap().mark();
        e.counted(count, '(');
        assert_eq!(e.text(), "alpha");
        assert_eq!(e.session.view().point, point);
        assert_eq!(e.session.buffer().unwrap().mark(), mark);
        assert!(e.session.buffer().unwrap().mark_active());
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}

// Unsafe quote transformations refuse atomically; selected bytes are never
// escaped or rewritten. Ordinary character and raw-string profiles remain distinct.
#[test]
fn quotes() {
    for (language, text, c, expected) in [
        ("c", "x", '\'', Some("'|x'")),
        ("c", "word", '\'', None),
        ("go", "界", '\'', Some("'|界'")),
        ("go", "a\\b\nline", '`', Some("`|a\\b\nline`")),
        ("rust", "word", '\'', None),
        ("plain", "a\"b", '"', None),
        ("plain", "a\\b", '"', None),
        ("json", "a\nb", '"', None),
        ("toml", "a\\b", '\'', Some("'|a\\b'")),
    ] {
        let mut e = selected("", language, text, false);
        let point = e.session.view().point;
        e.type_key(c);
        match expected {
            Some(expected) => e.expect(expected),
            None => {
                assert_eq!(e.text(), text);
                assert_eq!(e.session.view().point, point);
                assert!(e.session.buffer().unwrap().history().is_empty());
                assert!(!e.session.echo().is_empty());
            }
        }
    }
    let mut e = selected("", "plain", "alpha", false);
    e.counted("2", '"');
    assert_eq!(e.text(), "alpha");
    assert!(e.session.buffer().unwrap().mark_active());
}
