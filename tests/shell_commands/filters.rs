use super::regions::whole;
use super::*;
use rev::command::{History, Motion, RegionCommand, Split};

fn filter(editor: &mut Session, command: &str) {
    editor.execute(Command::ShellFilterRegion);
    answer(editor, command);
}
fn insert(editor: &mut Session, command: &str) {
    editor.execute(Command::ShellCommandInsert);
    answer(editor, command);
}
pub(super) fn source_editor(
    f: &Fixture,
    initial: &str,
    preferences: &str,
    readonly: bool,
) -> (Session, std::path::PathBuf) {
    let path = f.write("source.txt", initial);
    let config = f.write(
        "config.toml",
        &format!("files.external_change_debounce_ms=0\n[shell]\nprogram='/bin/sh'\n{preferences}"),
    );
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(readonly);
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
pub(super) fn settle(editor: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while editor.remaining_input(Instant::now()).is_some() {
        editor.expire_input(Instant::now());
        editor.frame().unwrap();
        assert!(
            Instant::now() < deadline,
            "native work did not finish: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

// A selected unsaved range is sorted in one checked edit, with adjacent text,
// source format and the initiating point restored by ordinary single-step undo.
#[test]
fn checked_replacement() {
    let f = Fixture::new();
    let (mut editor, source) = source_editor(&f, "disk version\n", "", false);
    let unsaved = "pre\nb\na\npost";
    editor
        .replace(0..editor.buffer().unwrap().len_chars(), unsaved)
        .unwrap();
    editor.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..4 {
        editor.execute(Command::Motion(Motion::ForwardChar));
    }
    editor.execute(Command::Region(RegionCommand::Set));
    for _ in 0..4 {
        editor.execute(Command::Motion(Motion::ForwardChar));
    }
    let point = editor.view().point.position;
    let history = editor.buffer().unwrap().history().len();
    filter(&mut editor, "LC_ALL=C sort");
    settle(&mut editor);
    assert_eq!(text(editor.buffer().unwrap()), "pre\na\nb\npost");
    assert_eq!(editor.buffer().unwrap().history().len(), history + 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(editor.buffer().unwrap()), unsaved);
    assert_eq!(editor.view().point.position, point);
    assert_eq!(fs::read_to_string(source).unwrap(), "disk version\n");
}

// Empty replacement deletes its captured range; empty insertion is a real no-op.
// Complete stdout preserves Unicode, BOM data, CRLF and trailing whitespace.
#[test]
fn output_edits() {
    for (name, command, operation, expected, edits) in [
        ("empty filter", ":", "filter", "", 1),
        ("tr filter", "LC_ALL=C tr a-z A-Z", "filter", "BASE", 1),
        ("empty insert", ":", "insert", "base", 0),
        (
            "literal insert",
            "printf '\\357\\273\\277界é\\r\\n  '",
            "insert",
            "\u{feff}界é\r\n  base",
            1,
        ),
        ("literal filter", "cat", "filter", "base", 1),
    ] {
        let f = Fixture::new();
        let (mut editor, _) = source_editor(&f, "base", "", false);
        let before = editor.buffer().unwrap().stamp();
        let initial = editor.buffer().unwrap().history().len();
        if operation == "filter" {
            whole(&mut editor);
            filter(&mut editor, command);
        } else {
            insert(&mut editor, command);
        }
        settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), expected, "{name}");
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            initial + edits,
            "{name}"
        );
        if edits == 0 {
            assert_eq!(editor.buffer().unwrap().stamp(), before);
        } else {
            editor.execute(Command::History(History::Undo));
            assert_eq!(text(editor.buffer().unwrap()), "base", "{name}");
        }
    }
}

// Nonzero/invalid/overflow/deadline/cancelled results cannot become source edits.
// Valid failure stdout remains inspectable separately from its diagnostics.
#[test]
fn failed_filters() {
    for (name, command, preferences, cancel, valid_stdout) in [
        (
            "nonzero",
            "printf partial; printf diagnostic >&2; exit 9",
            "",
            false,
            Some("partial"),
        ),
        ("invalid utf8", "printf '\\377'", "", false, None),
        ("nul", "printf '\\000'", "", false, None),
        (
            "overflow",
            "printf excess",
            "max_output_bytes=3",
            false,
            None,
        ),
        (
            "deadline",
            "printf ready > started; sleep 30",
            "timeout_seconds=1",
            false,
            None,
        ),
        ("cancel", "printf ready > started; sleep 30", "", true, None),
    ] {
        let f = Fixture::new();
        let (mut editor, source) = source_editor(&f, "source\n", preferences, false);
        whole(&mut editor);
        let stamp = editor.buffer().unwrap().stamp();
        filter(&mut editor, command);
        if cancel {
            ready(&f.0.join("started"));
            editor.execute(Command::ShellCommandCancel);
        }
        settle(&mut editor);
        assert_eq!(text(editor.buffer().unwrap()), "source\n", "{name}");
        assert_eq!(editor.buffer().unwrap().stamp(), stamp, "{name}");
        assert!(editor.buffer().unwrap().history().is_empty());
        let retained = output(&mut editor);
        if let Some(expected) = valid_stdout {
            assert_eq!(retained, expected);
        } else {
            assert!(
                editor
                    .buffer()
                    .unwrap()
                    .name()
                    .starts_with("*Shell Diagnostics*")
            );
        }
        assert_eq!(fs::read_to_string(source).unwrap(), "source\n");
    }
}

// Readonly data may be piped to a display-only command. Both edit operations
// reject at invocation, before any command prompt or child can acquire authority.
#[test]
fn readonly_admission() {
    let f = Fixture::new();
    let (mut editor, source) = source_editor(&f, "b\na\n", "", true);
    whole(&mut editor);
    editor.execute(Command::ShellCommandOnRegion);
    answer(&mut editor, "LC_ALL=C sort");
    assert_eq!(output(&mut editor), "a\nb\n");
    editor.load(&source).unwrap();
    for command in [Command::ShellFilterRegion, Command::ShellCommandInsert] {
        whole(&mut editor);
        editor.execute(command);
        assert!(editor.echo().contains("read-only"), "{}", editor.echo());
        editor.execute(Command::ShellCommandCancel);
        assert!(editor.echo().contains("No shell command is active"));
        assert_eq!(text(editor.buffer().unwrap()), "b\na\n");
        assert!(editor.buffer().unwrap().history().is_empty());
    }
}

// Cancellation of each argument prompt retains the source selection, stamp,
// point and history and never starts the proposed file-writing command.
#[test]
fn cancelled_prompts() {
    for command in [
        Command::ShellCommandOnRegion,
        Command::ShellFilterRegion,
        Command::ShellCommandInsert,
    ] {
        let f = Fixture::new();
        let (mut editor, _) = source_editor(&f, "source\n", "", false);
        whole(&mut editor);
        let stamp = editor.buffer().unwrap().stamp();
        let point = editor.view().point;
        let region = editor.buffer().unwrap().region(point).unwrap().unwrap();
        editor.execute(command);
        editor.handle(Event::Paste("printf created > marker".into()));
        editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        assert_eq!(editor.buffer().unwrap().stamp(), stamp);
        assert_eq!(editor.view().point, point);
        assert_eq!(
            editor.buffer().unwrap().region(point).unwrap().unwrap(),
            region
        );
        assert!(editor.buffer().unwrap().history().is_empty());
        assert!(!f.0.join("marker").exists());
    }
}

// A different selected pane is never the publication target. The original
// source is mapped without focus changes and remains reversible in one undo.
#[test]
fn pane_switch() {
    let f = Fixture::new();
    let (mut editor, source) = source_editor(&f, "abc", "", false);
    editor.execute(Command::Motion(Motion::ForwardChar));
    let origin = editor.windows().selected_id();
    insert(
        &mut editor,
        "printf started > started; while test ! -f ready; do sleep 0.01; done; printf X",
    );
    ready(&f.0.join("started"));
    editor.execute(Command::Split(Split::Below));
    editor.execute(Command::Window(rev::command::WindowCommand::Other));
    let other = f.write("other.txt", "different buffer");
    editor.load(&other).unwrap();
    let selected = editor.windows().selected_id();
    assert_ne!(selected, origin);
    fs::write(f.0.join("ready"), "").unwrap();
    settle(&mut editor);
    assert_eq!(editor.windows().selected_id(), selected);
    assert_eq!(text(editor.buffer().unwrap()), "different buffer");
    editor.load(&source).unwrap();
    assert_eq!(text(editor.buffer().unwrap()), "aXbc");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(editor.buffer().unwrap()), "abc");
    assert_eq!(editor.windows().get(origin).unwrap().point.position.0, 1);
}

