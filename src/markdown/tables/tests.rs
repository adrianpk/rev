use super::*;
use crate::buffer::{Buffers, edit::Point};
use crate::command::MarkdownCommand as C;

// Native caps refuse before source publication, including canceled action work,
// displayed cell width and output padding that would exceed the byte limit.
#[test]
fn work_bounds() {
    for (name, columns, body, valid) in [
        ("header only", 256, 0, true),
        ("last row", 1, 9998, true),
        ("column overflow", 257, 0, false),
        ("row overflow", 1, 9999, false),
        ("cell limit", 100, 998, true),
        ("cell overflow", 100, 999, false),
    ] {
        assert_eq!(observe::dimensions(columns, body).is_ok(), valid, "{name}");
    }
    for (name, value, cancel) in [
        ("wide cell", "界".repeat(2049), Cancellation::default()),
        ("canceled", "ok".into(), canceled()),
        (
            "expired",
            "ok".into(),
            Cancellation::default().bounded(std::time::Instant::now()),
        ),
    ] {
        let source = format!("|A|\n|---|\n|{value}|\n");
        let (mut buffers, id) = buffer(&source);
        let canonical = buffers.get_mut(id).unwrap();
        let model = super::super::Work::new(canonical.stamp(), Rope::from_str(&source))
            .run(&Cancellation::default())
            .unwrap();
        let point = Point {
            stamp: canonical.stamp(),
            position: Scalar(1),
        };
        let work = model.table_work(canonical, point).unwrap().unwrap();
        assert!(work.run(C::TableAlign, &cancel).is_err(), "{name}");
        assert!(canonical.history().is_empty(), "{name}");
    }
}

// GFM treats a pipe preceded by backslashes as cell source, including the
// parser's double-backslash case. No decoded inline event authorizes its loss.
#[test]
fn escaped_source() {
    for value in [r"a\|b", r"`a\|b`", r"a\\|b", r"a\\\|b", "&amp; **界é**"] {
        let source = format!("|A|B|\n|---|---|\n|{value}|tail|\n");
        let (buffers, id) = buffer(&source);
        let canonical = buffers.get(id).unwrap();
        let model = super::super::Work::new(canonical.stamp(), Rope::from_str(&source))
            .run(&Cancellation::default())
            .unwrap();
        let point = Point {
            stamp: canonical.stamp(),
            position: Scalar(1),
        };
        let work = model.table_work(canonical, point).unwrap().unwrap();
        let plan = work.run(C::TableAlign, &Cancellation::default()).unwrap();
        assert_eq!(plan.edits.len(), 1, "{value}");
        assert!(plan.edits[0].text.contains(value), "{value}");
        assert!(plan.edits[0].text.contains("tail"), "{value}");
    }
}

// Padding can exceed the byte budget even when the admitted source is small.
// Refusal happens before an edit plan or retained point destination escapes.
#[test]
fn output_bound() {
    let source = format!(
        "|A|\n|---|\n|{}|\n{}",
        "x".repeat(4096),
        "| |\n".repeat(2500)
    );
    let (buffers, id) = buffer(&source);
    let canonical = buffers.get(id).unwrap();
    let model = super::super::Work::new(canonical.stamp(), Rope::from_str(&source))
        .run(&Cancellation::default())
        .unwrap();
    let point = Point {
        stamp: canonical.stamp(),
        position: Scalar(1),
    };
    let work = model.table_work(canonical, point).unwrap().unwrap();
    let result = work.run(C::TableAlign, &Cancellation::default());
    assert!(result.err().unwrap().contains("output limit"));
    assert!(canonical.history().is_empty());
}

fn canceled() -> Cancellation {
    let cancel = Cancellation::default();
    cancel.cancel();
    cancel
}

fn buffer(source: &str) -> (Buffers, crate::buffer::BufferId) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    (buffers, id)
}
