use super::*;
use crate::{
    buffer::{BufferId, Buffers, edit::Replacement},
    config::prepare,
    coordinates::Scalar,
    fixture::Fixture,
    org::{Cancellation, agenda::SourceScan},
    view::{Size, View},
};
fn editor() -> (Fixture, Session, BufferId, BufferId) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "org.refile.targets=['destination.org']"));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let source = buffers
        .visit(&fixture.write("source.org", "* Note\n* Stay\n"))
        .unwrap();
    let destination = buffers
        .visit(&fixture.write("destination.org", "* Target\n"))
        .unwrap();
    let view = View::new(buffers.get(source).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    (fixture, session, source, destination)
}
fn plan(session: &Session) -> Plan {
    let source = session.buffer().unwrap();
    let subtree = Subtree::at(source, session.view().point, &Cancellation::default()).unwrap();
    let selection = SourceSelection {
        paths: Vec::new(),
        buffers: session
            .buffers
            .iter()
            .filter(|b| b.is_org())
            .map(|b| b.id())
            .collect(),
    };
    let limits = &session.buffers.snapshot().settings().org.limits;
    let sources = SourceScan::new(&session.buffers, selection, limits)
        .unwrap()
        .run(&Cancellation::default());
    let targets = Targets::observe(sources, 4096, &Cancellation::default()).unwrap();
    let index = targets
        .headings
        .iter()
        .position(|h| h.label.ends_with("* Target"))
        .unwrap();
    targets
        .plan(index, subtree, false, &Cancellation::default())
        .unwrap()
}
fn snapshot(session: &Session, id: BufferId) -> (String, crate::buffer::Stamp, usize) {
    let buffer = session.buffers.get(id).unwrap();
    (
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        buffer.stamp(),
        buffer.history().len(),
    )
}
fn change(session: &mut Session, id: BufferId) {
    let buffer = session.buffers.get_mut(id).unwrap();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "changed\n".into(),
    };
    let points = session.windows.capture_points(buffer).unwrap();
    let prepared = buffer.prepare(&[edit], &points.values, None).unwrap();
    session.windows.publish(buffer, prepared, points).unwrap();
}
// Both participants are checked before any text or history is published. Real
// edits/read-only transitions after planning leave the other owner untouched.
#[test]
fn refile_atomic_rejection() {
    for (name, source, readonly) in [
        ("source revision", true, false),
        ("destination revision", false, false),
        ("source readonly", true, true),
        ("destination readonly", false, true),
    ] {
        let (_fixture, mut session, a, b) = editor();
        let plan = plan(&session);
        let selected = if source { a } else { b };
        if readonly {
            session
                .buffers
                .get_mut(selected)
                .unwrap()
                .set_readonly(true);
        } else {
            change(&mut session, selected);
        }
        let before = [snapshot(&session, a), snapshot(&session, b)];
        assert!(session.publish_refile(plan).is_err(), "{name}");
        assert_eq!(
            [snapshot(&session, a), snapshot(&session, b)],
            before,
            "{name}"
        );
        assert!(session.refile_results().is_empty());
    }
}
struct Fault(bool);
impl crate::file::Output for Fault {
    fn write(
        &mut self,
        save: &crate::buffer::Save,
        file: &mut std::fs::File,
    ) -> std::io::Result<()> {
        if self.0 {
            save.write(file)
        } else {
            Err(std::io::Error::other("injected write failure"))
        }
    }
    fn sync_directory(&mut self, directory: &std::fs::File) -> std::io::Result<()> {
        if self.0 {
            Err(std::io::Error::other("injected directory sync failure"))
        } else {
            directory.sync_all()
        }
    }
}
// Actual source save can succeed before target save rejects or has unresolved
// completion after real rename. Canonical target/undo survives, and save retry
// acknowledges that same insertion without repeating a move or inventing success.
#[test]
fn refile_partial_saves() {
    for (name, uncertain) in [("write rejection", false), ("completion unresolved", true)] {
        let (fixture, mut session, a, b) = editor();
        let plan = plan(&session);
        session.publish_refile(plan).unwrap();
        let outcome = session.buffers.save(a).unwrap();
        session.saved_state(
            session.buffers.get(a).unwrap().state(),
            &outcome,
            super::super::AfterSave::Stay,
        );
        let save = session.buffers.prepare_save(b, None).unwrap();
        let outcome = session.buffers.publish_with(save, &mut Fault(uncertain));
        session.saved_state(
            session.buffers.get(b).unwrap().state(),
            &outcome,
            super::super::AfterSave::Stay,
        );
        let results = session.refile_results();
        assert_eq!(results[0].status, Status::Saved, "{name}");
        assert_eq!(
            results[1].status,
            if uncertain {
                Status::CompletionUnresolved
            } else {
                Status::Rejected
            }
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
            "* Stay\n"
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
            if uncertain {
                "* Target\n** Note\n"
            } else {
                "* Target\n"
            }
        );
        assert_eq!(snapshot(&session, b).0, "* Target\n** Note\n");
        assert_eq!(session.buffers.get(b).unwrap().history().len(), 1);
        let outcome = session.buffers.save(b).unwrap();
        session.saved_state(
            session.buffers.get(b).unwrap().state(),
            &outcome,
            super::super::AfterSave::Stay,
        );
        assert!(
            session
                .refile_results()
                .iter()
                .all(|e| e.status == Status::Saved)
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org"))
                .unwrap()
                .matches("** Note")
                .count(),
            1
        );
        assert_eq!(session.view().point.position, Scalar(0));
    }
}

// A save captured before the move can publish old bytes while the new canonical
// state stays dirty. Its completion must never acknowledge the later refile pin.
#[test]
fn refile_captured_save() {
    for (name, uncertain) in [
        ("complete older save", false),
        ("unresolved older save", true),
    ] {
        let (fixture, mut session, _a, b) = editor();
        let save = session.buffers.prepare_save(b, None).unwrap();
        let old = save.state();
        let plan = plan(&session);
        session.publish_refile(plan).unwrap();
        let outcome = if uncertain {
            session.buffers.publish_with(save, &mut Fault(true))
        } else {
            session.buffers.publish_save(save)
        };
        session.saved_state(old, &outcome, super::super::AfterSave::Stay);
        assert_eq!(
            session.refile_results()[1].status,
            Status::ChangedInBuffer,
            "{name}"
        );
        assert_eq!(snapshot(&session, b).0, "* Target\n** Note\n");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
            "* Target\n"
        );
        assert!(session.buffers.get(b).unwrap().is_modified());
    }
}
