use std::fs;

use rev::command::{History, Split};
use rev::config::prepare;

use super::*;
use crate::config_fixture::Fixture;

fn configured(source_width: usize, target_width: usize) -> (Session, Fixture) {
    let fixture = Fixture::new();
    let config = fixture.write("config.toml", &format!(
        "language.rust.tab_width = {source_width}\nlanguage.go.tab_width = {target_width}\nhistory.group_limit = 1\nhistory.payload_target_bytes = 1\nkill_ring.capacity = 1"
    ));
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    let source = fixture.write("source.rs", "\u{feff}a\tb\r\nc\td");
    fixture.write("target.go", "\u{feff}\tX\r\nq");
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(buffer.len_chars());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 14,
        },
    )
    .unwrap();
    (session, fixture)
}

// CFG10/CFG16-CFG17/UI01: a cross-width rectangle is one retained history
// group despite a one-byte soft target. Shared panes and explicit save preserve
// literal capture bytes, CRLF/BOM and saved-state truth in both history arms.
#[test]
fn formatted_transfer() {
    for (source_width, target_width) in [(4, 8), (8, 2), (32, 1)] {
        for capture in [RectangleCommand::Copy, RectangleCommand::Kill] {
            transfer_case(source_width, target_width, capture);
        }
    }
}

fn transfer_case(source_width: usize, target_width: usize, capture: RectangleCommand) {
    let (mut session, fixture) = configured(source_width, target_width);
    session.execute(Command::Split(Split::Right));
    let source_peer = session.windows().order()[1];
    session.execute(Command::Rectangle(capture));
    let rows = session.rectangle().unwrap().rows().to_vec();
    let expected_rows = if capture == RectangleCommand::Kill {
        vec!["a\tb".to_owned(), "c\td".to_owned()]
    } else {
        vec![
            format!("a{}b", " ".repeat(source_width - 1)),
            format!("c{}d", " ".repeat(source_width - 1)),
        ]
    };
    assert_eq!(rows, expected_rows);
    assert_eq!(
        session.buffer().unwrap().is_modified(),
        capture == RectangleCommand::Kill
    );
    let source = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    session.load(&fixture.0.join("target.go")).unwrap();
    at(&mut session, cell(0, target_width));
    session.execute(Command::Split(Split::Below));
    let target_id = session.buffer().unwrap().id();
    let peer = session
        .windows()
        .iter()
        .find(|(id, view)| {
            *id != session.windows().selected_id() && view.point.stamp.buffer == target_id
        })
        .unwrap()
        .0;
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    let expected = format!(
        "\t{}X\r\nq{}{}",
        rows[0],
        " ".repeat(target_width - 1),
        rows[1]
    );
    assert_eq!(text(&session), expected);
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
        2
    );
    assert_eq!(session.kill_ring().len(), 0);
    assert_eq!(
        fs::read(fixture.0.join("target.go")).unwrap(),
        b"\xef\xbb\xbf\tX\r\nq"
    );
    session.select_view(peer).unwrap();
    history_and_save(&mut session, &fixture, &expected);
    assert_eq!(session.rectangle().unwrap().rows(), rows);
    assert_eq!(session.buffer().unwrap().policy().tab_width, target_width);
    session.select_view(source_peer).unwrap();
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        source
    );
    assert_eq!(
        text(&session),
        if capture == RectangleCommand::Kill {
            "\r\n"
        } else {
            "a\tb\r\nc\td"
        }
    );
    assert_eq!(
        fs::read(fixture.0.join("source.rs")).unwrap(),
        b"\xef\xbb\xbfa\tb\r\nc\td"
    );
}

fn history_and_save(session: &mut Session, fixture: &Fixture, expected: &str) {
    session.execute(Command::History(History::Undo));
    assert_eq!(text(session), "\tX\r\nq");
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(session), expected);
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::SaveBuffer);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        fs::read(fixture.0.join("target.go")).unwrap(),
        format!("\u{feff}{expected}").as_bytes()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(session), "\tX\r\nq");
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(session), expected);
    assert!(!session.buffer().unwrap().is_modified());
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.buffer().unwrap().newline(), "\r\n");
    session.frame().unwrap();
    for (_, view) in session.windows().iter() {
        if view.point.stamp.buffer == session.buffer().unwrap().id() {
            assert_eq!(view.point.stamp, session.buffer().unwrap().stamp());
            assert!(
                Coordinates::new(session.buffer().unwrap())
                    .unwrap()
                    .is_boundary(view.point.position)
                    .unwrap()
            );
        }
    }
}
