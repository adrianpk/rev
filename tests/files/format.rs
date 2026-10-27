use rev::config::{Snapshot, prepare};

use super::*;

// Format defaults apply once to scratch, explicitly created and absent visiting
// files. Selection/serialization cannot create the destination or its parents.
#[test]
fn new_formats() {
    for (newline, separator, bom) in [
        ("lf", "\n", false),
        ("lf", "\n", true),
        ("crlf", "\r\n", false),
        ("crlf", "\r\n", true),
    ] {
        let fixture = Fixture::new();
        let config = format!("files.newline = '{newline}'\nfiles.bom = {bom}");
        let root = fixture.file("config.toml", config.as_bytes());
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let scratch = buffers.initial();
        let created = buffers.create("").unwrap();
        let path = fixture.0.join("absent/new.txt");
        let visited = buffers.visit(&path).unwrap();
        let expected = if bom { b"\xef\xbb\xbf".as_slice() } else { b"" };
        for id in [scratch, created, visited] {
            let buffer = buffers.get_mut(id).unwrap();
            assert_eq!(buffer.newline(), separator);
            assert_eq!(buffer.has_bom(), bom);
            assert_eq!(bytes(buffer), expected);
            assert!(!buffer.is_modified());
            assert!(buffer.history().is_empty());
            let before = (buffer.stamp(), buffer.state());
            buffer.apply_settings(&Snapshot::default());
            assert_eq!(buffer.newline(), separator);
            assert_eq!(bytes(buffer), expected);
            assert_eq!((buffer.stamp(), buffer.state()), before);
        }
        assert!(!fixture.0.join("absent").exists());
        assert_eq!(buffers.visit(&path).unwrap(), visited);
        assert!(!path.exists());
    }
}

// Opposing defaults cannot rewrite existing or raw-loaded files, including
// empty input, mixed separators, a recognized BOM and missing final newline.
#[test]
fn loaded_formats() {
    let cases = [
        ("empty", "", false, "\n"),
        ("LF", "α\nlast", false, "\n"),
        ("CRLF", "α\r\nlast", false, "\r\n"),
        ("mixed", "a\r\nb\nc\r\nlast", false, "\r\n"),
        ("tie", "a\r\nb\nlast", false, "\n"),
        ("BOM", "\u{feff}α\r\nlast", true, "\r\n"),
        ("BOM only", "\u{feff}", true, "\n"),
        ("no separator", "last", false, "\n"),
    ];
    for (name, source, bom, newline) in cases {
        let fixture = Fixture::new();
        let config = format!(
            "files.newline = '{}'\nfiles.bom = {}",
            if newline == "\n" { "crlf" } else { "lf" },
            !bom
        );
        let root = fixture.file("config.toml", config.as_bytes());
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let path = fixture.file("existing.txt", source.as_bytes());
        let visited = buffers.visit(&path).unwrap();
        let raw = buffers
            .load(Loaded::read(source.as_bytes()).unwrap())
            .unwrap();
        for id in [visited, raw] {
            let buffer = buffers.get(id).unwrap();
            assert_eq!(bytes(buffer), source.as_bytes(), "{name}");
            assert_eq!(buffer.newline(), newline, "{name}");
            assert_eq!(buffer.has_bom(), bom, "{name}");
            assert!(!buffer.is_modified(), "{name}");
        }
        assert!(buffers.save(visited).unwrap().is_complete(), "{name}");
        assert_eq!(fs::read(path).unwrap(), source.as_bytes(), "{name}");
    }
}

// Return uses the creation default before any separators exist. Paste retains
// literal tabs, explicit save publishes BOM once, and undo/redo keep saved truth.
#[test]
fn new_file_journey() {
    use rev::command::History;

    for (newline, separator, bom) in [("lf", "\n", false), ("crlf", "\r\n", true)] {
        let fixture = Fixture::new();
        let config = format!("files.newline = '{newline}'\nfiles.bom = {bom}");
        let root = fixture.file("config.toml", config.as_bytes());
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let path = fixture.0.join("new.txt");
        let id = buffers.visit(&path).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 40,
                height: 8,
            },
        )
        .unwrap();
        session.handle(press(KeyCode::Char('a')));
        session.handle(press(KeyCode::Enter));
        session.handle(Event::Paste("B\t界".into()));
        assert_eq!(
            text(session.buffer().unwrap()),
            format!("a{separator}B\t界")
        );
        assert_eq!(session.buffer().unwrap().history().len(), 3);
        assert!(!path.exists());
        let expected = format!("{}a{separator}B\t界", if bom { "\u{feff}" } else { "" });
        session.execute(Command::SaveBuffer);
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(session.buffer().unwrap()), format!("a{separator}"));
        assert!(session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(bytes(session.buffer().unwrap()), expected.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// An unrelated word edit must retain every untouched byte under opposing
// defaults. Saving and history preserve the existing BOM and absent final LF.
#[test]
fn existing_word_edit() {
    use rev::command::History;

    for source in ["a\r\nb", "\u{feff}a\r\nb\nlast", "a\nb", "a\rb\nlast"] {
        let fixture = Fixture::new();
        let config = format!(
            "files.newline = '{}'\nfiles.bom = {}",
            if source.contains("\r\n") {
                "lf"
            } else {
                "crlf"
            },
            !source.starts_with('\u{feff}')
        );
        let root = fixture.file("config.toml", config.as_bytes());
        let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
        let path = fixture.file("existing.txt", source.as_bytes());
        let id = buffers.visit(&path).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 40,
                height: 8,
            },
        )
        .unwrap();
        session.replace(0..1, "Ω").unwrap();
        let expected = source.replacen('a', "Ω", 1);
        session.execute(Command::SaveBuffer);
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Undo));
        assert_eq!(bytes(session.buffer().unwrap()), source.as_bytes());
        assert!(session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(bytes(session.buffer().unwrap()), expected.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
    }
}

// Closing the last owner creates another new buffer with the registry's format
// defaults. Replacing editing preferences cannot convert that initial metadata.
#[test]
fn replacement_format() {
    let fixture = Fixture::new();
    let root = fixture.file("config.toml", b"files.newline = 'crlf'\nfiles.bom = true");
    let mut buffers = Buffers::with_settings(prepare(&root).snapshot).unwrap();
    let kill = buffers.prepare_kill(buffers.initial()).unwrap();
    let id = buffers.kill(kill).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let before = (buffer.stamp(), buffer.state());
    buffer.apply_settings(&Snapshot::default());
    assert_eq!(buffer.newline(), "\r\n");
    assert_eq!(bytes(buffer), b"\xef\xbb\xbf");
    assert_eq!((buffer.stamp(), buffer.state()), before);
    assert!(!buffer.is_modified());
}
