use super::*;
use crate::{
    buffer::{Buffers, edit::Point},
    coordinates::Scalar,
    file::{Publication, Rename},
    fixture::Fixture,
};

fn capture(buffers: &Buffers, id: crate::buffer::BufferId) {
    let buffer = buffers.get(id).unwrap();
    buffers
        .bookmarks
        .lock()
        .unwrap()
        .set(
            "here",
            buffer.file().unwrap().path().into(),
            Point {
                stamp: buffer.stamp(),
                position: Scalar(3),
            },
            true,
            Context::capture(buffer, Scalar(3)).unwrap(),
        )
        .unwrap();
}

// Canonical file and directory renames update closed records as well as live
// associations. An occupied destination preserves the complete collection.
#[test]
fn renamed_records() {
    for directory in [false, true] {
        let f = Fixture::new();
        let old = f.write("old/sub/source", "retained");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&old).unwrap();
        capture(&buffers, id);
        let kill = buffers.prepare_kill(id).unwrap();
        buffers.kill(kill).unwrap();
        let prior = buffers.bookmarks().get("here").unwrap().clone();
        let source = if directory {
            f.0.join("old")
        } else {
            old.clone()
        };
        let target = if directory {
            f.0.join("moved/sub/source")
        } else {
            f.0.join("old/sub/moved")
        };
        f.write(
            if directory {
                "occupied/keep"
            } else {
                "old/sub/occupied"
            },
            "occupied",
        );
        assert!(
            buffers
                .rename(Rename::capture(&source).unwrap(), "occupied")
                .is_err()
        );
        assert_eq!(buffers.bookmarks().get("here"), Some(&prior));
        buffers
            .rename(Rename::capture(&source).unwrap(), "moved")
            .unwrap();
        let record = buffers.bookmarks().get("here").unwrap().clone();
        assert_eq!(record.path(), target);
        assert_eq!(record.id(), prior.id());
        assert_eq!(record.context(), prior.context());
        assert_eq!(record.resolution(), Resolution::Unresolved);
        assert!(!old.exists());
        assert_eq!(std::fs::read_to_string(target).unwrap(), "retained");
    }
}

// A real destination disappearing after save capture rejects staging. Canonical
// source identity, bookmark path/context and disk source all remain unchanged.
#[test]
fn rejected_save() {
    let f = Fixture::new();
    let source = f.write("source", "retained");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&source).unwrap();
    capture(&buffers, id);
    let before = buffers.bookmarks().get("here").unwrap().clone();
    let directory = f.0.join("destination");
    std::fs::create_dir(&directory).unwrap();
    let save = buffers
        .prepare_save(id, Some(&directory.join("new")))
        .unwrap();
    std::fs::remove_dir(directory).unwrap();
    let outcome = buffers.publish_save(save);
    assert_eq!(outcome.publication(), Publication::NotPublished);
    assert!(outcome.error().is_some());
    assert_eq!(buffers.bookmarks().get("here"), Some(&before));
    assert_eq!(buffers.get(id).unwrap().file().unwrap().path(), source);
    assert_eq!(std::fs::read_to_string(source).unwrap(), "retained");
}
