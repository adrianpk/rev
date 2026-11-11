use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::{BufferCommand, Command, PluginCommand},
    config,
    input::{Input, Outcome},
    session::Session,
    view::{Size, View},
};

use super::fixture::Fixture;

fn manifest(id: &str) -> String {
    format!(
        r#"[plugin]
id = "{id}"
version = "0.1.0"
api = "rev.plugins/0.1"
[process]
program = "./probe"
args = []
[[commands]]
name = "collect"
description = "Collect task lines."
prefix = "none"
recording = "reject"
[activation]
filetypes = []
events = []
"#
    )
}
fn prepare(fixture: &Fixture, paths: &[&str], extra: &str) -> config::Startup {
    let paths = serde_json::to_string(paths).unwrap();
    config::prepare(&fixture.write(
        "config.toml",
        &format!("[plugins]\nenabled=true\nmanifests={paths}\n{extra}"),
    ))
}
fn candidates(snapshot: &config::Snapshot, query: &str) -> Vec<String> {
    let mut input = Input::with_settings(snapshot);
    input.commands().unwrap();
    input.handle(Event::Paste(query.into()));
    input
        .palette()
        .unwrap()
        .candidates()
        .iter()
        .map(|c| c.name().to_owned())
        .collect()
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

// A real executable would create the marker. Passive configuration, both
// native selectors, binding declarations and inventory must never run it.
#[test]
fn dormant_catalog() {
    for profile in ["non-modal", "modal"] {
        let f = Fixture::new();
        let marker = f.0.join("started");
        let program = f.write("probe", "#!/bin/sh\nprintf active > \"$1\"\n");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        let declaration = manifest("field.notes").replace(
            "args = []",
            &format!("args = [{}]", serde_json::to_string(&marker).unwrap()),
        ) + "\n[[bindings]]\ncommand='field.notes.collect'\nkeys='C-c n'\ncontext='editor'\nscope='global'\n";
        f.write("package.toml", &declaration);
        let path = f.write("profile.toml", &format!("[editor]\ninteraction_profile='{profile}'\n[plugins]\nenabled=true\nmanifests=['package.toml']\nstartup=[]"));
        let prepared = config::prepare(&path);
        assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
        let mut input = Input::with_settings(&prepared.snapshot);
        input.commands().unwrap();
        input.handle(Event::Paste("field.notes.collect".into()));
        let palette = input.palette().unwrap();
        let command = palette.candidates()[0];
        assert_eq!(command.name(), "field.notes.collect");
        assert_eq!(palette.description(command), Some("Collect task lines."));
        input.cancel();
        let binding = [
            Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            key(KeyCode::Char('n')),
        ];
        for event in binding {
            assert!(!matches!(
                input.handle(event),
                Outcome::Command(Command::PluginDefined(_))
            ));
        }
        let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
        let source = buffers.create("source 🦀\n").unwrap();
        let source_name = buffers.get(source).unwrap().name().to_owned();
        let view = View::new(buffers.get(source).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 120,
                height: 18,
            },
        )
        .unwrap();
        let open = if profile == "modal" {
            key(KeyCode::Char(':'))
        } else {
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
        };
        session.handle(open);
        session.handle(Event::Paste("field.notes.collect".into()));
        session.execute(Command::KeyboardQuit);
        assert_eq!(text(&session), "source 🦀\n");
        session.execute(Command::Plugin(PluginCommand::List));
        assert!(session.buffer().unwrap().is_readonly());
        assert!(text(&session).contains("field.notes 0.1.0: installed, inactive"));
        assert!(text(&session).contains("Effective"));
        session.execute(Command::Buffer(BufferCommand::Switch));
        session.handle(Event::Paste(source_name));
        session.handle(key(KeyCode::Enter));
        assert_eq!(session.buffer().unwrap().id(), source);
        assert_eq!(text(&session), "source 🦀\n");
        assert!(
            !marker.exists(),
            "{profile}: passive metadata started a program"
        );
    }
}

