#![forbid(unsafe_code)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{
    Buffer, Buffers,
    edit::{History, Point, Replacement},
};
use rev::command::{Command, Motion, Split};
use rev::coordinates::{Coordinates, Scalar};
use rev::file;
use rev::session::Session;
use rev::view::{Size, View};

static NEXT: AtomicU64 = AtomicU64::new(0);
const LIMIT: usize = 1024;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rev-reload-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn bytes(buffer: &Buffer) -> Vec<u8> {
    let mut result = Vec::new();
    file::write(buffer, &mut result).unwrap();
    result
}
fn edit(buffer: &mut Buffer, value: &str) {
    let source = buffer.span(0..0).unwrap();
    let prepared = buffer
        .prepare(
            &[Replacement {
                source,
                text: value.into(),
            }],
            &[],
            None,
        )
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}
fn visiting(path: &Path) -> (Buffers, rev::buffer::BufferId) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(path).unwrap();
    (buffers, id)
}

// Complete byte images, including BOM-only changes, use native history. These
// real files establish serialization and clean-state semantics, not notifications.
#[test]
fn native_history() {
    for (name, old, new) in [
        ("ordinary", "same old tail", "same new tail"),
        (
            "unicode seams",
            "Ae\u{301}界tail",
            "Ae\u{300}👩\u{200d}💻tail",
        ),
        ("separators", "a\r\nb\r\n", "a\nb"),
        ("bom added", "text", "\u{feff}text"),
        ("bom removed", "\u{feff}text", "text"),
        ("empty", "old", ""),
    ] {
        let fixture = Fixture::new();
        let path = fixture.file("source.txt", old.as_bytes());
        let (mut buffers, id) = visiting(&path);
        fs::write(&path, new).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let before = buffer.stamp();
        let mut points = [
            Point {
                stamp: before,
                position: Scalar(0),
            },
            Point {
                stamp: before,
                position: Scalar(buffer.len_chars()),
            },
        ];
        let checked = buffer
            .request_reload(LIMIT)
            .unwrap()
            .read()
            .unwrap()
            .verify(false)
            .unwrap();
        let publication = buffer.prepare_reload(checked, &points).unwrap();
        assert!(
            buffer.publish_reload(publication, &mut points).unwrap(),
            "{name}"
        );
        assert_eq!(bytes(buffer), new.as_bytes(), "{name}");
        assert_eq!(buffer.history().len(), 1, "{name}");
        assert!(!buffer.is_modified(), "{name}");
        for point in points {
            assert_eq!(point.stamp, buffer.stamp());
            assert!(
                Coordinates::new(buffer)
                    .unwrap()
                    .is_boundary(point.position)
                    .unwrap()
            );
        }
        buffer.navigate(History::Undo, &mut points, None).unwrap();
        assert_eq!(bytes(buffer), old.as_bytes(), "{name}");
        assert!(buffer.is_modified(), "{name}");
        assert_eq!(fs::read(&path).unwrap(), new.as_bytes());
        buffer.navigate(History::Redo, &mut points, None).unwrap();
        assert_eq!(bytes(buffer), new.as_bytes(), "{name}");
        assert!(!buffer.is_modified(), "{name}");
    }
}

// An identical serialized image changes disk evidence only. A later save must
// not ask for overwrite consent for that already acknowledged observation.
#[test]
fn identical_image() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"same bytes");
    let (mut buffers, id) = visiting(&path);
    let original = buffers.get(id).unwrap().stamp();
    let replacement = fixture.file("replacement", b"same bytes");
    fs::rename(replacement, &path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [Point {
        stamp: original,
        position: Scalar(3),
    }];
    let checked = buffer
        .request_reload(LIMIT)
        .unwrap()
        .read()
        .unwrap()
        .verify(false)
        .unwrap();
    let publication = buffer.prepare_reload(checked, &points).unwrap();
    assert!(!buffer.publish_reload(publication, &mut points).unwrap());
    assert_eq!(buffer.stamp(), original);
    assert_eq!(points[0].position, Scalar(3));
    assert!(buffer.history().is_empty());
    assert_eq!(
        buffer.file().unwrap().observation(),
        file::observe(&path).unwrap().as_ref()
    );
    assert!(matches!(
        buffers.request_save(id, None).unwrap(),
        rev::buffer::SaveRequest::Ready(_)
    ));
}

