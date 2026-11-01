use super::navigation::{keys, settle};
use super::visual::{block, region};
use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use rev::command::{Motion, Split, WindowCommand};

// A native block payload retains logical columns for modal and native paste,
// including whole source tabs. Counted paste remains a single undoable batch.
#[test]
fn block_transfer() {
    for (paste, expected) in [
        ("P", "界a界b\n\tc\td"),
        ("2P", "界界a界b\n\t\tc\td"),
        ("p", "a界界b\nc\t\td"),
    ] {
        let mut editor = session(VIM, "a界b\nc\td", 1);
        block(&mut editor);
        keys(&mut editor, "jygg");
        assert_eq!(editor.rectangle().unwrap().rows(), ["界", "\t"]);
        assert_eq!(editor.killed_text().unwrap().to_string(), "界\n\t");
        keys(&mut editor, paste);
        assert_eq!(text(&editor), expected, "{paste}: {}", editor.echo());
        assert_eq!(editor.buffer().unwrap().history().len(), 1);
        keys(&mut editor, "u");
        assert_eq!(text(&editor), "a界b\nc\td");
    }
    let mut editor = session(VIM, "a界b\nc\td", 1);
    block(&mut editor);
    keys(&mut editor, "jygg");
    editor.execute(Command::Rectangle(rev::command::RectangleCommand::Yank));
    assert_eq!(text(&editor), "界a界b\n\tc\td");
}

// Reversing source rows/columns and exchanging active ends must describe the
// same logical block. Display widths never turn a tab into multiple columns.
#[test]
fn reverse_block() {
    for exchange in [false, true] {
        let mut editor = session(VIM, "ab界\nc\td", 6);
        block(&mut editor);
        keys(&mut editor, "ggl");
        if exchange {
            keys(&mut editor, "o");
        }
        keys(&mut editor, "y");
        assert_eq!(editor.rectangle().unwrap().rows(), ["b界", "\td"]);
        assert_eq!(editor.rectangle().unwrap().width(), 2);
    }
}

// Read-only failure leaves every row, history, endpoints and prior transfer
// unchanged. A valid copy remains available for all three selection shapes.
#[test]
fn readonly_visual() {
    for shape in ["v", "V", "block"] {
        for operation in ["d", "c"] {
            let mut editor = source_session(VIM, "one two\nthree four", 0, true);
            keys(&mut editor, "yw");
            let retained = editor.killed_text().unwrap().to_string();
            if shape == "block" {
                block(&mut editor);
            } else {
                keys(&mut editor, shape);
            }
            keys(&mut editor, "j");
            let state = editor.editing_state();
            let point = editor.view().point;
            let mark = editor.buffer().unwrap().mark();
            keys(&mut editor, operation);
            assert_eq!(text(&editor), "one two\nthree four");
            assert!(editor.buffer().unwrap().history().is_empty());
            assert_eq!(editor.editing_state(), state);
            assert_eq!(editor.view().point, point);
            assert_eq!(editor.buffer().unwrap().mark(), mark);
            assert_eq!(editor.killed_text().unwrap().to_string(), retained);
            assert!(editor.rectangle().is_none());
            keys(&mut editor, "y");
            assert_eq!(editor.editing_state(), Some(EditingState::Normal));
            assert!(editor.buffer().unwrap().history().is_empty());
        }
    }
}

fn positions(editor: &Session) -> Vec<(rev::window::ViewId, Scalar)> {
    editor
        .windows()
        .iter()
        .map(|(id, view)| (id, view.point.position))
        .collect()
}

// Shared-buffer views retain one selection while their own points stay native.
// One compound block edit/undo/redo reconciles and restores every live endpoint.
#[test]
fn shared_visual_history() {
    let mut editor = session(VIM, "abcd\nefgh\nijkl", 1);
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Window(WindowCommand::Other));
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Window(WindowCommand::Other));
    block(&mut editor);
    keys(&mut editor, "jl");
    let before = positions(&editor);
    let initiator = editor.windows().selected_id();
    editor.execute(Command::Window(WindowCommand::Other));
    assert_eq!(editor.editing_state(), Some(EditingState::VisualBlock));
    editor.select_view(initiator).unwrap();
    keys(&mut editor, "d");
    assert_eq!(text(&editor), "ad\neh\nijkl");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "abcd\nefgh\nijkl");
    assert_eq!(positions(&editor), before);
    editor.execute(Command::History(History::Redo));
    assert_eq!(text(&editor), "ad\neh\nijkl");
    assert!(
        editor
            .windows()
            .iter()
            .all(|(_, view)| view.cell(editor.buffer().unwrap()).is_ok())
    );
}

// A source edit or buffer change retires Visual authority before any following
// key can apply a range to another revision or document.
#[test]
fn visual_retirement() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "v2l");
    editor.replace(0..1, "X").unwrap();
    keys(&mut editor, "d");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert_eq!(text(&editor), "Xne two");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.handle(key(KeyCode::Esc));
    let fixture = Fixture::new();
    let target = fixture.write("other.txt", "target");
    editor.execute(Command::Split(Split::Right));
    let peer = editor.windows().order()[1];
    keys(&mut editor, "v");
    editor.load(&target).unwrap();
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    assert!(!editor.buffer().unwrap().mark_active());
    keys(&mut editor, "y");
    assert_eq!(text(&editor), "target");
    editor.handle(key(KeyCode::Esc));
    editor.select_view(peer).unwrap();
    assert!(!editor.buffer().unwrap().mark_active());
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
}

