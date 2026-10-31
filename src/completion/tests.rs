use super::*;
use crate::buffer::{Buffers, edit::Point};
use crate::coordinates::Scalar;
use crate::view::View;
use crate::window::Windows;

// Real retained Rust queries distinguish declarations from identifier uses and
// labels from literal edits; preparation leaves canonical text/history intact.
#[test]
fn declaration_identity() {
    let fixture = crate::fixture::Fixture::new();
    let path = fixture.write(
        "sample.rs",
        "fn config() {}\nfn cfg_value() {}\nfn main() { cfg }\n",
    );
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let stamp = buffer.stamp();
    let symbols = buffer
        .completion_symbols(std::time::Instant::now() + WORK_TIME)
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(46);
    let windows = Windows::new(view.clone()).unwrap();
    let completion =
        Completion::new(buffer, windows.selected_id(), 1, (view.point, symbols)).unwrap();
    assert_eq!(
        completion
            .candidates
            .iter()
            .map(Candidate::text)
            .collect::<Vec<_>>(),
        ["cfg_value", "config"]
    );
    assert_eq!(completion.candidates[0].label(), "cfg_value  function");
    let edit = completion.replacement(buffer).unwrap();
    assert_eq!(edit.text, "cfg_value");
    assert_eq!(buffer.slice(&edit.source).unwrap().to_string(), "cfg");
    assert_eq!(buffer.stamp(), stamp);
    assert!(buffer.history().is_empty());
}

// Foreign/stale points, invalid scalar ranges and split graphemes cannot admit
// a replacement, even if a candidate list was previously valid.
#[test]
fn target_admission() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("cafe\u{301}_value").unwrap();
    let other = buffers.create("").unwrap();
    let buffer = buffers.get(id).unwrap();
    for (name, point) in [
        (
            "foreign",
            Point {
                stamp: buffers.get(other).unwrap().stamp(),
                position: Scalar(0),
            },
        ),
        (
            "stale",
            Point {
                stamp: crate::buffer::Stamp {
                    revision: 0,
                    ..buffer.stamp()
                },
                position: Scalar(0),
            },
        ),
        (
            "bounds",
            Point {
                stamp: buffer.stamp(),
                position: Scalar(99),
            },
        ),
        (
            "grapheme",
            Point {
                stamp: buffer.stamp(),
                position: Scalar(4),
            },
        ),
    ] {
        assert!(target::at(buffer, point).is_err(), "{name}");
    }
    assert!(buffer.history().is_empty());
}

// An expired real query reports a work limit rather than a misleading empty
// result, and neither cancellation nor read-only refusal creates an edit.
#[test]
fn extraction_limits() {
    let fixture = crate::fixture::Fixture::new();
    let path = fixture.write("sample.rs", "fn sample() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.refresh_syntax();
    assert!(
        buffer
            .completion_symbols(std::time::Instant::now())
            .unwrap_err()
            .contains("limit")
    );
    buffer.set_readonly(true);
    assert!(
        buffer
            .completion_symbols(std::time::Instant::now() + WORK_TIME)
            .unwrap_err()
            .contains("read-only")
    );
    assert!(buffer.history().is_empty());
}
