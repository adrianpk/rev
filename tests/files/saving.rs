use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use rev::buffer::edit::History;
use rev::buffer::{Buffer, BufferId, Buffers, Save, Stamp, State};
use rev::command::Command;
use rev::file::{self, Location, Observation, Output, Publication};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::{Fixture, bytes, insert_text, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    None,
    Write,
    Flush,
    FileSync,
    Replace,
    DirectorySync,
    Observe,
    BothPost,
    Cleanup,
}

struct Failing {
    fault: Fault,
    root: PathBuf,
    temporary: Option<PathBuf>,
}

impl Failing {
    fn new(fault: Fault, root: &Path) -> Self {
        Self {
            fault,
            root: root.into(),
            temporary: None,
        }
    }
}

fn injected(phase: &str) -> io::Error {
    io::Error::other(format!("injected {phase} failure"))
}

fn temporaries(root: &Path) -> Vec<PathBuf> {
    fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".rev-save-")
        })
        .collect()
}

impl Output for Failing {
    fn write(&mut self, save: &Save, file: &mut File) -> io::Result<()> {
        let owned = temporaries(&self.root);
        assert_eq!(owned.len(), 1, "one real same-directory temporary output");
        assert_eq!(owned[0].parent(), Some(self.root.as_path()));
        assert_eq!(file.metadata()?.mode() & 0o777, 0o600);
        self.temporary = Some(owned[0].clone());
        if matches!(self.fault, Fault::Write | Fault::Cleanup) {
            file.write_all(b"partial")?;
            file.flush()?;
            assert_eq!(fs::read(&owned[0])?, b"partial");
            return Err(io::Error::new(
                io::ErrorKind::StorageFull,
                "injected partial write failure",
            ));
        }
        save.write(file)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        if self.fault == Fault::Flush {
            return Err(injected("flush"));
        }
        file.flush()
    }

    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        if self.fault == Fault::FileSync {
            return Err(injected("file sync"));
        }
        file.sync_all()
    }

    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        assert_eq!(Some(temporary), self.temporary.as_deref());
        assert_eq!(target.parent(), Some(self.root.as_path()));
        if self.fault == Fault::Replace {
            return Err(injected("replace"));
        }
        fs::rename(temporary, target)
    }

    fn sync_directory(&mut self, directory: &File) -> io::Result<()> {
        if matches!(self.fault, Fault::DirectorySync | Fault::BothPost) {
            return Err(injected("directory sync"));
        }
        directory.sync_all()
    }

    fn observe(&mut self, path: &Path) -> io::Result<Observation> {
        if matches!(self.fault, Fault::Observe | Fault::BothPost) {
            return Err(injected("observation"));
        }
        file::observe(path)?.ok_or_else(|| injected("missing observation"))
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        if self.fault == Fault::Cleanup {
            return Err(injected("cleanup"));
        }
        fs::remove_file(path)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Facts {
    bytes: Vec<u8>,
    stamp: Stamp,
    state: State,
    file: Option<Location>,
    name: String,
    history: usize,
    redo: usize,
    dirty: bool,
    read_only: bool,
    warning: Option<String>,
}

fn facts(buffer: &Buffer) -> Facts {
    Facts {
        bytes: bytes(buffer),
        stamp: buffer.stamp(),
        state: buffer.state(),
        file: buffer.file().cloned(),
        name: buffer.name().into(),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        dirty: buffer.is_modified(),
        read_only: buffer.is_read_only(),
        warning: buffer.save_warning().map(str::to_owned),
    }
}

fn editable() -> (Fixture, Buffers, BufferId, PathBuf) {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    insert_text(buffer, "!");
    insert_text(buffer, "?");
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    (fixture, buffers, id, path)
}

// Only real filesystem boundaries fail: partial bytes reach an owned temporary,
// but every pre-replacement failure preserves destination, history and clean marker.
#[test]
fn failed_output() {
    let cases = [
        (Fault::Write, "write temporary"),
        (Fault::Flush, "flush temporary"),
        (Fault::FileSync, "synchronize temporary"),
        (Fault::Replace, "replace destination"),
        (Fault::Cleanup, "remove temporary"),
    ];
    for (fault, phase) in cases {
        let (fixture, mut buffers, id, path) = editable();
        let before = facts(buffers.get(id).unwrap());
        let save = buffers.prepare_save(id, None).unwrap();
        let mut output = Failing::new(fault, &fixture.0);
        let result = buffers.publish_save_with(save, &mut output);
        assert_eq!(result.publication(), Publication::NotPublished, "{fault:?}");
        assert!(!result.is_complete());
        assert!(
            result.error().unwrap().to_string().contains(phase),
            "{result:?}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        assert!(temporaries(&fixture.0).is_empty());
        assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
        assert_eq!(buffers.visit(&path).unwrap(), id);
        let buffer = buffers.get_mut(id).unwrap();
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert_eq!(text(buffer), "original");
        assert!(!buffer.is_modified());
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert!(buffer.is_modified());
    }
}

// Directory-sync and observation failures occur after an actual successful
// rename. They retain publication/path facts, both errors and a pending save.
#[test]
fn uncertain_output() {
    for fault in [Fault::DirectorySync, Fault::Observe, Fault::BothPost] {
        let (fixture, mut buffers, id, path) = editable();
        let before = facts(buffers.get(id).unwrap());
        let save = buffers.prepare_save(id, None).unwrap();
        let result = buffers.publish_save_with(save, &mut Failing::new(fault, &fixture.0));
        assert_eq!(result.publication(), Publication::Uncertain);
        assert!(!result.is_complete());
        assert_eq!(result.target(), path);
        assert_eq!(fs::read(&path).unwrap(), b"!original");
        let buffer = buffers.get_mut(id).unwrap();
        assert_eq!(buffer.stamp(), before.stamp);
        assert_eq!(buffer.state(), before.state);
        assert_eq!(buffer.history().len(), before.history);
        assert_eq!(buffer.redo_history().len(), before.redo);
        assert!(buffer.is_modified());
        assert!(buffer.save_warning().unwrap().contains("published"));
        assert!(buffer.file().unwrap().is_present());
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert_eq!(text(buffer), "original");
        assert!(
            buffer.is_modified(),
            "a warning cannot satisfy a required save at an old clean state"
        );
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert!(temporaries(&fixture.0).is_empty());
        check_uncertain(fault, &mut buffers, id, &result);
    }
}

fn check_uncertain(fault: Fault, buffers: &mut Buffers, id: BufferId, result: &file::Outcome) {
    if fault == Fault::BothPost {
        let error = result.error().unwrap().to_string();
        assert!(error.contains("directory sync") && error.contains("observation"));
    }
    if fault == Fault::DirectorySync {
        assert!(buffers.save(id).unwrap().is_complete());
        assert!(!buffers.get(id).unwrap().is_modified());
        assert!(buffers.get(id).unwrap().save_warning().is_none());
    } else {
        let retry = buffers.save(id).unwrap();
        assert_eq!(retry.publication(), Publication::NotPublished);
        assert!(buffers.get(id).unwrap().is_modified());
    }
}

// Even an otherwise clean buffer must remain guarded when a required save
// completion failed after publication; quit cannot treat replacement as success.
#[test]
fn pending_clean() {
    let fixture = Fixture::new();
    let path = fixture.file("clean.txt", b"clean");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let save = buffers.prepare_save(id, None).unwrap();
    let result =
        buffers.publish_save_with(save, &mut Failing::new(Fault::DirectorySync, &fixture.0));
    assert_eq!(result.publication(), Publication::Uncertain);
    assert!(buffers.get(id).unwrap().is_modified());
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 8,
        },
    )
    .unwrap();
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert!(session.echo().starts_with("Unsaved changes"));
    assert_eq!(fs::read(path).unwrap(), b"clean");
}

