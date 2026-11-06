#[path = "shell.rs"]
mod shell;

#[path = "context_safety.rs"]
mod safety;

#[path = "context_lsp.rs"]
mod lsp;

#[path = "completion.rs"]
mod completion;

use super::navigation::{keys, settle};
use super::*;
use rev::command::{Motion, Split, WindowCommand};

pub(super) fn control(editor: &mut Session, character: char) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    )));
}

pub(super) fn answer(editor: &mut Session, value: &str) {
    editor.handle(Event::Paste(value.into()));
    editor.handle(key(KeyCode::Enter));
    settle(editor);
}

fn pane(editor: &mut Session, suffix: char) {
    control(editor, 'w');
    keys(editor, &suffix.to_string());
}

// Colon and M-x share the same full vertical catalog, completion, history and
// argument dispatch. Command names remain canonical with no Ex interpretation.
#[test]
fn colon_catalog() {
    let mut editor = session(VIM, "one\ntwo\nthree", 0);
    keys(&mut editor, ":goto-li");
    editor.handle(key(KeyCode::Tab));
    let colon = editor.frame().unwrap();
    assert!(
        colon
            .rows()
            .iter()
            .any(|r| r.text().contains("M-x goto-line"))
    );
    editor.handle(key(KeyCode::Enter));
    answer(&mut editor, "2");
    assert_eq!(editor.view().point.position, Scalar(4));
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('p'),
        KeyModifiers::ALT,
    )));
    let recalled = editor.frame().unwrap();
    assert!(
        recalled
            .rows()
            .iter()
            .any(|r| r.text().contains("M-x goto-line"))
    );
    control(&mut editor, 'g');
    keys(&mut editor, ":w");
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("windmove"))
    );
    control(&mut editor, 'g');
    assert_eq!(text(&editor), "one\ntwo\nthree");
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Both entry gestures share session usage. Canceling an argument still counts
// its named invocation; internal calls, Tab and selector cancellation do not.
#[test]
fn colon_ranking() {
    for colon in [true, false] {
        let mut editor = session(VIM, "one\ntwo\nthree", 0);
        editor.execute(Command::GotoLine);
        control(&mut editor, 'g');
        let open = |editor: &mut Session, colon: bool| {
            if colon {
                keys(editor, ":");
            } else {
                editor.handle(Event::Key(KeyEvent::new(
                    KeyCode::Char('x'),
                    KeyModifiers::ALT,
                )));
            }
        };
        open(&mut editor, colon);
        editor.handle(Event::Paste("goto-line".into()));
        editor.handle(key(KeyCode::Enter));
        assert!(
            editor
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().starts_with("Goto line: "))
        );
        control(&mut editor, 'g');
        editor.execute(Command::History(History::Undo));
        open(&mut editor, !colon);
        let frame = editor.frame().unwrap();
        let first = frame
            .rows()
            .iter()
            .find(|row| row.band() == rev::display::Band::Completion)
            .unwrap();
        assert_eq!(first.text().trim(), "goto-line");
        editor.handle(Event::Paste("undo".into()));
        editor.handle(key(KeyCode::Tab));
        control(&mut editor, 'g');
        open(&mut editor, colon);
        let frame = editor.frame().unwrap();
        let first = frame
            .rows()
            .iter()
            .find(|row| row.band() == rev::display::Band::Completion)
            .unwrap();
        assert_eq!(first.text().trim(), "goto-line");
        assert_eq!(text(&editor), "one\ntwo\nthree");
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    }
}

// A cancelled command selector retains the authoritative Visual shape/anchor
// only while its exact native view, point and source observations survive.
#[test]
fn colon_visual() {
    for (entry, cancel) in [('v', 'g'), ('V', 'g'), ('v', '['), ('V', '[')] {
        let mut editor = session(VIM, "one\ntwo", 0);
        keys(&mut editor, &format!("{entry}l:"));
        let point = editor.view().point;
        let mark = editor.buffer().unwrap().mark();
        control(&mut editor, cancel);
        if cancel == '[' {
            editor.expire_input(std::time::Instant::now() + rev::input::META_TIMEOUT);
        }
        assert_eq!(editor.view().point, point);
        assert_eq!(editor.buffer().unwrap().mark(), mark);
        assert!(
            editor
                .editing_state()
                .unwrap()
                .label()
                .starts_with("VISUAL")
        );
        keys(&mut editor, "y");
        assert!(editor.killed_text().is_some());
        assert!(editor.buffer().unwrap().history().is_empty());
    }
    let mut changed = session(VIM, "one two", 0);
    keys(&mut changed, "vl:");
    changed.replace(0..0, "X").unwrap();
    let point = changed.view().point;
    control(&mut changed, 'g');
    assert_eq!(changed.view().point, point);
    assert_eq!(changed.editing_state(), Some(EditingState::Normal));
    assert!(!changed.buffer().unwrap().mark_active());
    assert_eq!(text(&changed), "Xone two");
}

