#![forbid(unsafe_code)]

#[path = "files/format.rs"]
mod format;

#[path = "files/associations.rs"]
mod associations;

#[path = "files/buffers.rs"]
mod buffers;

#[path = "files/saving.rs"]
mod saving;

#[path = "files/decisions.rs"]
mod decisions;

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::edit::{History, Replacement};
use rev::buffer::{Buffer, Buffers, Stamp, State};
use rev::command::Command;
use rev::coordinates::{Byte, Coordinates, Scalar};
use rev::file::{self, Loaded};
use rev::session::{Action, Session};
use rev::view::{Size, View};

fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn bytes(buffer: &Buffer) -> Vec<u8> {
    let mut bytes = Vec::new();
    file::write(buffer, &mut bytes).unwrap();
    bytes
}

// Valid ingress installs one fresh clean owner, separates only a leading BOM,
// and serializes exactly the original UTF-8/separator/final-newline bytes.
#[test]
fn loaded_text() {
    let cases = [
        ("empty", "", "", false, "\n"),
        ("LF", "a\nb\n", "a\nb\n", false, "\n"),
        ("CRLF", "a\r\nb\r\n", "a\r\nb\r\n", false, "\r\n"),
        (
            "mixed",
            "a\r\nb\nc\r\nlast",
            "a\r\nb\nc\r\nlast",
            false,
            "\r\n",
        ),
        ("tie", "a\r\nb\n", "a\r\nb\n", false, "\n"),
        (
            "Unicode",
            "e\u{301}界👩\u{200d}💻",
            "e\u{301}界👩\u{200d}💻",
            false,
            "\n",
        ),
        ("BOM", "\u{feff}a\r\nb", "a\r\nb", true, "\r\n"),
        ("BOM only", "\u{feff}", "", true, "\n"),
        ("double BOM", "\u{feff}\u{feff}x", "\u{feff}x", true, "\n"),
        ("inner BOM", "a\u{feff}b", "a\u{feff}b", false, "\n"),
        ("visible controls", "a\rb\x1b", "a\rb\x1b", false, "\n"),
    ];
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    for (name, source, body, bom, newline) in cases {
        let loaded = Loaded::read(source.as_bytes()).unwrap();
        let id = buffers.load(loaded).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(text(buffer), body, "{name}");
        assert_eq!(buffer.has_bom(), bom, "{name}");
        assert_eq!(buffer.newline(), newline, "{name}");
        assert_eq!(buffer.len_bytes(), body.len(), "{name}");
        assert_eq!(buffer.len_chars(), body.chars().count(), "{name}");
        assert_eq!(
            buffer.len_lines(),
            body.bytes().filter(|b| *b == b'\n').count() + 1,
            "{name}"
        );
        assert_eq!(buffer.stamp().revision, 1, "{name}");
        assert!(!buffer.is_modified(), "{name}");
        assert!(
            buffer.history().is_empty() && buffer.redo_history().is_empty(),
            "{name}"
        );
        assert_eq!(bytes(buffer), source.as_bytes(), "{name}");
    }
    assert_eq!(text(buffers.get(initial).unwrap()), "");
}

// Invalid UTF-8 and any NUL are errors, including after a recognized BOM;
// no lossy string or partially decoded candidate is returned to a publisher.
#[test]
fn rejected_text() {
    let cases: [(&str, &[u8], &str); 6] = [
        ("invalid byte", b"\xff", "UTF-8"),
        ("truncated scalar", b"\xe2\x82", "UTF-8"),
        ("overlong scalar", b"\xc0\xaf", "UTF-8"),
        ("BOM then invalid", b"\xef\xbb\xbf\xff", "UTF-8"),
        ("embedded NUL", b"a\0b", "NUL"),
        ("BOM then NUL", b"\xef\xbb\xbf\0", "NUL"),
    ];
    for (name, source, message) in cases {
        let error = Loaded::read(source)
            .err()
            .expect("invalid input must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{name}");
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
}

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("rev-files-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn denied(&self) -> PathBuf {
        let path = self.file("denied", b"private");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

// Real files round-trip through ingress and the serializer without changing
// the source. Reloading the output retains the same text and BOM metadata.
#[test]
fn file_roundtrip() {
    let fixture = Fixture::new();
    let cases = [
        ("empty", ""),
        ("LF", "α\nb\n"),
        ("CRLF", "a\r\nb\r\n"),
        ("mixed", "a\r\nb\nc\r\nlast"),
        ("BOM only", "\u{feff}"),
        ("BOM mixed", "\u{feff}α\r\nb\nlast"),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source) in cases {
        let path = fixture.file(name, source.as_bytes());
        let id = buffers.load(file::load(&path).unwrap()).unwrap();
        let buffer = buffers.get(id).unwrap();
        let copy = fixture.0.join(format!("{name}-copy"));
        let mut output = File::create(&copy).unwrap();
        file::write(buffer, &mut output).unwrap();
        output.flush().unwrap();
        assert!(!buffer.is_modified(), "{name}");
        assert_eq!(fs::read(&copy).unwrap(), source.as_bytes(), "{name}");
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes(), "{name}");
        let reloaded = buffers.load(file::load(&copy).unwrap()).unwrap();
        assert_eq!(bytes(buffers.get(reloaded).unwrap()), source.as_bytes());
        assert_eq!(
            buffers.get(reloaded).unwrap().has_bom(),
            buffers.get(id).unwrap().has_bom(),
            "{name}"
        );
    }
}

struct Pieces<'a> {
    source: &'a [u8],
    limit: usize,
}

