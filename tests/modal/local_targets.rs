use super::navigation::keys;
use super::*;

// Find/till consume one Unicode scalar but land on checked grapheme boundaries;
// a missing counted match is atomic and never crosses a logical separator.
#[test]
fn line_find() {
    for (name, source, point, input, expected) in [
        ("wide scalar", "a界b界c", 0, "2f界", 3),
        ("backward", "a界b界c", 4, "F界", 3),
        ("till forward", "a界b界c", 0, "2t界", 2),
        ("till backward", "a界b界c", 4, "T界", 4),
        ("combining scalar", "a e\u{301} z", 0, "f\u{301}", 2),
        ("missing count", "a界b界c", 0, "3f界", 0),
        ("line local", "abc\r\nx", 0, "fx", 0),
        ("reverse local", "x\r\nabc", 5, "Fx", 5),
        ("no split CRLF", "abc\r\nx", 0, "f\r", 0),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(
            editor.view().point.position,
            Scalar(expected),
            "{name}: {}",
            editor.echo()
        );
        assert_eq!(text(&editor), source, "{name}");
    }
    let mut editor = session(VIM, "a界b界c界d", 0);
    keys(&mut editor, "t界;");
    assert_eq!(editor.view().point.position, Scalar(2));
    keys(&mut editor, ";");
    assert_eq!(editor.view().point.position, Scalar(4));
    keys(&mut editor, ",");
    assert_eq!(editor.view().point.position, Scalar(4));
    keys(&mut editor, "h;");
    assert_eq!(editor.view().point.position, Scalar(4));
}

// Pending and retained finds cannot reuse a changed native source or pane.
// Paste/cancel clear pending syntax and cannot be mistaken for a character.
#[test]
fn find_retirement() {
    let mut editor = session(VIM, "abcabc", 0);
    keys(&mut editor, "f");
    editor.handle(Event::Paste("c".into()));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
    keys(&mut editor, "fc");
    assert_eq!(editor.view().point.position, Scalar(2));
    keys(&mut editor, "iX");
    editor.handle(key(KeyCode::Esc));
    let point = editor.view().point;
    keys(&mut editor, ";");
    assert_eq!(editor.view().point, point);
    assert!(editor.echo().contains("No current find"));
    keys(&mut editor, "f");
    editor.execute(Command::Split(rev::command::Split::Right));
    editor.execute(Command::Window(rev::command::WindowCommand::Other));
    keys(&mut editor, "c");
    assert_eq!(text(&editor), "abXcabc");
}

// Paragraph boundaries and nested delimiter targets are source positions;
// missing mates do not expose a partial nesting traversal as a movement.
#[test]
fn structural_targets() {
    for (name, source, point, input, expected) in [
        ("next paragraph", "one\n\ntwo\n\nthree", 0, "}", 4),
        ("count paragraphs", "one\n\ntwo\n\nthree", 0, "2}", 9),
        ("previous paragraph", "one\n\ntwo\n\nthree", 12, "{", 9),
        ("nested pair", "a (b [c] (d)) z", 0, "%", 12),
        ("backward pair", "a (b [c] (d)) z", 12, "%", 2),
        ("missing pair", "(missing", 0, "%", 0),
        ("no delimiter", "one\n(two)", 0, "%", 0),
        ("Unicode around pair", "界(e\u{301})", 0, "%", 4),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(
            editor.view().point.position,
            Scalar(expected),
            "{name}: {}",
            editor.echo()
        );
        assert!(editor.buffer().unwrap().history().is_empty());
    }
}
