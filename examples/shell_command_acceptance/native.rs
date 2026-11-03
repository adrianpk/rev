use super::support::*;
use crossterm::event::{KeyCode, KeyModifiers};
use rev::{
    command::{Command, History},
    session::Session,
};
use std::{fs, path::Path};

const INPUT: &str = "b界\r\na$(printf untrusted)\r\n";

pub(crate) fn run(root: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        commands(root, profile)?;
        readonly(root, profile)?;
        cancelled(root, profile)?;
        stale(root, profile)?;
        println!(
            "PASS native profile={profile}: explicit sh/sort/tr, unsaved UTF-8/CRLF, checked one-step undo, readonly admission, stale source, owned cancellation and idle"
        );
    }
    Ok(())
}

fn commands(root: &Path, profile: &str) -> Result<()> {
    let mut editor = unsaved(root, profile)?;
    let history = editor.buffer()?.history().len();
    named(&mut editor, profile, "shell-command");
    answer(&mut editor, "printf 'hello界\\n'");
    assert_eq!(output(&mut editor)?, "hello界\n");
    switch(&mut editor, "source.txt");
    region(&mut editor, profile, history)?;
    insertion(&mut editor, profile, history)?;
    failures(&mut editor, profile, history)?;
    assert_eq!(source(&editor), INPUT);
    assert_eq!(
        fs::read_to_string(editor.buffer()?.file().unwrap().path())?,
        "saved source\n"
    );
    idle(&mut editor)
}

fn unsaved(root: &Path, profile: &str) -> Result<Session> {
    let mut editor = open(root, profile, "commands", "saved source\n", false)?;
    editor.replace(0..editor.buffer()?.len_chars(), INPUT)?;
    Ok(editor)
}

fn region(editor: &mut Session, profile: &str, history: usize) -> Result<()> {
    whole(editor);
    named(editor, profile, "shell-command-on-region");
    answer(editor, "cat");
    assert_eq!(output(editor)?, INPUT);
    switch(editor, "source.txt");
    assert_eq!(editor.buffer()?.history().len(), history);
    whole(editor);
    named(editor, profile, "shell-filter-region");
    answer(editor, "sort");
    settle(editor)?;
    assert_eq!(source(editor), "a$(printf untrusted)\r\nb界\r\n");
    assert_eq!(editor.buffer()?.history().len(), history + 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(source(editor), INPUT);
    Ok(())
}

fn insertion(editor: &mut Session, profile: &str, history: usize) -> Result<()> {
    named(editor, profile, "shell-command-insert");
    answer(editor, "printf '✨\\r\\n'");
    settle(editor)?;
    assert!(source(editor).contains("✨\r\n"));
    assert_eq!(editor.buffer()?.history().len(), history + 1);
    editor.execute(Command::History(History::Undo));
    whole(editor);
    named(editor, profile, "shell-filter-region");
    answer(editor, "tr b B");
    settle(editor)?;
    assert_eq!(source(editor), INPUT.replacen('b', "B", 1));
    editor.execute(Command::History(History::Undo));
    Ok(())
}

fn failures(editor: &mut Session, profile: &str, history: usize) -> Result<()> {
    let point = editor.view().point;
    whole(editor);
    named(editor, profile, "shell-filter-region");
    answer(editor, "printf rejected; printf error >&2; exit 7");
    settle(editor)?;
    assert_eq!(source(editor), INPUT);
    assert_eq!(editor.buffer()?.history().len(), history);
    assert_eq!(output(editor)?, "rejected");
    switch(editor, "source.txt");
    named(editor, profile, "shell-command-insert");
    key(editor, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert_eq!(editor.view().point.stamp, point.stamp);
    Ok(())
}

fn readonly(root: &Path, profile: &str) -> Result<()> {
    let mut editor = open(root, profile, "readonly", INPUT, true)?;
    whole(&mut editor);
    named(&mut editor, profile, "shell-filter-region");
    assert!(editor.echo().contains("read-only"));
    assert!(editor.remaining_input(std::time::Instant::now()).is_none());
    named(&mut editor, profile, "shell-command-on-region");
    answer(&mut editor, "cat");
    assert_eq!(output(&mut editor)?, INPUT);
    switch(&mut editor, "source.txt");
    assert!(editor.buffer()?.history().is_empty());
    idle(&mut editor)
}

fn cancelled(root: &Path, profile: &str) -> Result<()> {
    let mut editor = open(root, profile, "cancel", INPUT, false)?;
    let directory = editor
        .buffer()?
        .file()
        .unwrap()
        .path()
        .parent()
        .unwrap()
        .to_owned();
    named(&mut editor, profile, "shell-command");
    answer(
        &mut editor,
        "printf '%s' $$ > leader; sleep 3 & printf '%s' $! > descendant; wait",
    );
    let descendant = ready(&directory.join("descendant"))?;
    let leader = ready(&directory.join("leader"))?;
    named(&mut editor, profile, "shell-command-cancel");
    settle(&mut editor)?;
    reaped(&leader)?;
    reaped(&descendant)?;
    assert_eq!(source(&editor), INPUT);
    assert!(editor.buffer()?.history().is_empty());
    idle(&mut editor)
}

fn stale(root: &Path, profile: &str) -> Result<()> {
    let mut editor = open(root, profile, "stale", INPUT, false)?;
    let directory = editor
        .buffer()?
        .file()
        .unwrap()
        .path()
        .parent()
        .unwrap()
        .to_owned();
    whole(&mut editor);
    named(&mut editor, profile, "shell-filter-region");
    answer(
        &mut editor,
        "printf started > started; n=0; while test ! -f ready && test $n -lt 200; do n=$((n+1)); sleep 0.01; done; cat",
    );
    ready(&directory.join("started"))?;
    editor.replace(0..0, "newer ")?;
    fs::write(directory.join("ready"), "")?;
    settle(&mut editor)?;
    assert_eq!(source(&editor), format!("newer {INPUT}"));
    assert_eq!(editor.buffer()?.history().len(), 1);
    assert_eq!(output(&mut editor)?, INPUT);
    idle(&mut editor)
}
