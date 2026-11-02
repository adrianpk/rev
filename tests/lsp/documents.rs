use super::*;
use rev::{
    buffer::{Buffers, edit::History},
    command::{BufferCommand, Command, LspCommand, Motion, RectangleCommand, Split, WindowCommand},
    session::Session,
    view::{Size, View},
};
use serde_json::Value;

fn session(fixture: &Fixture, mode: &str, extra: &str) -> Session {
    let script = fixture.write("documents.sh", include_str!("documents.sh"));
    let args = vec![
        script.to_str().unwrap().to_owned(),
        mode.to_owned(),
        fixture.0.join("documents").to_str().unwrap().to_owned(),
    ];
    let config = format!(
        "[lsp]\nenabled = true\ninitialize_timeout_ms = 2000\nshutdown_timeout_ms = 300\n{extra}\n[lsp.servers.rust]\ncommand = '/bin/sh'\nargs = {args:?}"
    );
    let startup = rev::config::prepare(&fixture.write("config.toml", &config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn frames(fixture: &Fixture) -> Vec<Value> {
    let source = std::fs::read_to_string(fixture.0.join("documents.frames")).unwrap_or_default();
    source
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "bounded protocol observation timed out"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn synchronized(session: &Session) -> rev::lsp::DocumentVersion {
    wait_for(|| session.lsp_document_version().is_some());
    session.lsp_document_version().unwrap()
}

fn notifications(fixture: &Fixture, method: &str, count: usize) -> Vec<Value> {
    wait_for(|| {
        frames(fixture)
            .iter()
            .filter(|frame| frame["method"] == method)
            .count()
            >= count
    });
    frames(fixture)
        .into_iter()
        .filter(|frame| frame["method"] == method)
        .collect()
}

// Unsaved text, not disk bytes, is opened. Incremental ranges use the original
// canonical revision and the negotiated units; undo/redo get fresh versions.
#[test]
fn unsaved_revisions() {
    for (mode, start, end) in [("utf8", 4, 7), ("utf16", 2, 4)] {
        let fixture = Fixture::new();
        let file = fixture.write("unicode space.rs", "😀e\u{301}\r\nlet x = 1;\r\n");
        let mut session = session(&fixture, mode, "");
        session.execute(Command::Lsp(LspCommand::Mode));
        session.load(&file).unwrap();
        session.replace(7..8, "value").unwrap();
        let original = text(&session);
        session.execute(Command::Lsp(LspCommand::Mode));
        let opened = synchronized(&session);
        let open = notifications(&fixture, "textDocument/didOpen", 1);
        assert_eq!(open[0]["params"]["textDocument"]["text"], original);
        assert!(
            open[0]["params"]["textDocument"]["uri"]
                .as_str()
                .unwrap()
                .contains("unicode%20space.rs")
        );
        session.replace(1..3, "Z").unwrap();
        let changed = synchronized(&session);
        assert_eq!(changed.version, opened.version + 1);
        let changes = notifications(&fixture, "textDocument/didChange", 1);
        let edit = &changes[0]["params"]["contentChanges"][0];
        assert_eq!(
            edit["range"]["start"],
            serde_json::json!({"line":0,"character":start})
        );
        assert_eq!(
            edit["range"]["end"],
            serde_json::json!({"line":0,"character":end})
        );
        session.execute(Command::History(History::Undo));
        let undone = synchronized(&session);
        assert_eq!(text(&session), original);
        assert!(undone.version > changed.version && undone.stamp.revision > changed.stamp.revision);
        session.execute(Command::History(History::Redo));
        assert!(synchronized(&session).version > undone.version);
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "😀e\u{301}\r\nlet x = 1;\r\n"
        );
    }
}

// Full sync remains full, and didSave is sent only after the existing save
// owner succeeds, with text only when the server negotiated includeText.
#[test]
fn synchronization_modes() {
    for (mode, full, save_text, save_count) in [
        ("full", true, false, 1),
        ("utf8", false, true, 1),
        ("no-save", false, false, 0),
    ] {
        let fixture = Fixture::new();
        let file = fixture.write("main.rs", "fn first() {}\n");
        let mut session = session(&fixture, mode, "");
        session.load(&file).unwrap();
        synchronized(&session);
        session.replace(3..8, "second").unwrap();
        synchronized(&session);
        let changes = notifications(&fixture, "textDocument/didChange", 1);
        assert_eq!(
            changes[0]["params"]["contentChanges"][0]
                .get("range")
                .is_none(),
            full
        );
        if full {
            assert_eq!(
                changes[0]["params"]["contentChanges"][0]["text"],
                text(&session)
            );
        }
        session.execute(Command::SaveBuffer);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), text(&session));
        if save_count > 0 {
            let saves = notifications(&fixture, "textDocument/didSave", 1);
            assert_eq!(saves[0]["params"].get("text").is_some(), save_text);
        }
        drop(session);
        assert_eq!(
            frames(&fixture)
                .iter()
                .filter(|frame| frame["method"] == "textDocument/didSave")
                .count(),
            save_count
        );
    }
}

// Multiple canonical edits from one rectangle transaction must use descending
// base ranges; applying one range cannot shift a later protocol range.
#[test]
fn multiple_ranges() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "abc\ndef\nghi\n");
    let mut session = session(&fixture, "utf16", "");
    session.load(&file).unwrap();
    synchronized(&session);
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Scroll(rev::command::Scroll::NextLine));
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    synchronized(&session);
    let changes = notifications(&fixture, "textDocument/didChange", 1);
    let ranges = changes[0]["params"]["contentChanges"].as_array().unwrap();
    assert_eq!(ranges.len(), 2, "{}", session.echo());
    assert_eq!(ranges[0]["range"]["start"]["line"], 1);
    assert_eq!(ranges[1]["range"]["start"]["line"], 0);
    assert_eq!(text(&session), "bc\nef\nghi\n");
}

