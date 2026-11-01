use super::navigation::keys;
use super::*;

fn control(character: char) -> Event {
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}

// Modal search opens the same literal, smart-case, nonwrapping prompt. Accept
// records a shared continuation; C-g restores the complete native origin view.
#[test]
fn search_lifecycle() {
    let mut editor = session(VIM, "cat Cat catapult cat\ncat", 0);
    keys(&mut editor, "/cat");
    editor.handle(key(KeyCode::Enter));
    assert_eq!(editor.view().point.position, Scalar(3));
    keys(&mut editor, "2n");
    assert_eq!(editor.view().point.position, Scalar(11));
    keys(&mut editor, "N");
    assert_eq!(editor.view().point.position, Scalar(4));
    keys(&mut editor, "n");
    assert_eq!(editor.view().point.position, Scalar(11));
    let origin = editor.view().clone();
    keys(&mut editor, "?Cat");
    assert_eq!(editor.view().point.position, Scalar(4));
    editor.handle(control('g'));
    assert_eq!(editor.view(), &origin);
    keys(&mut editor, "/catapult");
    editor.handle(key(KeyCode::Enter));
    let after = editor.view().point;
    keys(&mut editor, "4294967295n");
    assert_eq!(editor.view().point, after);
    assert_eq!(text(&editor), "cat Cat catapult cat\ncat");
}

// Word-at-point uses whole Unicode word boundaries in the canonical matcher,
// skipping substrings and reusing its confirmed query for n/N after acceptance.
#[test]
fn word_search() {
    for (name, source, point, input, expected) in [
        ("skip substring", "cat catapult Cat cat", 0, "*", 16),
        ("smart case", "Cat cat Cat", 0, "*", 11),
        ("combining", "e\u{301} e\u{301}x e\u{301}", 0, "*", 9),
        ("backward word", "cat catapult cat", 13, "#", 0),
        ("count word", "cat cat cat", 0, "2*", 11),
        ("no wrap", "cat dog", 0, "*", 0),
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
    let mut editor = session(VIM, "cat catapult cat cat", 0);
    keys(&mut editor, "*n");
    assert_eq!(editor.view().point.position, Scalar(20));
    keys(&mut editor, "N");
    assert_eq!(editor.view().point.position, Scalar(13));
}

// Native command entry and modal continuation share the confirmed query owner;
// an edit invalidates its retained source offsets before another n can move.
#[test]
fn shared_search_owner() {
    let mut editor = session(VIM, "one one one", 0);
    editor.execute(Command::SearchForward);
    editor.handle(Event::Paste("one".into()));
    editor.handle(key(KeyCode::Enter));
    keys(&mut editor, "n");
    assert_eq!(editor.view().point.position, Scalar(7));
    keys(&mut editor, "iX");
    editor.handle(key(KeyCode::Esc));
    let point = editor.view().point;
    keys(&mut editor, "n");
    assert_eq!(editor.view().point, point);
    assert_eq!(text(&editor), "one oneX one");
}
