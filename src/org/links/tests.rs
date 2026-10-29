use super::*;
use crate::{buffer::Buffers, coordinates::Scalar, fixture::Fixture, org::agenda::SourceSelection};
fn navigation(buffers: &Buffers, id: crate::buffer::BufferId) -> Navigation {
    let b = buffers.get(id).unwrap();
    let point = Point {
        stamp: b.stamp(),
        position: Scalar(0),
    };
    let cancel = Cancellation::default();
    let Output::Follow(target) = Snapshot::new(b, point, &crate::config::Org::default())
        .unwrap()
        .observe(crate::command::OrgCommand::OpenLink, false, &cancel)
        .unwrap()
    else {
        panic!("expected link");
    };
    Resolve::new(
        buffers,
        b.stamp(),
        target,
        SourceSelection {
            paths: Vec::new(),
            buffers: Vec::new(),
        },
    )
    .unwrap()
    .run(&cancel)
    .unwrap()
}
// A real destination observation is rechecked before registry publication.
// Changed disk identity/bytes or a changed canonical buffer rejects the visit.
#[test]
fn links_visit_stale() {
    for scenario in ["disk bytes", "disk identity", "canonical buffer"] {
        let f = Fixture::new();
        let target = f.write("destination.org", "* Target\n");
        let source = f.write("source.org", "[[file:destination.org::*Target]]\n");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&source).unwrap();
        let dest = (scenario == "canonical buffer").then(|| buffers.visit(&target).unwrap());
        let observed = navigation(&buffers, id);
        let count = buffers.iter().len();
        match dest {
            Some(dest) => {
                let b = buffers.get_mut(dest).unwrap();
                let edit = crate::buffer::edit::Replacement {
                    source: b.span(0..0).unwrap(),
                    text: "changed\n".into(),
                };
                let prepared = b.prepare(&[edit], &[], None).unwrap();
                b.publish(prepared, &mut []).unwrap();
            }
            None if scenario == "disk identity" => {
                std::fs::rename(f.write("replacement.org", "* Target\n"), &target).unwrap();
            }
            None => std::fs::write(&target, "* Target\nchanged\n").unwrap(),
        }
        assert!(observed.prepare(&buffers).is_err(), "{scenario}");
        assert_eq!(
            buffers.iter().len(),
            count,
            "{scenario}: stale visit published a buffer"
        );
        assert_eq!(buffers.get(id).unwrap().len_chars(), 34);
    }
}
// Recognition remains complete only within the native record and byte profile.
// An over-limit source or cancelled observation cannot certify link targets.
#[test]
fn links_observation_bounds() {
    let policy = crate::config::Org::default();
    let cancel = Cancellation::default();
    let text = Rope::from_str(&"[[*Target]]\n".repeat(4096));
    assert!(Document::observe(text, &policy, &cancel).is_ok());
    let text = Rope::from_str(&"[[*Target]]\n".repeat(4097));
    assert!(Document::observe(text, &policy, &cancel).is_err());
    cancel.cancel();
    assert!(Document::observe(Rope::from_str("[[*Target]]\n"), &policy, &cancel).is_err());
}
