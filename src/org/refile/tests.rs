use super::*;
use crate::{
    buffer::Buffers,
    coordinates::Scalar,
    fixture::Fixture,
    org::agenda::{SourceScan, SourceSelection},
};
fn targets(buffers: &Buffers) -> Targets {
    let selection = SourceSelection {
        paths: Vec::new(),
        buffers: buffers
            .iter()
            .filter(|b| b.is_org())
            .map(|b| b.id())
            .collect(),
    };
    let sources = SourceScan::new(buffers, selection, &crate::config::OrgLimits::default())
        .unwrap()
        .run(&Cancellation::default());
    Targets::observe(sources, 4096, &Cancellation::default()).unwrap()
}
fn value(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
// One checked plan relevels the complete entry, retains literal block content
// and BOM/newlines, and never writes either actual file during preparation.
#[test]
fn refile_plan() {
    for copy in [false, true] {
        let fixture = Fixture::new();
        let source = "* Note\n:PROPERTIES:\n:ID: keep\n:END:\n** Child\n#+begin_example\n,* literal\n#+end_example\n* Stay\n";
        let dest = "\u{feff}* Projects\r\nbody EOF";
        let from = fixture.write("source.org", source);
        let to = fixture.write("destination.org", dest);
        let mut buffers = Buffers::new().unwrap();
        let a = buffers.visit(&from).unwrap();
        let b = buffers.visit(&to).unwrap();
        let subtree = Subtree::at(
            buffers.get(a).unwrap(),
            Point {
                stamp: buffers.get(a).unwrap().stamp(),
                position: Scalar(0),
            },
            &Cancellation::default(),
        )
        .unwrap();
        let targets = targets(&buffers);
        let target = targets
            .headings
            .iter()
            .position(|h| h.label.ends_with("* Projects"))
            .unwrap();
        let plan = targets
            .plan(target, subtree, copy, &Cancellation::default())
            .unwrap();
        let edits = plan
            .edits(buffers.get(a).unwrap(), buffers.get(b).unwrap())
            .unwrap();
        for id in [a, b] {
            let changes: Vec<_> = edits
                .iter()
                .filter(|e| e.source.stamp.buffer == id)
                .cloned()
                .collect();
            if changes.is_empty() {
                continue;
            }
            let buffer = buffers.get_mut(id).unwrap();
            let prepared = buffer.prepare(&changes, &[], None).unwrap();
            buffer.publish(prepared, &mut []).unwrap();
        }
        assert_eq!(
            value(buffers.get(a).unwrap()),
            if copy { source } else { "* Stay\n" }
        );
        assert_eq!(
            value(buffers.get(b).unwrap()),
            "* Projects\r\nbody EOF\r\n** Note\r\n:PROPERTIES:\r\n:ID: keep\r\n:END:\r\n*** Child\r\n#+begin_example\r\n,* literal\r\n#+end_example\r\n"
        );
        assert!(buffers.get(b).unwrap().has_bom());
        assert_eq!(std::fs::read_to_string(&from).unwrap(), source);
        assert_eq!(std::fs::read_to_string(&to).unwrap(), dest);
        assert_eq!(buffers.get(a).unwrap().history().len(), usize::from(!copy));
        assert_eq!(buffers.get(b).unwrap().history().len(), 1);
    }
}
// Same-buffer boundary insertion becomes a single replacement, while moving a
// parent into a descendant and using a stale same-source observation both fail.
#[test]
fn refile_local() {
    let fixture = Fixture::new();
    let path = fixture.write("source.org", "* Parent\n* Note\n** Child\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let b = buffers.get(id).unwrap();
    let subtree = Subtree::at(
        b,
        Point {
            stamp: b.stamp(),
            position: Scalar(9),
        },
        &Cancellation::default(),
    )
    .unwrap();
    let targets = targets(&buffers);
    let plan = targets
        .plan(0, subtree.clone(), false, &Cancellation::default())
        .unwrap();
    let changes = plan.edits(b, b).unwrap();
    assert_eq!(changes.len(), 1);
    assert!(
        targets
            .plan(2, subtree, false, &Cancellation::default())
            .is_err()
    );
    let b = buffers.get_mut(id).unwrap();
    let prepared = b.prepare(&changes, &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    assert_eq!(value(b), "* Parent\n** Note\n*** Child\n");
    let current = Subtree::at(
        b,
        Point {
            stamp: b.stamp(),
            position: Scalar(9),
        },
        &Cancellation::default(),
    )
    .unwrap();
    assert!(
        targets
            .plan(0, current, false, &Cancellation::default())
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "* Parent\n* Note\n** Child\n"
    );
}
