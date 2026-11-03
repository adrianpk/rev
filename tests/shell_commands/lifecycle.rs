use super::*;
use crate::filters::settle;
use rev::command::{CompletionCommand, Motion};

fn control(editor: &mut Session, value: char) {
    editor.handle(key(KeyCode::Char(value), KeyModifiers::CONTROL));
}
fn named(editor: &mut Session, modal: bool, command: &str) {
    if modal {
        editor.handle(key(KeyCode::Char(':'), KeyModifiers::NONE));
    } else {
        editor.handle(key(KeyCode::Char('x'), KeyModifiers::ALT));
    }
    answer(editor, command);
}
fn session(f: &Fixture, modal: bool, text: &str) -> (Session, std::path::PathBuf) {
    let path = f.write("source.rs", text);
    let config = f.write("config.toml", &format!("editor.interaction_profile='{}'\nshell.program='/bin/sh'\nfiles.external_change_debounce_ms=0", if modal { "modal" } else { "non-modal" }));
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    (
        Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap(),
        path,
    )
}
fn waiting(editor: &mut Session, source: &Path) -> String {
    start(
        editor,
        "printf '%s' $$ > leader; sleep 30 & printf '%s' $! > descendant; wait",
    );
    ready(&source.parent().unwrap().join("descendant"));
    fs::read_to_string(source.parent().unwrap().join("leader")).unwrap()
}
fn alive(pid: &str) -> bool {
    Path::new(&format!("/proc/{}", pid.trim())).exists() && !stopped(pid)
}

// M-x and colon execute the same native filter and one-step undo. Visual line
// ranges survive colon entry; Visual blocks reject rather than flattening input.
#[test]
fn named_regions() {
    for modal in [false, true] {
        let f = Fixture::new();
        let (mut editor, _) = session(&f, modal, "b\na\n");
        if modal {
            for value in ['V', 'G'] {
                editor.handle(key(KeyCode::Char(value), KeyModifiers::NONE));
            }
        } else {
            crate::regions::whole(&mut editor);
        }
        named(&mut editor, modal, "shell-filter-region");
        answer(&mut editor, "sort");
        settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), "a\nb\n");
        assert_eq!(editor.buffer().unwrap().history().len(), 1);
        editor.execute(Command::History(rev::command::History::Undo));
        assert_eq!(text(editor.buffer().unwrap()), "b\na\n");
        named(&mut editor, modal, "shell-command-output");
        assert_eq!(text(editor.buffer().unwrap()), "a\nb\n");
    }
    let f = Fixture::new();
    let (mut editor, _) = session(&f, true, "ab\ncd");
    control(&mut editor, 'v');
    editor.handle(key(KeyCode::Char('j'), KeyModifiers::NONE));
    named(&mut editor, true, "shell-command-on-region");
    assert!(
        editor.echo().contains("linear selection"),
        "{}",
        editor.echo()
    );
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert!(editor.buffer().unwrap().history().is_empty());
}

// Inner argument/search/selectors and modal Escape consume cancellation before
// the job. A subsequent source C-g cancels and reaps the owned leader/descendant.
#[test]
fn inner_cancellation() {
    for modal in [false, true] {
        for inner in [
            Command::GotoLine,
            Command::SearchForward,
            Command::ExecuteExtended,
        ] {
            let f = Fixture::new();
            let (mut editor, path) = session(&f, modal, "source");
            let leader = waiting(&mut editor, &path);
            editor.execute(inner);
            control(&mut editor, 'g');
            assert!(alive(&leader), "{modal}: {inner:?}");
            assert!(editor.remaining_input(Instant::now()).is_some());
            if modal {
                editor.handle(key(KeyCode::Esc, KeyModifiers::NONE));
                assert!(alive(&leader));
            }
            control(&mut editor, 'g');
            settle(&mut editor);
            assert!(!alive(&leader));
            assert!(stopped(
                &fs::read_to_string(path.parent().unwrap().join("descendant")).unwrap()
            ));
            assert_eq!(text(editor.buffer().unwrap()), "source");
        }
    }
}

// A real native completion popup owns its C-g. It closes without cancelling
// the child; the next C-g in the source is the explicit job cancellation.
#[test]
fn popup_cancellation() {
    let f = Fixture::new();
    let (mut editor, path) = session(&f, false, "fn cfg_value() {}\nfn main() { cf }\n");
    editor.execute(Command::Motion(Motion::BufferEnd));
    for _ in 0..4 {
        editor.execute(Command::Motion(Motion::BackwardChar));
    }
    let leader = waiting(&mut editor, &path);
    editor.execute(Command::Completion(CompletionCommand::AtPoint));
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.completion().is_none() {
        editor.expire_input(Instant::now());
        editor.frame().unwrap();
        assert!(
            Instant::now() < deadline,
            "popup unavailable: {}",
            editor.echo()
        );
    }
    control(&mut editor, 'g');
    assert!(editor.completion().is_none());
    assert!(alive(&leader));
    control(&mut editor, 'g');
    settle(&mut editor);
    assert!(!alive(&leader));
}

