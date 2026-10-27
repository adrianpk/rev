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