// Disabled startup does not require or read manifest files, and one snapshot's
// admitted dynamic commands cannot leak into a different editor configuration.
#[test]
fn disabled_isolation() {
    let f = Fixture::new();
    f.write("package.toml", &manifest("field.notes"));
    let enabled = prepare(&f, &["package.toml"], "");
    assert!(enabled.diagnostic.is_none());
    assert_eq!(
        candidates(&enabled.snapshot, "field.notes.collect"),
        ["field.notes.collect"]
    );
    let disabled = config::prepare(&f.write(
        "disabled.toml",
        "[plugins]\nenabled=false\nmanifests=['missing.toml']\nstartup=['field.notes']",
    ));
    assert!(disabled.diagnostic.is_none());
    assert!(disabled.snapshot.plugins().packages().next().is_none());
    assert!(candidates(&disabled.snapshot, "field.notes").is_empty());
    assert!(candidates(&config::Snapshot::default(), "field.notes").is_empty());
}

// Array precedence retains the assigning include's origin. Program and argv
// identity come from the manifest, never the process launch directory.
#[test]
fn source_origins() {
    let f = Fixture::new();
    let program = f.write("packages/probe", "#!/bin/sh\nexit 0\n");
    f.write(
        "packages/notes.toml",
        &manifest("field.notes")
            .replace("args = []", "args = ['one argument', 'literal;$(value)']"),
    );
    let included = f.write(
        "layers/packages.toml",
        "[plugins]\nmanifests=['../packages/notes.toml']",
    );
    let path = f.write(
        "root.toml",
        "include=['layers/packages.toml']\n[plugins]\nenabled=true",
    );
    let prepared = config::prepare(&path);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    assert_eq!(
        prepared.snapshot.origin(&["plugins", "manifests"]),
        Some(included.as_path())
    );
    let package = prepared.snapshot.plugins().packages().next().unwrap();
    assert!(package.source.is_absolute() && package.program.is_absolute());
    assert_eq!(package.program.canonicalize().unwrap(), program);
    assert_eq!(
        package
            .manifest
            .process
            .args
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>(),
        ["one argument", "literal;$(value)"]
    );
    let clear = f.write(
        "clear.toml",
        "include=['layers/packages.toml']\n[plugins]\nenabled=true\nmanifests=[]",
    );
    assert!(
        config::prepare(&clear)
            .snapshot
            .plugins()
            .packages()
            .next()
            .is_none()
    );
}

// A late invalid declaration rejects the whole catalog and the ordinary
// configuration candidate, without publishing the earlier valid namespace.
#[test]
fn atomic_catalog() {
    let f = Fixture::new();
    f.write("first.toml", &manifest("field.notes"));
    for (name, invalid) in [
        ("duplicate", manifest("field.notes")),
        ("unknown", manifest("other.notes") + "extra=true\n"),
        (
            "api",
            manifest("other.notes").replace("rev.plugins/0.1", "rev.plugins/9.0"),
        ),
    ] {
        let bad = f.write("second.toml", &invalid);
        let prepared = prepare(&f, &["first.toml", "second.toml"], "");
        assert!(prepared.diagnostic.is_some(), "{name}");
        assert_eq!(prepared.diagnostic.unwrap().source, bad);
        assert!(!prepared.snapshot.settings().plugins.enabled);
        assert!(prepared.snapshot.plugins().packages().next().is_none());
        assert!(candidates(&prepared.snapshot, "field.notes").is_empty());
    }
    f.write("second.toml", &manifest("other.notes"));
    let valid = prepare(&f, &["first.toml", "second.toml"], "");
    assert!(valid.diagnostic.is_none());
    assert_eq!(
        candidates(&valid.snapshot, "collect"),
        ["field.notes.collect", "other.notes.collect"]
    );
}