// Read-only admission applies only to disk refresh. Existing history remains,
// and ordinary edits cannot inherit the refresh's permission.
#[test]
fn readonly_and_prior_history() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"base");
    let (mut buffers, id) = visiting(&path);
    let buffer = buffers.get_mut(id).unwrap();
    edit(buffer, "local ");
    buffer.mark_saved(buffer.stamp()).unwrap();
    buffer.set_readonly(true);
    fs::write(&path, "fresh").unwrap();
    let checked = buffer
        .request_reload(LIMIT)
        .unwrap()
        .read()
        .unwrap()
        .verify(false)
        .unwrap();
    let publication = buffer.prepare_reload(checked, &[]).unwrap();
    buffer.publish_reload(publication, &mut []).unwrap();
    assert!(buffer.is_readonly());
    assert_eq!(buffer.history().len(), 2);
    let source = buffer.span(0..0).unwrap();
    assert!(
        buffer
            .prepare(
                &[Replacement {
                    source,
                    text: "x".into()
                }],
                &[],
                None
            )
            .is_err()
    );
    buffer.set_readonly(false);
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(bytes(buffer), b"local base");
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(bytes(buffer), b"base");
}

// Invalid/missing/rebound/oversized sources never install partial or lossy text,
// acknowledge save evidence, clear history or recreate a missing destination.
#[test]
fn unavailable_sources() {
    for name in [
        "utf8",
        "nul",
        "oversized",
        "missing",
        "directory",
        "symlink",
        "permissions",
    ] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"keep");
        let (mut buffers, id) = visiting(&path);
        let buffer = buffers.get_mut(id).unwrap();
        edit(buffer, "local ");
        let stamp = buffer.stamp();
        let evidence = buffer.file().unwrap().clone();
        match name {
            "utf8" => fs::write(&path, [0xff]).unwrap(),
            "nul" => fs::write(&path, b"a\0b").unwrap(),
            "oversized" => fs::write(&path, vec![b'x'; LIMIT + 1]).unwrap(),
            "missing" => fs::remove_file(&path).unwrap(),
            "directory" => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            "symlink" => {
                fs::remove_file(&path).unwrap();
                symlink(fixture.file("other", b"elsewhere"), &path).unwrap();
            }
            "permissions" => {
                fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
                if fs::read(&path).is_ok() {
                    continue;
                }
            }
            _ => unreachable!(),
        }
        assert!(
            buffer.request_reload(LIMIT).unwrap().read().is_err(),
            "{name}"
        );
        assert_eq!(bytes(buffer), b"local keep", "{name}");
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.file(), Some(&evidence));
        assert_eq!(buffer.history().len(), 1);
        assert!(buffer.is_modified());
        if name == "missing" {
            assert!(!path.exists());
        }
    }
}

