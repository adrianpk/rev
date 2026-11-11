//! Native acceptance bridge for a separately built public-protocol client.
//! Go source and builds belong to rev/plugin-examples, never this workspace.
#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::{BufferId, Buffers, Stamp, edit::History},
    command::{BufferCommand, Command, PluginCommand},
    config,
    plugins::lifecycle::State,
    session::Session,
    view::{Size, View},
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKAGE: &str = "field.collector";
const COMMAND: &str = "field.collector.collect";

struct Owned(Session);
impl Owned {
    fn session(&mut self) -> &mut Session {
        &mut self.0
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        let result = self.0.shutdown_plugins();
        if !std::thread::panicking() {
            result.expect("owned interoperability package cleanup");
        }
    }
}
fn key(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}
fn switch(session: &mut Session, name: &str) {
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter);
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().expect("native text buffer");
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn wait(session: &mut Session, condition: impl Fn(&Session) -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        session.expire_input(Instant::now());
        if condition(session) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "native interoperability deadline: {:?}; {}",
                session.plugin_status(PACKAGE),
                session.echo()
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn manage(session: &mut Session, action: PluginCommand) {
    session.execute(Command::Plugin(action));
    session.handle(Event::Paste(PACKAGE.into()));
    key(session, KeyCode::Enter);
}

// Two representative profiles exercise real native owners against the same
// separately compiled executable. Broader policy variants stay in tests/plugins.
fn run(program: &Path, root: &Path, profile: &str) -> Result<()> {
    let (fixture, mut owned) = Fixture::prepare(program, root, profile)?;
    let session = owned.session();
    let original = dirty_sources(session, &fixture)?;
    invoke_palette(session, profile)?;
    let first = collected(session, &fixture, &original)?;
    local_invocation(session, &fixture, &original, &first)?;
    let retired = retire_result(session)?;
    retained_sources(session, &fixture, &original, &first, &retired)?;
    println!(
        "Interop profile={profile}: passive discovery, hidden unsaved scalar reads, native result/navigation/local F8, unchanged sources/files and retained text/undo passed"
    );
    Ok(())
}
fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let program = fs::canonicalize(args.next().ok_or("expected PROGRAM ROOT")?)?;
    let root = std::path::PathBuf::from(args.next().ok_or("expected PROGRAM ROOT")?);
    if args.next().is_some() {
        return Err("expected PROGRAM ROOT".into());
    }
    for profile in ["non-modal", "modal"] {
        run(&program, &root.join(profile), profile)?;
    }
    println!(
        "Native public-protocol interoperability passed: api={} editor={}",
        rev_plugin_sdk::API,
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}

struct Sources {
    visible_path: PathBuf,
    hidden_path: PathBuf,
    visible_disk: &'static str,
    hidden_disk: String,
}
impl Sources {
    fn prepare(root: &Path) -> Result<Self> {
        let visible_path = root.join("visible.txt");
        let hidden_path = root.join("hidden.txt");
        let visible_disk = "visible saved\n";
        // The chunk boundary separates e from its combining mark and astral scalar.
        let hidden_disk = format!("{}e\u{301}🦀\r\nhidden tail\n", "a".repeat(16382));
        fs::write(&visible_path, visible_disk)?;
        fs::write(&hidden_path, &hidden_disk)?;
        Ok(Self {
            visible_path,
            hidden_path,
            visible_disk,
            hidden_disk,
        })
    }
}
struct Fixture {
    files: Sources,
    visible: BufferId,
    visible_name: String,
    hidden_name: String,
    orphan_name: String,
}
impl Fixture {
    fn prepare(program: &Path, root: &Path, profile: &str) -> Result<(Self, Owned)> {
        let settings = configuration(program, root, profile)?;
        let files = Sources::prepare(root)?;
        let (fixture, mut owned) = Self::native(settings, files)?;
        let session = owned.session();
        session.attach_plugin_state(&config::Roots {
            config: root.join("config"),
            data: root.join("data"),
            state: root.join("state"),
            cache: root.join("cache"),
        })?;
        Ok((fixture, owned))
    }
    fn native(settings: config::Snapshot, files: Sources) -> Result<(Self, Owned)> {
        let mut buffers = Buffers::with_settings(settings)?;
        let visible = buffers.visit(&files.visible_path)?;
        let hidden = buffers.visit(&files.hidden_path)?;
        let orphan = buffers.create("orphan e\u{301}🦀\n")?;
        let visible_name = buffers.get(visible)?.name().to_owned();
        let hidden_name = buffers.get(hidden)?.name().to_owned();
        let orphan_name = buffers.get(orphan)?.name().to_owned();
        let view = View::new(buffers.get(hidden)?);
        let owned = Owned(Session::new(
            buffers,
            view,
            Size {
                width: 160,
                height: 30,
            },
        )?);
        Ok((
            Self {
                files,
                visible,
                visible_name,
                hidden_name,
                orphan_name,
            },
            owned,
        ))
    }
}
fn configuration(program: &Path, root: &Path, profile: &str) -> Result<config::Snapshot> {
    fs::create_dir_all(root)?;
    fs::write(
        root.join("collector.toml"),
        format!(
            r#"[plugin]
id = "field.collector"
version = "0.1.0"
api = "rev.plugins/0.1"
[process]
program = {}
args = []
[[commands]]
name = "collect"
description = "Collect current text from open buffers."
parameters = []
prefix = "none"
recording = "reject"
"#,
            serde_json::to_string(program.to_str().ok_or("non-UTF-8 program path")?)?
        ),
    )?;
    let path = root.join("config.toml");
    fs::write(
        &path,
        format!(
            r#"[editor]
interaction_profile = "{profile}"
[plugins]
enabled = true
manifests = ["collector.toml"]
startup = []
"#
        ),
    )?;
    let prepared = config::prepare(&path);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    Ok(prepared.snapshot)
}

struct Original {
    hidden_text: String,
    hidden_stamp: Stamp,
    orphan_text: String,
    orphan_stamp: Stamp,
    visible_stamp: Stamp,
}
struct Collected {
    id: BufferId,
    name: String,
    text: String,
    generation: Option<rev_plugin_sdk::identity::Generation>,
}
struct Retired {
    name: String,
    text: String,
}

fn dirty_sources(session: &mut Session, fixture: &Fixture) -> Result<Original> {
    let hidden_disk = &fixture.files.hidden_disk;
    session.insert('!');
    let hidden_text = text(session);
    assert_eq!(hidden_text, format!("!{hidden_disk}"));
    assert!(session.buffer()?.is_modified());
    let hidden_stamp = session.buffer()?.stamp();
    switch(session, &fixture.orphan_name);
    session.insert('~');
    assert!(session.buffer()?.is_modified());
    assert!(session.buffer()?.file().is_none());
    let orphan_text = text(session);
    assert_eq!(orphan_text, "~orphan e\u{301}🦀\n");
    let orphan_stamp = session.buffer()?.stamp();
    switch(session, &fixture.visible_name);
    let visible_stamp = session.buffer()?.stamp();

    Ok(Original {
        hidden_text,
        hidden_stamp,
        orphan_text,
        orphan_stamp,
        visible_stamp,
    })
}

fn invoke_palette(session: &mut Session, profile: &str) -> Result<()> {
    session.handle(if profile == "modal" {
        Event::Key(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE))
    } else {
        Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
    });
    session.handle(Event::Paste(COMMAND.into()));
    assert!(
        session
            .frame()?
            .rows()
            .iter()
            .any(|r| r.text().contains(COMMAND))
    );
    assert_eq!(
        session.plugin_status(PACKAGE).unwrap().state,
        State::Installed
    );
    key(session, KeyCode::Enter);
    wait(session, |s| {
        s.buffer()
            .is_ok_and(|b| b.plugin_owner().is_some_and(|id| id.as_str() == PACKAGE))
            && s.plugin_status(PACKAGE)
                .is_some_and(|p| p.state == State::Active && p.pending == 0)
            && s.echo().contains("completed")
    })?;
    Ok(())
}