// These valid TOML variations exercise semantic and unknown-field admission,
// including namespaces, command contracts, argv and activation metadata.
#[test]
fn manifest_rejections() {
    let f = Fixture::new();
    let base = manifest("field.notes");
    let binding = "\n[[bindings]]\ncommand='other.notes.collect'\nkeys='C-c n'\ncontext='editor'\nscope='global'\n";
    let cases = [
        ("reserved", base.replace("field.notes", "rev.notes")),
        (
            "namespace",
            base.replace("field.notes", "field.notes.extra"),
        ),
        (
            "empty description",
            base.replace("Collect task lines.", "   "),
        ),
        (
            "control description",
            base.replace("Collect task lines.", "Line\\nBreak"),
        ),
        (
            "version",
            base.replace("version = \"0.1.0\"", "version = \"\""),
        ),
        (
            "prefix",
            base.replace("prefix = \"none\"", "prefix = \"repeat\""),
        ),
        ("unknown root", base.clone() + "\n[other]\nvalue=true\n"),
        (
            "unknown command",
            base.replace(
                "recording = \"reject\"",
                "recording = \"reject\"\nextra = true",
            ),
        ),
        (
            "path",
            base.replace("./probe", "https://example.invalid/probe"),
        ),
        (
            "argv NUL",
            base.replace("args = []", "args = [\"\\u0000\"]"),
        ),
        (
            "argv total",
            base.replace(
                "args = []",
                &format!(
                    "args = [{}]",
                    vec![format!("'{}'", "x".repeat(4096)); 17].join(",")
                ),
            ),
        ),
        ("foreign binding", base.clone() + binding),
        (
            "duplicate command",
            base.clone()
                + "\n[[commands]]\nname='collect'\ndescription='Duplicate.'\nprefix='none'\nrecording='reject'\n",
        ),
        (
            "bad filetype",
            base.replace("filetypes = []", "filetypes = ['../rust']"),
        ),
        (
            "resnapshot trigger",
            base.replace("events = []", "events = ['resnapshot']"),
        ),
    ];
    for (name, source) in cases {
        f.write("package.toml", &source);
        let prepared = prepare(&f, &["package.toml"], "");
        assert!(prepared.diagnostic.is_some(), "{name}");
        assert!(
            prepared.snapshot.plugins().packages().next().is_none(),
            "{name}"
        );
    }
}

// Persistent buffer selectors must be explicit and constrained even when the
// catalog is disabled; they never turn into global bindings accidentally.
#[test]
fn selectors() {
    let f = Fixture::new();
    for (name, scope, extra, valid) in [
        ("global", "global", "", true),
        ("filetype", "buffer", "filetypes=['rust']", true),
        ("owner", "buffer", "buffer_owner='field.notes'", true),
        (
            "intersection",
            "buffer",
            "filetypes=['rust']\nbuffer_owner='field.notes'",
            true,
        ),
        ("missing", "buffer", "", false),
        ("global selector", "global", "filetypes=['rust']", false),
        (
            "unknown selector",
            "buffer",
            "buffer_owners=['field.notes']",
            false,
        ),
    ] {
        let source = format!(
            "[plugins]\nenabled=false\n[[plugins.keybindings]]\ncommand='field.notes.collect'\nkeys='C-c n'\ncontext='editor'\nscope='{scope}'\n{extra}"
        );
        let prepared = config::prepare(&f.write("selectors.toml", &source));
        assert_eq!(
            prepared.diagnostic.is_none(),
            valid,
            "{name}: {:?}",
            prepared.diagnostic
        );
    }
    f.write("package.toml", &manifest("field.notes"));
    assert!(
        prepare(&f, &["package.toml"], "startup=['other.notes']")
            .diagnostic
            .is_some()
    );
    let unknown = "[[plugins.keybindings]]\ncommand='field.notes.absent'\nkeys='C-c n'\ncontext='editor'\nscope='global'";
    assert!(prepare(&f, &["package.toml"], unknown).diagnostic.is_some());
}

