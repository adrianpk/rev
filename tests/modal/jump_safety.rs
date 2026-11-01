use super::jumps::{backward, control, forward};
use super::navigation::{keys, settle};
use super::*;
use rev::command::{BufferCommand, Motion, TreeCommand};

fn answer(editor: &mut Session, text: &str) {
    editor.handle(Event::Paste(text.into()));
    editor.handle(key(KeyCode::Enter));
    settle(editor);
}

// Canonical directory rename updates even closed jump paths. Reopening checks
// the same native file identity/context without creating a named bookmark.
#[test]
fn renamed_jumps() {
    let fixture = Fixture::new();
    let directory = fixture.0.join("owned");
    let first = fixture.write("owned/first.txt", "source");
    let second = fixture.write("owned/second.txt", "target");
    let mut editor = session(VIM, "", 0);
    editor.load(&first).unwrap();
    editor.load(&second).unwrap();
    editor.execute(Command::Buffer(BufferCommand::Kill));
    answer(&mut editor, "first.txt");
    editor.execute(Command::Tree(TreeCommand::Focus));
    assert_eq!(editor.tree().selected().unwrap().path(), directory);
    editor.execute(Command::Tree(TreeCommand::Rename));
    answer(&mut editor, "renamed");
    editor.execute(Command::KeyboardQuit);
    backward(&mut editor);
    assert_eq!(text(&editor), "source", "{}", editor.echo());
    assert_eq!(
        editor.buffer().unwrap().file().unwrap().path(),
        fixture.0.join("renamed/first.txt")
    );
    assert!(editor.bookmarks().is_empty());
    assert!(!editor.bookmarks().is_dirty());
}

// Read-only file visits and dirty live source reuse preserve canonical owners;
// recording or traversing a jump cannot acquire any source history or disk write.
#[test]
fn readonly_jump() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", VIM);
    let startup = prepare(&root);
    let first = fixture.write("first.txt", "one\ntwo");
    let second = fixture.write("second.txt", "target");
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&first).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    keys(&mut editor, "G");
    editor.load(&second).unwrap();
    backward(&mut editor);
    assert_eq!(editor.buffer().unwrap().id(), id);
    assert!(editor.buffer().unwrap().is_readonly());
    assert_eq!(editor.view().point.position, Scalar(7));
    assert!(editor.buffer().unwrap().history().is_empty());
    assert_eq!(std::fs::read_to_string(first).unwrap(), "one\ntwo");
}

// A grapheme join invalidates the original source boundary rather than moving
// it to a nearby cursor. Native undo restores the actual recorded location.
#[test]
fn joined_anchor() {
    let mut editor = session(VIM, "aX", 1);
    keys(&mut editor, "G");
    editor.replace(1..1, "\u{301}").unwrap();
    let point = editor.view().point;
    backward(&mut editor);
    assert_eq!(editor.view().point, point);
    assert!(editor.echo().contains("removed"), "{}", editor.echo());
    editor.execute(Command::History(History::Undo));
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(1));
}

// Reopening the already-selected file cannot invent a destination or prune the
// forward branch. A failed file ingress also leaves traversal state unchanged.
#[test]
fn admitted_jumps() {
    let fixture = Fixture::new();
    let path = fixture.write("source.txt", "one\ntwo\nthree");
    let mut editor = session(VIM, "", 0);
    editor.load(&path).unwrap();
    keys(&mut editor, "2G");
    keys(&mut editor, "G");
    backward(&mut editor);
    let point = editor.view().point;
    editor.load(&path).unwrap();
    assert_eq!(editor.view().point, point);
    let invalid = fixture.write("invalid.bin", "bad\0source");
    assert!(editor.load(&invalid).is_err());
    forward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(13));
}

// A deferred page command completes before later FIFO input is interpreted.
// Resuming Insert must not wait for that later queue to drain in Normal state.
#[test]
fn deferred_once() {
    let source = "abcdefgh ijklmnop\n".repeat(4000);
    let mut editor = session(VIM, &source, 0);
    editor.handle(Event::Resize(12, 10));
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    keys(&mut editor, "4294967295");
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('f'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.view().point.position, Scalar(0));
    editor.handle(key(KeyCode::Char('X')));
    settle(&mut editor);
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    assert_eq!(text(&editor), format!("{source}X"));
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
}

// Complete sample maps preserve terminal C-i/Tab aliases, while prompt input
// and incompatible buffer/view changes retain their native input ownership.
#[test]
fn sample_jumps() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        include_str!("../../docs/reference/config.toml"),
    );
    let config = format!("include = ['{}']\n{VIM}", base.display());
    let mut editor = session(&config, "one\ntwo", 0);
    keys(&mut editor, "G");
    backward(&mut editor);
    editor.handle(key(KeyCode::Tab));
    settle(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(7));
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    keys(&mut editor, "h");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    control(&mut editor, 'o');
    keys(&mut editor, "r");
    let target = fixture.write("other.txt", "other");
    editor.load(&target).unwrap();
    keys(&mut editor, "l");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "other");
    editor.execute(Command::Motion(Motion::BufferStart));
}

// A new admitted jump must retain its valid origin even when an older removed
// location has the same scalar metadata. Stale evidence cannot deduplicate it.
#[test]
fn fresh_jump_origin() {
    let mut editor = session(VIM, "one\ntwo\nthree", 0);
    keys(&mut editor, "2G");
    editor.replace(4..7, "").unwrap();
    keys(&mut editor, "G");
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(4));
    assert_eq!(text(&editor), "one\n\nthree");
}

// Reopening one closed location qualifies every matching retained observation
// in that canonical source, so later edits rebase other destinations as well.
#[test]
fn reopened_anchors() {
    let fixture = Fixture::new();
    let first = fixture.write("first.txt", "one\ntwo\nthree");
    let second = fixture.write("second.txt", "target");
    let mut editor = session(VIM, "", 0);
    editor.load(&first).unwrap();
    keys(&mut editor, "2GG");
    editor.load(&second).unwrap();
    editor.execute(Command::Buffer(BufferCommand::Kill));
    answer(&mut editor, "first.txt");
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(13));
    editor.replace(0..0, "XX").unwrap();
    backward(&mut editor);
    assert_eq!(editor.view().point.position, Scalar(6));
    assert_eq!(text(&editor), "XXone\ntwo\nthree");
}

// Servicing native deadlines cannot complete a continuation before a command
// starts, or restore insertion after an unrelated source revision changes.
#[test]
fn continuation_deadline() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "i");
    control(&mut editor, 'o');
    editor.expire_input(std::time::Instant::now());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "w");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    control(&mut editor, 'o');
    editor.replace(0..0, "X").unwrap();
    editor.expire_input(std::time::Instant::now());
    keys(&mut editor, "l");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "Xone two");
}