// Later edits cannot change captured output or become clean by its completion.
// Undo can still return to the exact state that was actually saved.
#[test]
fn captured_state() {
    let (_fixture, mut buffers, id, path) = editable();
    let save = buffers.prepare_save(id, None).unwrap();
    let saved = save.state();
    let captured = save.stamp();
    insert_text(buffers.get_mut(id).unwrap(), "newer");
    let current = buffers.get(id).unwrap().stamp();
    assert_ne!(captured, current);
    let result = buffers.publish_save(save);
    assert!(result.is_complete());
    assert_eq!(fs::read(path).unwrap(), b"!original");
    let buffer = buffers.get_mut(id).unwrap();
    assert_eq!(text(buffer), "newer!original");
    assert_eq!(buffer.stamp(), current);
    assert!(buffer.is_modified());
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(buffer.state(), saved);
    assert_eq!(text(buffer), "!original");
    assert!(!buffer.is_modified());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert!(buffer.is_modified());
}

// A first association appears only on actual publication. An unresolved result
// retains the new target and index while keeping the save pending.
#[test]
fn initial_outcomes() {
    for fault in [Fault::Write, Fault::DirectorySync, Fault::None] {
        let fixture = Fixture::new();
        let path = fixture.0.join("first.txt");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("").unwrap();
        insert_text(buffers.get_mut(id).unwrap(), "new");
        let before = facts(buffers.get(id).unwrap());
        let save = buffers.prepare_save(id, Some(&path)).unwrap();
        let result = buffers.publish_save_with(save, &mut Failing::new(fault, &fixture.0));
        if fault == Fault::Write {
            assert_eq!(result.publication(), Publication::NotPublished);
            assert_eq!(facts(buffers.get(id).unwrap()), before);
            assert!(!path.exists());
            assert_ne!(buffers.visit(&path).unwrap(), id);
        } else {
            let buffer = buffers.get(id).unwrap();
            assert_eq!(buffer.file().unwrap().path(), path);
            assert!(buffer.file().unwrap().is_present());
            assert_eq!(buffer.name(), "first.txt");
            assert_eq!(buffer.is_modified(), fault != Fault::None);
            assert_eq!(fs::read(&path).unwrap(), b"new");
            assert_eq!(buffers.visit(&path).unwrap(), id);
        }
        assert!(temporaries(&fixture.0).is_empty());
    }
}