// Consent is bound to buffer revision and the observed destination. Controlled
// interleavings exercise races without timing assumptions or production switches.
#[test]
fn stale_consent() {
    for phase in ["decline", "disk", "typing", "prepared", "saved"] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"old");
        let (mut buffers, id) = visiting(&path);
        let buffer = buffers.get_mut(id).unwrap();
        edit(buffer, "local ");
        fs::write(&path, "external").unwrap();
        let reload = buffer.request_reload(LIMIT).unwrap().read().unwrap();
        assert!(reload.needs_confirmation());
        match phase {
            "decline" => assert!(reload.verify(false).is_err()),
            "disk" => {
                fs::write(&path, "new external").unwrap();
                assert!(reload.verify(true).is_err());
            }
            "typing" => {
                edit(buffer, "typed ");
                assert!(
                    buffer
                        .prepare_reload(reload.verify(true).unwrap(), &[])
                        .is_err()
                );
            }
            "prepared" => {
                let publication = buffer
                    .prepare_reload(reload.verify(true).unwrap(), &[])
                    .unwrap();
                edit(buffer, "typed ");
                assert!(buffer.publish_reload(publication, &mut []).is_err());
            }
            "saved" => {
                let checked = reload.verify(true).unwrap();
                buffer.mark_saved(buffer.stamp()).unwrap();
                assert!(buffer.prepare_reload(checked, &[]).is_err());
            }
            _ => unreachable!(),
        }
        assert!(bytes(buffer).ends_with(b"local old"), "{phase}");
        if phase != "saved" {
            assert!(
                matches!(
                    buffers.request_save(id, None).unwrap(),
                    rev::buffer::SaveRequest::Confirm(_)
                ),
                "{phase}"
            );
        }
    }
}

// A closed source never lends its admitted image to a newly allocated identity.
#[test]
fn closed_source() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"old");
    let (mut buffers, id) = visiting(&path);
    fs::write(&path, "external").unwrap();
    let checked = buffers
        .get(id)
        .unwrap()
        .request_reload(LIMIT)
        .unwrap()
        .read()
        .unwrap()
        .verify(false)
        .unwrap();
    let kill = buffers.prepare_kill(id).unwrap();
    let replacement = buffers.kill(kill).unwrap();
    assert!(
        buffers
            .get(replacement)
            .unwrap()
            .prepare_reload(checked, &[])
            .is_err()
    );
}

fn editor(fixture: &Fixture, path: &Path, profile: &str, policy: &str) -> Session {
    let root = fixture.file("config.toml", format!("editor.interaction_profile = '{profile}'\ntree.visible = false\nfiles.external_changes = '{policy}'").as_bytes());
    let startup = rev::config::prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(path).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap()
}
fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}
fn wait(editor: &mut Session, ready: impl Fn(&Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        editor.expire_input(Instant::now());
        if ready(editor) {
            break;
        }
        assert!(Instant::now() < deadline, "{}", editor.echo());
        std::thread::sleep(Duration::from_millis(1));
    }
}

// Both native command surfaces share reload under both policies. Real async
// ingress updates every pane once and retains focus and unchanged prefix points.
#[test]
fn native_surfaces() {
    for profile in ["non-modal", "modal"] {
        for policy in ["reload", "ask"] {
            let fixture = Fixture::new();
            let path = fixture.file("source", b"prefix old tail");
            let mut editor = editor(&fixture, &path, profile, policy);
            editor.execute(Command::Motion(Motion::ForwardChar));
            editor.execute(Command::Split(Split::Below));
            let selected = editor.windows().selected_id();
            let id = editor.buffer().unwrap().id();
            fs::write(&path, "prefix fresh tail").unwrap();
            if profile == "modal" {
                editor.handle(key(KeyCode::Char(':')));
            } else {
                editor.execute(Command::ExecuteExtended);
            }
            editor.handle(Event::Paste("revert-buffer".into()));
            editor.handle(key(KeyCode::Enter));
            wait(&mut editor, |s| s.echo().starts_with("Reloaded "));
            let buffer = editor.buffer().unwrap();
            assert_eq!(buffer.id(), id);
            assert_eq!(bytes(buffer), b"prefix fresh tail");
            assert_eq!(buffer.history().len(), 1);
            assert_eq!(editor.windows().selected_id(), selected);
            assert_eq!(editor.windows().iter().len(), 2);
            for (_, view) in editor.windows().iter() {
                assert_eq!(view.point.stamp, buffer.stamp());
                assert_eq!(view.point.position, Scalar(1));
            }
            assert!(editor.frame().is_ok());
            editor.execute(Command::History(History::Undo));
            assert_eq!(bytes(editor.buffer().unwrap()), b"prefix old tail");
            assert!(editor.buffer().unwrap().is_modified());
        }
    }
}

