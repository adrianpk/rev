use super::*;
use crate::{
    buffer::{
        Buffers,
        edit::{Observer, Point},
    },
    coordinates::Scalar,
    view::View,
};
use std::sync::{Arc, Mutex};

fn fixture() -> (Buffers, Windows, [BufferId; 2]) {
    let mut buffers = Buffers::new().unwrap();
    let a = buffers.create("first").unwrap();
    let b = buffers.create("second").unwrap();
    let windows = Windows::new(View::new(buffers.get(a).unwrap())).unwrap();
    (buffers, windows, [a, b])
}
fn prepare(buffers: &mut Buffers, windows: &Windows, ids: [BufferId; 2]) -> Atomic {
    let targets = ids.map(|id| {
        (
            id,
            vec![Replacement {
                source: buffers.get(id).unwrap().span(0..0).unwrap(),
                text: "+".into(),
            }],
        )
    });
    Atomic::prepare(buffers, windows, &targets).unwrap()
}
type BufferSnapshot = (
    Stamp,
    crate::buffer::State,
    String,
    usize,
    usize,
    Option<Scalar>,
);
type Snapshot = (Vec<BufferSnapshot>, Vec<Point>);

fn snapshot(buffers: &Buffers, windows: &Windows) -> Snapshot {
    (
        buffers
            .iter()
            .map(|b| {
                (
                    b.stamp(),
                    b.state(),
                    b.slice(&b.span(0..b.len_chars()).unwrap())
                        .unwrap()
                        .to_string(),
                    b.history().len(),
                    b.redo_history().len(),
                    b.mark(),
                )
            })
            .collect(),
        windows.iter().map(|(_, v)| v.point).collect(),
    )
}
// These transitions occur after every candidate/history/view reservation has
// completed. Final admission refuses all participants before the first commit.
#[test]
fn recheck_participants() {
    for change in ["revision", "readonly", "mark", "view", "closed", "location"] {
        let (mut buffers, mut windows, [a, b]) = fixture();
        let prepared = prepare(&mut buffers, &windows, [a, b]);
        match change {
            "revision" => {
                let buffer = buffers.get_mut(b).unwrap();
                let edit = buffer
                    .prepare(
                        &[Replacement {
                            source: buffer.span(0..0).unwrap(),
                            text: "user".into(),
                        }],
                        &[],
                        None,
                    )
                    .unwrap();
                buffer.publish(edit, &mut []).unwrap();
            }
            "readonly" => buffers.get_mut(b).unwrap().set_readonly(true),
            "mark" => buffers
                .get_mut(b)
                .unwrap()
                .set_mark(Some(Scalar(1)))
                .unwrap(),
            "view" => {
                windows
                    .get_mut(windows.selected_id())
                    .unwrap()
                    .point
                    .position = Scalar(1)
            }
            "closed" => {
                let kill = buffers.prepare_kill(b).unwrap();
                buffers.kill(kill).unwrap();
            }
            "location" => {
                let buffer = buffers.get(b).unwrap();
                let anchor = buffer
                    .locations
                    .lock()
                    .unwrap()
                    .capture(
                        buffer,
                        Point {
                            stamp: buffer.stamp(),
                            position: Scalar(1),
                        },
                    )
                    .unwrap();
                // Keep its native strong identity alive through final admission.
                let before = snapshot(&buffers, &windows);
                assert!(matches!(
                    prepared.publish(&mut buffers, &mut windows, || true),
                    Err(Failure::Edit(_))
                ));
                assert_eq!(snapshot(&buffers, &windows), before, "{change}");
                drop(anchor);
                continue;
            }
            _ => unreachable!(),
        }
        let before = snapshot(&buffers, &windows);
        assert!(
            matches!(
                prepared.publish(&mut buffers, &mut windows, || true),
                Err(Failure::Edit(_))
            ),
            "{change}"
        );
        assert_eq!(snapshot(&buffers, &windows), before, "{change}");
    }
}
// Cancellation is observed after full native preparation and final stamp checks.
// A canceled request cannot publish any participant or alter its redo branch.
#[test]
fn late_cancellation() {
    let (mut buffers, mut windows, ids) = fixture();
    // A canceled replacement must preserve an existing redo branch as well.
    let buffer = buffers.get_mut(ids[1]).unwrap();
    let edit = buffer
        .prepare(
            &[Replacement {
                source: buffer.span(0..0).unwrap(),
                text: "user".into(),
            }],
            &[],
            None,
        )
        .unwrap();
    buffer.publish(edit, &mut []).unwrap();
    buffer
        .navigate(crate::buffer::edit::History::Undo, &mut [], None)
        .unwrap();
    assert_eq!(buffer.redo_history().len(), 1);
    let prepared = prepare(&mut buffers, &windows, ids);
    let before = snapshot(&buffers, &windows);
    let mut called = false;
    assert!(matches!(
        prepared.publish(&mut buffers, &mut windows, || {
            called = true;
            false
        }),
        Err(Failure::Cancelled)
    ));
    assert!(called);
    assert_eq!(snapshot(&buffers, &windows), before);
}
struct Probe {
    other: crate::location::Anchor,
    seen: Arc<Mutex<Vec<Stamp>>>,
}
impl Observer for Probe {
    fn active(&self) -> bool {
        true
    }
    fn published(&self, _: Published) {
        self.seen
            .lock()
            .unwrap()
            .push(self.other.source().point.stamp);
    }
}
// An observation of the first participant must already see the second source's
// mapped live anchor at its committed revision, never an intermediate batch.
#[test]
fn observe_complete_batch() {
    let (mut buffers, mut windows, [a, b]) = fixture();
    let buffer = buffers.get(b).unwrap();
    let other = buffer
        .locations
        .lock()
        .unwrap()
        .capture(
            buffer,
            Point {
                stamp: buffer.stamp(),
                position: Scalar(1),
            },
        )
        .unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    buffers.get_mut(a).unwrap().observe(Some(Arc::new(Probe {
        other,
        seen: Arc::clone(&seen),
    })));
    let batch = prepare(&mut buffers, &windows, [a, b]);
    assert!(seen.lock().unwrap().is_empty());
    let stamps = batch.publish(&mut buffers, &mut windows, || true).unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![stamps[1]]);
    assert_eq!(windows.selected().point.stamp, stamps[0]);
}