// Relocation follows publication, never preparation. An unpublished alternate
// save keeps the old path, while either published result exposes the actual new target.
#[test]
fn association_move() {
    for fault in [Fault::Write, Fault::DirectorySync, Fault::None] {
        let (fixture, mut buffers, id, original) = editable();
        let target = fixture.0.join("other.txt");
        let before = facts(buffers.get(id).unwrap());
        let save = buffers.prepare_save(id, Some(&target)).unwrap();
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        let result = buffers.publish_save_with(save, &mut Failing::new(fault, &fixture.0));
        assert_eq!(fs::read(&original).unwrap(), b"original");
        if result.publication() == Publication::NotPublished {
            assert_eq!(facts(buffers.get(id).unwrap()), before);
            assert!(!target.exists());
            assert_eq!(buffers.visit(&original).unwrap(), id);
        } else {
            assert_eq!(buffers.get(id).unwrap().file().unwrap().path(), target);
            assert_eq!(buffers.visit(&target).unwrap(), id);
            assert_ne!(buffers.visit(&original).unwrap(), id);
            assert_eq!(fs::read(&target).unwrap(), b"!original");
        }
        assert!(temporaries(&fixture.0).is_empty());
    }
}

// Admission checks both actual OS write permissions and native read-only state,
// including a restriction introduced after capture and before publication.
#[test]
fn access_admission() {
    let fixture = Fixture::new();
    let path = fixture.file("readonly.txt", b"original");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "!");
    let before = facts(buffers.get(id).unwrap());
    let result = buffers.save(id).unwrap();
    assert_eq!(result.publication(), Publication::NotPublished);
    assert_eq!(
        result.error().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(facts(buffers.get(id).unwrap()), before);
    let save = buffers.prepare_save(id, None).unwrap();
    buffers.get_mut(id).unwrap().set_read_only(true);
    let restricted = facts(buffers.get(id).unwrap());
    assert!(buffers.prepare_save(id, None).is_err());
    assert_eq!(
        buffers.publish_save(save).publication(),
        Publication::NotPublished
    );
    assert_eq!(facts(buffers.get(id).unwrap()), restricted);
    assert_eq!(fs::read(path).unwrap(), b"original");
    assert!(temporaries(&fixture.0).is_empty());
}

