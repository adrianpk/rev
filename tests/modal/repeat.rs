use super::navigation::{keys, settle};
use super::*;
use rev::command::{Motion, Split, WindowCommand};

fn place(editor: &mut Session, position: usize) {
    editor.execute(Command::Motion(Motion::BufferStart));
    if position > 0 {
        let mut request = rev::command::Navigation::new(
            rev::command::navigation::Target::Character(rev::search::Direction::Forward),
        );
        request.count = Some(position as u32);
        editor.execute(Command::Navigate(request));
    }
    settle(editor);
    assert_eq!(editor.view().point.position, Scalar(position));
}

fn escape(editor: &mut Session) {
    editor.handle(key(KeyCode::Esc));
    settle(editor);
}

// Replace consumes complete source graphemes and inserts at logical EOL. One
// connected entry session is one native undo/redo group, including CRLF and joins.
#[test]
fn replace_session() {
    for (name, source, point, input, expected) in [
        ("clusters", "e\u{301}界!", 0, "Rxy", "xy!"),
        ("EOL", "ab\r\nnext", 1, "Rxyz", "axyz\r\nnext"),
        ("empty", "", 0, "R界e\u{301}", "界e\u{301}"),
        ("joined input", "abc", 0, "Re\u{301}x", "e\u{301}xc"),
    ] {
        let mut editor = session(VIM, source, point);
        keys(&mut editor, input);
        assert_eq!(
            editor.editing_state(),
            Some(EditingState::Replace),
            "{name}"
        );
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        escape(&mut editor);
        assert_eq!(editor.buffer().unwrap().history().len(), 1, "{name}");
        keys(&mut editor, "u");
        assert_eq!(text(&editor), source, "{name}");
        editor.execute(Command::History(History::Redo));
        assert_eq!(text(&editor), expected, "{name}");
    }
}

// Replace retains native modified controls, component-owned prompt text and
// read-only admission; a different buffer cannot inherit text-entry state.
#[test]
fn replace_boundaries() {
    let mut editor = session(VIM, "abcd", 0);
    keys(&mut editor, "Rx");
    editor.execute(Command::Motion(Motion::ForwardChar));
    keys(&mut editor, "y");
    assert_eq!(text(&editor), "xbyd");
    editor.execute(Command::ExecuteExtended);
    keys(&mut editor, "goto-line");
    editor.handle(key(KeyCode::Enter));
    keys(&mut editor, "1");
    editor.handle(key(KeyCode::Enter));
    assert_eq!(text(&editor), "xbyd");
    let fixture = Fixture::new();
    let file = fixture.write("other.txt", "other");
    editor.load(&file).unwrap();
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    let mut readonly = source_session(VIM, "abcd", 0, true);
    keys(&mut readonly, "Rno");
    readonly.handle(Event::Paste("refused".into()));
    assert_eq!(text(&readonly), "abcd");
    assert!(readonly.buffer().unwrap().history().is_empty());
}

// Dot resolves the original entry form at the current point, publishes one
// transaction, stays Normal and permits one-step undo of only that repetition.
#[test]
fn repeat_entries() {
    for (name, source, entry, payload, target, expected) in [
        ("insert", "abcd", "i", "XY", 4, "XYabXYcd"),
        ("after", "abcd", "a", "Z", 3, "aZbcZd"),
        ("line first", "  one\n  two", "I", "X", 9, "  Xone\n  Xtwo"),
        ("line end", "one\ntwo", "A", "X", 5, "oneX\ntwoX"),
        ("below", "one\ntwo", "o", "X", 6, "one\nX\ntwo\nX"),
        ("above", "one\ntwo", "O", "X", 7, "X\none\nX\ntwo"),
        ("replace", "abcd", "R", "XY", 2, "XYXY"),
    ] {
        let mut editor = session(VIM, source, 0);
        keys(&mut editor, entry);
        editor.handle(Event::Paste(payload.into()));
        escape(&mut editor);
        let before = text(&editor);
        place(&mut editor, target);
        let history = editor.buffer().unwrap().history().len();
        keys(&mut editor, ".");
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            history + 1,
            "{name}"
        );
        keys(&mut editor, "u");
        assert_eq!(text(&editor), before, "{name}");
    }
}

