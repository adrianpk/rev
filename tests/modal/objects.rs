use super::navigation::keys;
use super::*;

// Inner/around word, WORD and paragraph objects select native grapheme/line
// ranges; punctuation classes and surrounding whitespace have explicit roles.
#[test]
fn word_objects() {
    for (name, source, point, input, expected) in [
        ("inner word", "one two", 1, "diw", " two"),
        ("around word", "one two", 1, "daw", "two"),
        ("trailing word", "one two", 5, "daw", "one"),
        ("inner punctuation", "one,!!! two", 4, "diw", "one two"),
        ("inner WORD", "one,!!! two", 3, "diW", " two"),
        ("around WORD", "one,!!! two", 3, "daW", "two"),
        ("counted inner", "one two three", 0, "d2iw", " three"),
        ("combining", "e\u{301}界 next", 0, "diw", " next"),
        ("inner paragraph", "one\ntwo\n\nthree", 4, "dip", "\nthree"),
        ("around paragraph", "one\ntwo\n\nthree", 4, "dap", "three"),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert_eq!(editor.buffer().unwrap().history().len(), 1, "{name}");
        keys(&mut editor, "u");
        assert_eq!(text(&editor), source, "{name}");
    }
}

// Paired and quoted objects resolve nesting, closing aliases and escapes
// without syntax parsing; absent delimiters and nesting counts remain atomic.
#[test]
fn delimited_objects() {
    for (name, source, point, input, expected) in [
        ("inner pair", "a (b (c) d) z", 6, "di(", "a (b () d) z"),
        ("around outer", "a (b (c) d) z", 6, "d2a)", "a  z"),
        ("brackets", "a [b] z", 3, "di]", "a [] z"),
        ("braces", "a {b} z", 3, "da}", "a  z"),
        ("double quote", "a \"b c\" z", 4, "di\"", "a \"\" z"),
        ("single quote", "a 'b c' z", 4, "da'", "a  z"),
        ("backtick", "a `b c` z", 4, "di`", "a `` z"),
        ("escaped quote", "a \"b\\\"c\" z", 4, "di\"", "a \"\" z"),
        ("missing close", "a (b c", 4, "di(", "a (b c"),
        ("missing nesting", "a (b) c", 3, "d2i(", "a (b) c"),
        ("missing quote", "a b c", 2, "di\"", "a b c"),
        ("line local quote", "a \"b\nc\"", 3, "di\"", "a \"b\nc\""),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            usize::from(source != expected),
            "{name}"
        );
    }
}