// Aggregate package admission and file reads obey their exact finite ceilings;
// a growing or oversized manifest cannot allocate an unbounded input buffer.
#[test]
fn admission_bounds() {
    let f = Fixture::new();
    let base = manifest("field.notes");
    let mut exact = base;
    exact.push('#');
    exact.push_str(&"x".repeat(rev::plugins::declarations::MANIFEST_BYTES - exact.len()));
    f.write("package.toml", &exact);
    assert!(prepare(&f, &["package.toml"], "").diagnostic.is_none());
    exact.push('x');
    f.write("package.toml", &exact);
    assert!(
        prepare(&f, &["package.toml"], "")
            .diagnostic
            .unwrap()
            .message
            .contains("256 KiB")
    );
    fs::write(f.0.join("package.toml"), [0xff]).unwrap();
    assert!(
        prepare(&f, &["package.toml"], "")
            .diagnostic
            .unwrap()
            .message
            .contains("UTF-8")
    );
    let mut paths = Vec::new();
    for index in 0..9 {
        let mut source = manifest(&format!("field.package-{index}"));
        for command in 1..64 {
            source.push_str(&format!("\n[[commands]]\nname='command-{command}'\ndescription='Describe the command.'\nprefix='none'\nrecording='reject'\n"));
        }
        let name = format!("package-{index}.toml");
        f.write(&name, &source);
        paths.push(name);
    }
    let names: Vec<_> = paths.iter().map(String::as_str).collect();
    let exact = prepare(&f, &names[..8], "");
    assert!(exact.diagnostic.is_none(), "{:?}", exact.diagnostic);
    assert_eq!(
        exact
            .snapshot
            .plugins()
            .packages()
            .map(|p| p.manifest.commands.len())
            .sum::<usize>(),
        512
    );
    assert!(
        prepare(&f, &names, "")
            .diagnostic
            .unwrap()
            .message
            .contains("512")
    );
    let path = f.write(
        "too-many.toml",
        &format!(
            "[plugins]\nmanifests={}",
            serde_json::to_string(&vec!["absent"; 65]).unwrap()
        ),
    );
    assert!(config::prepare(Path::new(&path)).diagnostic.is_some());
}

// Qualified discovery must not suspend existing native command persistence.
// Keep the schema-1 disk grammar unchanged even for the longest public ID.
#[test]
fn usage_persistence() {
    use rev::command::usage::{History, Store};
    let f = Fixture::new();
    let package = format!("{}.{}", "a".repeat(32), "b".repeat(32));
    let command = "c".repeat(64);
    let qualified = format!("{package}.{command}");
    assert_eq!(qualified.len(), 130);
    f.write(
        "package.toml",
        &manifest(&package).replace("name = \"collect\"", &format!("name = \"{command}\"")),
    );
    let prepared = prepare(&f, &["package.toml"], "");
    assert!(prepared.diagnostic.is_none());
    let path = f.write("usage.toml", "version=1\ncommands=['backward-char']\n");
    let mut buffers = Buffers::with_settings(prepared.snapshot).unwrap();
    let id = buffers.create("source").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 200,
            height: 18,
        },
    )
    .unwrap()
    .with_command_usage(History::open(&path));
    for name in [&qualified, "forward-char"] {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
        session.handle(Event::Paste(name.to_owned()));
        session.handle(key(KeyCode::Enter));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
        let frame = session.frame().unwrap();
        let first = frame
            .rows()
            .iter()
            .find(|r| r.band() == rev::display::Band::Completion)
            .unwrap();
        assert_eq!(first.text().split_whitespace().next(), Some(name));
        session.execute(Command::KeyboardQuit);
        session.flush_command_usage().unwrap();
    }
    assert_eq!(
        Store::open(&path).unwrap().1,
        ["backward-char", "forward-char"]
    );
    assert_eq!(text(&session), "source");
    session.shutdown_plugins().unwrap();
}
