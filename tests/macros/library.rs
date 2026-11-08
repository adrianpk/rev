use super::*;
use rev::command::MacroName;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

fn roots(fixture: &fixture::Fixture) -> rev::config::Roots {
    rev::config::Roots {
        config: fixture.0.join("config"),
        data: fixture.0.join("data"),
        state: fixture.0.join("state"),
        cache: fixture.0.join("cache"),
    }
}
fn name(session: &mut Session, value: &str) {
    feed(
        session,
        [
            control('x'),
            control('k'),
            character('n'),
            Event::Paste(value.into()),
            key(KeyCode::Enter),
        ],
    );
}
fn select(session: &mut Session, command: MacroCommand, value: &str, prefix: bool) {
    if prefix {
        session.handle(control('u'));
        feed(
            session,
            [
                meta('x'),
                Event::Paste(command.name().into()),
                key(KeyCode::Enter),
            ],
        );
    } else {
        session.execute(Command::Macro(command));
    }
    feed(session, [Event::Paste(value.into()), key(KeyCode::Enter)]);
}
fn save(session: &mut Session, name: &str, prefix: bool) {
    select(session, MacroCommand::Save, name, prefix);
    session.handle(character('y'));
}
fn call(session: &mut Session, name: &str) {
    session.execute(Command::MacroNamed(MacroName::new(name).unwrap()));
    settle(session);
}
fn library(roots: &rev::config::Roots) -> std::path::PathBuf {
    roots.state.join("macros/library.toml")
}
fn read(roots: &rev::config::Roots) -> toml::Value {
    toml::from_str(&fs::read_to_string(library(roots)).unwrap()).unwrap()
}
fn records(value: &toml::Value) -> &Vec<toml::Value> {
    value["macros"].as_array().unwrap()
}
fn named(session: &mut Session, value: &str, events: impl IntoIterator<Item = Event>) {
    record(session, events);
    name(session, value);
}

