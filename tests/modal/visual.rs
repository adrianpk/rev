use super::navigation::{keys, settle};
use super::*;
use rev::buffer::Visual;
use rev::command::RegionCommand;

pub(super) fn region(editor: &Session) -> String {
    let buffer = editor.buffer().unwrap();
    buffer
        .slice(&buffer.region(editor.view().point).unwrap().unwrap())
        .unwrap()
        .to_string()
}

pub(super) fn block(editor: &mut Session) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    settle(editor);
}

// Inclusive native regions retain whole Unicode graphemes and CRLF clusters.
// Reversing direction changes the active endpoint without losing the anchor.
#[test]
fn character_regions() {
    for (name, source, point, input, selected) in [
        ("current", "e\u{301}界!", 0, "v", "e\u{301}"),
        ("forward", "e\u{301}界!", 0, "vl", "e\u{301}界"),
        ("backward", "e\u{301}界!", 2, "vh", "e\u{301}界"),
        ("newline", "a\r\nb", 0, "vl", "a\r\n"),
        ("empty", "", 0, "v", ""),
        ("eof", "abc", 3, "v", ""),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(region(&editor), selected, "{name}");
        assert_eq!(editor.buffer().unwrap().visual(), Some(Visual::Character));
        assert_eq!(text(&editor), source);
        assert!(editor.buffer().unwrap().history().is_empty());
    }
    let mut editor = session(VIM, "abcde", 1);
    keys(&mut editor, "v2lo");
    assert_eq!(editor.view().point.position, Scalar(1));
    assert_eq!(editor.buffer().unwrap().mark(), Some(Scalar(3)));
    assert_eq!(region(&editor), "bcd");
}

// Shape conversion preserves the shared anchor; repeated entry and recovery
// end presentation without creating history or altering source text.
#[test]
fn shape_transitions() {
    let mut editor = session(VIM, "ab\r\ncd\r\nef", 1);
    keys(&mut editor, "vjlV");
    assert_eq!(editor.editing_state(), Some(EditingState::VisualLine));
    assert_eq!(region(&editor), "ab\r\ncd\r\n");
    assert_eq!(editor.buffer().unwrap().mark(), Some(Scalar(1)));
    block(&mut editor);
    assert_eq!(editor.editing_state(), Some(EditingState::VisualBlock));
    keys(&mut editor, "v");
    assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
    keys(&mut editor, "v");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(!editor.buffer().unwrap().mark_active());
    keys(&mut editor, "V");
    editor.handle(key(KeyCode::Esc));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "v");
    editor.execute(Command::KeyboardQuit);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(!editor.buffer().unwrap().mark_active());
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Immediate Visual operations use native prepared edits and one history batch;
// change admits Insert only after publication and line copy preserves its shape.
#[test]
fn visual_operators() {
    for (source, point, input, expected, copied, state) in [
        ("one two", 0, "v2ld", " two", "one", EditingState::Normal),
        ("one two", 0, "v2lc", " two", "one", EditingState::Insert),
        (
            "one\r\ntwo",
            1,
            "Vd",
            "two",
            "one\r\n",
            EditingState::Normal,
        ),
        (
            "one\r\ntwo",
            1,
            "Vc",
            "\r\ntwo",
            "one\r\n",
            EditingState::Insert,
        ),
        (
            "one\ntwo",
            1,
            "Vyp",
            "one\none\ntwo",
            "one\n",
            EditingState::Normal,
        ),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{input}: {}", editor.echo());
        assert_eq!(editor.killed_text().unwrap().to_string(), copied);
        assert_eq!(editor.editing_state(), Some(state));
        assert!(!editor.buffer().unwrap().mark_active());
        editor.execute(Command::History(History::Undo));
        assert_eq!(text(&editor), source);
    }
}

// Block columns count each tab and wide/combining grapheme once. Short rows
// contribute empty fragments, and deletion never consumes a line separator.
#[test]
fn logical_block() {
    let source = "a\t界!\nb界e\u{301}Z\nx\n1234";
    let mut editor = session(VIM, source, 1);
    block(&mut editor);
    editor.execute(Command::Motion(rev::command::Motion::BufferEnd));
    keys(&mut editor, "hy");
    assert_eq!(
        editor.rectangle().unwrap().rows(),
        ["\t界!", "界e\u{301}Z", "", "234"]
    );
    assert_eq!(editor.rectangle().unwrap().width(), 3);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(editor.buffer().unwrap().history().is_empty());
    let mut editor = session(VIM, source, 1);
    block(&mut editor);
    editor.execute(Command::Motion(rev::command::Motion::BufferEnd));
    keys(&mut editor, "hd");
    assert_eq!(text(&editor), "a\nb\nx\n1");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), source);
}

// Linear Visual text objects replace the shared endpoints; block state rejects
// object/indentation commands rather than quietly applying a linear operation.
#[test]
fn visual_objects() {
    for (input, expected) in [("viwy", "one"), ("vawy", "one "), ("v2iwy", "one two")] {
        let mut editor = session(VIM, "one two three", 1);
        keys(&mut editor, input);
        assert_eq!(
            editor.killed_text().unwrap().to_string(),
            expected,
            "{input}"
        );
        assert_eq!(text(&editor), "one two three");
    }
    let mut editor = session(VIM, "one\ntwo", 0);
    block(&mut editor);
    keys(&mut editor, ">iw");
    assert_eq!(text(&editor), "one\ntwo");
    assert_eq!(editor.editing_state(), Some(EditingState::VisualBlock));
    editor.execute(Command::Region(RegionCommand::Exchange));
    assert_eq!(editor.buffer().unwrap().visual(), Some(Visual::Block));
}
