use super::navigation::keys;
use super::*;

// Linewise transfer shares the existing ring, but adds only the separators
// needed by its native shape at EOF. Linear paste retains native yank endpoints.
#[test]
fn shaped_paste() {
    for (name, source, point, input, expected) in [
        ("unterminated line after", "one", 0, "yyp", "one\none"),
        ("unterminated line before", "one", 0, "yyP", "one\none"),
        ("terminated line", "one\n", 0, "yyp", "one\none\n"),
        ("middle line", "one\ntwo", 0, "yyp", "one\none\ntwo"),
        ("last line", "one\ntwo", 4, "yyp", "one\ntwo\ntwo"),
        ("counted line", "one", 0, "yy3p", "one\none\none\none"),
        ("CRLF", "one\r\ntwo", 0, "yyp", "one\r\none\r\ntwo"),
        ("linear before", "one two", 0, "ywP", "one one two"),
        ("linear after", "one two", 0, "ywp", "oone ne two"),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert_eq!(editor.buffer().unwrap().history().len(), 1, "{name}");
        keys(&mut editor, "u");
        assert_eq!(text(&editor), source, "{name}");
    }
}

// Small deletion deliberately leaves retained text untouched. Replacement
// operates on whole graphemes, is line-local and refuses incomplete counts.
#[test]
fn direct_edits() {
    for (name, source, point, input, expected) in [
        ("forward grapheme", "e\u{301}界!", 0, "x", "界!"),
        ("backward grapheme", "e\u{301}界!", 2, "X", "界!"),
        ("counted delete", "abc\r\ndef", 0, "9x", "\r\ndef"),
        ("backward line bound", "abc\r\ndef", 5, "X", "abc\r\ndef"),
        ("replace graphemes", "e\u{301}界!", 0, "2rλ", "λλ!"),
        ("missing replacement", "a\r\nb", 0, "2rx", "a\r\nb"),
        ("huge replacement", "abc", 0, "4294967295rx", "abc"),
        ("join", "one\n  two\nthree", 1, "J", "one two\nthree"),
        ("join blank lines", "one\n\nthree", 1, "3J", "one three"),
        (
            "counted join",
            "one\r\n  two\r\nthree",
            1,
            "3J",
            "one two three",
        ),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert!(editor.killed_text().is_none(), "{name}");
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            usize::from(source != expected),
            "{name}"
        );
    }
}

// Shifts resolve intersecting logical lines and publish one native batch;
// source policies choose the unit, including a partially consumed leading tab.
#[test]
fn indentation() {
    let config = "[editor]\ninteraction_profile = 'modal'\nindent_style = 'spaces'\nindent_width = 2\ntab_width = 4\n[tree]\nvisible = false";
    let mut editor = session(config, "a\n b\n\tc", 0);
    keys(&mut editor, "2>>");
    assert_eq!(text(&editor), "  a\n   b\n\tc");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "a\n b\n\tc");
    keys(&mut editor, "G<<");
    assert_eq!(text(&editor), "a\n b\n  c");
    keys(&mut editor, "u>j");
    assert_eq!(text(&editor), "a\n b\n  \tc");
}