// Stored operator and character requests re-resolve whole native targets.
// An explicit dot count replaces the multiplied original count, never multiplies it.
#[test]
fn repeat_operators() {
    for (name, source, input, repeated, expected) in [
        ("delete", "one two three", "dw", ".", "three"),
        ("count reuse", "abcdefghi", "2x", ".", "efghi"),
        ("count override", "abcdefghij", "2x", "3.", "fghij"),
        ("motion override", "a b c d e f g h", "2d2w", "2.", "g h"),
        ("replace char", "abcdef", "2rX", "ll3.", "XXXXXf"),
        ("indent", "a\nb\nc", ">>", "j.", "    a\n    b\nc"),
    ] {
        let mut editor = session(VIM, source, 0);
        keys(&mut editor, input);
        let before = text(&editor);
        keys(&mut editor, repeated);
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        keys(&mut editor, "u");
        assert_eq!(text(&editor), before, "{name}");
    }
}

// Change plus inserted text is one native group, including a preserved line
// separator. Dot replaces the next semantic target atomically without entering Insert.
#[test]
fn repeat_changes() {
    for (name, source, command, payload, motion, expected) in [
        ("word", "one two three", "cw", "X ", "", "X X three"),
        (
            "line",
            "one\r\ntwo\r\nthree",
            "cc",
            "X",
            "j",
            "X\r\nX\r\nthree",
        ),
        ("object", "one two three", "ciw", "X", "w", "X X three"),
    ] {
        let mut editor = session(VIM, source, 0);
        keys(&mut editor, command);
        editor.handle(Event::Paste(payload.into()));
        escape(&mut editor);
        assert_eq!(editor.buffer().unwrap().history().len(), 1, "{name}");
        let before = text(&editor);
        keys(&mut editor, motion);
        keys(&mut editor, ".");
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        keys(&mut editor, "u");
        assert_eq!(text(&editor), before, "{name}");
        keys(&mut editor, "u");
        assert_eq!(text(&editor), source, "{name}");
    }
}

// A retained paste carries its admitted linear/linewise/block shape and text;
// a later copy into the shared kill ring cannot silently change dot's payload.
#[test]
fn repeat_transfer() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "ywP");
    keys(&mut editor, "Gyiw");
    place(&mut editor, 0);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "one one one two");
    let mut lines = session(VIM, "one\ntwo\nthree", 0);
    keys(&mut lines, "yyp");
    lines.execute(Command::Motion(Motion::BufferEnd));
    keys(&mut lines, ".");
    assert_eq!(text(&lines), "one\none\ntwo\nthree\none\n");
}

// Completed repeat data is session-local and portable to another pane, while
// native undo restores every shared point through the canonical initiator policy.
#[test]
fn repeat_panes() {
    let mut editor = session(VIM, "abcdef", 0);
    editor.execute(Command::Split(Split::Right));
    keys(&mut editor, "RXY");
    escape(&mut editor);
    editor.execute(Command::Window(WindowCommand::Other));
    place(&mut editor, 3);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "XYcXYf");
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "XYcdef");
    keys(&mut editor, "u");
    assert_eq!(text(&editor), "abcdef");
}

// Oversized sessions still publish all admitted input and undo coherently, but
// dot cannot replay a truncated prefix or a prior unrelated retained operation.
#[test]
fn repeat_limit() {
    for size in [1024, 1025] {
        let mut editor = session(VIM, "abcdef", 0);
        keys(&mut editor, "xR");
        let payload = "z".repeat(size);
        editor.handle(Event::Paste(payload.clone()));
        escape(&mut editor);
        assert_eq!(text(&editor), payload);
        place(&mut editor, size);
        keys(&mut editor, ".");
        assert_eq!(
            text(&editor).len(),
            if size == 1024 { size * 2 } else { size }
        );
    }
}

// Failed read-only/missing target repeats preserve the record and do not add
// history. Empty insertion entry and cancelled grammar retain the prior change.
#[test]
fn repeat_failures() {
    let mut editor = session(VIM, "abcd", 0);
    keys(&mut editor, "x");
    keys(&mut editor, "i");
    escape(&mut editor);
    keys(&mut editor, "d");
    escape(&mut editor);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "cd");
    place(&mut editor, 2);
    let before = editor.buffer().unwrap().history().len();
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "cd");
    assert_eq!(editor.buffer().unwrap().history().len(), before);
    place(&mut editor, 0);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "d");
}

