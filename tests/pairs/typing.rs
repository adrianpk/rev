#[path = "typing/lifecycle.rs"]
mod lifecycle;

use super::editor::*;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, ModalCommand, Motion, Split};
use rev::coordinates::Scalar;
use std::time::{Duration, Instant};

// Independent expected edits cover balance on both sides, not only an adjacent
// closer. Existing balanced outer pairs cannot donate their closer to new input.
#[test]
fn insertion() {
    for (name, source, c, expected) in [
        ("parentheses", "|", '(', "(|)"),
        ("brackets", "|", '[', "[|]"),
        ("braces", "|", '{', "{|}"),
        ("balanced outer", "(|)", '(', "((|))"),
        ("free closer", "|)", '(', "(|)"),
        ("free later closer", "|text)", '(', "(|text)"),
        ("free after balanced", "|[])", '(', "(|[])"),
        ("free outside outer", "(|))", '(', "((|))"),
        ("mismatch", "|]", '(', "(|]"),
        ("prior mismatch", "[)|", '(', "[)(|"),
        (
            "Unicode CRLF",
            "界🙂e\u{301}\r\n|",
            '[',
            "界🙂e\u{301}\r\n[|]",
        ),
        ("angles literal", "|", '<', "<|"),
    ] {
        let mut e = Editor::new("", "plain", source);
        e.type_key(c);
        e.expect(expected);
        assert!(e.session.echo().is_empty(), "{name}: {}", e.session.echo());
    }
}

// Only a confirmed, unescaped native closer can move point without history.
// Unmatched and crossed punctuation remains editable as ordinary literal text.
#[test]
fn skipping() {
    for (language, source, c, expected, changed) in [
        ("plain", "(|)", ')', "()|", false),
        ("plain", "(|)\u{301}", ')', "()\u{301}|", false),
        ("plain", "([|])", ']', "([]|)", false),
        ("plain", "|)", ')', ")|)", true),
        ("plain", "([|)]", ')', "([)|)]", true),
        ("plain", "\"x|\"", '"', "\"x\"|", false),
        ("rust", "\"x\\|\"", '"', "\"x\\\"|\"", true),
        ("rust", "// (|)", ')', "// ()|)", true),
        ("rust", "\"(|)\"", ')', "\"()|)\"", true),
    ] {
        let mut e = Editor::new("", language, source);
        let stamp = e.session.buffer().unwrap().stamp();
        e.type_key(c);
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().stamp() != stamp, changed);
        assert_eq!(
            e.session.buffer().unwrap().history().len(),
            usize::from(changed)
        );
        assert!(e.session.echo().is_empty(), "{}", e.session.echo());
    }
}

// Quotes require the declared language and an eligible boundary. This protects
// literal comments, words, raw/template forms and markup attributes without colors.
#[test]
fn quote_contexts() {
    for (language, source, c, expected) in [
        ("plain", "|", '"', "\"|\""),
        ("plain", "don|t", '\'', "don'|t"),
        ("plain", "word|", '"', "word\"|"),
        ("plain", "|word", '"', "\"|word"),
        ("rust", "let x = |", '"', "let x = \"|\""),
        ("rust", "r|", '"', "r\"|"),
        ("rust", "|", '\'', "'|"),
        ("c", "|", '\'', "'|'"),
        ("go", "|", '`', "`|`"),
        ("javascript", "|", '\'', "'|'"),
        ("javascript", "`x ${|}`", '(', "`x ${(|}`"),
        ("toml", "|", '\'', "'|'"),
        ("html", "<a x=|>", '"', "<a x=\"|\">"),
        ("html", "<a |>", '(', "<a (|>"),
        ("html", "|text", '"', "\"|text"),
        ("org", "=|=", '(', "=(|="),
        ("markdown", "`|`", '[', "`[|`"),
    ] {
        let mut e = Editor::new("", language, source);
        e.type_key(c);
        e.expect(expected);
    }
}

// Positive counts compose nested structural effects and reuse free closers
// once. Zero/negative input retains the established prefix admission contract.
#[test]
fn counts() {
    for (source, count, c, expected) in [
        ("|", "3", '(', "(((|)))"),
        ("|))", "3", '(', "(((|)))"),
        ("|a)b)", "3", '(', "(((|)a)b)"),
        ("((|))", "2", ')', "(())|"),
        ("((|))", "3", ')', "(()))|"),
        ("(|)\u{301}", "2", ')', "()\u{301})|"),
        ("|", "0", '(', "|"),
        ("|", "-2", '(', "|"),
        ("|", "2", '"', "\"\"|"),
        ("|", "3", '"', "\"\"\"|"),
        ("|\"abc\"", "2", '"', "\"\"|abc\""),
        ("|\"", "3", '"', "\"\"\"|"),
        ("|\"\"", "3", '"', "\"\"\"|\"\""),
        ("\"|\"", "3", '"', "\"\"\"\"|"),
    ] {
        let mut e = Editor::new("", "plain", source);
        e.counted(count, c);
        e.expect(expected);
        assert!(e.session.buffer().unwrap().history().len() <= 1);
    }
}

