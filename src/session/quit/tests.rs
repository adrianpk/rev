use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

use crate::buffer::Buffers;
use crate::buffer::edit::Replacement;
use crate::file::{Output, Publication};
use crate::view::{Size, View};

use super::{Action, AfterSave, Pending, Session};

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

struct Uncertain;

impl Output for Uncertain {
    fn sync_directory(&mut self, _directory: &File) -> io::Result<()> {
        Err(io::Error::other("directory sync interrupted"))
    }
}

// Real writes/replacement reach the shared lifecycle continuation. Only the
// unavailable directory-sync error is injected; this does not prove durability
// under a physical filesystem failure. Published bytes cannot count as exit.
#[test]
fn unresolved_exit() {
    let fixture = Fixture(std::env::temp_dir().join(format!("rev-quit-{}", std::process::id())));
    fs::create_dir(&fixture.0).unwrap();
    let path = fixture.0.join("target.txt");
    fs::write(&path, b"base").unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let replacement = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "new ".into(),
    };
    let prepared = buffer.prepare(&[replacement], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 240,
            height: 12,
        },
    )
    .unwrap();
    assert_eq!(session.quit(), Action::Redraw);
    let Some(Pending::Quit(quit)) = session.pending.take() else {
        panic!("quit decision")
    };
    session.input.cancel();
    let save = session.buffers.prepare_save(id, None).unwrap();
    let outcome = session.buffers.publish_with(save, &mut Uncertain);
    assert_eq!(outcome.publication(), Publication::Uncertain);
    assert_eq!(fs::read(&path).unwrap(), b"new base");
    assert_eq!(
        session.saved(&outcome, AfterSave::Quit(quit)),
        Action::Redraw
    );
    assert!(session.echo().contains("published; completion unresolved"));
    assert!(session.buffer().unwrap().is_modified());
    assert!(session.buffer().unwrap().save_warning().is_some());
    assert_eq!(session.quit(), Action::Redraw);
    session.cancel_quit();
    assert_eq!(session.save_buffer(), Action::Redraw);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.quit(), Action::Exit);
}
