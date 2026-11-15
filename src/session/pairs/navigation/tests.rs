use crate::buffer::Buffers;
use crate::command::Command;
use crate::coordinates::Scalar;
use crate::session::Session;
use crate::view::{Size, View};
use std::time::{Duration, Instant};

fn editor() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&format!("({})", "a ".repeat(4096))).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

// External source owners may change point, mark or focus while native work is
// pending. Resume must retire its ticket and publish neither the old target nor
// a candidate cloned from the old view.
#[test]
fn mate_stale() {
    for change in ["point", "mark", "focus"] {
        stale_navigation(Command::MatchingDelimiter, change);
    }
}

// Group work uses the same captured point/mark/focus owner and retirement path.
#[test]
fn groups_stale() {
    for change in ["point", "mark", "focus"] {
        stale_navigation(Command::ForwardPair, change);
    }
}

fn stale_navigation(command: Command, change: &str) {
    let mut session = editor();
    session.execute(command);
    let ticket = session.layout.navigation.as_ref().unwrap().ticket;
    match change {
        "point" => session.windows.selected_mut().point.position = Scalar(1),
        "mark" => session
            .buffers
            .get_mut(ticket.stamp.buffer)
            .unwrap()
            .set_mark(Some(Scalar(1)))
            .unwrap(),
        "focus" => session.input.argument("Value: ", "").unwrap(),
        _ => unreachable!(),
    }
    let point = session.view().point;
    let mark = session.buffer().unwrap().mark();
    let deadline = Instant::now() + Duration::from_secs(10);
    while session.layout_active() {
        assert!(Instant::now() < deadline, "{change}");
        session.expire_input(Instant::now());
    }
    assert_eq!(session.view().point, point, "{change}");
    assert_eq!(session.buffer().unwrap().mark(), mark, "{change}");
    assert!(
        session.buffer().unwrap().pair_stats(ticket).is_none(),
        "{change}"
    );
    assert!(session.buffer().unwrap().history().is_empty(), "{change}");
}