impl Read for Pieces<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let end = output.len().min(self.limit);
        self.source.read(&mut output[..end])
    }
}

// Scalar, BOM and CRLF bytes may straddle any read boundary. Large input
// remains chunked and coordinate byte zero excludes the recognized BOM.
#[test]
fn chunked_ingress() {
    let source = format!(
        "\u{feff}{}last",
        "e\u{301}界👩\u{200d}💻\r\nx\n".repeat(600)
    );
    let mut buffers = Buffers::new().unwrap();
    for limit in [1, 2, 3, 17, 1024] {
        let loaded = Loaded::read(Pieces {
            source: source.as_bytes(),
            limit,
        })
        .unwrap();
        let id = buffers.load(loaded).unwrap();
        let buffer = buffers.get(id).unwrap();
        let span = buffer.span(0..buffer.len_chars()).unwrap();
        assert!(buffer.chunks(&span).unwrap().count() > 2, "{limit}");
        assert_eq!(bytes(buffer), source.as_bytes(), "{limit}");
        assert_eq!(buffer.newline(), "\n", "{limit}");
        assert!(buffer.has_bom(), "{limit}");
        let coordinates = Coordinates::new(buffer).unwrap();
        assert_eq!(coordinates.to_byte(Scalar(0)).unwrap(), Byte(0));
        assert_eq!(coordinates.to_byte(Scalar(2)).unwrap(), Byte(3));
        assert_eq!(coordinates.next(Scalar(0)).unwrap(), Scalar(2));
    }
}

// Late invalid data must discard an already decoded prefix, including when
// the malformed scalar or NUL appears beyond the reader's internal buffer.
#[test]
fn late_invalid() {
    for suffix in [b"\xff".as_slice(), b"\xe2\x82", b"\0"] {
        let mut source = vec![b'a'; 8192];
        source.extend(suffix);
        for limit in [1, 7, 2048] {
            let error = Loaded::read(Pieces {
                source: &source,
                limit,
            })
            .err()
            .expect("late invalid data must reject the whole candidate");
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
    }
}

struct Broken<'a> {
    source: &'a [u8],
    kind: io::ErrorKind,
}

impl Read for Broken<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.source.is_empty() {
            return Err(io::Error::new(self.kind, "injected read failure"));
        }
        self.source.read(output)
    }
}

// A read failure after valid bytes keeps its I/O kind and cannot expose a
// partially loaded document as a successful candidate.
#[test]
fn failed_reader() {
    let prefix = "\u{feff}valid\r\n界".repeat(1000);
    for kind in [
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::UnexpectedEof,
        io::ErrorKind::Other,
    ] {
        let error = Loaded::read(Broken {
            source: prefix.as_bytes(),
            kind,
        })
        .err()
        .expect("read failure must reject the candidate");
        assert_eq!(error.kind(), kind);
        assert!(error.to_string().contains("injected read failure"));
    }
}