// Busy attempts never queue/replace the operation, including the stopping slot.
// Named cancellation is shared by both selectors and permits a later fresh job.
#[test]
fn single_job() {
    for modal in [false, true] {
        let f = Fixture::new();
        let (mut editor, path) = session(&f, modal, "source");
        let leader = waiting(&mut editor, &path);
        editor.execute(Command::ShellCommandInsert);
        assert!(editor.echo().contains("already active"));
        assert!(alive(&leader));
        named(&mut editor, modal, "shell-command-cancel");
        editor.execute(Command::ShellCommand);
        assert!(editor.echo().contains("already active"));
        settle(&mut editor);
        assert!(!alive(&leader));
        start(&mut editor, "printf next");
        assert_eq!(output(&mut editor), "next");
    }
}

// A newer action at the same source point owns the echo. Completion retains
// output without replacing that interaction or selecting its output buffer.
#[test]
fn newer_interaction() {
    let f = Fixture::new();
    let (mut editor, path) = session(&f, false, "source");
    let source = editor.view().point;
    start(
        &mut editor,
        "while test ! -f ready; do sleep 0.01; done; printf done",
    );
    editor.execute(Command::ShellCommand);
    let echo = editor.echo().to_owned();
    fs::write(path.parent().unwrap().join("ready"), "").unwrap();
    settle(&mut editor);
    assert_eq!(editor.echo(), echo);
    assert_eq!(editor.view().point, source);
    assert_eq!(output(&mut editor), "done");
}

// The explicit output command reopens the retained identity. Closing it makes
// that result unavailable, and never resurrects it under a reused buffer name.
#[test]
fn output_identity() {
    let f = Fixture::new();
    let (mut editor, _) = session(&f, false, "source");
    start(&mut editor, "printf retained");
    assert_eq!(output(&mut editor), "retained");
    let result = editor.view().point.stamp.buffer;
    switch(&mut editor, "source.rs");
    editor.execute(Command::ShellCommandOutput);
    assert_eq!(editor.view().point.stamp.buffer, result);
    editor.execute(Command::Buffer(BufferCommand::Kill));
    answer(&mut editor, "");
    assert_ne!(editor.view().point.stamp.buffer, result);
    let before = editor.view().point;
    editor.execute(Command::ShellCommandOutput);
    assert!(editor.echo().contains("unavailable"));
    assert_eq!(editor.view().point, before);
}

// Writing to another visiting path revokes the captured edit authority while
// preserving complete stdout and the user's new association and saved content.
#[test]
fn source_rebind() {
    let f = Fixture::new();
    let (mut editor, path) = session(&f, false, "source");
    editor.execute(Command::ShellCommandInsert);
    answer(
        &mut editor,
        "while test ! -f ready; do sleep 0.01; done; printf output",
    );
    let other = f.0.join("other.rs");
    editor.execute(Command::WriteFile);
    answer(&mut editor, other.to_str().unwrap());
    fs::write(path.parent().unwrap().join("ready"), "").unwrap();
    settle(&mut editor);
    assert_eq!(editor.buffer().unwrap().file().unwrap().path(), other);
    assert_eq!(text(editor.buffer().unwrap()), "source");
    assert!(editor.buffer().unwrap().history().is_empty());
    assert_eq!(fs::read_to_string(other).unwrap(), "source");
    assert_eq!(output(&mut editor), "output");
}

// Cancelled quit preserves the running job and dirty source. Only completed
// save/discard consent stops/reaps the group and returns the native exit action.
#[test]
fn quit_lifecycle() {
    for save in [false, true] {
        let f = Fixture::new();
        let (mut editor, path) = session(&f, false, "source");
        editor.replace(0..0, "dirty ").unwrap();
        let leader = waiting(&mut editor, &path);
        assert_eq!(editor.execute(Command::Quit), Action::Redraw);
        control(&mut editor, 'g');
        assert!(alive(&leader));
        assert!(editor.buffer().unwrap().is_modified());
        editor.execute(Command::Quit);
        assert!(alive(&leader));
        let action = editor.handle(key(
            KeyCode::Char(if save { 'y' } else { 'n' }),
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Exit, "{}", editor.echo());
        assert!(!alive(&leader));
        assert!(stopped(
            &fs::read_to_string(path.parent().unwrap().join("descendant")).unwrap()
        ));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            if save { "dirty source" } else { "source" }
        );
        settle(&mut editor);
        assert!(editor.remaining_input(Instant::now()).is_none());
    }
}

// A different selected source owns its own C-g and cannot cancel an operation
// launched from the original source. Returning there restores contextual cancel.
#[test]
fn other_source_cancel() {
    let f = Fixture::new();
    let (mut editor, path) = session(&f, false, "source");
    let other = f.write("other.txt", "other");
    let leader = waiting(&mut editor, &path);
    editor.execute(Command::FindFile);
    answer(&mut editor, other.to_str().unwrap());
    control(&mut editor, 'g');
    assert!(alive(&leader));
    assert_eq!(text(editor.buffer().unwrap()), "other");
    switch(&mut editor, "source.rs");
    control(&mut editor, 'g');
    settle(&mut editor);
    assert!(!alive(&leader));
}