// Repeated Visual edits retain linear, linewise and logical-block geometry,
// resolve fresh ranges and publish all affected rows in one transaction.
#[test]
fn repeat_visual() {
    for (name, source, input, motion, expected) in [
        ("character", "abcdef", "vld", "ll", "cd"),
        ("line", "one\ntwo\nthree\nfour", "Vd", "j", "two\nfour"),
    ] {
        let mut editor = session(VIM, source, 0);
        keys(&mut editor, input);
        let before = text(&editor);
        keys(&mut editor, motion);
        keys(&mut editor, ".");
        assert_eq!(text(&editor), expected, "{name}: {}", editor.echo());
        keys(&mut editor, "u");
        assert_eq!(text(&editor), before, "{name}");
    }
    for (action, expected) in [('d', "cd\ngh\nkl\nop"), ('c', "Xcd\ngh\nXkl\nop")] {
        let mut editor = session(VIM, "abcd\nefgh\nijkl\nmnop", 0);
        editor.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('v'),
            KeyModifiers::CONTROL,
        )));
        keys(&mut editor, &format!("jl{action}"));
        if action == 'c' {
            keys(&mut editor, "X");
            escape(&mut editor);
            assert_eq!(editor.buffer().unwrap().history().len(), 1);
        }
        let before = text(&editor);
        keys(&mut editor, "jj0.");
        assert_eq!(text(&editor), expected, "{action}: {}", editor.echo());
        keys(&mut editor, "u");
        assert_eq!(text(&editor), before);
    }
}

// Text may be inserted into an empty top block row before any removed row.
// That insertion joins the native change and its undo restores every original row.
#[test]
fn repeat_short_block() {
    let source = "\nabcd\n\nefgh";
    let mut editor = session(VIM, source, 0);
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    keys(&mut editor, "jlcX");
    escape(&mut editor);
    assert_eq!(text(&editor), "X\ncd\n\nefgh");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "jj0.");
    assert_eq!(text(&editor), "X\ncd\nX\ngh");
    keys(&mut editor, "uu");
    assert_eq!(text(&editor), source);
    editor.execute(Command::History(History::Redo));
    editor.execute(Command::History(History::Redo));
    assert_eq!(text(&editor), "X\ncd\nX\ngh");
}

// An external revision retires an unfinished entry; native controls still
// execute, but unsupported destructive entry edits cannot leave a partial replay.
#[test]
fn repeat_retirement() {
    let mut editor = session(VIM, "abcd", 0);
    keys(&mut editor, "iXY");
    editor.replace(0..0, "external").unwrap();
    escape(&mut editor);
    let before = text(&editor);
    keys(&mut editor, ".");
    assert_eq!(text(&editor), before);
    let mut replaced = session(VIM, "abcd", 0);
    keys(&mut replaced, "RXY");
    replaced.execute(Command::BackwardDeleteChar);
    escape(&mut replaced);
    let before = text(&replaced);
    keys(&mut replaced, ".");
    assert_eq!(text(&replaced), before);
}

// A read-only canonical file rejects dot without losing the portable completed
// record. Returning to an editable file admits the same semantic change.
#[test]
fn repeat_readonly() {
    let fixture = Fixture::new();
    let readonly = fixture.write("readonly.txt", "readonly");
    let editable = fixture.write("editable.txt", "editable");
    let root = fixture.write("config.toml", VIM);
    let startup = prepare(&root);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&readonly).unwrap();
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
    editor.load(&editable).unwrap();
    keys(&mut editor, "2x");
    editor.load(&readonly).unwrap();
    let point = editor.view().point;
    keys(&mut editor, ".");
    assert_eq!(editor.view().point, point);
    assert_eq!(text(&editor), "readonly");
    assert!(editor.buffer().unwrap().history().is_empty());
    editor.load(&editable).unwrap();
    keys(&mut editor, ".");
    assert_eq!(text(&editor), "able");
}

// Insert deletion within newly inserted text updates the semantic payload;
// oversized count expansion fails before mutation and leaves a later retry valid.
#[test]
fn repeat_text_controls() {
    let mut editor = session(VIM, "abc", 0);
    keys(&mut editor, "iXY");
    editor.execute(Command::BackwardDeleteChar);
    keys(&mut editor, "Z");
    escape(&mut editor);
    assert_eq!(text(&editor), "XZabc");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    keys(&mut editor, "4294967295.");
    assert_eq!(text(&editor), "XZabc");
    keys(&mut editor, "2.");
    assert_eq!(text(&editor), "XZXZXZabc");
    keys(&mut editor, "u.");
    assert_eq!(text(&editor), "XZXZabc");
}
