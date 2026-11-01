use super::navigation::{keys, settle};
use super::*;

fn control(character: char) -> Event {
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}

// Paging and visible-window targets use the same bounded fold/wrap geometry
// as rendering; alignment moves only the viewport and preserves native point.
#[test]
fn viewport_targets() {
    let source = (0..50).map(|n| format!("line {n}\n")).collect::<String>();
    let mut editor = session(VIM, &source, 0);
    editor.handle(Event::Resize(30, 10));
    settle(&mut editor);
    editor.handle(control('d'));
    settle(&mut editor);
    assert_eq!(editor.view().cell(editor.buffer().unwrap()).unwrap().row, 4);
    editor.handle(control('u'));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "21Gzt");
    assert_eq!(editor.view().top, 20);
    keys(&mut editor, "L");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        27
    );
    keys(&mut editor, "H");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        20
    );
    keys(&mut editor, "M");
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        23
    );
    let point = editor.view().point;
    keys(&mut editor, "zz");
    assert_eq!(editor.view().point, point);
    assert_eq!(editor.view().top, 19);
    keys(&mut editor, "zb");
    assert_eq!(editor.view().point, point);
    assert_eq!(editor.view().top, 16);
    keys(&mut editor, "4294967295");
    editor.handle(control('f'));
    settle(&mut editor);
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        50
    );
}

// Vertical Normal movement skips hidden source bodies through the native fold
// projection. It never creates a parallel list of visible or wrapped lines.
#[test]
fn folded_lines() {
    let mut editor = session(VIM, "* First\nbody\n** Child\nhidden\n* Next\ntail\n", 0);
    editor.execute(Command::Mode(rev::buffer::Mode::Org));
    editor.execute(Command::Org(rev::command::OrgCommand::Overview));
    settle(&mut editor);
    keys(&mut editor, "j");
    assert_eq!(editor.view().cell(editor.buffer().unwrap()).unwrap().row, 4);
    editor.handle(key(KeyCode::Up));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Maximum counts over many wrapped rows finish through the native layout
// deadline queue rather than count-sized replay or a separate viewport model.
#[test]
fn large_page_count() {
    let source = "abcdefgh ijklmnop\n".repeat(4000);
    let mut editor = session(VIM, &source, 0);
    editor.handle(Event::Resize(12, 10));
    keys(&mut editor, "4294967295");
    editor.handle(control('f'));
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.handle(key(KeyCode::Char('k')));
    settle(&mut editor);
    assert_eq!(
        editor.view().cell(editor.buffer().unwrap()).unwrap().row,
        3999
    );
    editor.handle(key(KeyCode::PageUp));
    settle(&mut editor);
    assert!(editor.view().cell(editor.buffer().unwrap()).unwrap().row < 4000);
}

// C-g interrupts a deferred source-sized page before publication. Subsequent
// Normal input remains ordered and cannot revive the discarded destination.
#[test]
fn cancel_large_page() {
    let source = "abcdefgh ijklmnop\n".repeat(4000);
    let mut editor = session(VIM, &source, 0);
    editor.handle(Event::Resize(12, 10));
    keys(&mut editor, "4294967295");
    editor.handle(control('f'));
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.handle(control('g'));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "l");
    assert_eq!(editor.view().point.position, Scalar(1));
    assert!(editor.buffer().unwrap().history().is_empty());
}
