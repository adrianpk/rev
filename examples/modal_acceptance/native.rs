use crate::support::{Result, complete, key, open, select, settle, source};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    command::{Command, History},
    config::{InteractionProfile, Snapshot},
    input::EditingState,
    session::Session,
};
use std::path::Path;

pub(super) fn settings(root: &Path) -> Result<Snapshot> {
    let path = root.join("modal.toml");
    std::fs::write(
        &path,
        "[editor]\ninteraction_profile = 'modal'\n[tree]\nvisible = false\n[completion]\nautomatic = false\n",
    )?;
    let startup = rev::config::prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    Ok(startup.snapshot)
}

pub(super) fn keys(editor: &mut Session, input: &str) -> Result<()> {
    for character in input.chars() {
        key(editor, KeyCode::Char(character));
    }
    settle(editor)
}

pub(super) fn control(editor: &mut Session, character: char) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    )));
}

pub(super) fn answer(editor: &mut Session, value: &str) -> Result<()> {
    editor.handle(Event::Paste(value.into()));
    key(editor, KeyCode::Enter);
    settle(editor)
}

pub(super) fn run(root: &Path) -> Result<()> {
    isolation(root)?;
    editing(root)?;
    commands(root)?;
    completion(root)?;
    println!(
        "PASS native modal journey: immutable profiles, operators, three Visual shapes, jumps, repeat, shared colon/M-x arguments, popup controls and one-step undo"
    );
    Ok(())
}

fn isolation(root: &Path) -> Result<()> {
    let mut emacs = open(root, "emacs.txt", "|one two", Snapshot::default())?;
    keys(&mut emacs, "i:/")?;
    assert_eq!(source(&emacs), "i:/one two");
    assert_eq!(emacs.editing_state(), None);
    assert_eq!(emacs.interaction_profile(), InteractionProfile::NonModal);
    let mut vim = open(root, "normal.txt", "|one two", settings(root)?)?;
    keys(&mut vim, "q")?;
    vim.handle(Event::Paste("refused".into()));
    assert_eq!(source(&vim), "one two");
    assert!(vim.buffer()?.history().is_empty());
    assert_eq!(vim.editing_state(), Some(EditingState::Normal));
    std::fs::write(
        root.join("modal.toml"),
        "[editor]\ninteraction_profile = 'non-modal'\n",
    )?;
    assert_eq!(vim.interaction_profile(), InteractionProfile::Modal);
    assert_eq!(emacs.interaction_profile(), InteractionProfile::NonModal);
    Ok(())
}

fn editing(root: &Path) -> Result<()> {
    let original = "one two\nthree four\n";
    let mut editor = open(
        root,
        "editing.txt",
        &format!("|{original}"),
        settings(root)?,
    )?;
    selections(&mut editor)?;
    repeat_jumps(&mut editor)?;
    assert_eq!(source(&editor), original);
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/editing.txt"))?,
        original
    );
    Ok(())
}

fn selections(editor: &mut Session) -> Result<()> {
    keys(editor, "dw")?;
    assert_eq!(source(editor), "two\nthree four\n");
    keys(editor, "uvld")?;
    assert_eq!(source(editor), "e two\nthree four\n");
    keys(editor, "uVd")?;
    assert_eq!(source(editor), "three four\n");
    keys(editor, "u0")?;
    control(editor, 'v');
    keys(editor, "jld")?;
    assert_eq!(source(editor), "e two\nree four\n");
    Ok(())
}

fn repeat_jumps(editor: &mut Session) -> Result<()> {
    keys(editor, "ugg0iX")?;
    key(editor, KeyCode::Esc);
    keys(editor, "j0.")?;
    assert_eq!(source(editor), "Xone two\nXthree four\n");
    assert_eq!(editor.buffer()?.history().len(), 2);
    keys(editor, "uu2G")?;
    let newer = editor.view().point;
    control(editor, 'o');
    settle(editor)?;
    assert_ne!(editor.view().point, newer);
    control(editor, 'i');
    settle(editor)?;
    assert_eq!(editor.view().point, newer);
    Ok(())
}

fn commands(root: &Path) -> Result<()> {
    let config = settings(root)?;
    let mut colon = open(root, "colon.txt", "|one\ntwo\n", config.clone())?;
    let mut extended = open(root, "extended.txt", "|one\ntwo\n", config)?;
    colon_line(&mut colon)?;
    extended.execute(Command::ExecuteExtended);
    answer(&mut extended, "goto-line")?;
    answer(&mut extended, "2")?;
    assert_eq!(colon.view().point.position, extended.view().point.position);
    assert_eq!(source(&colon), source(&extended));
    visual_cancel(&mut colon)
}

fn colon_line(editor: &mut Session) -> Result<()> {
    keys(editor, ":goto-li")?;
    control(editor, 'i');
    key(editor, KeyCode::Enter);
    answer(editor, "2")
}

fn visual_cancel(colon: &mut Session) -> Result<()> {
    keys(colon, "v:")?;
    let mark = colon.buffer()?.mark();
    control(colon, 'g');
    assert_eq!(colon.editing_state(), Some(EditingState::VisualCharacter));
    assert_eq!(colon.buffer()?.mark(), mark);
    assert!(colon.buffer()?.history().is_empty());
    Ok(())
}

fn completion(root: &Path) -> Result<()> {
    let original = "fn cfg_value() {}\nfn main() { cfg| }\n";
    let mut editor = open(root, "complete.rs", original, settings(root)?)?;
    keys(&mut editor, "i")?;
    complete(&mut editor)?;
    select(&mut editor, "cfg_value");
    assert_eq!(
        editor.completion().unwrap().candidates()[0].kind(),
        rev::session::CompletionKind::Function
    );
    assert!(
        editor
            .frame()?
            .rows()
            .iter()
            .any(|row| !row.syntax().is_empty())
    );
    control(&mut editor, 'i');
    settle(&mut editor)?;
    assert_eq!(
        source(&editor),
        original
            .replace('|', "")
            .replace("{ cfg }", "{ cfg_value }")
    );
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    assert_eq!(editor.buffer()?.history().len(), 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(source(&editor), original.replace('|', ""));
    complete(&mut editor)?;
    key(&mut editor, KeyCode::Esc);
    assert_eq!(editor.editing_state(), Some(EditingState::Insert));
    key(&mut editor, KeyCode::Esc);
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    Ok(())
}