// Missing components preserve NotFound; directories and devices are refused
// before their contents can become an editable document.
#[test]
fn missing_paths() {
    let fixture = Fixture::new();
    let cases = [
        (fixture.0.join("missing"), io::ErrorKind::NotFound),
        (fixture.0.join("missing/child"), io::ErrorKind::NotFound),
        (fixture.0.clone(), io::ErrorKind::InvalidInput),
        (PathBuf::from("/dev/null"), io::ErrorKind::InvalidInput),
    ];
    for (path, kind) in cases {
        let error = file::load(&path).err().expect("path must be rejected");
        assert_eq!(error.kind(), kind, "{path:?}");
        assert!(error.to_string().contains("load "), "{error}");
    }
}

// Exercise an actual denied regular-file read, rather than substituting a
// fake successful filesystem boundary or skipping the permission case.
#[test]
fn denied_file() {
    let fixture = Fixture::new();
    let path = fixture.denied();
    let error = file::load(&path).err().expect("unreadable file");
    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert!(error.to_string().contains("denied"));
}

fn chord(character: char) -> Event {
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}

fn dirty_session() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let source = "0123456789\r\n".repeat(20) + "last";
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(1))).unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(buffer.len_chars());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 8,
            height: 5,
        },
    )
    .unwrap();
    session.insert('!');
    session.execute(Command::KeyboardQuit);
    session.insert('?');
    session.execute(Command::named("undo").unwrap());
    session.frame().unwrap();
    assert!(session.view().top > 0);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    session.handle(chord('x'));
    session
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    bytes: Vec<u8>,
    stamp: Stamp,
    state: State,
    view: View,
    mark: Option<Scalar>,
    history: usize,
    redo: usize,
    read_only: bool,
    newline: &'static str,
    bom: bool,
    file: Option<file::Location>,
    name: String,
    warning: Option<String>,
}

fn snapshot(session: &Session) -> Snapshot {
    let buffer = session.buffer().unwrap();
    Snapshot {
        bytes: bytes(buffer),
        stamp: buffer.stamp(),
        state: buffer.state(),
        view: session.view().clone(),
        mark: buffer.mark(),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        read_only: buffer.is_readonly(),
        newline: buffer.newline(),
        bom: buffer.has_bom(),
        file: buffer.file().cloned(),
        name: buffer.name().into(),
        warning: buffer.save_warning().map(str::to_owned),
    }
}

// Every rejected ingress preserves the active dirty document, mark, scroll,
// retained undo/redo and pending prefix. Only the readable error echo changes.
#[test]
fn rejected_session() {
    let fixture = Fixture::new();
    let cases = [
        (
            fixture.file("invalid", b"text\xff"),
            io::ErrorKind::InvalidData,
        ),
        (fixture.file("NUL", b"a\0b"), io::ErrorKind::InvalidData),
        (
            fixture.file("bad-BOM", b"\xef\xbb\xbf\xff"),
            io::ErrorKind::InvalidData,
        ),
        (fixture.0.clone(), io::ErrorKind::InvalidInput),
        (fixture.denied(), io::ErrorKind::PermissionDenied),
    ];
    let mut session = dirty_session();
    let before = snapshot(&session);
    for (path, kind) in cases {
        let error = session.load(&path).unwrap_err();
        assert_eq!(error.kind(), kind, "{path:?}");
        assert_eq!(snapshot(&session), before, "{path:?}");
        assert!(
            session
                .echo()
                .contains(path.file_name().unwrap().to_str().unwrap())
        );
        assert!(session.frame().unwrap().cursor().is_some());
        assert_eq!(snapshot(&session), before);
    }
    assert_eq!(session.handle(chord('c')), Action::Redraw);
    assert!(session.echo().ends_with("y=save, n=discard, C-g=cancel"));
    session.execute(Command::KeyboardQuit);
    session.execute(Command::named("undo-redo").unwrap());
    assert!(text(session.buffer().unwrap()).ends_with("last!?"));
    session.execute(Command::named("undo").unwrap());
    session.execute(Command::named("undo").unwrap());
    assert!(!session.buffer().unwrap().is_modified());
    assert!(text(session.buffer().unwrap()).ends_with("last"));
}