fn collected(session: &mut Session, fixture: &Fixture, original: &Original) -> Result<Collected> {
    let first = session.buffer()?.id();
    let first_name = session.buffer()?.name().to_owned();
    let first_text = text(session);
    assert!(first_text.starts_with("# Collected open text\n"));
    assert!(first_text.contains(fixture.files.visible_disk));
    assert!(first_text.contains(&original.hidden_text));
    assert!(first_text.contains(&original.orphan_text));
    assert!(session.buffer()?.is_modified());
    assert!(session.buffer()?.file().is_none());
    assert!(!session.buffer()?.is_transient());
    let generation = session.plugin_status(PACKAGE).unwrap().generation;

    Ok(Collected {
        id: first,
        name: first_name,
        text: first_text,
        generation,
    })
}

fn local_invocation(
    session: &mut Session,
    fixture: &Fixture,
    original: &Original,
    first: &Collected,
) -> Result<()> {
    switch(session, &fixture.visible_name);
    key(session, KeyCode::F(8));
    assert_eq!(session.buffer()?.id(), fixture.visible);
    assert_eq!(session.plugin_status(PACKAGE).unwrap().pending, 0);
    assert_eq!(session.buffer()?.stamp(), original.visible_stamp);
    switch(session, &first.name);
    key(session, KeyCode::F(8));
    wait(session, |s| {
        s.buffer().is_ok_and(|b| {
            b.id() != first.id && b.plugin_owner().is_some_and(|id| id.as_str() == PACKAGE)
        }) && s.plugin_status(PACKAGE).is_some_and(|p| p.pending == 0)
            && s.echo().contains("completed")
    })?;
    assert_eq!(
        session.plugin_status(PACKAGE).unwrap().generation,
        first.generation
    );
    assert_eq!(
        text(session),
        first.text,
        "previous results must not be collected again"
    );
    Ok(())
}