// Cancellation retains dirty text and does not acknowledge external overwrite.
// An immediate local edit also wins over the worker's captured clean revision.
#[test]
fn native_cancel_and_typing() {
    for cancel in [true, false] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"old");
        let mut editor = editor(&fixture, &path, "non-modal", "reload");
        if cancel {
            editor.replace(0..0, "local ").unwrap();
        }
        fs::write(&path, "external").unwrap();
        editor.execute(Command::RevertBuffer);
        if cancel {
            wait(&mut editor, |s| s.echo().starts_with("Discard unsaved"));
            editor.execute(Command::KeyboardQuit);
            assert!(editor.echo().contains("canceled"));
            assert_eq!(bytes(editor.buffer().unwrap()), b"local old");
        } else {
            editor.replace(0..0, "typed ").unwrap();
            wait(&mut editor, |s| s.echo().starts_with("reload:"));
            assert_eq!(bytes(editor.buffer().unwrap()), b"typed old");
        }
    }
}

// The byte bound includes the serialized BOM and permits exactly the limit.
#[test]
fn byte_limit() {
    for (name, value, limit, accepted) in [
        ("empty", "", 1, true),
        ("exact", "text", 4, true),
        ("excess", "text", 3, false),
        ("bom exact", "\u{feff}x", 4, true),
        ("bom excess", "\u{feff}x", 3, false),
    ] {
        let fixture = Fixture::new();
        let path = fixture.file("source", value.as_bytes());
        let (buffers, id) = visiting(&path);
        assert_eq!(
            buffers
                .get(id)
                .unwrap()
                .request_reload(limit)
                .unwrap()
                .read()
                .is_ok(),
            accepted,
            "{name}"
        );
    }
}

// Native association changes invalidate a previously checked image even when
// the document revision itself did not change during the rename.
#[test]
fn rebound_source() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"old");
    let (mut buffers, id) = visiting(&path);
    fs::write(&path, "external").unwrap();
    let checked = buffers
        .get(id)
        .unwrap()
        .request_reload(LIMIT)
        .unwrap()
        .read()
        .unwrap()
        .verify(false)
        .unwrap();
    buffers
        .rename(file::Rename::capture(&path).unwrap(), "renamed")
        .unwrap();
    let buffer = buffers.get(id).unwrap();
    assert!(buffer.prepare_reload(checked, &[]).is_err());
    assert_eq!(bytes(buffer), b"old");
}

// Explicit consent publishes the captured dirty-buffer image in one operation;
// clean work also remains associated with its source after focus moves elsewhere.
#[test]
fn consent_and_hidden_source() {
    for dirty in [true, false] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"old");
        let other = fixture.file("other", b"other buffer");
        let mut editor = editor(&fixture, &path, "non-modal", "ask");
        if dirty {
            editor.replace(0..0, "local ").unwrap();
        }
        fs::write(&path, "external").unwrap();
        editor.execute(Command::RevertBuffer);
        if dirty {
            wait(&mut editor, |s| s.echo().starts_with("Discard unsaved"));
            editor.handle(key(KeyCode::Char('y')));
        } else {
            editor.load(&other).unwrap();
        }
        wait(&mut editor, |s| s.echo().starts_with("Reloaded "));
        if !dirty {
            assert_eq!(bytes(editor.buffer().unwrap()), b"other buffer");
        }
        editor.load(&path).unwrap();
        assert_eq!(bytes(editor.buffer().unwrap()), b"external");
        assert!(!editor.buffer().unwrap().is_modified());
        assert_eq!(
            editor.buffer().unwrap().history().len(),
            if dirty { 2 } else { 1 }
        );
    }
}