// Construction/reload and ordinary naming/replay/export never create state.
// An explicit admitted save creates only the fixed private macro concern.
#[test]
fn explicit_only() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut session = editor("", "").with_macro_library(&roots);
    assert!(!roots.state.exists());
    named(&mut session, "alpha", [character('A')]);
    call(&mut session, "alpha");
    session.execute(Command::Macro(MacroCommand::Load));
    select(&mut session, MacroCommand::Export, "alpha", false);
    assert!(!roots.state.exists());
    save(&mut session, "alpha", false);
    assert!(library(&roots).is_file(), "{}", session.echo());
    assert_eq!(
        fs::metadata(roots.state.join("macros"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    for file in ["library.toml", "library.toml.lock"] {
        assert_eq!(
            fs::metadata(roots.state.join("macros").join(file))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert_eq!(fs::read_dir(&roots.state).unwrap().count(), 1);
    assert_eq!(fs::read_dir(roots.state.join("macros")).unwrap().count(), 2);
}

// Reopening in either profile admits inert definitions and counters. Replay
// still enters through native editing and owns one ordinary undo boundary.
#[test]
fn restart() {
    for modal in [false, true] {
        let fixture = fixture::Fixture::new();
        let roots = roots(&fixture);
        let config = if modal {
            "[editor]\ninteraction_profile='modal'\n"
        } else {
            ""
        };
        let mut first = editor(config, "").with_macro_library(&roots);
        named(
            &mut first,
            "stamp",
            if modal {
                vec![character('i'), character('λ'), key(KeyCode::Esc)]
            } else {
                vec![Event::Paste("λ\n".into())]
            },
        );
        save(&mut first, "stamp", false);
        let bytes = fs::read(library(&roots)).unwrap();
        drop(first);
        let mut second = editor(config, "").with_macro_library(&roots);
        assert_eq!(text(&second), "");
        assert_eq!(second.macro_status(), Status::Idle);
        assert_eq!(fs::read(library(&roots)).unwrap(), bytes);
        call(&mut second, "stamp");
        assert_eq!(text(&second), if modal { "λ" } else { "λ\n" });
        second.execute(Command::History(History::Undo));
        assert_eq!(text(&second), "");
    }
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "").with_macro_library(&roots);
    named(&mut first, "counter", [key(KeyCode::F(3))]);
    call(&mut first, "counter");
    save(&mut first, "counter", false);
    let mut second = editor("", "").with_macro_library(&roots);
    call(&mut second, "counter");
    assert_eq!(text(&second), "2");
}

// Bindings are explicit optional saved data. Unprefixed replacement announces
// removal, and startup leaves an occupied binding inactive without losing name.
#[test]
fn bindings() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "").with_macro_library(&roots);
    named(&mut first, "alpha", [character('A')]);
    feed(
        &mut first,
        [
            control('x'),
            control('k'),
            character('b'),
            key(KeyCode::F(9)),
        ],
    );
    save(&mut first, "alpha", true);
    assert_eq!(records(&read(&roots))[0]["binding"].as_str(), Some("F9"));
    let mut second = editor("", "").with_macro_library(&roots);
    second.handle(key(KeyCode::F(9)));
    settle(&mut second);
    assert_eq!(text(&second), "A");
    let mut occupied = editor("[keys.editor]\nF9='forward-char'\n", "").with_macro_library(&roots);
    assert!(occupied.echo().contains("inactive"), "{}", occupied.echo());
    call(&mut occupied, "alpha");
    assert_eq!(text(&occupied), "A");
    select(&mut first, MacroCommand::Save, "alpha", false);
    assert!(
        first
            .echo()
            .contains("without its previously saved binding"),
        "{}",
        first.echo()
    );
    first.handle(character('y'));
    assert!(records(&read(&roots))[0].get("binding").is_none());
    let mut third = editor("", "").with_macro_library(&roots);
    third.handle(key(KeyCode::F(9)));
    settle(&mut third);
    assert_eq!(text(&third), "");
    call(&mut third, "alpha");
    assert_eq!(text(&third), "A");
}

// Export is a valid standalone single-record document inserted through native
// editing. It creates no library entry and one undo restores the source.
#[test]
fn inert_export() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut session = editor("", "").with_macro_library(&roots);
    named(&mut session, "alpha", [character('A')]);
    session.replace(0..1, "").unwrap();
    select(&mut session, MacroCommand::Export, "alpha", false);
    let value: toml::Value = toml::from_str(&text(&session)).unwrap();
    assert_eq!(value["schema_version"].as_integer(), Some(1));
    assert_eq!(records(&value).len(), 1);
    assert_eq!(records(&value)[0]["name"].as_str(), Some("alpha"));
    assert!(!roots.state.exists());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
}

// Each save rereads the aggregate under its own lock. Independent additions
// survive, while stale same-record and same-name writes preserve disk/session.
#[test]
fn concurrent_records() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "").with_macro_library(&roots);
    let mut second = editor("", "").with_macro_library(&roots);
    named(&mut first, "alpha", [character('A')]);
    named(&mut second, "bravo", [character('B')]);
    save(&mut first, "alpha", false);
    save(&mut second, "bravo", false);
    assert_eq!(records(&read(&roots)).len(), 2);
    second.execute(Command::Macro(MacroCommand::Load));
    call(&mut first, "alpha");
    save(&mut first, "alpha", false);
    let bytes = fs::read(library(&roots)).unwrap();
    call(&mut second, "alpha");
    save(&mut second, "alpha", false);
    assert!(second.echo().contains("conflict"), "{}", second.echo());
    assert_eq!(fs::read(library(&roots)).unwrap(), bytes);
    named(&mut second, "charlie", [character('C')]);
    save(&mut second, "charlie", false);
    named(&mut first, "charlie", [character('D')]);
    save(&mut first, "charlie", false);
    assert!(first.echo().contains("conflict"));
    assert_eq!(records(&read(&roots)).len(), 3);
    named(&mut second, "alpha", [character('X')]);
    second.handle(character('y'));
    second.execute(Command::Macro(MacroCommand::Load));
    assert!(second.echo().contains("preserved"));
    let before = text(&second);
    call(&mut second, "alpha");
    assert_eq!(text(&second), before + "X");
}

