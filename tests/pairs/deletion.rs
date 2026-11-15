use super::editor::*;
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, History, ModalCommand, Motion, RegionCommand, Split, WindowCommand};
use rev::coordinates::Scalar;

fn backspace(e: &mut Editor) {
    e.session
        .handle(key(KeyCode::Backspace, KeyModifiers::NONE));
    settle(&mut e.session);
}

// Only a confirmed adjacent empty pair extends the ordinary deletion range.
// Whitespace, malformed context and literal text cannot donate a closing byte.
#[test]
fn admission() {
    for (name, language, source, expected) in [
        ("parentheses", "plain", "(|)", "|"),
        ("brackets", "plain", "[|]", "|"),
        ("braces", "plain", "{|}", "|"),
        ("nested", "plain", "([|])", "(|)"),
        ("double", "plain", "\"|\"", "|"),
        ("single", "c", "'|'", "|"),
        ("raw", "go", "`|`", "|"),
        ("space before", "plain", "( |)", "(|)"),
        ("space after", "plain", "(| )", "| )"),
        ("content after", "plain", "(|x)", "|x)"),
        ("content before", "plain", "(x|)", "(|)"),
        ("closing deletion", "plain", "()|", "(|"),
        ("crossed", "plain", "([)|]", "([|]"),
        ("prior mismatch", "plain", "[(](|)", "[(]|)"),
        ("comment", "rust", "// (|)", "// |)"),
        ("string", "rust", "\"(|)\"", "\"|)\""),
        ("escaped quote", "rust", "\"x\\\"|\"", "\"x\\|\""),
        ("apostrophe", "plain", "don'|'t", "don|'t"),
        ("Rust lifetime", "rust", "'a (|)", "'a |)"),
        ("Rust character", "rust", "'|'", "|'"),
        ("template", "javascript", "`x ${(|)}`", "`x ${|)}`"),
        ("fence", "markdown", "```text\n(|)\n```", "```text\n|)\n```"),
        ("Org verbatim", "org", "=(|)=", "=|)="),
        ("HTML attribute", "html", "<a x=\"|\">", "<a x=|>"),
        ("HTML text", "html", "text \"|\"", "text |\""),
        (
            "CRLF Unicode",
            "plain",
            "界🙂e\u{301}\r\n(|)\r\n",
            "界🙂e\u{301}\r\n|\r\n",
        ),
        ("combining before", "plain", "(e\u{301}|)", "(|)"),
        ("combining closer", "plain", "(|)\u{301}", "|)\u{301}"),
        ("combining opener", "plain", "(\u{301}|)", "|)"),
        ("CRLF grapheme", "plain", "(\r\n|)", "(|)"),
    ] {
        let mut e = Editor::new("", language, source);
        backspace(&mut e);
        e.expect(expected);
        assert!(e.session.echo().is_empty(), "{name}: {}", e.session.echo());
    }
}

// Each automatic-editing leaf is independent. Language overrides and the
// shared runtime toggle affect deletion without changing startup settings.
#[test]
fn policy() {
    for (config, source, expected) in [
        ("pairs.enabled=false", "(|)", "|)"),
        ("pairs.delete_empty=false", "(|)", "|)"),
        ("pairs.quotes=false", "\"|\"", "|\""),
        ("pairs.quotes=false", "(|)", "|"),
        ("pairs.skip_closing=false", "(|)", "|"),
        ("language.rust.pairs.delete_empty=false", "(|)", "|)"),
    ] {
        let mut e = Editor::new(config, "rust", source);
        backspace(&mut e);
        e.expect(expected);
    }
    let mut e = Editor::new("", "plain", "(|)");
    e.session.execute(Command::Split(Split::Right));
    e.session.execute(Command::AutoPairMode);
    e.session.execute(Command::Window(WindowCommand::Other));
    backspace(&mut e);
    e.expect("|)");
    e.session.execute(Command::History(History::Undo));
    settle(&mut e.session);
    e.session.execute(Command::AutoPairMode);
    backspace(&mut e);
    e.expect("|");
}