// Accepted ingress installs a fresh clean root at point/scroll zero, cancels
// old prefixes and retains previous unsaved buffers in the session quit guard.
#[test]
fn accepted_session() {
    let fixture = Fixture::new();
    let source = "\u{feff}e\u{301}界\r\nlast";
    let path = fixture.file("valid", source.as_bytes());
    let mut session = dirty_session();
    let previous = session.view().point.stamp.buffer;
    session.load(&path).unwrap();
    let buffer = session.buffer().unwrap();
    assert_ne!(buffer.id(), previous);
    assert_eq!(bytes(buffer), source.as_bytes());
    assert_eq!(text(buffer), "e\u{301}界\r\nlast");
    assert!(buffer.has_bom());
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert!(buffer.redo_history().is_empty());
    assert_eq!(session.view(), &View::new(buffer));
    assert!(session.echo().is_empty());
    assert!(session.frame().unwrap().cursor().is_some());
    assert_eq!(session.handle(chord('c')), Action::Redraw);
    assert_eq!(session.echo(), "Unbound key");
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert!(session.echo().ends_with("y=save, n=discard, C-g=cancel"));
    session.execute(Command::KeyboardQuit);
}

// Serialization emits the BOM once after edits, preserves untouched bytes and
// does not mark an edit saved. Undo restores exact imported bytes and clean state.
#[test]
fn edited_format() {
    let cases = [
        ("BOM no final", "\u{feff}α\r\nlast", "\u{feff}!α\r\nlast"),
        ("BOM only", "\u{feff}", "\u{feff}!"),
        ("plain no final", "α\nlast", "!α\nlast"),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, expected) in cases {
        let id = buffers
            .load(Loaded::read(source.as_bytes()).unwrap())
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let state = buffer.state();
        let edit = Replacement {
            source: buffer.span(0..0).unwrap(),
            text: "!".into(),
        };
        let prepared = buffer.prepare(&[edit], &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        assert_eq!(bytes(buffer), expected.as_bytes(), "{name}");
        assert!(buffer.is_modified(), "{name}");
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert_eq!(bytes(buffer), source.as_bytes(), "{name}");
        assert_eq!(buffer.state(), state, "{name}");
        assert!(!buffer.is_modified(), "{name}");
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert_eq!(bytes(buffer), expected.as_bytes(), "{name}");
        assert!(buffer.is_modified(), "{name}");
    }
}

// Canonical aliases select the live dirty owner, even after disk changes;
// reopening must not decode another copy or replace its retained history.
#[test]
fn visited_alias() {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"source");
    let alias = fixture.0.join("alias.txt");
    symlink(&path, &alias).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let stamp = buffer.stamp();
    fs::write(&path, b"\xff").unwrap();
    assert_eq!(buffers.visit(&alias).unwrap(), id);
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(text(buffer), "!source");
    assert!(buffer.is_modified());
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(
        buffer.file().unwrap().path(),
        fs::canonicalize(&path).unwrap()
    );
    assert_eq!(buffer.name(), "document.txt");
}

// New destinations have normalized identities and a clean empty root; opening
// neither creates missing parents nor loses reuse across lexical aliases.
#[test]
fn intended_visit() {
    let fixture = Fixture::new();
    let path = fixture.0.join("missing/deep/new.txt");
    let alias = fixture.0.join("unused/../missing/deep/./new.txt");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    assert_eq!(buffers.visit(&alias).unwrap(), id);
    let buffer = buffers.get(id).unwrap();
    assert_eq!(text(buffer), "");
    assert!(!buffer.is_modified());
    assert!(!buffer.file().unwrap().is_present());
    assert_eq!(buffer.file().unwrap().path(), path);
    assert!(!fixture.0.join("missing").exists());
    assert_eq!(buffer.name(), "new.txt");
}

fn type_argument(session: &mut Session, text: &str) {
    for character in text.chars() {
        assert_eq!(
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(character),
                KeyModifiers::NONE
            ))),
            Action::Redraw
        );
    }
}

// Binding and named dispatch share a focused argument owner. Typed paths stay
// in the minibuffer until Enter, with the cursor on its actual echo row.
#[test]
fn find_argument() {
    let fixture = Fixture::new();
    let path = fixture.file("found.txt", b"loaded");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    let before = snapshot(&session);
    session.handle(chord('x'));
    assert_eq!(session.handle(chord('f')), Action::Redraw);
    assert_eq!(session.echo(), "");
    type_argument(&mut session, path.to_str().unwrap());
    assert_eq!(snapshot(&session), before);
    let frame = session.frame().unwrap();
    let prompt = frame
        .rows()
        .iter()
        .find(|row| row.band() == rev::display::Band::Echo)
        .unwrap();
    assert!(prompt.text().starts_with("Find file: "));
    assert_eq!(frame.cursor().unwrap().row, prompt.row());
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(text(session.buffer().unwrap()), "loaded");
    let id = session.buffer().unwrap().id();
    session.insert('!');
    let edited = snapshot(&session);
    session.execute(Command::FindFile);
    type_argument(&mut session, path.to_str().unwrap());
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(snapshot(&session), edited);
    assert_eq!(fs::read(path).unwrap(), b"loaded");
}

