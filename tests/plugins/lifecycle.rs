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

pub(super) fn peer() -> &'static Path {
    static BINARY: OnceLock<(Fixture, PathBuf)> = OnceLock::new();
    &BINARY
        .get_or_init(|| {
            let fixture = Fixture::new();
            let path = fixture.0.join("plugin-peer");
            let artifacts = super::artifacts::get();
            let deps = artifacts.json.parent().expect("SDK dependencies");
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
            compiler
                .arg("--extern")
                .arg(format!("rev_plugin_sdk={}", artifacts.sdk.display()))
                .arg("--extern")
                .arg(format!("serde_json={}", artifacts.json.display()));

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
    declaration: Option<&'a str>,
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
                declaration: None,
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
                declaration: None,
            })
            .collect();
        let startup: Vec<_> = if rpc {
            bindings.iter().map(|(id, _)| *id).collect()
        } else {
            Vec::new()
        };
        Self::setup(profile, &specs, &startup, bindings, config, false)
    }
    pub(super) fn recording(profile: &str, mode: &str, declaration: Option<&str>) -> Self {
        let context = if profile == "modal" {
            "vim_normal"
        } else {
            "editor"
        };
        let binding = format!(
            "\n[[bindings]]\ncommand='field.notes.collect'\nkeys='F8'\ncontext='{context}'\nscope='global'\n"
        );
        let specs: Vec<_> = declaration
            .into_iter()
            .flat_map(|declaration| {
                ["field.notes", "other.notes"].map(|id| Spec {
                    id,
                    mode,
                    activation: "",
                    ignore_term: false,
                    declaration: Some(declaration),
                })
            })
            .collect();
        let bindings = declaration.map(|_| ("field.notes", binding.as_str()));
        Self::setup(
            profile,
            specs.as_slice(),
            &[],
            bindings.as_slice(),
            "",
            true,
        )
    }
    pub(super) fn events(profile: &str, count: usize) -> Self {
        let ids = [
            "field.notes",
            "other.notes",
            "third.notes",
            "fourth.notes",
            "fifth.notes",
            "sixth.notes",
            "seventh.notes",
            "eighth.notes",
        ];
        let specs: Vec<_> = ids
            .iter()
            .take(count)
            .map(|id| Spec {
                id,
                mode: "events",
                activation: "",
                ignore_term: false,
                declaration: None,
            })
            .collect();
        Self::packages(profile, &specs, &ids[..count])
    }
    pub(super) fn state(root: &Path, packages: usize) -> Self {
        let ids = ["field.notes", "other.notes"];
        let specs: Vec<_> = ids
            .iter()
            .take(packages)
            .map(|id| Spec {
                id,
                mode: "events",
                activation: "",
                ignore_term: false,
                declaration: None,
            })
            .collect();
        let mut editor = Self::packages("non-modal", &specs, &[]);
        editor
            .session
            .attach_plugin_state(&config::Roots {
                config: root.join("config"),
                data: root.join("data"),
                state: root.join("state"),
                cache: root.join("cache"),
            })
            .unwrap();
        for id in &ids[..packages] {
            editor
                .session
                .execute(Command::Plugin(PluginCommand::Activate));
            editor.session.handle(Event::Paste((*id).into()));
            editor.session.handle(key(KeyCode::Enter));
        }
        editor
    }
    fn packages(profile: &str, specs: &[Spec<'_>], startup: &[&str]) -> Self {
        Self::setup(profile, specs, startup, &[], "", false)
    }
    fn setup(
        profile: &str,
        specs: &[Spec<'_>],
        startup: &[&str],
        bindings: &[(&str, &str)],
        config: &str,
        library: bool,
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
{}
[activation]
{}
"#,
                spec.id,
                serde_json::to_string(&program).unwrap(),
                serde_json::to_string(&args).unwrap(),
                spec.declaration.unwrap_or("parameters = [{name='label', kind='string', required=false}]\nprefix='none'\nrecording='reject'"),
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
        let session = if library {
            session.with_macro_library(&config::Roots {
                config: fixture.0.join("config"),
                data: fixture.0.join("data"),
                state: fixture.0.join("state"),
                cache: fixture.0.join("cache"),
            })
        } else {
            session
        };
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
                "plugin deadline: {:?}, pending={}, cleanup={}; {}; {}",
                self.status().state,
                self.status().pending,
                self.status().cleanup,
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
pub(super) fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
pub(super) fn lines(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
pub(super) fn stopped(pid: u32) -> bool {
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
                "keymaps",
                "events"
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
            declaration: None,
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