fn retire_result(session: &mut Session) -> Result<Retired> {
    let result_name = session.buffer()?.name().to_owned();
    session.insert('!');
    let result_text = text(session);
    manage(session, PluginCommand::Deactivate);
    wait(session, |s| {
        s.plugin_status(PACKAGE)
            .is_some_and(|p| p.state == State::Inactive && p.pending == 0)
    })?;
    key(session, KeyCode::F(8));
    assert_eq!(
        session.plugin_status(PACKAGE).unwrap().state,
        State::Inactive
    );
    assert_eq!(text(session), result_text);

    Ok(Retired {
        name: result_name,
        text: result_text,
    })
}

fn retained_sources(
    session: &mut Session,
    fixture: &Fixture,
    original: &Original,
    first: &Collected,
    retired: &Retired,
) -> Result<()> {
    switch(session, &fixture.hidden_name);
    assert_eq!(text(session), original.hidden_text);
    assert_eq!(session.buffer()?.stamp(), original.hidden_stamp);
    assert!(session.buffer()?.is_modified());
    switch(session, &fixture.orphan_name);
    assert_eq!(text(session), original.orphan_text);
    assert_eq!(session.buffer()?.stamp(), original.orphan_stamp);
    switch(session, &fixture.visible_name);
    assert_eq!(text(session), fixture.files.visible_disk);
    assert_eq!(session.buffer()?.stamp(), original.visible_stamp);
    switch(session, &retired.name);
    assert_eq!(text(session), retired.text);
    session.execute(Command::History(History::Undo));
    assert_eq!(
        text(session),
        first.text,
        "native undo remains available after retirement"
    );
    assert_eq!(
        fs::read_to_string(&fixture.files.visible_path)?,
        fixture.files.visible_disk
    );
    assert_eq!(
        fs::read_to_string(&fixture.files.hidden_path)?,
        fixture.files.hidden_disk
    );
    Ok(())
}
