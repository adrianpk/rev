use super::*;
use crate::{buffer::Buffers, coordinates::Scalar};
fn snapshot(source: &str) -> Snapshot {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    Snapshot::new(
        buffer,
        Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        },
    )
    .unwrap()
}
// Owned source snapshots honor cancellation and expired native deadlines before
// any plan can authorize publication. No test clock or source identity is forged.
#[test]
fn document_job_bounds() {
    for scenario in ["cancelled", "expired"] {
        let cancel = Cancellation::default();
        let cancellation = if scenario == "cancelled" {
            cancel.cancel();
            cancel
        } else {
            cancel.bounded(std::time::Instant::now())
        };
        assert!(
            snapshot("Read [fn:x:body].")
                .run(C::FootnoteRenumber, Argument::None, &cancellation)
                .is_err(),
            "{scenario}"
        );
    }
    let source = "x".repeat(Limits::default().bytes + 1);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get(id).unwrap();
    assert!(
        Snapshot::new(
            buffer,
            Point {
                stamp: buffer.stamp(),
                position: Scalar(0)
            }
        )
        .is_err()
    );
}
// Global note/target operations reject complete observations above their record
// budget rather than acting on a prefix, while real source snapshots stay intact.
#[test]
fn document_record_bounds() {
    for (name, source, expected) in [
        ("notes", "Read [fn:x:body] ".repeat(4097), "4096 records"),
        (
            "radio",
            "<<<target>>> ".repeat(4097),
            "native record/name profile",
        ),
    ] {
        let result = snapshot(&source).run(
            C::FootnoteRenumber,
            Argument::None,
            &Cancellation::default(),
        );
        assert!(
            matches!(result, Err(ref error) if error.contains(expected)),
            "{name}"
        );
    }
}