// Edits, closure, explicit reload and a command's own disk write revoke captured
// edit authority. Complete stdout stays available while native reload policy wins.
#[test]
fn stale_targets() {
    for mode in [
        "edited",
        "edited then undone",
        "closed",
        "reopened",
        "reloaded",
        "own disk write",
    ] {
        let f = Fixture::new();
        let (mut editor, source) = source_editor(&f, "original\n", "", false);
        whole(&mut editor);
        filter(
            &mut editor,
            "printf started > started; while test ! -f ready; do sleep 0.01; done; if test -f write-disk; then printf 'disk changed\\n' > source.txt; fi; printf output",
        );
        ready(&f.0.join("started"));
        match mode {
            "edited" => {
                editor.replace(0..0, "local ").unwrap();
            }
            "edited then undone" => {
                editor.replace(0..0, "local ").unwrap();
                editor.execute(Command::History(History::Undo));
            }
            "closed" | "reopened" => {
                editor.execute(Command::Buffer(BufferCommand::Kill));
                answer(&mut editor, "");
                if mode == "reopened" {
                    editor.load(&source).unwrap();
                }
            }
            "reloaded" => {
                fs::write(&source, "disk changed\n").unwrap();
                editor.execute(Command::RevertBuffer);
                settle_reload(&mut editor, "disk changed\n");
            }
            _ => {
                fs::write(f.0.join("write-disk"), "").unwrap();
            }
        }
        fs::write(f.0.join("ready"), "").unwrap();
        settle(&mut editor);
        assert_eq!(output(&mut editor), "output", "{mode}");
        switch(&mut editor, "*Shell Diagnostics*");
        assert!(
            text(editor.buffer().unwrap()).contains("Not applied:"),
            "{mode}"
        );
        editor.load(&source).unwrap();
        if mode == "own disk write" {
            settle_reload(&mut editor, "disk changed\n");
        }
        let actual = text(editor.buffer().unwrap());
        assert_eq!(
            actual,
            match mode {
                "edited" => "local original\n",
                "closed" | "reopened" | "edited then undone" => "original\n",
                _ => "disk changed\n",
            },
            "{mode}"
        );
        assert!(!actual.contains("output"));
    }
}
fn settle_reload(editor: &mut Session, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while text(editor.buffer().unwrap()) != expected {
        editor.expire_input(Instant::now());
        assert!(
            Instant::now() < deadline,
            "reload did not complete: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

// A real discard-consent prompt retains authority over its captured disk image.
// Shell completion cannot accept it or edit the still-unsaved native source.
#[test]
fn pending_reload_consent() {
    let f = Fixture::new();
    let (mut editor, source) = source_editor(&f, "disk old\n", "", false);
    editor
        .replace(0..editor.buffer().unwrap().len_chars(), "unsaved\n")
        .unwrap();
    whole(&mut editor);
    filter(
        &mut editor,
        "printf started > started; while test ! -f ready; do sleep 0.01; done; printf output",
    );
    ready(&f.0.join("started"));
    fs::write(&source, "disk new\n").unwrap();
    editor.execute(Command::RevertBuffer);
    let deadline = Instant::now() + Duration::from_secs(3);
    while !editor.echo().contains("Discard unsaved contents") {
        editor.expire_input(Instant::now());
        assert!(
            Instant::now() < deadline,
            "discard consent did not arrive: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    fs::write(f.0.join("ready"), "").unwrap();
    settle(&mut editor);
    assert!(
        editor
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("Discard unsaved contents"))
    );
    assert_eq!(text(editor.buffer().unwrap()), "unsaved\n");
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert_eq!(output(&mut editor), "output");
    switch(&mut editor, "*Shell Diagnostics*");
    assert!(text(editor.buffer().unwrap()).contains("Not applied:"));
    assert_eq!(fs::read_to_string(source).unwrap(), "disk new\n");
}

// A later point move does not redirect insertion or restore a stale viewport.
// Undo still restores the original command's initiating source point.
#[test]
fn captured_point() {
    let f = Fixture::new();
    let (mut editor, _) = source_editor(&f, "abc", "", false);
    editor.execute(Command::Motion(Motion::ForwardChar));
    insert(
        &mut editor,
        "printf started > started; while test ! -f ready; do sleep 0.01; done; printf X",
    );
    ready(&f.0.join("started"));
    editor.execute(Command::Motion(Motion::BufferEnd));
    fs::write(f.0.join("ready"), "").unwrap();
    settle(&mut editor);
    assert_eq!(text(editor.buffer().unwrap()), "aXbc");
    assert_eq!(editor.view().point.position.0, 4);
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(editor.buffer().unwrap()), "abc");
    assert_eq!(editor.view().point.position.0, 1);
}

// A native target must still be current when Return accepts the prompt. Host
// edits during that prompt revoke launch authority instead of running old data.
#[test]
fn stale_before_launch() {
    for command in [Command::ShellFilterRegion, Command::ShellCommandInsert] {
        let f = Fixture::new();
        let (mut editor, _) = source_editor(&f, "source", "", false);
        whole(&mut editor);
        editor.execute(command);
        editor.replace(0..0, "local ").unwrap();
        answer(&mut editor, "printf spawned > marker");
        assert!(editor.echo().contains("changed"), "{}", editor.echo());
        editor.execute(Command::ShellCommandCancel);
        assert!(editor.echo().contains("No shell command is active"));
        assert_eq!(text(editor.buffer().unwrap()), "local source");
        assert!(!f.0.join("marker").exists());
    }
}