// Neither default save nor an explicit spelling of the same destination may
// refresh away an external-change observation and silently overwrite new bytes.
#[test]
fn changed_destination() {
    for explicit in [false, true] {
        let (fixture, mut buffers, id, path) = editable();
        let before = facts(buffers.get(id).unwrap());
        fs::write(&path, b"external").unwrap();
        let target = explicit.then_some(path.as_path());
        let save = buffers.prepare_save(id, target).unwrap();
        let result = buffers.publish_save(save);
        assert_eq!(result.publication(), Publication::NotPublished);
        assert!(result.error().unwrap().to_string().contains("changed"));
        assert_eq!(fs::read(&path).unwrap(), b"external");
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        assert!(temporaries(&fixture.0).is_empty());
    }
}

// Occupied targets need explicit authority, and a live visiting owner introduced
// after capture must still prevent collision before any filesystem mutation.
#[test]
fn target_collision() {
    let (fixture, mut buffers, id, path) = editable();
    let occupied = fixture.file("occupied.txt", b"keep");
    let error = buffers.prepare_save(id, Some(&occupied)).err().unwrap();
    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    let target = fixture.0.join("new.txt");
    let save = buffers.prepare_save(id, Some(&target)).unwrap();
    let other = buffers.visit(&target).unwrap();
    let before = facts(buffers.get(id).unwrap());
    let result = buffers.publish_save(save);
    assert_eq!(result.publication(), Publication::NotPublished);
    assert!(
        result
            .error()
            .unwrap()
            .to_string()
            .contains("another live buffer")
    );
    assert_eq!(facts(buffers.get(id).unwrap()), before);
    assert_eq!(buffers.visit(&target).unwrap(), other);
    assert!(!target.exists());
    assert_eq!(fs::read(path).unwrap(), b"original");
    assert_eq!(fs::read(occupied).unwrap(), b"keep");
}

// A completed association change invalidates an older save capture before I/O;
// the stale operation cannot move the buffer back or acknowledge obsolete state.
#[test]
fn stale_save() {
    let (fixture, mut buffers, id, path) = editable();
    let old = buffers.prepare_save(id, None).unwrap();
    let target = fixture.0.join("new.txt");
    let current = buffers.prepare_save(id, Some(&target)).unwrap();
    assert!(buffers.publish_save(current).is_complete());
    let before = facts(buffers.get(id).unwrap());
    let result = buffers.publish_save(old);
    assert_eq!(result.publication(), Publication::NotPublished);
    assert!(
        result
            .error()
            .unwrap()
            .to_string()
            .contains("association changed")
    );
    assert_eq!(facts(buffers.get(id).unwrap()), before);
    assert_eq!(fs::read(path).unwrap(), b"original");
    assert_eq!(fs::read(target).unwrap(), b"!original");
}

// Saving through a visiting symlink replaces its canonical referent in that
// directory and preserves the alias entry itself, including after a real rename.
#[test]
fn symlink_save() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("real")).unwrap();
    let path = fixture.file("real/document.txt", b"original");
    let alias = fixture.0.join("alias.txt");
    symlink(&path, &alias).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&alias).unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "!");
    let save = buffers.prepare_save(id, None).unwrap();
    let mut output = Failing::new(Fault::None, path.parent().unwrap());
    assert!(buffers.publish_save_with(save, &mut output).is_complete());
    assert!(fs::symlink_metadata(&alias).unwrap().is_symlink());
    assert_eq!(fs::read_link(&alias).unwrap(), path);
    assert_eq!(fs::read(&path).unwrap(), b"!original");
    assert_eq!(buffers.visit(&alias).unwrap(), id);
    assert!(temporaries(path.parent().unwrap()).is_empty());
}

