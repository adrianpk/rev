#![forbid(unsafe_code)]

#[path = "shell_commands/filters.rs"]
mod filters;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "shell_commands/interaction.rs"]
mod interaction;
#[path = "shell_commands/lifecycle.rs"]
mod lifecycle;
#[path = "shell_commands/regions.rs"]
mod regions;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::{
    buffer::{Buffer, Buffers},
    command::{BufferCommand, Command},
    session::{Action, Session},
    view::{Size, View},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn editor(f: &Fixture, preferences: &str) -> (Session, std::path::PathBuf) {
    let config = f.write(
        "config.toml",
        &format!("[shell]\nprogram='/bin/sh'\n{preferences}"),
    );
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let source = f.write(
        "nested/source.txt",
        "unsaved input is data; never a command\n",
    );
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    (session, source)
}
fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}
fn answer(editor: &mut Session, command: &str) {
    editor.handle(Event::Paste(command.into()));
    editor.handle(key(KeyCode::Enter, KeyModifiers::NONE));
}
fn start(editor: &mut Session, command: &str) {
    editor.execute(Command::ShellCommand);
    answer(editor, command);
}
fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn output(editor: &mut Session) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        if editor.remaining_input(Instant::now()).is_none() {
            editor.execute(Command::ShellCommandOutput);
        }
        if editor.buffer().unwrap().name().starts_with("*Shell") {
            assert!(editor.buffer().unwrap().is_readonly());
            assert!(!editor.buffer().unwrap().is_org());
            return text(editor.buffer().unwrap());
        }
        assert!(
            Instant::now() < deadline,
            "shell result unavailable: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn switch(editor: &mut Session, name: &str) {
    editor.execute(Command::Buffer(BufferCommand::Switch));
    answer(editor, name);
}
fn ready(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !fs::metadata(path).is_ok_and(|m| m.len() > 0) {
        assert!(Instant::now() < deadline, "fixture did not become ready");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn stopped(pid: &str) -> bool {
    fs::read_to_string(format!("/proc/{}/stat", pid.trim())).map_or(true, |stat| {
        stat.split_once(") ").unwrap().1.starts_with('Z')
    })
}

// Native named invocation keeps unsaved input out of stdin, captures the file
// directory and retains complete literal Unicode/newline/control text readonly.
#[test]
fn literal_output() {
    let f = Fixture::new();
    let (mut editor, source) = editor(&f, "");
    editor.insert('X');
    let original = text(editor.buffer().unwrap());
    start(
        &mut editor,
        "if read -r line; then printf unexpected; else printf '界é\\r\\n\\033[31m'; fi; pwd",
    );
    let result = output(&mut editor);
    assert_eq!(
        result,
        format!("界é\r\n\u{1b}[31m{}\n", source.parent().unwrap().display())
    );
    assert!(editor.buffer().unwrap().history().is_empty());
    editor.load(&source).unwrap();
    assert_eq!(text(editor.buffer().unwrap()), original);
    assert!(editor.buffer().unwrap().is_modified());
    assert!(!fs::read_to_string(&source).unwrap().starts_with('X'));
}

// Both byte streams share one bound, with complete stdout preserved separately
// from nonzero status/stderr. Overflow and invalid text never expose partial data.
#[test]
fn bounded_results() {
    let cases = [
        (
            "exact nonzero",
            "printf abc; printf def >&2; exit 7",
            "abc",
            Some("def"),
        ),
        (
            "aggregate overflow",
            "printf abcd; printf def >&2",
            "output limit",
            None,
        ),
        ("invalid utf8", "printf '\\377'", "not valid UTF-8", None),
        ("nul", "printf '\\000'", "unsupported NUL", None),
        ("empty", ":", "", None),
    ];
    for (name, command, expected, diagnostic) in cases {
        let f = Fixture::new();
        let (mut editor, source) = editor(&f, "max_output_bytes=6");
        let original = text(editor.buffer().unwrap());
        start(&mut editor, command);
        let result = output(&mut editor);
        if diagnostic.is_some() || name == "empty" {
            assert_eq!(result, expected, "{name}");
        } else {
            assert!(result.contains(expected), "{name}: {result}");
            assert!(
                editor
                    .buffer()
                    .unwrap()
                    .name()
                    .starts_with("*Shell Diagnostics*")
            );
        }
        if let Some(stderr) = diagnostic {
            switch(&mut editor, "*Shell Diagnostics*");
            let result = text(editor.buffer().unwrap());
            assert!(
                result.contains("exit status: 7") && result.contains(stderr),
                "{result}"
            );
        }
        editor.load(&source).unwrap();
        assert_eq!(text(editor.buffer().unwrap()), original, "{name}");
        assert!(editor.buffer().unwrap().history().is_empty());
    }
}

// A child fills both pipes beyond socket capacity while the owner alternates
// nonblocking reads. Waiting for only one pipe would deadlock this real shell.
#[test]
fn simultaneous_streams() {
    let f = Fixture::new();
    let (mut editor, _) = editor(&f, "max_output_bytes=262144");
    let chunk = "x".repeat(128);
    start(
        &mut editor,
        &format!(
            "(i=0; while test $i -lt 1024; do printf '{chunk}'; i=$((i+1)); done) & (i=0; while test $i -lt 1024; do printf '{chunk}' >&2; i=$((i+1)); done) & wait"
        ),
    );
    assert_eq!(output(&mut editor), "x".repeat(131072));
    switch(&mut editor, "*Shell Diagnostics*");
    let diagnostics = text(editor.buffer().unwrap());
    assert_eq!(diagnostics.split_once('\n').unwrap().1, "x".repeat(131072));
}

// No launch is authorized by a cancelled, empty or over-byte-limit prompt.
// The Unicode payload fits the scalar limit but exceeds the UTF-8 byte limit.
#[test]
fn command_admission() {
    for (name, command, cancel) in [
        ("cancel", "printf created > marker".into(), true),
        ("empty", String::new(), false),
        ("whitespace", "   ".into(), false),
        ("bytes", "é".repeat(2049), false),
    ] {
        let f = Fixture::new();
        let (mut editor, source) = editor(&f, "");
        let original = text(editor.buffer().unwrap());
        editor.execute(Command::ShellCommand);
        editor.handle(Event::Paste(command));
        if cancel {
            editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        } else {
            editor.handle(key(KeyCode::Enter, KeyModifiers::NONE));
        }
        editor.execute(Command::ShellCommandCancel);
        assert!(
            editor.echo().contains("No shell command is active"),
            "{name}: {}",
            editor.echo()
        );
        assert_eq!(text(editor.buffer().unwrap()), original, "{name}");
        assert!(!source.parent().unwrap().join("marker").exists(), "{name}");
    }
}

// Slot ownership persists until cancellation cleanup is reaped. Both explicit
// cancel and native source C-g stop the leader and its waiting descendants.
#[test]
fn cancel_and_timeout() {
    for mode in ["named", "source key", "timeout"] {
        let f = Fixture::new();
        let (mut editor, source) = editor(&f, "timeout_seconds=1");
        start(
            &mut editor,
            "printf '%s' $$ > leader; sleep 30 & printf '%s' $! > descendant; wait",
        );
        let directory = source.parent().unwrap();
        ready(&directory.join("descendant"));
        editor.execute(Command::ShellCommand);
        assert!(editor.echo().contains("already active"));
        match mode {
            "named" => {
                editor.execute(Command::ShellCommandCancel);
            }
            "source key" => {
                editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
            }
            _ => {}
        }
        let result = output(&mut editor);
        assert!(
            result.contains(if mode == "timeout" {
                "timed out"
            } else {
                "cancelled"
            }),
            "{mode}: {result}"
        );
        let leader = fs::read_to_string(directory.join("leader")).unwrap();
        assert!(
            !Path::new(&format!("/proc/{}", leader.trim())).exists(),
            "leader not reaped"
        );
        assert!(
            stopped(&fs::read_to_string(directory.join("descendant")).unwrap()),
            "{mode}"
        );
        editor.load(&source).unwrap();
        // A completed owner releases the slot and its temporary input deadline.
        start(&mut editor, "printf fresh");
        assert_eq!(output(&mut editor), "fresh");
    }
}

// Normal leader exit and dropping a library-owned session both retire owned
// descendants; a background shell child is never promoted into a session.
#[test]
fn descendant_lifetime() {
    for dropping in [false, true] {
        let f = Fixture::new();
        let (mut editor, source) = editor(&f, "");
        let command = if dropping {
            "printf '%s' $$ > leader; sleep 30 & printf '%s' $! > descendant; wait"
        } else {
            "sleep 30 & printf '%s' $! > descendant; printf done"
        };
        start(&mut editor, command);
        let directory = source.parent().unwrap();
        ready(&directory.join("descendant"));
        if dropping {
            drop(editor);
        } else {
            assert_eq!(output(&mut editor), "done");
        }
        assert!(stopped(
            &fs::read_to_string(directory.join("descendant")).unwrap()
        ));
        if dropping {
            let leader = fs::read_to_string(directory.join("leader")).unwrap();
            assert!(!Path::new(&format!("/proc/{}", leader.trim())).exists());
        }
    }
}

// A later native prompt retains its input/echo while a child completes. The
// explicit output command can still visit the retained text afterward.
#[test]
fn prompt_ownership() {
    let f = Fixture::new();
    let (mut editor, source) = editor(&f, "");
    start(
        &mut editor,
        "while test ! -f ready; do sleep 0.01; done; printf done",
    );
    editor.execute(Command::GotoLine);
    let before = editor.echo().to_owned();
    fs::write(source.parent().unwrap().join("ready"), "").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while editor.remaining_input(Instant::now()).is_some() {
        editor.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "owned work did not become idle");
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(editor.echo(), before);
    editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert_eq!(output(&mut editor), "done");
}

// Fresh subprocesses supply startup environments without unsafe global test
// mutation. Empty/missing SHELL falls back, and explicit configuration wins.
#[test]
fn startup_environment() {
    for mode in [
        "missing",
        "empty",
        "environment",
        "configured",
        "missing program",
    ] {
        let f = Fixture::new();
        let wrapper = f.write(
            "wrapper",
            "#!/bin/sh\nprintf 'selected:'\nexec /bin/sh \"$@\"\n",
        );
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "environment_child", "--nocapture"])
            .env("REV_SHELL_CHILD", mode)
            .env("REV_SHELL_FIXTURE", &f.0)
            .env("REV_SHELL_TOKEN", "inherited")
            .env_remove("SHELL");
        match mode {
            "empty" => {
                child.env("SHELL", "");
            }
            "environment" => {
                child.env("SHELL", &wrapper);
            }
            "configured" | "missing program" => {
                child.env("SHELL", "/unavailable-shell");
            }
            _ => {}
        }
        let result = child.output().unwrap();
        assert!(
            result.status.success(),
            "{mode}: {}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

// This bounded child is selected only by startup_environment's owned harness.
#[test]
fn environment_child() {
    let Ok(mode) = std::env::var("REV_SHELL_CHILD") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("REV_SHELL_FIXTURE").unwrap());
    let source = root.join("source.txt");
    fs::write(&source, "source").unwrap();
    let program = if mode == "configured" { "/bin/sh" } else { "" };
    let config = root.join("config.toml");
    fs::write(
        &config,
        format!("[shell]\nprogram='{program}'\nargs=['-c','--']"),
    )
    .unwrap();
    let snapshot = rev::config::prepare(&config).snapshot;
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &snapshot);
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    start(&mut editor, "printf '%s' \"$REV_SHELL_TOKEN\"");
    let result = output(&mut editor);
    if mode == "missing program" {
        assert!(result.contains("Start configured program"), "{result}");
    } else {
        assert_eq!(
            result,
            if mode == "environment" {
                "selected:inherited"
            } else {
                "inherited"
            }
        );
    }
    assert_ne!(editor.execute(Command::ShellCommandOutput), Action::Exit);
}
