use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BufferCommand, Command, History, Split, WindowCommand};
use rev::display::Frame;
use rev::gutter::{Change, CurrentLine, Mode};
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::{ViewId, Windows};

use super::relative::{point, prefix};
use super::{Buffers, Composition, area, surface};

const SIZE: Size = Size {
    width: 80,
    height: 24,
};

fn submit(session: &mut Session, value: &str) {
    session.handle(Event::Paste(value.into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    submit(session, &line.to_string());
    assert_eq!(session.echo(), "");
}

fn editor(source: &str) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.bookmark_markers = false;
    view.gutter.apply(Change::Mode(Mode::Relative));
    Session::new(buffers, view, SIZE).unwrap()
}

fn fields(session: &mut Session, expected: &[(ViewId, usize)]) {
    let frame = session.frame().unwrap();
    let composition = Composition::new(session.windows(), area(0, 0, 80, 24), 0).unwrap();
    for &(id, current) in expected {
        let view = session.windows().get(id).unwrap();
        assert_eq!(view.cell(session.buffer().unwrap()).unwrap().row, current);
        let content = composition
            .panels()
            .iter()
            .find(|(v, _)| *v == id)
            .unwrap()
            .1
            .content();
        let row = usize::from(content.row()) + current - view.top;
        let cells = surface::draw(&frame);
        for (offset, label) in [
            (row - 1, "   1 │".to_owned()),
            (row, format!("{:>4} │", current + 1)),
        ] {
            let column = usize::from(content.column());
            let actual: String = (column..column + 6)
                .map(|c| cells[&(offset, c)].character)
                .collect();
            assert_eq!(actual, label, "{id:?}/{current}/{offset}");
        }
        assert!(cells[&(row, usize::from(content.column()) + 3)].bold);
    }
}

// Shared panes retain independent logical origins even when inactive. Real
// source publication maps both points, and history restores coherent labels.
#[test]
fn shared_origins() {
    for direction in [Split::Below, Split::Right] {
        origins_case(direction);
    }
}

fn origins_case(direction: Split) {
    let source = "\n".repeat(100);
    let mut session = editor(&source);
    goto(&mut session, 20);
    let original = session.windows().selected_id();
    let policy = session.view().gutter;
    session.execute(Command::Split(direction));
    let peer = session.windows().next_view().unwrap();
    assert_eq!(session.windows().get(peer).unwrap().gutter, policy);
    session.select_view(peer).unwrap();
    goto(&mut session, 60);
    fields(&mut session, &[(original, 19), (peer, 59)]);
    let peer_before = session.windows().get(peer).unwrap().clone();
    session.select_view(original).unwrap();
    fields(&mut session, &[(original, 19), (peer, 59)]);
    assert_eq!(session.windows().get(peer).unwrap(), &peer_before);
    session.handle(Event::Paste("\n".into()));
    fields(&mut session, &[(original, 20), (peer, 60)]);
    assert_eq!(session.buffer().unwrap().len_lines(), 102);
    session.execute(Command::History(History::Undo));
    fields(&mut session, &[(original, 19), (peer, 59)]);
    assert!(!session.buffer().unwrap().is_modified());
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, source.as_bytes());
    session.execute(Command::History(History::Redo));
    fields(&mut session, &[(original, 20), (peer, 60)]);
    for (_, view) in session.windows().iter() {
        assert_eq!(view.gutter, policy);
    }
    let retained = session.windows().get(peer).unwrap().clone();
    session.execute(Command::Window(WindowCommand::Delete));
    assert!(session.windows().get(original).is_err());
    assert_eq!(session.view(), &retained);
    fields(&mut session, &[(peer, 60)]);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Native policy updates act only on the addressed view. The composed cells
// distinguish modes/forms, preserve hidden mode, and reset startup provenance.
#[test]
fn divergent_modes() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"x\n".repeat(100)).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    point(&mut view, buffer, 41);
    view.top = 39;
    view.gutter.apply(Change::Mode(Mode::Relative));
    let original_view = view.clone();
    let mut windows = Windows::new(view).unwrap();
    let original = windows.selected_id();
    let peer = windows.split(rev::window::Split::Right, SIZE).unwrap();
    assert_eq!(windows.get(peer).unwrap(), &original_view);
    windows
        .change_gutter(peer, Change::Mode(Mode::Absolute))
        .unwrap();
    windows
        .change_gutter(peer, Change::CurrentLine(CurrentLine::Hidden))
        .unwrap();
    let frame = Frame::compose(&buffers, &windows, SIZE, "").unwrap();
    let left = ["   2 │", "   1 │", "  42 │", "   1 │", "   2 │"];
    let right = ["  40 │", "  41 │", "     │", "  43 │", "  44 │"];
    for (row, expected) in right.iter().enumerate() {
        assert_eq!(prefix(&frame, row), left[row]);
        assert_eq!(
            frame.rows()[row]
                .text()
                .chars()
                .skip(40)
                .take(6)
                .collect::<String>(),
            *expected
        );
    }
    assert_eq!(windows.get(original).unwrap(), &original_view);
    assert_eq!(windows.get(peer).unwrap().point, original_view.point);
    windows.change_gutter(peer, Change::Visible(false)).unwrap();
    let hidden = Frame::compose(&buffers, &windows, SIZE, "").unwrap();
    assert!(
        hidden.rows()[0]
            .text()
            .chars()
            .skip(40)
            .collect::<String>()
            .starts_with('x')
    );
    windows.change_gutter(peer, Change::Visible(true)).unwrap();
    assert_eq!(
        windows.get(peer).unwrap().gutter.effective().mode,
        Mode::Absolute
    );
    assert_eq!(
        Frame::compose(&buffers, &windows, SIZE, "").unwrap().rows()[0].text(),
        frame.rows()[0].text()
    );
    windows.change_gutter(peer, Change::Reset).unwrap();
    let reset = Frame::compose(&buffers, &windows, SIZE, "").unwrap();
    assert!(
        reset.rows()[2]
            .text()
            .chars()
            .skip(40)
            .collect::<String>()
            .starts_with("  42 │")
    );
    assert_eq!(windows.get(original).unwrap(), &original_view);
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// Real buffer/file ingress recomputes labels for new text while retaining the
// view's policy. Closing shared text rebinds peers without stale source state.
#[test]
fn rebound_labels() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("other.txt", "\u{feff}界\r\nlast\n");
    let mut session = editor("first\nsecond");
    let original_source = session.view().point.stamp.buffer;
    let policy = session.view().gutter;
    session.execute(Command::Split(Split::Right));
    let original = session.windows().selected_id();
    let peer = session.windows().next_view().unwrap();
    let name = session.buffer().unwrap().name().to_owned();
    session.select_view(peer).unwrap();
    session.load(&path).unwrap();
    let loaded_name = session.buffer().unwrap().name().to_owned();
    goto(&mut session, 3);
    let frame = Frame::project(
        session.buffer().unwrap(),
        session.view(),
        super::relative::SIZE,
        "",
    )
    .unwrap();
    assert_eq!(prefix(&frame, 0), "   2 │");
    assert_eq!(prefix(&frame, 2), "   3 │");
    assert_eq!(prefix(&frame, 3), "     │");
    assert_eq!(session.view().gutter, policy);
    assert_eq!(
        session.windows().get(original).unwrap().point.stamp.buffer,
        original_source
    );
    session.execute(Command::Buffer(BufferCommand::Switch));
    submit(&mut session, &name);
    assert_eq!(session.view().point.stamp.buffer, original_source);
    let frame = session.frame().unwrap();
    assert_eq!(prefix(&frame, 0), "   1 │");
    assert_eq!(session.view().gutter, policy);
    session.execute(Command::Buffer(BufferCommand::Kill));
    submit(&mut session, "");
    assert_eq!(session.echo(), "Buffer closed");
    for (_, view) in session.windows().iter() {
        assert_ne!(view.point.stamp.buffer, original_source);
        assert_eq!(view.gutter, policy);
    }
    let frame = session.frame().unwrap();
    assert_eq!(prefix(&frame, 0), "   1 │");
    session.execute(Command::Buffer(BufferCommand::Switch));
    submit(&mut session, &loaded_name);
    let frame = session.frame().unwrap();
    assert_eq!(prefix(&frame, 0), "   1 │");
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, "\u{feff}界\r\nlast\n".as_bytes());
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}