// Resolving symlinks before parent traversal preserves filesystem semantics;
// missing suffixes and relative paths converge without creating directories.
#[test]
fn normalized_paths() {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("real/deep")).unwrap();
    let source = fixture.file("real/parent.txt", b"real");
    let link = fixture.0.join("shortcut");
    symlink(fixture.0.join("real/deep"), &link).unwrap();
    let mut relative = PathBuf::new();
    for _ in std::env::current_dir().unwrap().ancestors().skip(1) {
        relative.push("..");
    }
    relative.push(source.strip_prefix("/").unwrap());
    let cases = [
        (
            "symlink parent",
            link.join("../parent.txt"),
            source.clone(),
            true,
        ),
        ("relative file", relative, source, true),
        (
            "missing suffix",
            link.join("missing/../new.txt"),
            fixture.0.join("real/deep/new.txt"),
            false,
        ),
        (
            "rejoined symlink",
            fixture.0.join("unused/../shortcut/new.txt"),
            fixture.0.join("real/deep/new.txt"),
            false,
        ),
        (
            "missing parents",
            fixture.0.join("absent/deep/./new.txt"),
            fixture.0.join("absent/deep/new.txt"),
            false,
        ),
    ];
    for (name, input, expected, present) in cases {
        let location = file::resolve(&input).unwrap();
        assert_eq!(location.path(), expected, "{name}");
        assert_eq!(location.is_present(), present, "{name}");
    }
    assert!(!fixture.0.join("unused").exists());
    assert!(!fixture.0.join("absent").exists());
}

// Basenames, generated labels and literal suffixes never identify the rope.
// Colliding live labels remain unique while every destination stays explicit.
#[test]
fn duplicate_labels() {
    let fixture = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let cases = [
        ("one", "note.txt"),
        ("two", "note.txt"),
        ("three", "note.txt<2>"),
        ("four", "*scratch*"),
    ];
    let mut labels = std::collections::BTreeSet::from(["*scratch*".to_string()]);
    for (directory, basename) in cases {
        fs::create_dir(fixture.0.join(directory)).unwrap();
        let path = fixture.file(&format!("{directory}/{basename}"), directory.as_bytes());
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert!(labels.insert(buffer.name().to_owned()), "{basename}");
        assert_eq!(text(buffer), directory);
        assert_eq!(buffer.file().unwrap().path(), path);
        assert_eq!(buffers.visit(&path).unwrap(), id);
    }
    let anonymous = buffers.create("unnamed").unwrap();
    assert!(labels.insert(buffers.get(anonymous).unwrap().name().to_owned()));
}

// Non-UTF-8 paths retain exact OS identity. Lossy display labels can collide,
// but their suffixes and associations still select distinct canonical buffers.
#[test]
fn raw_names() {
    let fixture = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let mut labels = std::collections::BTreeSet::new();
    for (name, body) in [
        (b"raw-\xff.txt".as_slice(), "first"),
        (b"raw-\xfe.txt", "second"),
    ] {
        let path = fixture.0.join(OsString::from_vec(name.into()));
        fs::write(&path, body).unwrap();
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert!(labels.insert(buffer.name().to_owned()));
        assert_eq!(buffer.file().unwrap().path(), path);
        assert_eq!(text(buffer), body);
        assert_eq!(buffers.visit(&path).unwrap(), id);
    }
}