// Canonical aliases and split views share one document. Manual exclusion spans
// those views, survives activation, and restoring it never enables disabled LSP.
#[test]
fn buffer_exceptions() {
    let fixture = Fixture::new();
    fixture.write("Cargo.toml", "");
    let first = fixture.write("one.rs", "fn one() {}\n");
    let second = fixture.write("two.rs", "fn two() {}\n");
    let alias = fixture.0.join("alias.rs");
    std::os::unix::fs::symlink(&first, &alias).unwrap();
    let mut session = session(&fixture, "utf8", "");
    session.load(&first).unwrap();
    let version = synchronized(&session);
    session.load(&alias).unwrap();
    assert_eq!(synchronized(&session), version);
    session.execute(Command::Split(Split::Right));
    assert_eq!(synchronized(&session), version);
    session.load(&second).unwrap();
    let other = synchronized(&session);
    session.load(&first).unwrap();
    assert_eq!(synchronized(&session), version);
    session.execute(Command::Lsp(LspCommand::Disconnect));
    assert_eq!(session.lsp_status(), Status::Excluded);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.lsp_status(), Status::Excluded);
    session.load(&second).unwrap();
    assert_eq!(synchronized(&session), other);
    session.load(&first).unwrap();
    assert_eq!(session.lsp_status(), Status::Excluded);
    session.replace(0..0, "// native\n").unwrap();
    session.execute(Command::Lsp(LspCommand::Connect));
    let restored = synchronized(&session);
    assert_ne!(restored.generation, version.generation);
    assert_eq!(restored.connection, other.connection);
    notifications(&fixture, "textDocument/didOpen", 3);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("documents"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    session.execute(Command::Lsp(LspCommand::Mode));
    session.execute(Command::Lsp(LspCommand::Disconnect));
    session.execute(Command::Lsp(LspCommand::Connect));
    assert_eq!(session.lsp_status(), Status::Disabled);
}

// A retired initialization attachment cannot publish its old snapshot. Restore
// sends only the latest unsaved document through the retained connection.
#[test]
fn retired_open() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "fn original() {}\n");
    let fifo = fixture.0.join("documents.gate");
    assert!(
        std::process::Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let mut session = session(&fixture, "paused", "");
    session.load(&file).unwrap();
    wait_for(|| fixture.0.join("documents.waiting").exists());
    session.execute(Command::Lsp(LspCommand::Disconnect));
    session.replace(3..11, "current").unwrap();
    session.execute(Command::Lsp(LspCommand::Connect));
    let released = std::process::Command::new("timeout")
        .args(["1", "sh", "-c", "printf 'ready\\n' > \"$1\"", "fixture"])
        .arg(&fifo)
        .status()
        .unwrap();
    assert!(released.success());
    synchronized(&session);
    drop(session);
    let opens: Vec<_> = frames(&fixture)
        .into_iter()
        .filter(|frame| frame["method"] == "textDocument/didOpen")
        .collect();
    assert_eq!(opens.len(), 1);
    assert_eq!(
        opens[0]["params"]["textDocument"]["text"],
        "fn current() {}\n"
    );
}