// An unnamed buffer cannot acquire a false save acknowledgment from the binding;
// its edited rope remains guarded until a real destination can be supplied.
#[test]
fn unnamed_save() {
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.insert('!');
    let before = facts(session.buffer().unwrap());
    assert_eq!(session.execute(Command::SaveBuffer), Action::Redraw);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Write file: ")
    );
    session.execute(Command::KeyboardQuit);
    assert_eq!(facts(session.buffer().unwrap()), before);
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert!(session.echo().starts_with("Unsaved changes"));
}

fn occupied_save(buffers: &Buffers, id: BufferId, path: &Path) -> rev::buffer::Confirmation {
    match buffers.request_save(id, Some(path)).unwrap() {
        rev::buffer::SaveRequest::Confirm(confirmation) => confirmation,
        rev::buffer::SaveRequest::Ready(_) => panic!("occupied target requires consent"),
    }
}

// Confirmed save-as obeys the same actual-publication boundary as ordinary save.
// Partial disk-full writes preserve the old association; later failures expose
// the new target, keep the warning, and require a completed retry before clean.
#[test]
fn confirmed_outcomes() {
    for fault in [
        Fault::Write,
        Fault::Replace,
        Fault::DirectorySync,
        Fault::Observe,
        Fault::None,
    ] {
        let (fixture, mut buffers, id, original) = editable();
        let target = fixture.file("occupied.txt", b"keep");
        let before = facts(buffers.get(id).unwrap());
        let confirmation = occupied_save(&buffers, id, &target);
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        let result = buffers.confirm_save_with(confirmation, &mut Failing::new(fault, &fixture.0));
        assert_eq!(fs::read(&original).unwrap(), b"original");
        confirmed_result(&mut buffers, id, &target, &result, &before);
        assert!(temporaries(&fixture.0).is_empty());
    }
}

fn confirmed_result(
    buffers: &mut Buffers,
    id: BufferId,
    target: &Path,
    result: &file::Outcome,
    before: &Facts,
) {
    if result.publication() == Publication::NotPublished {
        assert_eq!(fs::read(target).unwrap(), b"keep");
        assert_eq!(facts(buffers.get(id).unwrap()), *before);
        return;
    }
    assert_eq!(fs::read(target).unwrap(), b"!original");
    assert_eq!(buffers.get(id).unwrap().file().unwrap().path(), target);
    assert_eq!(buffers.visit(target).unwrap(), id);
    let buffer = buffers.get_mut(id).unwrap();
    assert_eq!(buffer.is_modified(), !result.is_complete());
    assert_eq!(buffer.save_warning().is_some(), !result.is_complete());
    assert_eq!(buffer.stamp(), before.stamp);
    assert_eq!(buffer.history().len(), before.history);
    assert_eq!(buffer.redo_history().len(), before.redo);
    if !result.is_complete() {
        complete_retry(buffers, id);
    }
    let buffer = buffers.get_mut(id).unwrap();
    assert!(!buffer.is_modified());
    assert!(buffer.save_warning().is_none());
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert!(buffer.is_modified());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert!(!buffer.is_modified());
}

fn complete_retry(buffers: &mut Buffers, id: BufferId) {
    let result = match buffers.request_save(id, None).unwrap() {
        rev::buffer::SaveRequest::Ready(save) => buffers.publish_save(save),
        rev::buffer::SaveRequest::Confirm(confirmation) => buffers.confirm_save(confirmation),
    };
    assert!(result.is_complete(), "{result:?}");
}

