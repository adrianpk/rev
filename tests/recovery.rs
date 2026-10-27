#![forbid(unsafe_code)]

#[path = "recovery/quit.rs"]
mod quit;

#[path = "recovery/failures.rs"]
mod failures;

#[path = "recovery/publication.rs"]
mod publication;

#[path = "recovery/ingress.rs"]
mod ingress;

#[path = "recovery/configured.rs"]
mod configured;

#[path = "recovery/combined.rs"]
mod combined;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::edit::Replacement;
use rev::buffer::{Buffer, Buffers};
use rev::session::{Action, Session};
use rev::view::{Size, View};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("rev-recovery-{}-{id}", std::process::id()));
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

fn press(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn answer(session: &mut Session, character: char) -> Action {
    session.handle(press(KeyCode::Char(character)))
}

fn cancel(session: &mut Session) {
    assert_eq!(
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL
        ))),
        Action::Redraw
    );
}

fn path(session: &mut Session, path: &Path) -> Action {
    session.handle(Event::Paste(path.to_str().unwrap().into()));
    session.handle(press(KeyCode::Enter))
}

fn edit(buffer: &mut Buffer, text: &str) {
    let replacement = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: text.into(),
    };
    let prepared = buffer.prepare(&[replacement], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn session(buffers: Buffers) -> Session {
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 240,
            height: 12,
        },
    )
    .unwrap()
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    bytes: Vec<u8>,
    stamp: rev::buffer::Stamp,
    state: rev::buffer::State,
    location: Option<rev::file::Location>,
    name: String,
    mark: Option<rev::coordinates::Scalar>,
    active: bool,
    rectangle: bool,
    history: usize,
    redo: usize,
    modified: bool,
    read_only: bool,
    warning: Option<String>,
}

fn snapshot(buffer: &Buffer) -> Snapshot {
    let mut bytes = Vec::new();
    rev::file::write(buffer, &mut bytes).unwrap();
    Snapshot {
        bytes,
        stamp: buffer.stamp(),
        state: buffer.state(),
        location: buffer.file().cloned(),
        name: buffer.name().into(),
        mark: buffer.mark(),
        active: buffer.mark_active(),
        rectangle: buffer.rectangle_active(),
        history: buffer.history().len(),
        redo: buffer.redo_history().len(),
        modified: buffer.is_modified(),
        read_only: buffer.is_readonly(),
        warning: buffer.save_warning().map(str::to_owned),
    }
}

#[path = "recovery/paths.rs"]
mod paths;