// Closing clears a manual exception and its document generation. Default
// retention allows reopening to reuse the owned connection without another PID.
#[test]
fn close_reopen() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "fn main() {}\n");
    let mut session = session(&fixture, "utf8", "");
    session.load(&file).unwrap();
    let first = synchronized(&session);
    session.execute(Command::Lsp(LspCommand::Disconnect));
    session.execute(Command::Buffer(BufferCommand::Kill));
    session.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ),
    ));
    session.load(&file).unwrap();
    let second = synchronized(&session);
    assert_eq!(second.connection, first.connection);
    assert_ne!(second.generation, first.generation);
    drop(session);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("documents"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

// Protocol positions reject split UTF-8 bytes, UTF-16 surrogates and CRLF
// interiors while round-tripping scalar boundaries independently of display cells.
#[test]
fn checked_positions() {
    use rev::lsp::position::{Position, decode, encode};
    let text = ropey::Rope::from_str("😀e\u{301}\r\n界\n");
    for encoding in [Encoding::Utf8, Encoding::Utf16] {
        for scalar in [0, 1, 2, 3, 5, 6, 7] {
            assert_eq!(
                decode(&text, encode(&text, scalar, &encoding).unwrap(), &encoding).unwrap(),
                scalar
            );
        }
        assert!(encode(&text, 4, &encoding).is_err());
        assert!(
            decode(
                &text,
                Position {
                    line: 0,
                    character: 1
                },
                &encoding
            )
            .is_err()
        );
        assert!(
            decode(
                &text,
                Position {
                    line: 3,
                    character: 0
                },
                &encoding
            )
            .is_err()
        );
        assert!(
            decode(
                &text,
                Position {
                    line: 1,
                    character: 9
                },
                &encoding
            )
            .is_err()
        );
    }
}

// Oversized documents remain native and editable without a partial didOpen or
// an unnecessary server launch. The source is never saved to obtain eligibility.
#[test]
fn document_limit() {
    let fixture = Fixture::new();
    let original = "x".repeat(8 * 1024 * 1024 + 1);
    let file = fixture.write("large.rs", &original);
    let mut session = session(&fixture, "utf8", "");
    session.load(&file).unwrap();
    assert_eq!(session.lsp_status(), Status::Limited);
    assert!(session.lsp_document_version().is_none());
    session.replace(0..0, "// native\n").unwrap();
    assert!(session.buffer().unwrap().is_modified());
    assert!(!fixture.0.join("documents").exists());
    assert_eq!(
        std::fs::metadata(&file).unwrap().len(),
        original.len() as u64
    );
}

// The selected last-buffer policy retires the owned child on close. Reopening
// receives a different connection and document generation, with no stale proof.
#[test]
fn last_buffer_shutdown() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "fn main() {}\n");
    let mut session = session(&fixture, "utf8", "shutdown_on_last_buffer = true");
    session.load(&file).unwrap();
    let first = synchronized(&session);
    let pid: u32 = std::fs::read_to_string(fixture.0.join("documents"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    session.execute(Command::Buffer(BufferCommand::Kill));
    session.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ),
    ));
    wait_for(|| {
        session.expire_input(Instant::now());
        !Path::new(&format!("/proc/{pid}")).exists()
    });
    session.load(&file).unwrap();
    let second = synchronized(&session);
    assert_ne!(second.connection, first.connection);
    assert_ne!(second.generation, first.generation);
}

// A ready shared transport is not synchronization or eligibility evidence for
// an unrelated canonical buffer that has never attached to that connection.
#[test]
fn unattached_status() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "");
    let mut client = Client::new(policy(&fixture, "ready"));
    client.activate(&file, "rust", None);
    assert!(matches!(wait(&client), Status::Ready(_)));
    let buffers = Buffers::new().unwrap();
    assert_eq!(client.buffer_status(buffers.initial()), Status::Unavailable);
    assert!(client.document_version(buffers.initial()).is_none());
}

// Real kernel reload uses the canonical publication observer: one didChange
// version, no source didSave, and the same native undo advances the LSP version.
#[test]
fn external_reload() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "fn first() {}\n");
    let mut session = session(&fixture, "utf8", "");
    session.load(&file).unwrap();
    let opened = synchronized(&session);
    notifications(&fixture, "textDocument/didOpen", 1);
    std::fs::write(&file, "fn external() {}\n").unwrap();
    wait_for(|| {
        session.expire_input(Instant::now());
        text(&session) == "fn external() {}\n"
    });
    let changed = synchronized(&session);
    assert_eq!(changed.version, opened.version + 1);
    let changes = notifications(&fixture, "textDocument/didChange", 1);
    assert_eq!(changes.len(), 1);
    assert_eq!(
        changes[0]["params"]["textDocument"]["version"],
        changed.version
    );
    assert!(
        frames(&fixture)
            .iter()
            .all(|f| f["method"] != "textDocument/didSave")
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "fn first() {}\n");
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(synchronized(&session).version, changed.version + 1);
}
