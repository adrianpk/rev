use super::{fixture::Fixture, lifecycle::text};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::{Command, PluginCommand},
    config,
    plugins::lifecycle::State,
    session::Session,
    view::{Size, View},
};
use rev_plugin_sdk::{limits::Limits, methods, rpc::Message};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    time::{Duration, Instant},
};

const MANIFEST: &str = include_str!("../../docs/reference/plugin-example.toml");
const CONFIG: &str = include_str!("../../docs/reference/plugin-config-example.toml");
const PROTOCOL: &str = include_str!("../../docs/reference/plugin-protocol.md");

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn select(session: &mut Session, profile: &str, name: &str) {
    session.handle(if profile == "modal" {
        key(KeyCode::Char(':'))
    } else {
        Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
    });
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}
fn wait(session: &mut Session, condition: impl Fn(&Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        session.expire_input(Instant::now());
        if condition(session) {
            return;
        }
        assert!(Instant::now() < deadline, "{}", session.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
}
struct Owned(Session);
impl Drop for Owned {
    fn drop(&mut self) {
        let cleanup = self.0.shutdown_plugins();
        if !std::thread::panicking() {
            cleanup.unwrap();
        }
    }
}

// Compile the exact downloadable source using only its public SDK dependency.
// Execute the unchanged manifest/config assets through both native selectors,
// their user shortcut and generation management, with private owned processes.
#[test]
fn hello_workflow() {
    let fixture = Fixture::new();
    fixture.write("plugin-example.toml", MANIFEST);
    let binary = fixture.0.join("hello-plugin/target/debug/hello-plugin");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let library = fs::read_dir(&deps)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("librev_plugin_sdk-")
                && path.extension().is_some_and(|ext| ext == "rlib")
        })
        .max_by_key(|path| path.metadata().unwrap().modified().unwrap())
        .unwrap();
    let output =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args([
                "--edition=2024",
                "--crate-name",
                "plugin_hello",
                "--forbid=unsafe_code",
            ])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/reference/plugin-hello.rs"))
            .arg("-L")
            .arg(format!("dependency={}", deps.display()))
            .arg("--extern")
            .arg(format!("rev_plugin_sdk={}", library.display()))
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for profile in ["non-modal", "modal"] {
        let path = fixture.write(
            "config.toml",
            &format!("{CONFIG}\n[editor]\ninteraction_profile='{profile}'\n"),
        );
        let prepared = config::prepare(&path);
        assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
        assert_eq!(
            prepared
                .snapshot
                .plugins()
                .package("field.hello")
                .unwrap()
                .program,
            binary
        );
        let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
        let id = buffers.create("practice e\u{301}🦀\n").unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut owned = Owned(
            Session::new(
                buffers,
                view,
                Size {
                    width: 120,
                    height: 24,
                },
            )
            .unwrap(),
        );
        let session = &mut owned.0;
        assert_eq!(
            session.plugin_status("field.hello").unwrap().state,
            State::Installed
        );
        // Discovery remains passive until the accepted qualified command.
        session.handle(if profile == "modal" {
            key(KeyCode::Char(':'))
        } else {
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
        });
        session.handle(Event::Paste("field.hello.ping".into()));
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().contains("field.hello.ping"))
        );
        assert_eq!(
            session.plugin_status("field.hello").unwrap().state,
            State::Installed
        );
        session.handle(key(KeyCode::Enter));
        wait(session, |s| s.echo().contains("completed"));
        assert_eq!(
            session.plugin_status("field.hello").unwrap().state,
            State::Active
        );
        assert_eq!(text(session), "practice e\u{301}🦀\n");
        let first = session.plugin_status("field.hello").unwrap().generation;
        session.execute(Command::Plugin(PluginCommand::Deactivate));
        session.handle(Event::Paste("field.hello".into()));
        session.handle(key(KeyCode::Enter));
        wait(session, |s| {
            s.plugin_status("field.hello").unwrap().state == State::Inactive
        });
        // The explicit configured key lazily starts a fresh generation.
        session.handle(key(KeyCode::F(8)));
        wait(session, |s| {
            s.echo().contains("completed")
                && s.plugin_status("field.hello").unwrap().state == State::Active
        });
        assert_ne!(
            session.plugin_status("field.hello").unwrap().generation,
            first
        );
        assert_eq!(text(session), "practice e\u{301}🦀\n");
        select(session, profile, "plugin-restart");
        session.handle(Event::Paste("field.hello".into()));
        session.handle(key(KeyCode::Enter));
        wait(session, |s| {
            s.plugin_status("field.hello").unwrap().state == State::Active
                && s.plugin_status("field.hello").unwrap().pending == 0
        });
    }
}

// Compare the reference with the public method/profile catalogs, then admit
// every published JSON frame through the real SDK parser and method schemas.
// This catches omitted surface rows, stale ceilings and invalid copyable wire.
#[test]
fn protocol_examples() {
    let documented: BTreeSet<_> = PROTOCOL
        .lines()
        .filter_map(|line| {
            line.strip_prefix("| `")?
                .split_once('`')
                .map(|(name, _)| name)
        })
        .filter(|name| name.contains('.') || *name == "initialize")
        .collect();
    let expected: BTreeSet<_> = methods::METHODS
        .iter()
        .map(|entry| entry.0)
        .chain([methods::BUFFER_EVENT])
        .collect();
    assert_eq!(documented, expected);
    for (field, ceiling) in serde_json::to_value(Limits::default())
        .unwrap()
        .as_object()
        .unwrap()
    {
        assert!(
            PROTOCOL
                .lines()
                .any(|line| line == format!("| {field} | {ceiling} |")),
            "{field}"
        );
    }
    let mut frames = 0;
    for block in PROTOCOL.split("```json\n").skip(1) {
        let frame = block.split_once("\n```").unwrap().0;
        let message: Message = serde_json::from_str(frame).unwrap();
        assert!(message.validate());
        match message {
            Message::Request(request) => {
                methods::validate_params(&request.method, request.params, &Limits::default())
                    .unwrap()
            }
            Message::Failure(error) => assert!(error.error.kind().is_some()),
            _ => panic!("unexpected example envelope"),
        }
        frames += 1;
    }
    assert_eq!(frames, 3);
}