// The shared native handlers consume Visual selections after colon invocation;
// a complete argument workflow remains one semantic Insert C-o command lifetime.
#[test]
fn colon_invocation() {
    let mut editor = session(VIM, "one two", 0);
    keys(&mut editor, "vl:");
    answer(&mut editor, "kill-region");
    assert_eq!(text(&editor), "e two");
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    keys(&mut editor, "ui");
    control(&mut editor, 'o');
    keys(&mut editor, ":goto-line");
    editor.handle(key(KeyCode::Enter));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    answer(&mut editor, "1");
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    keys(&mut editor, ":/");
    assert_eq!(text(&editor), ":/one two");
}

// Native pane navigation uses screen geometry and overlapping orthogonal
// extents, retains points/text and treats a missing adjacent pane as a no-op.
#[test]
fn pane_directions() {
    let mut editor = session(VIM, "source", 2);
    let first = editor.windows().selected_id();
    pane(&mut editor, 'v');
    pane(&mut editor, 'l');
    let right = editor.windows().selected_id();
    assert_ne!(right, first);
    pane(&mut editor, 's');
    pane(&mut editor, 'j');
    let lower = editor.windows().selected_id();
    assert_ne!(lower, right);
    pane(&mut editor, 'h');
    assert_eq!(editor.windows().selected_id(), first);
    pane(&mut editor, 'k');
    assert_eq!(editor.windows().selected_id(), first);
    pane(&mut editor, 'l');
    assert_eq!(editor.windows().selected_id(), right);
    pane(&mut editor, 'j');
    assert_eq!(editor.windows().selected_id(), lower);
    pane(&mut editor, 'c');
    assert_eq!(editor.windows().iter().len(), 2);
    pane(&mut editor, 'o');
    assert_eq!(editor.windows().iter().len(), 1);
    assert_eq!(text(&editor), "source");
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Balancing shares native geometry, preserves topology/focus and startup
// policy, equalizes three same-axis leaves, and survives deletion and resize.
#[test]
fn pane_balance() {
    let config = format!("{VIM}\n[windows]\nsplit_ratio = 0.3");
    let mut editor = session(&config, "source", 0);
    pane(&mut editor, 'v');
    pane(&mut editor, 'v');
    let selected = editor.windows().selected_id();
    let layout = editor.windows().layout().clone();
    pane(&mut editor, '=');
    let geometry = editor
        .windows()
        .geometry(Size {
            width: 80,
            height: 24,
        })
        .unwrap();
    let widths: Vec<_> = geometry
        .panes()
        .iter()
        .map(|(_, area)| area.width())
        .collect();
    assert!(
        widths.iter().max().unwrap() - widths.iter().min().unwrap() <= 1,
        "{widths:?}"
    );
    assert_eq!(editor.windows().layout(), &layout);
    assert_eq!(editor.windows().selected_id(), selected);
    editor.execute(Command::Window(WindowCommand::Delete));
    editor.handle(Event::Resize(50, 12));
    pane(&mut editor, '=');
    assert_eq!(editor.windows().iter().len(), 2);
    assert!(editor.frame().is_ok());
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Command/pane prefixes stay local: prompts own C-w, tree owns colon, while
// compatible modified editing and literal punctuation survive text-entry states.
#[test]
fn context_ownership() {
    let mut editor = session(VIM, "source", 0);
    keys(&mut editor, ":");
    control(&mut editor, 'w');
    keys(&mut editor, "v");
    assert_eq!(editor.windows().iter().len(), 1);
    control(&mut editor, 'g');
    editor.execute(Command::Split(Split::Right));
    editor.execute(Command::Motion(Motion::BufferStart));
    keys(&mut editor, "i:/");
    assert_eq!(text(&editor), ":/source");
    editor.handle(key(KeyCode::Esc));
    pane(&mut editor, 'l');
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
}

// Org source drafts use the immutable session profile and native local save/
// abort bindings. Cancelling a command selector cannot discard the outer draft.
#[test]
fn source_draft_context() {
    use rev::command::OrgCommand;
    let original = "#+BEGIN_SRC rust\nfn demo() {}\n#+END_SRC\n";
    for save in [false, true] {
        let mut editor = session(VIM, original, 0);
        editor.execute(Command::Mode(rev::buffer::Mode::Org));
        editor.execute(Command::Org(OrgCommand::EditSpecial));
        settle(&mut editor);
        assert!(editor.source_edit_draft().is_some(), "{}", editor.echo());
        assert_eq!(editor.interaction_profile(), InteractionProfile::Modal);
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        keys(&mut editor, "iX");
        editor.handle(key(KeyCode::Esc));
        keys(&mut editor, ":");
        control(&mut editor, 'g');
        assert!(editor.source_edit_draft().is_some());
        control(&mut editor, 'c');
        control(&mut editor, if save { 'c' } else { 'k' });
        settle(&mut editor);
        assert!(editor.source_edit_draft().is_none(), "{}", editor.echo());
        assert_eq!(editor.editing_state(), Some(EditingState::Normal));
        if save {
            assert!(text(&editor).contains("Xfn demo()"));
            assert_eq!(editor.buffer().unwrap().history().len(), 1);
            keys(&mut editor, "u");
        }
        assert_eq!(text(&editor), original);
    }
}
