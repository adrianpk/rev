use rev::buffer::{
    Buffers,
    edit::{History, Point, Replacement},
};
use rev::command::{Command, RectangleCommand};
use rev::config::prepare;
use rev::coordinates::Scalar;
use rev::session::{Action, Session};
use rev::view::{Size, View};

// A multi-row rectangle may exceed the entire soft payload target and still
// undoes/redoes as one unit. Older groups evict without changing saved identity.
#[test]
fn compound_retention() {
    for config in [
        "history.group_limit = 1",
        "history.payload_target_bytes = 1",
    ] {
        let fixture = super::config_fixture::Fixture::new();
        let startup = prepare(&fixture.write("config.toml", config));
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.create("0123\r\n4567\r\n89ab").unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let edit = Replacement {
            source: buffer.span(0..1).unwrap(),
            text: "X".into(),
        };
        let prepared = buffer.prepare(&[edit], &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        buffer.mark_saved(buffer.stamp()).unwrap();
        buffer
            .toggle_rectangle(Point {
                stamp: buffer.stamp(),
                position: Scalar(1),
            })
            .unwrap();
        let mark = buffer.mark();
        let mut view = View::new(buffer);
        view.point.position = Scalar(15);
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 40,
                height: 8,
            },
        )
        .unwrap();
        assert_eq!(
            session.execute(Command::Rectangle(RectangleCommand::Kill)),
            Action::Redraw
        );
        assert_eq!(super::text(&session), "X3\r\n47\r\n8b");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(
            session
                .buffer()
                .unwrap()
                .history()
                .back()
                .unwrap()
                .changes()
                .len(),
            3
        );
        assert_eq!(session.rectangle().unwrap().rows(), ["12", "56", "9a"]);
        assert_eq!(session.kill_ring().len(), 0);
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(&session), "X123\r\n4567\r\n89ab");
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert_eq!(session.view().point.position, Scalar(15));
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Undo));
        assert_eq!(super::text(&session), "X123\r\n4567\r\n89ab");
        session.execute(Command::History(History::Redo));
        assert_eq!(super::text(&session), "X3\r\n47\r\n8b");
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(session.rectangle().unwrap().rows(), ["12", "56", "9a"]);
    }
}
