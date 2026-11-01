use super::navigation::{keys, settle};
use super::*;
use rev::command::{BufferCommand, Motion, Split, WindowCommand};

pub(super) fn control(editor: &mut Session, character: char) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    )));
    settle(editor);
}

pub(super) fn backward(editor: &mut Session) {
    control(editor, 'o');
}
pub(super) fn forward(editor: &mut Session) {
    control(editor, 'i');
}

// Nonlocal navigation records origins/destinations; ordinary cursor steps do
// not flood history. Count traversal clamps once and a new jump prunes forward.
#[test]
fn jump_traversal() {
    let source = "one\ntwo\nthree\nfour";
    let mut editor = session(VIM, source, 0);
    keys(&mut editor, "3G");
    assert_eq!(editor.view().point.position, Scalar(8));
    keys(&mut editor, "ll");
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(10));
    keys(&mut editor, "G");
    keys(&mut editor, "4294967295");
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "2G");
    let point = editor.view().point;
    forward(&mut editor);
    assert_eq!(editor.view().point, point);
    assert_eq!(text(&editor), source);
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Jump locations follow insertions and native history, become unavailable when
// removed and return only when undo restores their original native boundary.
#[test]
fn jump_edits() {
    let mut editor = session(VIM, "one\ntwo\nthree", 4);
    keys(&mut editor, "G");
    editor.replace(0..0, "XX").unwrap();
    editor.execute(Command::Motion(Motion::BufferEnd));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(6));
    editor.execute(Command::History(History::Undo));
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(13));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(4));
    forward(&mut editor);
    editor.replace(4..7, "").unwrap();
    let before = editor.view().point;
    backward(&mut editor);
    assert_eq!(editor.view().point, before);
    assert!(editor.echo().contains("removed"), "{}", editor.echo());
    editor.execute(Command::History(History::Undo));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(4));
}

// Only accepted search destinations join jump history. Preview and C-g retain
// the original index and the canonical confirmed query remains independent.
#[test]
fn search_jumps() {
    let mut editor = session(VIM, "one two one two", 0);
    keys(&mut editor, "/two");
    editor.handle(key(KeyCode::Enter));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(7));
    keys(&mut editor, "n");
    assert_eq!(editor.view().point.position, Scalar(15));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(7));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "/one");
    control(&mut editor, 'g');
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(7));
}

// File traversal reuses the canonical dirty buffer and can reopen an unchanged
// closed file. It never creates another text owner or modifies disk/history.
#[test]
fn file_jumps() {
    let fixture = Fixture::new();
    let first = fixture.write("first.txt", "one\ntwo");
    let second = fixture.write("second.txt", "other");
    let mut editor = session(VIM, "", 0);
    editor.load(&first).unwrap();
    let id = editor.buffer().unwrap().id();
    keys(&mut editor, "2G");
    keys(&mut editor, "G");
    editor.replace(0..0, "X").unwrap();
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(5));
    editor.load(&second).unwrap();
    backward(&mut editor);
    assert_eq!(editor.buffer().unwrap().id(), id);
    assert_eq!(editor.view().point.position, Scalar(5));
    assert_eq!(text(&editor), "Xone\ntwo");
    editor.execute(Command::Buffer(BufferCommand::Kill));
    editor.handle(Event::Paste("second.txt".into()));
    editor.handle(key(KeyCode::Enter));
    settle(&mut editor);
    forward(&mut editor);
    assert_eq!(text(&editor), "other");
    assert_eq!(std::fs::read_to_string(&first).unwrap(), "one\ntwo");
}

// A failed closed-file visit leaves both point and history index unchanged;
// restoring the exact file identity permits the same traversal to be retried.
#[test]
fn unavailable_jump() {
    let fixture = Fixture::new();
    let first = fixture.write("first.txt", "source");
    let second = fixture.write("second.txt", "target");
    let parked = fixture.0.join("parked.txt");
    let mut editor = session(VIM, "", 0);
    editor.load(&first).unwrap();
    editor.load(&second).unwrap();
    editor.execute(Command::Buffer(BufferCommand::Kill));
    editor.handle(Event::Paste("first.txt".into()));
    editor.handle(key(KeyCode::Enter));
    std::fs::rename(&first, &parked).unwrap();
    let point = editor.view().point;
    backward(&mut editor);
    assert_eq!(editor.view().point, point);
    fixture.write("first.txt", "source");
    backward(&mut editor);
    assert_eq!(editor.view().point, point);
    std::fs::remove_file(&first).unwrap();
    std::fs::rename(&parked, &first).unwrap();
    backward(&mut editor);
    assert_eq!(editor.buffer().unwrap().file().unwrap().path(), first);
}

// Splits inherit the current bounded jump state, then move their traversal
// indices independently. Deleting one view does not alter the survivor.
#[test]
fn split_jumps() {
    let mut editor = session(VIM, "one\ntwo\nthree", 0);
    keys(&mut editor, "G");
    let original = editor.windows().selected_id();
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.select_view(original).unwrap();
    assert_eq!(editor.view().point.position, Scalar(13));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(0));
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(13));
    editor.execute(Command::Window(WindowCommand::Delete));
    assert_eq!(editor.view().point.position, Scalar(0));
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(13));
}

// Insert C-o keeps counts/operators/character targets pending until one native
// semantic command ends, then resumes insertion at the admitted native point.
#[test]
fn one_normal() {
    for (input, expected, point) in [
        ("2w", "one two three", 8),
        ("dw", "two three", 0),
        ("f界", "one 界 two", 4),
    ] {
        let source = if input == "f界" {
            "one 界 two"
        } else {
            "one two three"
        };
        let mut editor = session(VIM, source, 0);
        keys(&mut editor, "i");
        control(&mut editor, 'o');
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        keys(&mut editor, &input[..1]);
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        keys(&mut editor, &input[1..]);
        assert_eq!(text(&editor), expected, "{input}");
        assert_eq!(editor.view().point.position, Scalar(point));
        assert_eq!(editor.editing_state(), Some(EditingState::Insert));
        keys(&mut editor, "X");
        assert_eq!(editor.view().point.position, Scalar(point + 1));
    }
}

// A native command/argument prompt is one continuation lifetime. Tab stays with
// that prompt, while view/source changes retire incompatible insertion authority.
#[test]
fn one_normal_prompt() {
    let mut editor = session(VIM, "one\ntwo", 0);
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    keys(&mut editor, "goto-line");
    editor.handle(key(KeyCode::Enter));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    editor.handle(Event::Paste("2".into()));
    editor.handle(key(KeyCode::Tab));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    editor.handle(key(KeyCode::Enter));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(4));
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    control(&mut editor, 'o');
    editor.replace(0..0, "X").unwrap();
    keys(&mut editor, "l");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(editor.view().cell(editor.buffer().unwrap()).is_ok());
    editor.execute(Command::Motion(Motion::BufferStart));
}