// Reload cannot discard an unsaved replacement. Explicit deletion is checked,
// cancellable and leaves independent ring/session snapshots and siblings intact.
#[test]
fn reload_and_delete() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "").with_macro_library(&roots);
    named(&mut first, "alpha", [character('A')]);
    save(&mut first, "alpha", false);
    fixture.write("state/command-history.toml", "sibling history");
    fixture.write("state/contexts.toml", "sibling contexts");
    fixture.write("data/bookmarks.toml", "sibling bookmarks");
    let mut second = editor("", "").with_macro_library(&roots);
    named(&mut second, "alpha", [character('B')]);
    second.handle(character('y'));
    second.execute(Command::Macro(MacroCommand::Load));
    assert!(second.echo().contains("preserved"));
    let before = text(&second);
    call(&mut second, "alpha");
    assert_eq!(text(&second), before + "B");
    select(&mut first, MacroCommand::DeleteSaved, "alpha", false);
    first.handle(character('n'));
    assert_eq!(records(&read(&roots)).len(), 1);
    call(&mut second, "alpha");
    save(&mut second, "alpha", false);
    select(&mut first, MacroCommand::DeleteSaved, "alpha", false);
    first.handle(character('y'));
    assert!(first.echo().contains("conflict"));
    select(&mut second, MacroCommand::DeleteSaved, "alpha", false);
    second.handle(character('y'));
    assert!(records(&read(&roots)).is_empty());
    let before = text(&second);
    call(&mut second, "alpha");
    assert_eq!(text(&second), before + "B");
    assert_eq!(
        fs::read_to_string(roots.state.join("command-history.toml")).unwrap(),
        "sibling history"
    );
    assert_eq!(
        fs::read_to_string(roots.state.join("contexts.toml")).unwrap(),
        "sibling contexts"
    );
    assert_eq!(
        fs::read_to_string(roots.data.join("bookmarks.toml")).unwrap(),
        "sibling bookmarks"
    );
}

// Invalid stores and substituted filesystem entries remain untouched. Startup
// and reload do not substitute an empty library or disturb ordinary editing.
#[test]
fn rejected_state() {
    for variant in [
        "invalid",
        "unsupported",
        "oversized",
        "library-link",
        "lock-link",
        "directory-link",
        "nonregular",
    ] {
        let fixture = fixture::Fixture::new();
        let roots = roots(&fixture);
        let mut source = editor("", "").with_macro_library(&roots);
        named(&mut source, "alpha", [character('A')]);
        save(&mut source, "alpha", false);
        let path = library(&roots);
        match variant {
            "invalid" => fs::write(&path, "malformed").unwrap(),
            "unsupported" => {
                let t = fs::read_to_string(&path)
                    .unwrap()
                    .replace("schema_version = 1", "schema_version = 2");
                fs::write(&path, t).unwrap();
            }
            "oversized" => fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(8 * 1024 * 1024 + 1)
                .unwrap(),
            "library-link" => {
                fs::rename(&path, fixture.0.join("valid")).unwrap();
                symlink(fixture.0.join("valid"), &path).unwrap();
            }
            "lock-link" => {
                let p = roots.state.join("macros/library.toml.lock");
                fs::remove_file(&p).unwrap();
                symlink(fixture.0.join("unrelated"), p).unwrap();
            }
            "directory-link" => {
                fs::rename(
                    roots.state.join("macros"),
                    fixture.0.join("valid-directory"),
                )
                .unwrap();
                symlink(
                    fixture.0.join("valid-directory"),
                    roots.state.join("macros"),
                )
                .unwrap();
            }
            "nonregular" => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            _ => unreachable!(),
        }
        let mut session = editor("", "").with_macro_library(&roots);
        assert!(
            session.echo().contains("library"),
            "{variant}: {}",
            session.echo()
        );
        session.handle(character('Z'));
        assert_eq!(text(&session), "Z");
        session.execute(Command::Macro(MacroCommand::Load));
        assert!(session.echo().contains("library"));
        let before = fs::symlink_metadata(&path).unwrap();
        named(&mut session, "local", [character('B')]);
        save(&mut session, "local", false);
        assert!(
            session.echo().contains("library"),
            "{variant}: {}",
            session.echo()
        );
        let after = fs::symlink_metadata(&path).unwrap();
        assert_eq!(
            (after.ino(), after.len()),
            (before.ino(), before.len()),
            "{variant}"
        );
        assert!(!fixture.0.join("unrelated").exists());
    }
}