// Rejected ingress cannot change a visiting buffer's association, unique label,
// history or view. A dangling symlink is an error, never a new link replacement.
#[test]
fn failed_association() {
    let fixture = Fixture::new();
    let source = "0123456789\r\n".repeat(20) + "last";
    let active = fixture.file("active.txt", source.as_bytes());
    let invalid = fixture.file("invalid.txt", b"bad\xff");
    let dangling = fixture.0.join("dangling");
    symlink(fixture.0.join("missing-target"), &dangling).unwrap();
    let mut session = Session::empty(Size {
        width: 8,
        height: 5,
    })
    .unwrap();
    session.load(&active).unwrap();
    session.insert('!');
    session.execute(Command::named("end-of-buffer").unwrap());
    session.execute(Command::named("previous-line").unwrap());
    session.frame().unwrap();
    session.handle(chord('x'));
    let before = snapshot(&session);
    let cases = [
        (invalid.clone(), io::ErrorKind::InvalidData),
        (fixture.0.clone(), io::ErrorKind::InvalidInput),
        (invalid.join("child"), io::ErrorKind::NotADirectory),
        (fixture.denied(), io::ErrorKind::PermissionDenied),
        (PathBuf::new(), io::ErrorKind::InvalidInput),
        (dangling.clone(), io::ErrorKind::NotFound),
    ];
    for (path, kind) in cases {
        let error = session.load(&path).unwrap_err();
        assert_eq!(error.kind(), kind, "{path:?}");
        assert_eq!(session.echo(), error.to_string());
        assert_eq!(snapshot(&session), before, "{path:?}");
        assert!(session.frame().unwrap().cursor().is_some());
    }
    assert_eq!(session.handle(chord('c')), Action::Redraw);
    assert!(session.echo().ends_with("y=save, n=discard, C-g=cancel"));
    session.execute(Command::KeyboardQuit);
    let after_quit = snapshot(&session);
    session.load(&active).unwrap();
    assert_eq!(snapshot(&session), after_quit);
    assert!(fs::symlink_metadata(dangling).unwrap().is_symlink());
    assert!(!fixture.0.join("missing-target").exists());
}

// A failed candidate enters neither the index nor label inventory. A later
// successful retry receives one fresh owner and the next actual collision label.
#[test]
fn failed_index() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("one")).unwrap();
    fs::create_dir(fixture.0.join("two")).unwrap();
    let first = fixture.file("one/same.txt", b"one");
    let second = fixture.file("two/same.txt", b"\xff");
    let mut buffers = Buffers::new().unwrap();
    let one = buffers.visit(&first).unwrap();
    assert!(buffers.visit(&second).is_err());
    fs::write(&second, b"two").unwrap();
    let two = buffers.visit(&second).unwrap();
    assert_ne!(one, two);
    assert_eq!(buffers.get(one).unwrap().name(), "same.txt");
    assert_eq!(buffers.get(two).unwrap().name(), "same.txt<2>");
    assert_eq!(text(buffers.get(two).unwrap()), "two");
    assert_eq!(buffers.visit(&second).unwrap(), two);
}

// A newly appearing disk file and its symlink alias cannot fork a live intended
// destination or replace its unsaved text. Reuse keeps the original format state.
#[test]
fn new_reuse() {
    let fixture = Fixture::new();
    let path = fixture.0.join("new.txt");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&path).unwrap();
    session.insert('!');
    let before = snapshot(&session);
    fs::write(&path, b"external").unwrap();
    let alias = fixture.0.join("alias.txt");
    symlink(&path, &alias).unwrap();
    session.load(&alias).unwrap();
    assert_eq!(snapshot(&session), before);
    assert_eq!(text(session.buffer().unwrap()), "!");
    assert!(!session.buffer().unwrap().file().unwrap().is_present());
    assert_eq!(fs::read(path).unwrap(), b"external");
}

fn press(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

// Cancel returns to the same associated document and view; prompt text and a
// repeated Enter cannot become a visit or a document edit after cancellation.
#[test]
fn canceled_find() {
    let fixture = Fixture::new();
    let active = fixture.file("active", b"original");
    let other = fixture.file("other", b"other");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&active).unwrap();
    session.insert('!');
    let before = snapshot(&session);
    session.execute(Command::FindFile);
    type_argument(&mut session, other.to_str().unwrap());
    session.handle(press(KeyCode::Left));
    session.handle(press(KeyCode::Backspace));
    assert_eq!(snapshot(&session), before);
    session.handle(chord('g'));
    assert_eq!(session.echo(), "Quit");
    assert_eq!(snapshot(&session), before);
    let repeat = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Repeat,
    ));
    assert_eq!(session.handle(repeat), Action::Wait);
    assert_eq!(snapshot(&session), before);
    session.insert('z');
    assert_eq!(text(session.buffer().unwrap()), "!zoriginal");
    assert_eq!(
        session.buffer().unwrap().history().len(),
        before.history + 1
    );
    assert_eq!(fs::read(other).unwrap(), b"other");
}