fn mouse(kind: MouseEventKind, column: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row: 0,
        modifiers: KeyModifiers::NONE,
    })
}

// Pointer drag replaces the native anchor, retains the active shape and uses
// inclusive whole-grapheme presentation through the ordinary surface mapping.
#[test]
fn visual_pointer() {
    let mut editor = session(&format!("{VIM}\n[mouse]\nenabled = true"), "alpha", 0);
    keys(&mut editor, "v");
    let frame = editor.frame().unwrap();
    let row = frame.rows()[0].text();
    let offset = row.find("alpha").unwrap();
    let left = unicode_width::UnicodeWidthStr::width(&row[..offset]) as u16;
    editor.handle(mouse(MouseEventKind::Down(MouseButton::Left), left + 1));
    editor.handle(mouse(MouseEventKind::Drag(MouseButton::Left), left + 3));
    editor.handle(mouse(MouseEventKind::Up(MouseButton::Left), left + 3));
    settle(&mut editor);
    assert_eq!(region(&editor), "lph");
    assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
    keys(&mut editor, "d");
    assert_eq!(text(&editor), "aa");
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "alpha");
}

// Folding/wrapping/resize alter projection only. Native logical-line selection
// still includes hidden source rows and publishes its complete range once.
#[test]
fn visual_projection() {
    let source = "* First\nbody body body\n* Last\nend";
    let mut editor = session(VIM, source, 0);
    editor.execute(Command::Mode(rev::command::Mode::Org));
    editor.execute(Command::Org(rev::command::OrgCommand::Overview));
    settle(&mut editor);
    assert!(
        !editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("body body"))
    );
    editor.execute(Command::VisualLineMode);
    keys(&mut editor, "Vj");
    editor.handle(Event::Resize(18, 9));
    settle(&mut editor);
    assert_eq!(region(&editor), "* First\nbody body body\n* Last\n");
    keys(&mut editor, "d");
    assert_eq!(text(&editor), "end");
    keys(&mut editor, "u");
    assert_eq!(text(&editor), source);
}

// Linear indentation and native paragraph objects remain one ordinary edit;
// unsupported objects and huge block expansion preserve selection/transfer.
#[test]
fn visual_limits() {
    let mut editor = session(VIM, "one\ntwo", 0);
    keys(&mut editor, "Vj>");
    assert_eq!(text(&editor), "    one\n    two");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "one\ntwo");
    keys(&mut editor, "vip");
    assert_eq!(region(&editor), "one\ntwo");
    let point = editor.view().point;
    let mark = editor.buffer().unwrap().mark();
    keys(&mut editor, "i(");
    assert_eq!(editor.view().point, point);
    assert_eq!(editor.buffer().unwrap().mark(), mark);
    editor.handle(key(KeyCode::Esc));
    block(&mut editor);
    keys(&mut editor, "jy");
    let retained = editor.killed_text().unwrap().to_string();
    keys(&mut editor, "4294967295p");
    assert_eq!(text(&editor), "one\ntwo");
    assert_eq!(editor.killed_text().unwrap().to_string(), retained);
    assert!(editor.buffer().unwrap().history().is_empty());
}

// The exhaustive sample activates the same Visual factories and protected
// recovery. Component input remains literal and cannot consume a Visual edit.
#[test]
fn sample_visual() {
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        include_str!("../../docs/reference/config.toml"),
    );
    let config = format!("include = ['{}']\n{VIM}", base.display());
    let mut editor = session(&config, "one two", 0);
    keys(&mut editor, "v2ld");
    assert_eq!(text(&editor), " two");
    keys(&mut editor, "u");
    block(&mut editor);
    keys(&mut editor, "y");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "v2l");
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    keys(&mut editor, "d");
    editor.execute(Command::Edit(rev::command::Editing::new(
        rev::command::editing::Action::Delete,
    )));
    assert_eq!(text(&editor), "one two");
    assert!(editor.buffer().unwrap().history().is_empty());
    assert_eq!(editor.editing_state(), Some(EditingState::VisualCharacter));
}

// Block change removes all rows once, enters Insert only after publication and
// does not split CRLF. Empty valid changes enter Insert without invented edits.
#[test]
fn block_change() {
    let mut editor = session(VIM, "ab\r\ncd", 0);
    block(&mut editor);
    keys(&mut editor, "jc");
    assert_eq!(text(&editor), "b\r\nd");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(&editor), "ab\r\ncd");
    for shape in ["v", "V", "block"] {
        let mut editor = session(VIM, "", 0);
        if shape == "block" {
            block(&mut editor);
        } else {
            keys(&mut editor, shape);
        }
        keys(&mut editor, "c");
        assert_eq!(editor.editing_state(), Some(EditingState::Insert));
        assert!(editor.buffer().unwrap().history().is_empty());
    }
}