// A held concern lock refuses immediately and releases after each failed save;
// no other store's lock or process lifetime participates in macro publication.
#[test]
fn busy_lock() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut session = editor("", "").with_macro_library(&roots);
    named(&mut session, "alpha", [character('A')]);
    save(&mut session, "alpha", false);
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(roots.state.join("macros/library.toml.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    let bytes = fs::read(library(&roots)).unwrap();
    let start = Instant::now();
    save(&mut session, "alpha", false);
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(session.echo().contains("busy"));
    assert_eq!(fs::read(library(&roots)).unwrap(), bytes);
    lock.unlock().unwrap();
    save(&mut session, "alpha", false);
    assert_eq!(records(&read(&roots))[0]["revision"].as_integer(), Some(2));
}

// A structurally valid record can lose its name to a compiled/dynamic owner.
// Quarantine is independent, and explicit deletion can still target that record.
#[test]
fn namespace_admission() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "").with_macro_library(&roots);
    named(&mut first, "alpha", [character('A')]);
    save(&mut first, "alpha", false);
    named(&mut first, "bravo", [character('B')]);
    save(&mut first, "bravo", false);
    let path = library(&roots);
    let source = fs::read_to_string(&path)
        .unwrap()
        .replace("name = \"alpha\"", "name = \"forward-char\"");
    fs::write(&path, source).unwrap();
    let mut second = editor("", "").with_macro_library(&roots);
    assert!(
        second.echo().contains("command name collision"),
        "{}",
        second.echo()
    );
    call(&mut second, "bravo");
    assert_eq!(text(&second), "B");
    select(
        &mut second,
        MacroCommand::DeleteSaved,
        "forward-char",
        false,
    );
    second.handle(character('y'));
    assert_eq!(records(&read(&roots)).len(), 1);
    assert_eq!(records(&read(&roots))[0]["name"].as_str(), Some("bravo"));
}

// Persisted searches retain literal patterns, direction and repeat/accept
// behavior. After restart, native failure still stops before the next edit.
#[test]
fn search_restart() {
    let fixture = fixture::Fixture::new();
    let roots = roots(&fixture);
    let mut first = editor("", "hit hit\nhit hit\n").with_macro_library(&roots);
    record(
        &mut first,
        [
            control('s'),
            Event::Paste("hit".into()),
            control('s'),
            key(KeyCode::Enter),
            character('X'),
        ],
    );
    name(&mut first, "find-hit");
    save(&mut first, "find-hit", false);
    assert_eq!(
        records(&read(&roots))[0]["steps"][0]["kind"].as_str(),
        Some("search")
    );
    let mut second = editor("", "hit hit\nhit hit\n").with_macro_library(&roots);
    call(&mut second, "find-hit");
    assert_eq!(text(&second), "hit hitX\nhit hit\n");
    second.execute(Command::Motion(rev::command::Motion::BufferEnd));
    let before = text(&second);
    call(&mut second, "find-hit");
    assert_eq!(text(&second), before);
    assert_eq!(
        second.macro_stop().unwrap().reason,
        StopReason::NativeFailure
    );
}
