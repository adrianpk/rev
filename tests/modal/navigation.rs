use super::*;
use rev::command::{
    Motion,
    navigation::{Navigation, Target, Word},
};

pub(super) fn keys(editor: &mut Session, input: &str) {
    for character in input.chars() {
        editor.handle(key(KeyCode::Char(character)));
    }
    settle(editor);
}

pub(super) fn settle(editor: &mut Session) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        editor.frame().unwrap();
        let now = std::time::Instant::now();
        if editor.remaining_input(now).is_none() {
            return;
        }
        assert!(now < end, "native layout did not settle: {}", editor.echo());
        editor.expire_input(now);
        std::thread::yield_now();
    }
}

// Shared endpoints retain Odra's punctuation-run w, native word-start b and
// exclusive word-end e. Additional WORD/ge/count forms are rev requirements.
#[test]
fn word_endpoints() {
    for (name, source, point, input, expected) in [
        ("punctuation run", "one 🙂two", 0, "w", 4),
        ("leave punctuation", "one 🙂two", 4, "w", 5),
        ("Unicode start", "alpha  βeta\ngamma", 0, "w", 7),
        ("native b", "one !!! two", 8, "b", 0),
        ("native e", "one !!! two", 0, "e", 3),
        ("native e skips punctuation", "one !!! two", 3, "e", 11),
        ("big start", "one,!!! two", 0, "W", 8),
        ("big previous", "one,!!! two", 8, "B", 0),
        ("big end", "one,!!! two", 0, "E", 7),
        ("previous end", "one !!! two", 9, "ge", 3),
        ("previous big end", "one,!!! two", 9, "gE", 7),
        ("counted starts", "one,two three", 0, "3w", 8),
        ("counted ends", "one,two three", 0, "2e", 7),
        ("combining end", "e\u{301}界! next", 0, "e", 3),
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
        assert!(editor.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Horizontal movement observes whole graphemes and whole CRLF separators;
// logical endpoints remain independent of wrapped fragments and desired cells.
#[test]
fn logical_targets() {
    for (name, source, point, input, expected) in [
        ("cluster right", "e\u{301}界\r\nx", 0, "2l", 3),
        ("cluster left", "e\u{301}界\r\nx", 3, "2h", 0),
        ("whole separator", "e\u{301}界\r\nx", 3, "l", 5),
        ("CRLF end", "  e\u{301}界\r\nx", 0, "$", 5),
        ("first text", " \t界\r\nx", 3, "^", 2),
        ("logical start", " \t界\r\nx", 3, "0", 0),
        ("counted end", "abc\r\ndef\r\nx", 1, "2$", 8),
        ("line destination", "one\ntwo\nthree", 1, "2G", 4),
        ("counted gg", "one\ntwo\nthree", 1, "3gg", 8),
        ("buffer end", "one\ntwo", 1, "G", 7),
        ("buffer start", "one\ntwo", 5, "gg", 0),
        ("empty", "", 0, "hjk lGgg", 0),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(
            editor.view().point.position,
            Scalar(expected),
            "{name}: {}",
            editor.echo()
        );
    }
    let mut editor = session(VIM, "abcdefghij klmnop\n\t界z\nabcdefghijklmnop", 9);
    editor.handle(Event::Resize(8, 8));
    keys(&mut editor, "j");
    assert_eq!(editor.view().cell(editor.buffer().unwrap()).unwrap().row, 1);
    keys(&mut editor, "j");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().column,
        9
    );
    keys(&mut editor, "0");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().column,
        0
    );
}

// u32 overflow and cancellation never execute a partial count; huge valid
// counts traverse the finite source, preserving the existing native history.
#[test]
fn bounded_counts() {
    for (input, expected) in [
        ("4294967295l", 5),
        ("4294967295w", 5),
        ("4294967295e", 5),
        ("4294967295j", 4),
        ("4294967295G", 4),
    ] {
        let mut editor = session(VIM, "a b\nc", 0);
        keys(&mut editor, input);
        assert_eq!(
            editor.view().point.position,
            Scalar(expected),
            "{input}: {}",
            editor.echo()
        );
        assert!(editor.buffer().unwrap().history().is_empty());
    }
    let mut editor = session(VIM, "a b\nc", 0);
    keys(&mut editor, "4294967296");
    assert_eq!(editor.view().point.position, Scalar(0));
    assert!(editor.echo().contains("u32"));
    keys(&mut editor, "2");
    editor.handle(key(KeyCode::Esc));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
    keys(&mut editor, "2");
    editor.execute(Command::Motion(Motion::BufferStart));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
    editor.execute(Command::Navigate(Navigation::new(Target::Word(
        Word::NextStart,
    ))));
    assert_eq!(editor.view().point.position, Scalar(2));
}
