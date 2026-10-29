use super::*;
use crate::{
    buffer::Buffers,
    fixture::Fixture,
    org::Cancellation,
    view::{Size, View},
};
use std::time::{Duration, Instant};
fn editor() -> (Fixture, Session) {
    let f = Fixture::new();
    let source = f.write("source.org", "* Task\nbody\n");
    f.write("input.bin", "payload");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 14,
        },
    )
    .unwrap();
    s.execute(Command::Org(C::AttachCopy));
    let end = Instant::now() + Duration::from_secs(2);
    loop {
        s.frame().unwrap();
        s.expire_input(Instant::now() + Duration::from_millis(10));
        if s.attachment.is_some() {
            break;
        }
        assert!(Instant::now() < end, "{}", s.echo());
        std::thread::yield_now();
    }
    (f, s)
}
// Actual filesystem publication can precede a stale, read-only or cancelled
// heading. The retained generated ID and destination still identify that effect.
#[test]
fn attachments_heading_failure() {
    for scenario in ["changed", "read-only", "cancelled"] {
        let (f, mut s) = editor();
        let context = s.attachment.as_ref().unwrap().context.clone();
        let p =
            Plan::prepare(context, "input.bin", Method::Copy, &Cancellation::default()).unwrap();
        s.attachment.as_mut().unwrap().stage = Stage::Running;
        let file = p.run(&Cancellation::default());
        assert_eq!(file.effect, Effect::Published);
        match scenario {
            "changed" => {
                s.insert('X');
            }
            "read-only" => {
                let id = s.view().point.stamp.buffer;
                s.buffers.get_mut(id).unwrap().set_readonly(true);
            }
            _ => {
                s.cancel_attachment(Command::KeyboardQuit);
            }
        }
        let history = s.buffer().unwrap().history().len();
        s.finish_attachment_effect(file);
        let result = s.attachment_results().front().unwrap();
        assert_eq!(result.heading, HeadingStatus::Unresolved, "{scenario}");
        assert!(result.source_error.is_some());
        assert!(result.file.id.is_some());
        assert_eq!(std::fs::read(&result.file.destination).unwrap(), b"payload");
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            "* Task\nbody\n"
        );
        assert!(s.attachment.is_none());
    }
}