// Consent must not bypass OS write denial even when the containing directory
// would permit rename, nor a native restriction introduced after the question.
#[test]
fn confirmed_access() {
    for restriction in ["disk", "buffer"] {
        let (fixture, mut buffers, id, _) = editable();
        let target = fixture.file("occupied.txt", b"keep");
        if restriction == "disk" {
            fs::set_permissions(&target, fs::Permissions::from_mode(0o444)).unwrap();
        }
        let confirmation = occupied_save(&buffers, id, &target);
        if restriction == "buffer" {
            buffers.get_mut(id).unwrap().set_read_only(true);
        }
        let before = facts(buffers.get(id).unwrap());
        let result = buffers.confirm_save(confirmation);
        assert_eq!(result.publication(), Publication::NotPublished);
        assert_eq!(
            result.error().unwrap().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(fs::read(&target).unwrap(), b"keep");
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        assert!(temporaries(&fixture.0).is_empty());
    }
}

// A changed/removed target invalidates its captured observation. A new live
// owner or a relocated source independently invalidates registry admission.
#[test]
fn stale_consent() {
    for change in ["rewrite", "remove", "replace", "owner", "association"] {
        let (fixture, mut buffers, id, original) = editable();
        let target = fixture.file("occupied.txt", b"keep");
        let confirmation = occupied_save(&buffers, id, &target);
        invalidate_consent(change, &mut buffers, id, &target);
        let before = facts(buffers.get(id).unwrap());
        let disk = fs::read(&target).ok();
        let result = buffers.confirm_save(confirmation);
        assert_eq!(result.publication(), Publication::NotPublished, "{change}");
        assert_eq!(facts(buffers.get(id).unwrap()), before);
        assert_eq!(fs::read(&target).ok(), disk);
        assert_eq!(fs::read(&original).unwrap(), b"original");
        assert!(temporaries(&fixture.0).is_empty());
    }
}

fn invalidate_consent(change: &str, buffers: &mut Buffers, id: BufferId, target: &Path) {
    match change {
        "rewrite" => fs::write(target, b"external").unwrap(),
        "remove" => fs::remove_file(target).unwrap(),
        "replace" => {
            let replacement = target.with_extension("replacement");
            fs::write(&replacement, b"keep").unwrap();
            fs::rename(replacement, target).unwrap();
        }
        "owner" => {
            assert_ne!(buffers.visit(target).unwrap(), id);
        }
        "association" => {
            let new = target.with_extension("new");
            let save = buffers.prepare_save(id, Some(&new)).unwrap();
            assert!(buffers.publish_save(save).is_complete());
        }
        _ => panic!("unknown change"),
    }
}

// Editing during a captured question cannot be acknowledged by its old bytes.
// Undo still reaches the captured clean logical state after actual publication.
#[test]
fn confirmed_capture() {
    let (fixture, mut buffers, id, _) = editable();
    let target = fixture.file("occupied.txt", b"keep");
    let confirmation = occupied_save(&buffers, id, &target);
    let captured = buffers.get(id).unwrap().state();
    insert_text(buffers.get_mut(id).unwrap(), "newer");
    assert!(buffers.confirm_save(confirmation).is_complete());
    assert_eq!(fs::read(&target).unwrap(), b"!original");
    let buffer = buffers.get_mut(id).unwrap();
    assert_eq!(text(buffer), "newer!original");
    assert!(buffer.is_modified());
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(buffer.state(), captured);
    assert!(!buffer.is_modified());
}

// An occupied symlink discloses its canonical referent. Retargeting that alias
// during the question cannot redirect consent or replace the symlink itself.
#[test]
fn resolved_consent() {
    let (fixture, mut buffers, id, original) = editable();
    let target = fixture.file("referent.txt", b"keep");
    let other = fixture.file("other.txt", b"other");
    let alias = fixture.0.join("alias.txt");
    symlink(&target, &alias).unwrap();
    let confirmation = occupied_save(&buffers, id, &alias);
    assert_eq!(confirmation.target(), target);
    assert!(confirmation.message().contains(target.to_str().unwrap()));
    assert!(confirmation.message().contains(alias.to_str().unwrap()));
    fs::remove_file(&alias).unwrap();
    symlink(&other, &alias).unwrap();
    assert!(buffers.confirm_save(confirmation).is_complete());
    assert_eq!(fs::read_link(&alias).unwrap(), other);
    assert_eq!(fs::read(&target).unwrap(), b"!original");
    assert_eq!(fs::read(&other).unwrap(), b"other");
    assert_eq!(fs::read(&original).unwrap(), b"original");
    assert_eq!(buffers.get(id).unwrap().file().unwrap().path(), target);
}
