use super::navigation::keys;
use super::*;

// A complete operator resolves once against native endpoints and creates one
// transaction. Counts multiply before resolution; e preserves its separator.
#[test]
fn operator_ranges() {
    for (name, source, point, input, expected, retained) in [
        ("word start", "one two", 0, "dw", "two", "one "),
        ("word end", "one two", 0, "de", " two", "one"),
        (
            "multiplied count",
            "a b c d e f g",
            0,
            "2d3w",
            "g",
            "a b c d e f ",
        ),
        ("backward word", "one two", 4, "db", "two", "one "),
        (
            "logical end",
            "one two\r\nnext",
            4,
            "D",
            "one \r\nnext",
            "two",
        ),
        (
            "two lines",
            "one\ntwo\nthree",
            1,
            "2dd",
            "three",
            "one\ntwo\n",
        ),
        (
            "line motion",
            "one\ntwo\nthree",
            1,
            "dj",
            "three",
            "one\ntwo\n",
        ),
        (
            "buffer end",
            "one\ntwo\nthree",
            5,
            "dG",
            "one\n",
            "two\nthree",
        ),
        ("Unicode", "e\u{301}界 next", 0, "dl", "界 next", "e\u{301}"),
        ("find boundary", "abcXdef", 0, "dfX", "Xdef", "abc"),
        ("till boundary", "abcXdef", 0, "dtX", "cXdef", "ab"),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert_eq!(
            editor.killed_text().unwrap().to_string(),
            retained,
            "{name}"
        );
        assert_eq!(editor.buffer().unwrap().history().len(), 1, "{name}");
        keys(&mut editor, "u");
        assert_eq!(text(&editor), source, "{name}");
        assert_eq!(editor.view().point.position, Scalar(point), "{name}");
    }
}

// Copy uses the same retained text without history; change enters Insert only
// after publication, and a line change preserves the empty logical insertion slot.
#[test]
fn copy_and_change() {
    let mut editor = session(VIM, "one two\nnext", 0);
    keys(&mut editor, "yw");
    assert_eq!(editor.killed_text().unwrap().to_string(), "one ");
    assert!(editor.buffer().unwrap().history().is_empty());
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "cw");
    assert_eq!(text(&editor), "two\nnext");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    editor.handle(key(KeyCode::Esc));
    keys(&mut editor, "u");
    keys(&mut editor, "cc");
    assert_eq!(text(&editor), "\nnext");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    editor.handle(key(KeyCode::Esc));
    keys(&mut editor, "uY");
    assert_eq!(editor.killed_text().unwrap().to_string(), "one two");
}

// Pending syntax leaves point/text untouched. Overflow, missing targets and a
// source change retire the whole composition without replacing retained text.
#[test]
fn rejected_operators() {
    for input in ["d", "2d3", "di", "df", "4294967295d2w", "d9fZ", "diz"] {
        let mut editor = session(VIM, "one two", 0);
        keys(&mut editor, input);
        assert_eq!(text(&editor), "one two", "{input}");
        assert_eq!(editor.view().point.position, Scalar(0), "{input}");
        assert!(editor.buffer().unwrap().history().is_empty(), "{input}");
        assert!(editor.killed_text().is_none(), "{input}");
    }
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "d");
    editor.handle(key(KeyCode::Esc));
    keys(&mut editor, "w");
    assert_eq!(editor.view().point.position, Scalar(4));
    keys(&mut editor, "d");
    editor.insert('X');
    keys(&mut editor, "w");
    assert_eq!(text(&editor), "one Xtwo");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}

// A shared incremental-search target remains a preview until confirmation;
// cancellation restores origin, and acceptance applies one checked operator.
#[test]
fn operator_search() {
    let mut editor = session(VIM, "one two three", 0);
    keys(&mut editor, "d/two");
    assert_eq!(text(&editor), "one two three");
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "d/two");
    editor.handle(key(KeyCode::Enter));
    assert_eq!(text(&editor), " three");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "one two three");
}