// One deletion publishes both ends once. Native history restores the original
// initiating point/mark; other live points use the documented right affinity.
#[test]
fn history() {
    for (source, expected) in [("界([|])\r\n", "界(|)\r\n"), ("界 \"|\"\r\n", "界 |\r\n")] {
        let mut e = Editor::new("", "plain", source);
        e.session.execute(Command::Region(RegionCommand::Set));
        e.session.execute(Command::KeyboardQuit);
        e.session.execute(Command::Split(Split::Right));
        let initiating = e.session.windows().selected_id();
        let mark = e.session.buffer().unwrap().mark();
        backspace(&mut e);
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().history().len(), 1);
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(2)));
        e.session.execute(Command::History(History::Undo));
        settle(&mut e.session);
        e.expect(source);
        assert_eq!(e.session.buffer().unwrap().mark(), mark);
        for (id, view) in e.session.windows().iter() {
            let expected = if id == initiating {
                Scalar(3)
            } else {
                Scalar(4)
            };
            assert_eq!(view.point.position, expected);
            assert_eq!(view.point.stamp, e.session.buffer().unwrap().stamp());
        }
        e.session.execute(Command::History(History::Redo));
        settle(&mut e.session);
        e.expect(expected);
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(2)));
        assert!(!e.session.buffer().unwrap().mark_active());
    }
}

// Active regions retain the native deletion owner in either orientation.
// A physical Backspace must never extend a region to consume an extra closer.
#[test]
fn regions() {
    for (source, motion, expected) in [
        ("|()", Motion::ForwardChar, "|)"),
        ("(|)", Motion::BackwardChar, "|()"),
    ] {
        let mut e = Editor::new("", "plain", source);
        e.session.execute(Command::Region(RegionCommand::Set));
        e.session.execute(Command::Motion(motion));
        backspace(&mut e);
        e.expect(expected);
    }
}

// Non-source focus and read-only state cannot admit a pair edit. Modal text
// entry uses the same source transaction; Normal retains its existing keymap.
#[test]
fn focus_modes() {
    for command in [
        Command::FindFile,
        Command::SearchForward,
        Command::ExecuteExtended,
    ] {
        let mut e = Editor::new("", "plain", "(|)");
        e.session.execute(command);
        backspace(&mut e);
        assert_eq!(e.text(), "()");
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
    let mut e = Editor::build("", "plain", "(|)", true);
    backspace(&mut e);
    e.expect("(|)");
    assert!(e.session.buffer().unwrap().history().is_empty());
    let mut e = Editor::new("", "plain", "|()");
    let point = e.session.view().point;
    backspace(&mut e);
    e.expect("|()");
    assert_eq!(e.session.view().point, point);
    assert_eq!(e.session.echo(), "Buffer boundary");
    assert!(e.session.buffer().unwrap().history().is_empty());
    for state in [ModalCommand::Insert, ModalCommand::Replace] {
        let mut e = Editor::new("editor.interaction_profile='modal'", "plain", "(|)");
        e.session.execute(Command::Modal(state));
        backspace(&mut e);
        e.expect("|");
        e.session.execute(Command::History(History::Undo));
        settle(&mut e.session);
        e.expect("(|)");
    }
}

// A cold prefix requires several cooperative turns. C-g, changed source and
// queued text exercise the existing FIFO and stale-proof publication checks.
#[test]
fn pending() {
    let prefix = "x ".repeat(2048);
    let source = format!("{prefix}(|)");
    let mut e = Editor::new("", "plain", &source);
    e.session
        .handle(key(KeyCode::Backspace, KeyModifiers::NONE));
    e.session
        .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    settle(&mut e.session);
    e.expect(&source);
    assert!(e.session.buffer().unwrap().history().is_empty());
    e.session
        .handle(key(KeyCode::Backspace, KeyModifiers::NONE));
    e.session.replace(0..0, "changed ").unwrap();
    let changed = e.session.view().point;
    settle(&mut e.session);
    assert_eq!(e.text(), format!("changed {}", source.replace('|', "")));
    assert_eq!(e.session.view().point, changed);
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    let mut e = Editor::new("", "plain", &source);
    e.session
        .handle(key(KeyCode::Backspace, KeyModifiers::NONE));
    e.session.handle(character('a'));
    settle(&mut e.session);
    e.expect(&format!("{prefix}a|"));
}

// Budget exhaustion keeps ordinary Backspace available without a guessed mate.
#[test]
fn limited() {
    let prefix = "x".repeat(rev::pairs::MAX_BYTES + 1);
    let mut e = Editor::new("", "plain", &format!("{prefix}(|)"));
    backspace(&mut e);
    e.expect(&format!("{prefix}|)"));
}
