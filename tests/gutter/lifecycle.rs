use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BufferCommand, Command, Split};
use rev::gutter::{Change, Mode, Policy};
use rev::session::Session;
use rev::view::{Size, View};

use super::{Buffers, Composition, area};

pub(super) fn editor(source: &str) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.gutter = Policy::new(super::policy::startup());
    view.gutter.apply(Change::Visible(true));
    view.gutter.apply(Change::Mode(Mode::Absolute));
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

fn accept(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Actual file ingress preserves view policy, while an existing visit or failed
// visit retains the complete active view and never disturbs its shared peer.
#[test]
fn file_rebind() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("document.txt", "\u{feff}x\t界\r\n");
    let mut session = editor("original");
    session.execute(Command::Split(Split::Below));
    let original = session.windows().selected_id();
    let peer = session.windows().next_view().unwrap();
    let peer_before = session.windows().get(peer).unwrap().clone();
    let policy = session.view().gutter;
    let old_source = session.view().point.stamp;
    session.load(&path).unwrap();
    assert_ne!(session.view().point.stamp.buffer, old_source.buffer);
    assert_eq!(session.view().gutter, policy);
    assert_eq!(session.windows().get(peer).unwrap(), &peer_before);
    session.execute(Command::Motion(rev::command::Motion::BufferEnd));
    let loaded = session.view().clone();
    session.load(&path).unwrap();
    assert_eq!(session.view(), &loaded);
    assert!(session.load(&fixture.0).is_err());
    assert_eq!(session.view(), &loaded);
    assert_eq!(session.windows().selected_id(), original);
    assert_eq!(session.windows().get(peer).unwrap(), &peer_before);
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, "\u{feff}x\t界\r\n".as_bytes());
    assert!(!session.buffer().unwrap().is_modified());
    assert!(session.buffer().unwrap().history().is_empty());
}

// Buffer switching keeps only the existing view policy while recomputing source
// identity/width. Closing shared text rebinds every affected view with no stale
// source reference, and another view's policy and source are retained.
#[test]
fn buffer_lifetime() {
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.create("界\tfirst\r\nlast").unwrap();
    let second = buffers.create(&"\n".repeat(9_999)).unwrap();
    let second_name = buffers.get(second).unwrap().name().to_owned();
    let mut view = View::new(buffers.get(first).unwrap());
    view.gutter = Policy::new(super::policy::startup());
    view.gutter.apply(Change::Visible(true));
    let policy = view.gutter;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Right));
    let original = session.windows().selected_id();
    let peer = session.windows().next_view().unwrap();
    session.select_view(peer).unwrap();
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(second_name));
    accept(&mut session);
    assert_eq!(session.buffer().unwrap().id(), second);
    assert_eq!(session.view().gutter, policy);
    assert_eq!(
        session.windows().get(original).unwrap().point.stamp.buffer,
        first
    );
    let composition = Composition::new(session.windows(), area(0, 0, 80, 24), 0).unwrap();
    let projection = composition
        .gutter(session.windows(), session.buffer().unwrap(), peer)
        .unwrap()
        .unwrap();
    assert_eq!(projection.lines, 10_000);
    assert_eq!(projection.geometry.digits(), 5);
    session.select_view(original).unwrap();
    session.execute(Command::Split(Split::Below));
    session.execute(Command::Buffer(BufferCommand::Kill));
    accept(&mut session);
    assert_eq!(session.echo(), "Buffer closed");
    for (_, live) in session.windows().iter() {
        assert_ne!(live.point.stamp.buffer, first);
        assert_eq!(live.gutter, policy);
    }
    assert_eq!(
        session.windows().get(peer).unwrap().point.stamp.buffer,
        second
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}
