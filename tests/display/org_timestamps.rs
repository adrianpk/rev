use super::*;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use rev::buffer::Mode;
use rev::command::{Command, History, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use rev::session::Session;
use std::time::{Duration, Instant, UNIX_EPOCH};

fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    let env = ZoneEnvironment {
        localtime: "/unused-localtime".into(),
        zoneinfo: "/unused-zoneinfo".into(),
        tz: None,
    };
    ZonedOperation::capture(&ZoneSelection::Utc, &env, || {
        Ok(UNIX_EPOCH + Duration::from_secs(1793183400))
    })
}
fn native() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界e\u{301}\r\nlast").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let view = View::new(buffer);
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock)
}
fn settle(session: &mut Session) {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(Instant::now() < until, "owned calendar work did not finish");
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn press(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    settle(session);
}
fn click(session: &mut Session, column: u16, row: u16) {
    session.handle(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
    settle(session);
}
fn content(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// The real transient surface projects three months only when they fit. Tiny
// and suspended frames clip safely without moving source anchors or history.
#[test]
fn timestamp_calendar_geometry() {
    let mut session = native();
    let origin = session.view().clone();
    let source = content(&session);
    session.execute(Command::Org(C::Timestamp));
    settle(&mut session);
    for (width, height) in [(100, 12), (29, 12), (2, 2), (0, 0), (100, 12)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        calendar_bounds(&frame, Size { width, height });
        assert_eq!(session.view(), &origin);
        let rows = frame
            .rows()
            .iter()
            .map(|r| r.text())
            .collect::<Vec<_>>()
            .join("\n");
        if width >= 88 {
            assert!(rows.contains("September 2026"));
            assert!(rows.contains("October 2026"));
            assert!(rows.contains("November 2026"));
            assert!(rows.contains("[28]"));
        }
        if width == 29 {
            assert!(rows.contains("October 2026"));
            assert!(!rows.contains("September 2026"));
        }
    }
    session.execute(Command::KeyboardQuit);
    assert_eq!(content(&session), source);
    assert_eq!(session.view(), &origin);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Only visible day cells select a date. Header, inter-month gap and unused
// right cells cannot change the draft; a valid mouse selection commits on RET.
#[test]
fn timestamp_calendar_pointer() {
    let mut session = native();
    let source = content(&session);
    session.execute(Command::Org(C::Timestamp));
    settle(&mut session);
    let frame = session.frame().unwrap();
    let row = frame
        .rows()
        .iter()
        .find(|r| r.band() == Band::Completion)
        .unwrap()
        .row();
    for (x, y) in [(51, row), (29, row + 6), (95, row + 6)] {
        click(&mut session, x, y);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("[28]"))
        );
    }
    click(&mut session, 51, row + 6);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("[30]"))
    );
    assert_eq!(content(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    press(&mut session, KeyCode::Enter);
    assert_eq!(content(&session), format!("<2026-10-30 Fri>{source}"));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(content(&session), source);
}

fn calendar_bounds(frame: &Frame, size: Size) {
    for row in frame.rows() {
        assert!(row.row() < size.height);
        assert_eq!(row.text().width(), usize::from(size.width));
        assert!(!row.text().chars().any(char::is_control));
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < size.width);
        assert!(cursor.row < size.height);
    }
}
