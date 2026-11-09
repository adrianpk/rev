//! Native acceptance bridge for a separately built public-protocol client.
//! Go source and builds belong to rev/plugin-examples, never this workspace.
#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::{Buffers, edit::History},
    command::{BufferCommand, Command, PluginCommand},
    config,
    plugins::lifecycle::State,
    session::Session,
    view::{Size, View},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKAGE: &str = "field.collector";
const COMMAND: &str = "field.collector.collect";

struct Owned(Session);
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
    let visible_path = root.join("visible.txt");
    let hidden_path = root.join("hidden.txt");
    let visible_disk = "visible saved\n";
    // The chunk boundary separates e from its combining mark and astral scalar.
    let hidden_disk = format!("{}e\u{301}🦀\r\nhidden tail\n", "a".repeat(16382));
    fs::write(&visible_path, visible_disk)?;
    fs::write(&hidden_path, &hidden_disk)?;
    let mut buffers = Buffers::with_settings(prepared.snapshot)?;
    let visible = buffers.visit(&visible_path)?;
    let hidden = buffers.visit(&hidden_path)?;
    let orphan = buffers.create("orphan e\u{301}🦀\n")?;
    let visible_name = buffers.get(visible)?.name().to_owned();
    let hidden_name = buffers.get(hidden)?.name().to_owned();
    let orphan_name = buffers.get(orphan)?.name().to_owned();
    let view = View::new(buffers.get(hidden)?);
    let mut owned = Owned(Session::new(
        buffers,
        view,
        Size {
            width: 160,
            height: 30,
        },
    )?);
    let session = &mut owned.0;
    session.attach_plugin_state(&config::Roots {
        config: root.join("config"),
        data: root.join("data"),
        state: root.join("state"),
        cache: root.join("cache"),
    })?;
    session.insert('!');
    let hidden_text = text(session);
    assert_eq!(hidden_text, format!("!{hidden_disk}"));
    assert!(session.buffer()?.is_modified());
    let hidden_stamp = session.buffer()?.stamp();
    switch(session, &orphan_name);
    session.insert('~');
    assert!(session.buffer()?.is_modified());
    assert!(session.buffer()?.file().is_none());
    let orphan_text = text(session);
    assert_eq!(orphan_text, "~orphan e\u{301}🦀\n");
    let orphan_stamp = session.buffer()?.stamp();
    switch(session, &visible_name);
    let visible_stamp = session.buffer()?.stamp();

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
    let first = session.buffer()?.id();
    let first_name = session.buffer()?.name().to_owned();
    let first_text = text(session);
    assert!(first_text.starts_with("# Collected open text\n"));
    assert!(first_text.contains(visible_disk));
    assert!(first_text.contains(&hidden_text));
    assert!(first_text.contains(&orphan_text));
    assert!(session.buffer()?.is_modified());
    assert!(session.buffer()?.file().is_none());
    assert!(!session.buffer()?.is_transient());
    let generation = session.plugin_status(PACKAGE).unwrap().generation;

    switch(session, &visible_name);
    key(session, KeyCode::F(8));
    assert_eq!(session.buffer()?.id(), visible);
    assert_eq!(session.plugin_status(PACKAGE).unwrap().pending, 0);
    assert_eq!(session.buffer()?.stamp(), visible_stamp);
    switch(session, &first_name);
    key(session, KeyCode::F(8));
    wait(session, |s| {
        s.buffer().is_ok_and(|b| {
            b.id() != first && b.plugin_owner().is_some_and(|id| id.as_str() == PACKAGE)
        }) && s.plugin_status(PACKAGE).is_some_and(|p| p.pending == 0)
            && s.echo().contains("completed")
    })?;
    assert_eq!(
        session.plugin_status(PACKAGE).unwrap().generation,
        generation
    );
    assert_eq!(
        text(session),
        first_text,
        "previous results must not be collected again"
    );
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

    switch(session, &hidden_name);
    assert_eq!(text(session), hidden_text);
    assert_eq!(session.buffer()?.stamp(), hidden_stamp);
    assert!(session.buffer()?.is_modified());
    switch(session, &orphan_name);
    assert_eq!(text(session), orphan_text);
    assert_eq!(session.buffer()?.stamp(), orphan_stamp);
    switch(session, &visible_name);
    assert_eq!(text(session), visible_disk);
    assert_eq!(session.buffer()?.stamp(), visible_stamp);
    switch(session, &result_name);
    assert_eq!(text(session), result_text);
    session.execute(Command::History(History::Undo));
    assert_eq!(
        text(session),
        first_text,
        "native undo remains available after retirement"
    );
    assert_eq!(fs::read_to_string(&visible_path)?, visible_disk);
    assert_eq!(fs::read_to_string(&hidden_path)?, hidden_disk);
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
