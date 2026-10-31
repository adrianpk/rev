use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, History, Motion, OrgCommand as Org};
use rev::coordinates::Scalar;
use rev::gutter::Change;
use rev::session::Session;
use rev::view::{Size, View};

fn editor(source: &str, width: u16, height: u16) -> (super::fixture::Fixture, Session) {
    let fixture = super::fixture::Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.gutter.apply(Change::Visible(false));
    (
        fixture,
        Session::new(buffers, view, Size { width, height }).unwrap(),
    )
}

fn settle(session: &mut Session) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        let now = Instant::now();
        if session.remaining_input(now).is_none() {
            break;
        }
        assert!(now < end, "{}", session.echo());
        session.expire_input(now + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn action(session: &mut Session, command: Org) {
    session.execute(Command::Org(command));
    settle(session);
}

fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    session.handle(Event::Paste(line.to_string()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Native fold requests keep canonical heading/body identities across reflow.
// Visible headings, table rows and literal source lines all follow buffer wrap;
// hidden subtree bytes remain absent under either pane width or wrap choice.
#[test]
fn folded_source() {
    let source = "* A heading with several words\nHIDDEN body with words\n** Child\nhidden child body\n* Peer heading with words\n| alpha beta gamma | delta |\n#+begin_src text\nliteral source with many words\n#+end_src\n";
    let (_fixture, mut session) = editor(source, 12, 24);
    action(&mut session, Org::HideSubtree);
    let stamp = session.buffer().unwrap().stamp();
    let visible = session.view().visible_lines(session.buffer().unwrap());
    assert_eq!(visible, 7);
    for width in [8, 16, 32] {
        session.handle(Event::Resize(width, 24));
        let b = session.buffer().unwrap();
        let fragments = session.view().wrapped_rows(b, width, 64).unwrap();
        assert!(fragments.iter().all(|f| !matches!(f.line, 1..=3)));
        for line in [0, 4, 5, 7] {
            assert!(fragments.iter().any(|f| f.line == line), "{width}/{line}");
            if width == 8 {
                assert!(fragments.iter().filter(|f| f.line == line).count() > 1);
            }
        }
        assert_eq!(session.view().point.position, Scalar(0));
        assert_eq!(session.view().visible_lines(b), visible);
        session.frame().unwrap();
    }
    session.execute(Command::VisualLineMode);
    session.frame().unwrap();
    assert_eq!(
        session.view().visible_lines(session.buffer().unwrap()),
        visible
    );
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Table-special commands use logical fields even when a field crosses several
// display fragments. Explicit alignment is the sole canonical edit and undo
// restores its exact source, including tabs, Unicode and CRLF separators.
#[test]
fn table_fields() {
    let source = "| alpha beta gamma | 界e\u{301}\tZ |\r\n| x | second field |\r\n";
    let (_fixture, mut session) = editor(source, 9, 18);
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Motion(Motion::ForwardChar));
    action(&mut session, Org::TableNextField);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        0
    );
    assert_eq!(session.view().point.position, Scalar(21));
    action(&mut session, Org::TableBeginningField);
    assert_eq!(session.view().point.position, Scalar(21));
    assert_eq!(session.buffer().unwrap().history().len(), 1); // Native field motion aligns first.
    let aligned = text(&session);
    assert!(aligned.contains("alpha beta gamma"));
    assert!(aligned.contains("界e\u{301}\tZ"));
    action(&mut session, Org::TablePreviousField);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(text(&session), aligned);
    assert!(session.frame().unwrap().cursor().is_some());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}

// Source drafts and their parent both wrap normally. Native exit publishes one
// checked source edit; the block delimiters and unrelated text stay canonical.
#[test]
fn source_draft() {
    let source =
        "* Heading\r\n#+begin_src text\r\na long source body with words\r\n#+end_src\r\nTail";
    let (_fixture, mut session) = editor(source, 12, 15);
    goto(&mut session, 3);
    let parent = session.view().point.stamp.buffer;
    action(&mut session, Org::EditSpecial);
    assert_ne!(
        session.view().point.stamp.buffer,
        parent,
        "{}",
        session.echo()
    );
    assert!(session.buffer().unwrap().soft_wrap());
    assert!(
        session
            .view()
            .wrapped_rows(session.buffer().unwrap(), 12, 12)
            .unwrap()
            .len()
            > 1
    );
    session.handle(Event::Paste("X ".into()));
    action(&mut session, Org::EditSrcExit);
    assert_eq!(session.view().point.stamp.buffer, parent);
    assert_eq!(text(&session), source.replace("a long", "X a long"));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Optional image rectangles start on the actual visible anchor fragment after
// a wrapped heading. Reflow changes placement only; a folded or scrolled-past
// anchor cannot acquire a rectangle at a different logical source row.
#[test]
fn preview_anchor() {
    let source = "* Heading with several words\n[[file:image.png]]\nTail";
    let (fixture, mut session) = editor(source, 12, 18);
    std::fs::write(
        fixture.0.join("image.png"),
        include_bytes!("../fixtures/org-preview.png"),
    )
    .unwrap();
    let point = session.view().point;
    action(&mut session, Org::LinkPreviewRegion);
    let frame = session.frame().unwrap();
    assert_eq!(frame.previews().len(), 1, "{}", session.echo());
    assert_eq!(frame.previews()[0].area.row(), 4);
    session.handle(Event::Resize(40, 18));
    let frame = session.frame().unwrap();
    assert_eq!(frame.previews()[0].area.row(), 1);
    assert_eq!(session.view().point, point);
    action(&mut session, Org::HideSubtree);
    assert!(session.frame().unwrap().previews().is_empty());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Runtime policy belongs to a live buffer across unequal split widths. Closing
// it removes its override from every affected view; revisiting the unchanged
// file creates a new identity inheriting the frozen startup default again.
#[test]
fn override_lifetime() {
    use rev::command::{BufferCommand, Split};
    for startup in [true, false] {
        let fixture = super::fixture::Fixture::new();
        let config_text = format!("editor.soft_wrap = {startup}");
        let config = fixture.write("config.toml", &config_text);
        let source = "one two three four five six seven eight nine\r\n";
        let first = fixture.write("first.org", source);
        let other = fixture.write("other.org", "* other\n");
        let mut buffers = Buffers::with_settings(rev::config::prepare(&config).snapshot).unwrap();
        let original = buffers.visit(&first).unwrap();
        let mut view = View::new(buffers.get(original).unwrap());
        view.gutter.apply(Change::Visible(false));
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 40,
                height: 14,
            },
        )
        .unwrap();
        let stamp = session.buffer().unwrap().stamp();
        session.execute(Command::VisualLineMode);
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Split(Split::Below));
        let ids = session.windows().order();
        assert_eq!(ids.len(), 3);
        for id in &ids {
            session.select_view(*id).unwrap();
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().soft_wrap(), !startup);
            assert!(session.buffer().unwrap().history().is_empty());
        }
        session.load(&other).unwrap();
        assert_eq!(session.buffer().unwrap().soft_wrap(), startup);
        session.select_view(ids[0]).unwrap();
        session.execute(Command::Buffer(BufferCommand::Kill));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        assert_eq!(session.echo(), "Buffer closed");
        for id in ids {
            session.select_view(id).unwrap();
            assert_ne!(session.view().point.stamp.buffer, original);
            assert_eq!(session.buffer().unwrap().soft_wrap(), startup);
        }
        session.load(&first).unwrap();
        assert_ne!(session.view().point.stamp.buffer, original);
        assert_eq!(session.buffer().unwrap().soft_wrap(), startup);
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
        assert_eq!(std::fs::read(&first).unwrap(), source.as_bytes());
        assert_eq!(std::fs::read_to_string(config).unwrap(), config_text);
    }
}