// Argument cursors belong to the echo row and remain bounded through narrow,
// one-row and zero-size frames. Empty submission reports an error and consumes Enter.
#[test]
fn prompt_geometry() {
    use unicode_width::UnicodeWidthStr;
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.execute(Command::FindFile);
    type_argument(&mut session, "界e\u{301}");
    let before = snapshot(&session);
    for (width, height) in [(3, 3), (5, 1), (1, 1), (0, 0), (80, 8)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        if let Some(cursor) = frame.cursor() {
            let prompt = frame
                .rows()
                .iter()
                .find(|row| row.band() == rev::display::Band::Echo)
                .unwrap();
            assert_eq!(cursor.row, prompt.row());
            assert_eq!(cursor.column, 14.min(width - 1));
        } else {
            assert_eq!((width, height), (0, 0));
        }
        assert_eq!(snapshot(&session), before);
    }
    session.handle(chord('g'));
    session.execute(Command::FindFile);
    session.handle(press(KeyCode::Enter));
    assert!(session.echo().contains("empty file path"));
    assert_eq!(snapshot(&session), before);
    session.handle(press(KeyCode::Char('z')));
    assert_eq!(text(session.buffer().unwrap()), "z");
}

fn insert_text(buffer: &mut Buffer, text: &str) {
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: text.into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}

// Saving streams exact captured bytes through replacement, preserves access
// bits and history, and makes only the actually published logical state clean.
#[test]
fn saved_file() {
    use std::os::unix::fs::MetadataExt;
    let fixture = Fixture::new();
    let cases = [
        ("LF", "α\nlast", "!α\nlast"),
        ("BOM mixed", "\u{feff}α\r\nb\nlast", "\u{feff}!α\r\nb\nlast"),
        ("BOM only", "\u{feff}", "\u{feff}!"),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, expected) in cases {
        let path = fixture.file(name, source.as_bytes());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let inode = fs::metadata(&path).unwrap().ino();
        let id = buffers.visit(&path).unwrap();
        insert_text(buffers.get_mut(id).unwrap(), "!");
        let stamp = buffers.get(id).unwrap().stamp();
        let result = buffers.save(id).unwrap();
        assert!(result.is_complete(), "{name}: {result:?}");
        assert!(result.error().is_none());
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes(), "{name}");
        assert_ne!(fs::metadata(&path).unwrap().ino(), inode, "{name}");
        assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, 0o640);
        let buffer = buffers.get_mut(id).unwrap();
        assert_eq!(buffer.stamp(), stamp);
        assert!(!buffer.is_modified());
        assert_eq!(buffer.history().len(), 1);
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert_eq!(bytes(buffer), source.as_bytes());
        assert!(buffer.is_modified());
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert_eq!(bytes(buffer), expected.as_bytes());
        assert!(!buffer.is_modified());
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), cases.len());
}

// An explicit absent destination is associated only after actual publication;
// the live path index and display label then identify this same canonical owner.
#[test]
fn first_save() {
    let fixture = Fixture::new();
    let path = fixture.0.join("new.txt");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("").unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "new");
    let save = buffers.prepare_save(id, Some(&path)).unwrap();
    assert!(buffers.get(id).unwrap().file().is_none());
    assert!(!path.exists());
    let result = buffers.publish_save(save);
    assert!(result.is_complete(), "{result:?}");
    let buffer = buffers.get(id).unwrap();
    assert!(!buffer.is_modified());
    assert_eq!(buffer.file().unwrap().path(), path);
    assert!(buffer.file().unwrap().is_present());
    assert!(buffer.file().unwrap().observation().is_some());
    assert_eq!(buffer.name(), "new.txt");
    assert_eq!(fs::read(&path).unwrap(), b"new");
    assert_eq!(buffers.visit(&path).unwrap(), id);
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
}

// The actual binding uses the same save owner as named dispatch. Only a
// completed save clears the modified quit guard, without changing the view point.
#[test]
fn session_save() {
    let fixture = Fixture::new();
    let path = fixture.file("session.txt", b"source");
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&path).unwrap();
    session.insert('!');
    let point = session.view().point;
    session.handle(chord('x'));
    assert_eq!(session.handle(chord('s')), Action::Redraw);
    assert!(session.echo().starts_with("Saved "), "{}", session.echo());
    assert_eq!(session.view().point, point);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(fs::read(&path).unwrap(), b"!source");
    assert_eq!(session.execute(Command::Quit), Action::Exit);
    session.insert('?');
    session.execute(Command::SaveBuffer);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(fs::read(path).unwrap(), b"!?source");
}