// The source owner admits at most depth 256, even for a single counted key.
// Exhaustion leaves every byte, point, mark and history group unchanged.
#[test]
fn count_bounds() {
    for (source, count, admitted) in [
        ("|", "256", true),
        ("|", "257", false),
        ("(|)", "256", false),
    ] {
        let mut e = Editor::new("", "plain", source);
        let stamp = e.session.buffer().unwrap().stamp();
        let point = e.session.view().point;
        e.counted(count, '(');
        if admitted {
            e.expect(&format!("{}|{}", "(".repeat(256), ")".repeat(256)));
            assert_eq!(e.session.buffer().unwrap().history().len(), 1);
        } else {
            e.expect(source);
            assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
            assert_eq!(e.session.view().point, point);
            assert!(e.session.buffer().unwrap().history().is_empty());
            assert!(!e.session.echo().is_empty());
        }
    }
}

// The complete admitted pair uses one native transaction. A shared view maps
// through it, while undo/redo restore the initiating cursor and exact UTF-8 bytes.
#[test]
fn history() {
    let mut e = Editor::new("", "plain", "界|\r\n");
    e.session.execute(Command::Split(Split::Right));
    let before = e.session.view().point;
    e.counted("3", '[');
    e.expect("界[[[|]]]\r\n");
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    assert!(
        e.session
            .windows()
            .iter()
            .all(|(_, v)| v.point.stamp == e.session.buffer().unwrap().stamp())
    );
    e.session.execute(Command::History(History::Undo));
    settle(&mut e.session);
    e.expect("界|\r\n");
    assert_eq!(e.session.view().point.position, before.position);
    e.session.execute(Command::History(History::Redo));
    settle(&mut e.session);
    e.expect("界[[[|]]]\r\n");
}

// Modal state owns whether printable input is text. Replace counts only the
// physical characters when choosing overwritten graphemes, never generated mates.
#[test]
fn modal_entry() {
    for (state, source, c, expected) in [
        (ModalCommand::Insert, "|abc", '(', "(|)abc"),
        (ModalCommand::Replace, "|abc", '(', "(|)bc"),
        (ModalCommand::Replace, "|)", '(', "(|)"),
        (ModalCommand::Replace, "(|)abc", ')', "()|abc"),
        (ModalCommand::Normal, "|abc", '(', "|abc"),
    ] {
        let mut e = Editor::new("editor.interaction_profile='modal'", "plain", source);
        e.session.execute(Command::Modal(state));
        e.type_key(c);
        e.expect(expected);
    }
}

// Pair settings change only the corresponding automatic effect. Literal paste
// and direct native replacement retain their source provenance at every count.
#[test]
fn literal_policy() {
    for (config, c, source, expected) in [
        ("pairs.enabled=false", '(', "|", "(|"),
        ("pairs.quotes=false", '"', "|", "\"|"),
        ("pairs.skip_closing=false", ')', "(|)", "()|)"),
    ] {
        let mut e = Editor::new(config, "plain", source);
        e.type_key(c);
        e.expect(expected);
    }
    let mut e = Editor::new("", "plain", "|");
    e.session.handle(Event::Paste("(".into()));
    settle(&mut e.session);
    e.expect("(|");
    e.session.replace(0..1, "[").unwrap();
    e.expect("[|");
    e.session.execute(Command::Motion(Motion::BufferEnd));
    e.session
        .handle(key(KeyCode::Char('u'), KeyModifiers::CONTROL));
    e.session.handle(character('3'));
    e.session.handle(Event::Paste("{".into()));
    settle(&mut e.session);
    e.expect("[{{{|");
}

// Symmetric counts follow repeated admitted typing, including an existing quote
// and disabled overtyping. Generated quotes never become recursive key events.
#[test]
fn quote_counts() {
    for (config, language, source, count, c, expected) in [
        ("", "go", "|`", "2", '`', "``|"),
        ("", "go", "|``", "2", '`', "``|`"),
        ("", "c", "|'", "2", '\'', "''|"),
        (
            "pairs.skip_closing=false",
            "plain",
            "|",
            "2",
            '"',
            "\"\"|\"",
        ),
        (
            "pairs.skip_closing=false",
            "plain",
            "|\"",
            "2",
            '"',
            "\"\"|\"",
        ),
    ] {
        let mut e = Editor::new(config, language, source);
        e.counted(count, c);
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    }
}

// A physical skip completes the same native modal insertion capture. Semantic
// repeat replays its accepted source effects once, without synthesizing mates.
#[test]
fn modal_repeat() {
    let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "|target");
    e.session.execute(Command::Modal(ModalCommand::Insert));
    for c in ['(', 'a', ')'] {
        e.type_key(c);
    }
    e.session.execute(Command::Modal(ModalCommand::Normal));
    assert_eq!(e.text(), "(a)target");
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::Edit(rev::command::Editing::new(
        rev::command::editing::Action::Repeat,
    )));
    settle(&mut e.session);
    assert_eq!(e.text(), "(a)(a)target");
    e.session.execute(Command::History(History::Undo));
    settle(&mut e.session);
    assert_eq!(e.text(), "(a)target");
}
