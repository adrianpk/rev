use super::fixture::Fixture;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::{BufferCommand, Command, PluginCommand, PluginName},
    config,
    plugins::lifecycle::State,
    session::Session,
    view::{Size, View},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{Duration, Instant},
};

fn peer() -> &'static Path {
    static BINARY: OnceLock<(Fixture, PathBuf)> = OnceLock::new();
    &BINARY
        .get_or_init(|| {
            let fixture = Fixture::new();
            let path = fixture.0.join("plugin-peer");
            let deps = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf();
            let mut compiler = std::process::Command::new(
                std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()),
            );
            compiler
                .args([
                    "--edition=2024",
                    "--crate-name",
                    "plugin_lifecycle_peer",
                    "--forbid=unsafe_code",
                ])
                .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/plugins/peer.rs"))
                .arg("-L")
                .arg(format!("dependency={}", deps.display()))
                .arg("-o")
                .arg(&path);
            let library = fs::read_dir(&deps)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| {
                    p.file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("librev_plugin_sdk-")
                        && p.extension().is_some_and(|e| e == "rlib")
                })
                .max_by_key(|p| p.metadata().unwrap().modified().unwrap())
                .expect("Cargo-built public SDK");
            compiler
                .arg("--extern")
                .arg(format!("rev_plugin_sdk={}", library.display()));

            let output = compiler.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            (fixture, path)
        })
        .1
}
struct Spec<'a> {
    id: &'a str,
    mode: &'a str,
    activation: &'a str,
    ignore_term: bool,
}
pub(super) struct Editor {
    pub(super) fixture: Fixture,
    pub(super) session: Session,
}
impl Editor {
    pub(super) fn new(profile: &str, mode: &str, activation: &str, startup: bool) -> Self {
        Self::packages(
            profile,
            &[Spec {
                id: "field.notes",
                mode,
                activation,
                ignore_term: mode == "hold",
            }],
            if startup { &["field.notes"] } else { &[] },
        )
    }
    pub(super) fn keymaps(
        profile: &str,
        bindings: &[(&str, &str)],
        config: &str,
        rpc: bool,
    ) -> Self {
        let specs: Vec<_> = bindings
            .iter()
            .map(|(id, _)| Spec {
                id,
                mode: if rpc { "buffers" } else { "echo" },
                activation: "",
                ignore_term: false,
            })
            .collect();
        let startup: Vec<_> = if rpc {
            bindings.iter().map(|(id, _)| *id).collect()
        } else {
            Vec::new()
        };
        Self::setup(profile, &specs, &startup, bindings, config)
    }
    fn packages(profile: &str, specs: &[Spec<'_>], startup: &[&str]) -> Self {
        Self::setup(profile, specs, startup, &[], "")
    }
    fn setup(
        profile: &str,
        specs: &[Spec<'_>],
        startup: &[&str],
        bindings: &[(&str, &str)],
        config: &str,
    ) -> Self {
        let fixture = Fixture::new();
        let mut manifests = Vec::new();
        for spec in specs {
            fixture.write(&format!("{}/mode", spec.id), spec.mode);
            let directory = fixture.0.join(spec.id);
            let (program, args) = if spec.ignore_term {
                (
                    "/bin/sh".to_owned(),
                    vec![
                        "-c".to_owned(),
                        "trap '' TERM; exec \"$@\"".into(),
                        "plugin-peer".into(),
                        peer().to_str().unwrap().to_owned(),
                        directory.to_str().unwrap().to_owned(),
                        spec.id.into(),
                    ],
                )
            } else {
                (
                    if spec.mode == "missing" {
                        fixture
                            .0
                            .join("missing-program")
                            .to_str()
                            .unwrap()
                            .to_owned()
                    } else {
                        peer().to_str().unwrap().to_owned()
                    },
                    vec![directory.to_str().unwrap().to_owned(), spec.id.into()],
                )
            };
            let manifest = format!(
                r#"[plugin]
id = "{}"
version = "0.1.0"
api = "rev.plugins/0.1"
[process]
program = {}
args = {}
[[commands]]
name = "collect"
description = "Collect task lines."
parameters = [{{name="label", kind="string", required=false}}]
prefix = "none"
recording = "reject"
[activation]
{}
"#,
                spec.id,
                serde_json::to_string(&program).unwrap(),
                serde_json::to_string(&args).unwrap(),
                spec.activation
            );
            let name = format!("{}.toml", spec.id);
            let extra = bindings
                .iter()
                .find(|(id, _)| *id == spec.id)
                .map_or("", |(_, b)| *b);
            fixture.write(&name, &format!("{manifest}{extra}"));
            manifests.push(name);
        }
        let terminal = if specs.iter().any(|s| s.mode == "buffers") {
            "\n[terminal]\nprogram='/bin/sh'\nargs=['-c', 'exec sleep 30']"
        } else {
            ""
        };
        let path = fixture.write("config.toml", &format!("[editor]\ninteraction_profile='{profile}'\n[plugins]\nenabled=true\nmanifests={}\nstartup={}{}",
            serde_json::to_string(&manifests).unwrap(), serde_json::to_string(startup).unwrap(), format!("{terminal}\n{config}")));
        let prepared = config::prepare(&path);
        assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
        let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
        let id = buffers.create("source 🦀\n").unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let session = Session::new(
            buffers,
            view,
            Size {
                width: 120,
                height: 24,
            },
        )
        .unwrap();
        Self { fixture, session }
    }
    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.fixture.0.join("field.notes").join(name)
    }
    pub(super) fn status(&self) -> rev::plugins::lifecycle::Inspection {
        self.session.plugin_status("field.notes").unwrap()
    }
    pub(super) fn call(&mut self, id: &str) {
        self.session
            .execute(Command::PluginDefined(PluginName::new(id).unwrap()));
    }
    pub(super) fn manage(&mut self, command: PluginCommand) {
        self.session.execute(Command::Plugin(command));
        self.session.handle(Event::Paste("field.notes".into()));
        self.session.handle(key(KeyCode::Enter));
    }
    pub(super) fn wait(&mut self, mut condition: impl FnMut(&Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            self.session.expire_input(Instant::now());
            if condition(self) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "plugin deadline: {:?}, {}; {}",
                self.status().state,
                self.status().diagnostic,
                self.session.echo()
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
impl Drop for Editor {
    fn drop(&mut self) {
        let result = self.session.shutdown_plugins();
        let terminals = self.session.shutdown_terminals();
        if !std::thread::panicking() {
            result.unwrap();
            terminals.unwrap();
        }
    }
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn lines(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
fn stopped(pid: u32) -> bool {
    let path = format!("/proc/{pid}/stat");
    fs::read_to_string(path).map_or(true, |s| s.rsplit_once(") ").unwrap().1.starts_with('Z'))
}

// Both real selectors stay passive. Two first-use calls join one blocked
// handshake, retain distinct native captures, and do not block native typing.
#[test]
fn lazy_profiles() {
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::new(profile, "slow", "", false);
        let source = editor.session.buffer().unwrap().id().to_string();
        let open = if profile == "modal" {
            key(KeyCode::Char(':'))
        } else {
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
        };
        editor.session.handle(open);
        editor
            .session
            .handle(Event::Paste("field.notes.collect".into()));
        assert!(!editor.path("starts").exists());
        let start = Instant::now();
        editor.session.handle(key(KeyCode::Enter));
        let file = editor.fixture.write("other.rs", "second\n");
        editor.session.load(&file).unwrap();
        let other = editor.session.buffer().unwrap().id().to_string();
        let revision = editor
            .session
            .buffer()
            .unwrap()
            .stamp()
            .revision
            .to_string();
        editor.call("field.notes.collect");
        if profile == "modal" {
            editor.session.handle(key(KeyCode::Char('i')));
        }
        editor.session.handle(Event::Paste("!".into()));
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "input blocked by activation"
        );
        assert_eq!(text(&editor.session), "!second\n");
        editor.wait(|e| e.path("offers").exists());
        assert_eq!(editor.status().pending, 2);
        assert_eq!(editor.status().state, State::Activating);
        fs::write(editor.path("release"), "ready").unwrap();
        editor.wait(|e| e.status().state == State::Active && e.status().pending == 0);
        let calls = lines(&editor.path("calls"));
        assert_eq!(calls.len(), 2);
        for (call, target) in calls.iter().zip([source, other]) {
            assert_eq!(
                call["params"]["context"]["stamp"]["buffer"]["id"],
                format!("buffer-{target}")
            );
            assert_eq!(call["params"]["context"]["point"], 0);
        }
        assert_eq!(calls[1]["params"]["context"]["stamp"]["revision"], revision);
        let generation = editor.status().generation;
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().pending == 0 && lines(&e.path("calls")).len() == 3);
        assert_eq!(editor.status().generation, generation);
        assert_eq!(
            fs::read_to_string(editor.path("starts"))
                .unwrap()
                .lines()
                .count(),
            1
        );
        let offers = lines(&editor.path("offers"));
        assert_eq!(
            offers[0]["capabilities"],
            serde_json::json!([
                "commands",
                "buffers",
                "display",
                "edits",
                "invocation",
                "file_lifecycle",
                "keymaps"
            ])
        );
    }
}

// Configured startup and native filetype/open/change/close observations use the
// same lazy owner. Ordinary closed buffers never unload an already active peer.
#[test]
fn activation_triggers() {
    for (name, activation, startup) in [
        ("startup", "", true),
        ("filetype", "filetypes=['rust']", false),
        ("opened", "events=['opened']", false),
        ("changed", "events=['changed']", false),
        ("closed", "events=['closed']", false),
    ] {
        let mut editor = Editor::new("non-modal", "echo", activation, startup);
        if matches!(name, "changed" | "closed" | "filetype") {
            editor.wait(|e| e.session.remaining_input(Instant::now()).is_none());
            assert!(!editor.path("starts").exists(), "{name}");
        }
        match name {
            "filetype" => {
                let file = editor.fixture.write("trigger.rs", "fn sample() {}\n");
                editor.session.load(&file).unwrap();
            }
            "changed" => {
                editor.session.handle(Event::Paste("!".into()));
            }
            "closed" => {
                let source = editor.session.buffer().unwrap().name().to_owned();
                let file = editor.fixture.write("other.txt", "other\n");
                editor.session.load(&file).unwrap();
                editor.session.execute(Command::Buffer(BufferCommand::Kill));
                editor.session.handle(Event::Paste(source));
                editor.session.handle(key(KeyCode::Enter));
            }
            _ => {}
        }
        editor.wait(|e| e.status().state == State::Active);
        let pid = editor.status().pid.unwrap();
        editor.wait(|e| e.session.remaining_input(Instant::now()).is_none());
        assert_eq!(editor.status().pid, Some(pid), "{name}");
        assert!(!stopped(pid));
    }
}

// Malformed/oversized traffic, EOF/crash and foreign handshake agreement fail
// before handlers publish. A failure leaves all ordinary source text intact.
#[test]
fn peer_failures() {
    for mode in [
        "crash",
        "noise",
        "truncated",
        "oversize",
        "foreign-api",
        "foreign-generation",
        "undeclared",
    ] {
        let mut editor = Editor::new("non-modal", mode, "", false);
        let source = text(&editor.session);
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().state == State::Failed && e.status().pending == 0);
        assert!(!editor.status().diagnostic.is_empty(), "{mode}");
        assert!(
            editor.status().cleanup.contains("group reaped"),
            "{mode}: {:?}",
            editor.status()
        );
        assert_eq!(text(&editor.session), source, "{mode}");
        if let Some(pid) = editor.status().pid {
            assert!(
                !Path::new(&format!("/proc/{pid}")).exists(),
                "{mode}: leader not reaped"
            );
        }
    }
}

// Native cancellation retires callback authority first, sends the SDK cancel
// notification, and escalates a peer that ignores TERM. Restart uses a new epoch.
#[test]
fn cancellation_restart() {
    for profile in ["non-modal", "modal"] {
        let mut editor = Editor::new(profile, "hold", "", false);
        editor.call("field.notes.collect");
        editor.wait(|e| e.path("calls").exists());
        let old = editor.status();
        let start = Instant::now();
        editor.session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(editor.status().state, State::Stopping);
        editor.session.insert('!');
        assert!(start.elapsed() < Duration::from_secs(1));
        editor.wait(|e| e.status().state == State::Inactive);
        assert!(editor.path("cancels").exists());
        assert!(editor.path("retired").exists());
        assert_eq!(editor.status().pending, 0);
        assert!(!Path::new(&format!("/proc/{}", old.pid.unwrap())).exists());
        fs::write(editor.path("mode"), "echo").unwrap();
        editor.manage(PluginCommand::Restart);
        editor.wait(|e| e.status().state == State::Active);
        assert_ne!(editor.status().generation, old.generation);
        assert_ne!(editor.status().pid, old.pid);
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().pending == 0 && lines(&e.path("calls")).len() == 2);
        let calls = lines(&editor.path("calls"));
        assert_ne!(calls[0]["id"], calls[1]["id"]);
        assert_ne!(
            calls[0]["params"]["context"]["generation"],
            calls[1]["params"]["context"]["generation"]
        );
        assert_eq!(text(&editor.session), "!source 🦀\n");
    }
}

// Owned descendant settlement includes TERM-resistant members with open pipes.
// An independent editor's package must remain alive through the first shutdown.
#[test]
fn group_shutdown() {
    let mut first = Editor::new("non-modal", "descendant", "", true);
    let mut second = Editor::new("non-modal", "echo", "", true);
    first.wait(|e| e.status().state == State::Active && e.path("descendant").exists());
    second.wait(|e| e.status().state == State::Active);
    let leader = first.status().pid.unwrap();
    let descendant: u32 = fs::read_to_string(first.path("descendant"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let other = second.status().pid.unwrap();
    assert_ne!(leader, other);
    first.manage(PluginCommand::Deactivate);
    first.wait(|e| e.status().state == State::Inactive);
    assert!(!Path::new(&format!("/proc/{leader}")).exists());
    assert!(stopped(descendant));
    assert!(!stopped(other));
    assert_eq!(text(&first.session), "source 🦀\n");
    first.session.shutdown_plugins().unwrap();
    assert!(!stopped(other));
}

// A native context replacement swaps source registries, not editor-owned
// package processes. The next callback must reuse the same live generation.
#[test]
fn context_lifetime() {
    let mut editor = Editor::new("non-modal", "echo", "", true);
    editor.wait(|e| e.status().state == State::Active);
    let before = editor.status();
    editor
        .fixture
        .write("workspace/document.txt", "workspace source\n");
    let directory = editor.fixture.0.join("workspace");
    editor.session.load(&directory).unwrap();
    for _ in 0..4 {
        if editor.session.echo().contains("n=discard") {
            editor.session.handle(key(KeyCode::Char('n')));
        }
    }
    editor.wait(|e| matches!(e.session.context(), rev::context::Context::Directory(_)));
    assert_eq!(editor.status().pid, before.pid);
    assert_eq!(editor.status().generation, before.generation);
    editor.session.execute(Command::KeyboardQuit);
    editor.call("field.notes.collect");
    editor.wait(|e| e.status().pending == 0 && e.path("calls").exists());
    assert_eq!(editor.status().state, State::Active);
    assert_eq!(
        fs::read_to_string(editor.path("starts"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

// A completed background callback cannot replace a native question or consume
// its cancellation. Feedback waits until editing owns input again.
#[test]
fn prompt_ownership() {
    let mut editor = Editor::new("non-modal", "slow-call", "", false);
    let source = text(&editor.session);
    editor.call("field.notes.collect");
    editor.wait(|e| e.path("calls").exists());
    editor.session.execute(Command::FindFile);
    let question = editor.session.echo().to_owned();
    fs::write(editor.path("complete"), "ready").unwrap();
    editor.wait(|e| e.status().pending == 0);
    assert_eq!(editor.session.echo(), question);
    editor.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(editor.status().state, State::Active);
    assert_eq!(text(&editor.session), source);
    editor.wait(|e| e.session.echo().contains("completed"));
}

// The real SDK sends a package-to-host request and receives a correlated typed
// rejection. A noisy diagnostic stream stays bounded and cannot inject escapes.
#[test]
fn rpc_and_stderr() {
    for mode in ["rpc", "stderr"] {
        let mut editor = Editor::new("non-modal", mode, "", true);
        editor.wait(|e| e.status().state == State::Active);
        if mode == "rpc" {
            editor.wait(|e| e.path("errors").exists());
            let errors = lines(&editor.path("errors"));
            assert_eq!(errors[0]["error"]["data"]["kind"], "unsupported_method");
            assert_eq!(errors[0]["error"]["code"], -32601);
            assert!(
                errors[0]["error"]["data"]["request"]
                    .as_str()
                    .unwrap()
                    .starts_with(&format!("{}:", editor.status().generation.unwrap()))
            );
        } else {
            editor.wait(|e| e.status().stderr.contains("END"));
            let status = editor.status();
            assert_eq!(status.stderr_bytes, 128 * 1024);
            assert!(!status.stderr.contains('\x1b'));
            assert!(status.stderr.contains("END"));
        }
        editor.wait(|e| e.session.remaining_input(Instant::now()).is_none());
        editor.manage(PluginCommand::Deactivate);
        editor.wait(|e| e.status().state == State::Inactive);
    }
}

// Per-package and aggregate pending admission precedes further spawning. The
// active ceiling counts live generations until their owned cleanup completes.
#[test]
fn admission_limits() {
    let ids: Vec<_> = std::iter::once("field.notes".to_owned())
        .chain((1..9).map(|n| format!("field.p{n}")))
        .collect();
    let specs: Vec<_> = ids
        .iter()
        .map(|id| Spec {
            id,
            mode: "hold",
            activation: "",
            ignore_term: true,
        })
        .collect();
    let mut editor = Editor::packages("non-modal", &specs, &[]);
    for id in &ids[..4] {
        let command = format!("{id}.collect");
        for count in 1..=16 {
            let deadline = Instant::now() + Duration::from_secs(2);
            while editor.session.plugin_status(id).unwrap().pending < count {
                editor.call(&command);
                editor.session.expire_input(Instant::now());
                assert!(Instant::now() < deadline);
            }
        }
        editor.call(&command);
        let bound = if id == &ids[3] {
            "64"
        } else {
            "16 per package"
        };
        assert!(
            editor.session.echo().contains(bound),
            "{id}: {}",
            editor.session.echo()
        );
    }
    editor.call(&format!("{}.collect", ids[4]));
    assert!(editor.session.echo().contains("64"));
    assert!(!editor.fixture.0.join(&ids[4]).join("starts").exists());
    editor.session.execute(Command::KeyboardQuit);
    editor.wait(|e| {
        ids[..4]
            .iter()
            .all(|id| e.session.plugin_status(id).unwrap().state == State::Inactive)
    });
    for id in &ids[..8] {
        fs::write(editor.fixture.0.join(id).join("mode"), "echo").unwrap();
        editor.call(&format!("{id}.collect"));
        editor.wait(|e| {
            let s = e.session.plugin_status(id).unwrap();
            s.state == State::Active && s.pending == 0
        });
    }
    editor.call(&format!("{}.collect", ids[8]));
    assert!(editor.session.echo().contains("process limit exceeded (8)"));
    assert!(!editor.fixture.0.join(&ids[8]).join("starts").exists());
}

// Exercise the actual startup deadline without shortening the production
// contract. A silent peer cannot keep activation or request permits forever.
#[test]
fn startup_deadline() {
    let mut editor = Editor::new("non-modal", "timeout", "", true);
    editor.wait(|e| e.path("offers").exists());
    editor.wait(|e| e.status().state == State::Failed);
    assert!(editor.status().diagnostic.contains("deadline exceeded"));
    assert_eq!(editor.status().pending, 0);
    assert_eq!(text(&editor.session), "source 🦀\n");
}

// Isolate exact descriptor evidence from unrelated test workers. Repeated
// failed execs release their pipes, control eventfd and pending registrations.
#[test]
fn spawn_descriptors() {
    if std::env::var_os("REV_PLUGIN_FD_FIXTURE").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "lifecycle::spawn_descriptors",
                "--test-threads=1",
            ])
            .env("REV_PLUGIN_FD_FIXTURE", "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    let mut editor = Editor::new("non-modal", "missing", "", false);
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    for _ in 0..8 {
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().state == State::Failed && e.status().pending == 0);
    }
    editor.session.shutdown_plugins().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while fs::read_dir("/proc/self/fd").unwrap().count() != baseline && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
    assert_eq!(text(&editor.session), "source 🦀\n");
}
