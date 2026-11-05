#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use fixture::Fixture;
use rev::buffer::Buffers;
use rev::command::{Command, History, LinearCommand, Motion, OrgCommand, Scroll, Split};
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::{Frame, Position};
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::Area;
use std::time::{Duration, Instant};

fn editor(config: &str, source: &str, width: u16, height: u16) -> (Fixture, Session) {
    let f = Fixture::new();
    let config = format!(
        "bookmarks.file='{}'\nbookmarks.save='manual'\n{config}",
        f.0.join("bookmarks.toml").display()
    );
    let startup = rev::config::prepare(&f.write("config.toml", &config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&f.write("notes.org", source)).unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    view.bookmark_markers = false;
    view.gutter.apply(rev::gutter::Change::Visible(false));
    let mut session = Session::new(buffers, view, Size { width, height }).unwrap();
    settle(&mut session);
    (f, session)
}

fn settle(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        let now = Instant::now();
        if session.remaining_input(now).is_none() {
            break;
        }
        assert!(
            now < deadline,
            "Org indentation observation did not settle: {}",
            session.echo()
        );
        session.expire_input(now + Duration::from_millis(10));
        std::thread::yield_now();
    }
}

fn source(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn mouse(session: &mut Session, kind: MouseEventKind, row: u16, column: u16) {
    session.handle(Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

fn goto(session: &mut Session, line: usize) {
    session.execute(Command::GotoLine);
    session.handle(Event::Paste(line.to_string()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Real sessions admit the new default under both profiles and both wrap modes.
// Pretty/source choices remain independent, and disabled mode keeps legacy cells.
#[test]
fn configured_geometry() {
    let text = "preamble\n* Parent\nbody\n** Child\nchild\n*** Deep\ndeep\n* Return\nbody";
    for profile in ["non-modal", "modal"] {
        for wrapped in [true, false] {
            for (name, policy, pretty, expected) in [
                (
                    "default",
                    "",
                    true,
                    vec![
                        "preamble",
                        "◉ Parent",
                        "  body",
                        "  ○ Child",
                        "    child",
                        "    ✸ Deep",
                        "      deep",
                        "◉ Return",
                        "  body",
                    ],
                ),
                (
                    "disabled",
                    "org.indent=false",
                    true,
                    vec![
                        "preamble",
                        "◉ Parent",
                        "body",
                        " ○ Child",
                        "child",
                        "  ✸ Deep",
                        "deep",
                        "◉ Return",
                        "body",
                    ],
                ),
                (
                    "source",
                    "",
                    false,
                    vec![
                        "preamble",
                        "* Parent",
                        "  body",
                        " ** Child",
                        "    child",
                        "  *** Deep",
                        "      deep",
                        "* Return",
                        "  body",
                    ],
                ),
            ] {
                let config = format!(
                    "editor.interaction_profile='{profile}'\neditor.soft_wrap={wrapped}\norg.headings.pretty={pretty}\n{policy}"
                );
                let (f, mut session) = editor(&config, text, 60, 16);
                let stamp = session.buffer().unwrap().stamp();
                let state = session.buffer().unwrap().state();
                let frame = session.frame().unwrap();
                for (row, expected) in expected.iter().enumerate() {
                    assert_eq!(
                        frame.rows()[row].text().trim_end(),
                        *expected,
                        "{profile}/{wrapped}/{name}/{row}"
                    );
                }
                goto(&mut session, 3);
                assert_eq!(
                    session.frame().unwrap().cursor(),
                    Some(Position {
                        row: 2,
                        column: if name == "disabled" { 0 } else { 2 }
                    })
                );
                assert_eq!(
                    session.view().cell(session.buffer().unwrap()).unwrap(),
                    Cell { row: 2, column: 0 }
                );
                assert_eq!(session.buffer().unwrap().stamp(), stamp);
                assert_eq!(session.buffer().unwrap().state(), state);
                assert_eq!(source(&session), text);
                assert!(session.buffer().unwrap().history().is_empty());
                session.execute(Command::SaveBuffer);
                assert_eq!(
                    std::fs::read(f.0.join("notes.org")).unwrap(),
                    text.as_bytes()
                );
            }
        }
    }
}

// Native wrap, cursor, mouse and vertical motion agree on exact continuation
// rows. The same journey has source-only navigation under either input profile.
#[test]
fn wrapped_mappings() {
    let text = "** One two three\nalpha beta gamma delta";
    for profile in ["non-modal", "modal"] {
        let config = format!("editor.interaction_profile='{profile}'");
        let (_f, mut session) = editor(&config, text, 12, 14);
        let frame = session.frame().unwrap();
        for (row, expected) in [
            "  ○ One two ",
            "    three   ",
            "    alpha   ",
            "    beta    ",
            "    gamma   ",
            "    delta   ",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(frame.rows()[row].text(), *expected, "{profile}/{row}");
        }
        mouse(&mut session, MouseEventKind::Down(MouseButton::Left), 3, 1);
        mouse(&mut session, MouseEventKind::Up(MouseButton::Left), 3, 1);
        assert_eq!(session.view().point.position, Scalar(23));
        assert_eq!(
            session.frame().unwrap().cursor(),
            Some(Position { row: 3, column: 4 })
        );
        session.execute(Command::Scroll(Scroll::NextLine));
        assert_eq!(session.view().point.position, Scalar(28));
        assert_eq!(
            session.frame().unwrap().cursor(),
            Some(Position { row: 4, column: 4 })
        );
        session.execute(Command::Scroll(Scroll::PreviousLine));
        assert_eq!(session.view().point.position, Scalar(23));
        assert_eq!(source(&session), text);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Tabs use logical source stops; Unicode remains whole under wrap, clipping,
// resize and narrow caps. Each visible source glyph round-trips through a frame.
#[test]
fn unicode_round_trips() {
    let text = "*** Heading\na\t界e\u{301}👩‍💻 words";
    for wrapped in [true, false] {
        let (_f, session) = editor(&format!("editor.soft_wrap={wrapped}"), text, 24, 20);
        let b = session.buffer().unwrap();
        let coordinates = Coordinates::new(b).unwrap();
        let mut view = session.view().clone();
        for width in [3, 8, 12, 24] {
            let size = Size { width, height: 20 };
            for point in [Scalar(0), Scalar(2), Scalar(12), Scalar(b.len_chars())] {
                if !coordinates.is_boundary(point).unwrap() {
                    continue;
                }
                view.point.position = point;
                view.reveal(b, size).unwrap();
                let frame = Frame::project(b, &view, size, "").unwrap();
                let cursor = frame.cursor().unwrap();
                assert!(cursor.column < width);
                let hit = view
                    .hit(b, Area::new(0, 0, width, 18).unwrap(), cursor)
                    .unwrap()
                    .unwrap();
                assert_eq!(hit.position, point, "{wrapped}/{width}/{point:?}");
                assert!(
                    frame
                        .rows()
                        .iter()
                        .all(|row| unicode_width::UnicodeWidthStr::width(row.text())
                            == usize::from(width))
                );
            }
        }
        assert_eq!(source(&session), text);
    }
}

// Dragging from virtual padding selects only actual source. Search and source
// syntax paint begin after that same prefix in wrapped and unwrapped panes.
#[test]
fn source_overlays() {
    let text = "** H\nalpha beta\n#+begin_src text\n* literal\n#+end_src\n";
    for wrapped in [true, false] {
        let (_f, mut session) = editor(&format!("editor.soft_wrap={wrapped}"), text, 40, 14);
        mouse(&mut session, MouseEventKind::Down(MouseButton::Left), 1, 0);
        mouse(&mut session, MouseEventKind::Drag(MouseButton::Left), 1, 9);
        mouse(&mut session, MouseEventKind::Up(MouseButton::Left), 1, 9);
        assert_eq!(session.frame().unwrap().rows()[1].selection(), Some(4..9));
        session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(session.killed_text().unwrap().to_string(), "alpha");
        session.execute(Command::KeyboardQuit);
        session.execute(Command::Motion(Motion::BufferStart));
        session.execute(Command::SearchForward);
        session.handle(Event::Paste("beta".into()));
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows()[1].search(), Some(10..14));
        assert!(
            frame.rows()[2]
                .syntax()
                .iter()
                .all(|paint| paint.columns.start >= 4)
        );
        assert_eq!(frame.rows()[3].text().trim_end(), "    * literal");
        assert_eq!(source(&session), text);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Source revision retirement is coherent before the worker catches up. Undo
// reobserves the original hierarchy without inserting padding in edit history.
#[test]
fn structural_reflow() {
    let (_f, mut session) = editor("syntax.enabled=false", "** H\nbody\n", 30, 10);
    assert_eq!(
        session.frame().unwrap().rows()[1].text().trim_end(),
        "    body"
    );
    session.insert('*');
    let b = session.buffer().unwrap();
    let stale = Frame::project(
        b,
        session.view(),
        Size {
            width: 30,
            height: 10,
        },
        "",
    )
    .unwrap();
    assert_eq!(stale.rows()[0].text().trim_end(), "*** H");
    assert_eq!(stale.rows()[1].text().trim_end(), "body");
    settle(&mut session);
    assert_eq!(
        session.frame().unwrap().rows()[1].text().trim_end(),
        "      body"
    );
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(
        session.frame().unwrap().rows()[1].text().trim_end(),
        "    body"
    );
    assert_eq!(source(&session), "** H\nbody\n");
    assert!(!session.buffer().unwrap().is_modified());
}

// Split views share indentation while pretty-heading choices and geometry stay
// independent. Folding still hides logical rows, and buffer switches keep policy.
#[test]
fn pane_and_fold_identity() {
    let (f, mut session) = editor("", "** H\nbody words\n* Peer\nother\n", 60, 14);
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::Split(Split::Right));
    let ids = session.windows().order();
    session.execute(Command::Org(OrgCommand::PrettyHeadings));
    assert!(!session.view().pretty_headings());
    for id in ids {
        session.select_view(id).unwrap();
        assert!(session.buffer().unwrap().org_indentation());
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        let frame = session.frame().unwrap();
        assert!(frame.cursor().is_some());
    }
    session.execute(Command::Org(OrgCommand::HideSubtree));
    settle(&mut session);
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 4);
    session.load(&f.write("plain.txt", "body\n")).unwrap();
    assert!(!session.buffer().unwrap().org_indentation());
    session.load(&f.0.join("notes.org")).unwrap();
    assert!(session.buffer().unwrap().org_indentation());
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
}

// Rectangles remain logical source columns across body continuations. Neither
// the virtual prefix nor continuation padding enters the copied rectangle.
#[test]
fn rectangular_source() {
    use rev::command::RectangleCommand as R;
    let text = "** H\nabcdEFGH\nijklMNOP\n";
    let (_f, mut session) = editor("", text, 10, 14);
    goto(&mut session, 2);
    for _ in 0..2 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Rectangle(R::Mark));
    for _ in 0..12 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        Cell { row: 2, column: 5 }
    );
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[1].selection(), Some(6..9));
    assert_eq!(frame.rows()[2].selection(), None);
    assert_eq!(frame.rows()[3].selection(), Some(6..9));
    assert_eq!(frame.rows()[4].selection(), None);
    session.execute(Command::Rectangle(R::Copy));
    assert_eq!(session.rectangle().unwrap().rows(), ["cdE", "klM"]);
    assert_eq!(source(&session), text);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Completion placement uses the projected cursor, and accepting a candidate
// edits only its source span. Image results use the same outer heading margin.
#[test]
fn completion_and_preview_anchors() {
    use rev::command::CompletionCommand as C;
    for (enabled, column) in [(true, 8), (false, 4)] {
        let text = "** H\n#+TI\n";
        let (f, mut session) = editor(&format!("org.indent={enabled}"), text, 40, 14);
        goto(&mut session, 2);
        session.execute(Command::Motion(Motion::LineEnd));
        session.execute(Command::Completion(C::AtPoint));
        settle(&mut session);
        assert!(session.completion().is_some(), "{}", session.echo());
        let frame = session.frame().unwrap();
        assert_eq!(frame.cursor().unwrap().column, column);
        let border = frame
            .rows()
            .iter()
            .find(|row| row.text().contains('┌'))
            .unwrap();
        assert_eq!(
            border.text().chars().position(|ch| ch == '┌'),
            Some(usize::from(column))
        );
        session.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
        assert_eq!(source(&session), "** H\n#+TITLE:\n");
        session.execute(Command::History(History::Undo));
        assert_eq!(source(&session), text);
        std::fs::write(
            f.0.join("image.png"),
            include_bytes!("fixtures/org-preview.png"),
        )
        .unwrap();
        let image_source = "** H\n[[file:image.png]]\n";
        session.load(&f.write("image.org", image_source)).unwrap();
        settle(&mut session);
        session.execute(Command::Org(OrgCommand::LinkPreviewRegion));
        settle(&mut session);
        let frame = session.frame().unwrap();
        assert_eq!(frame.previews().len(), 1, "{}", session.echo());
        assert_eq!(
            frame.previews()[0].area.column(),
            if enabled { 4 } else { 0 }
        );
        assert_eq!(frame.previews()[0].area.row(), 1);
        assert_eq!(source(&session), image_source);
    }
}

// Number and bookmark fields keep logical line identity on either side of the
// pane. Only the first real fragment gets a marker; folds remove the source row.
#[test]
fn gutter_and_bookmark_identity() {
    use rev::command::{BookmarkCommand as B, GutterCommand as G};
    let (_f, mut session) = editor("", "** H\nalpha beta gamma\n* Peer\nbody\n", 20, 14);
    session.execute(Command::Gutter(G::Toggle));
    session.execute(Command::Bookmark(B::Markers));
    goto(&mut session, 2);
    session.execute(Command::Bookmark(B::Set));
    session.handle(Event::Paste("body".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert!(
        session.bookmarks().get("body").is_some(),
        "{}",
        session.echo()
    );
    for _ in 0..2 {
        let frame = session.frame().unwrap();
        let b = session.buffer().unwrap();
        let geometry = session
            .view()
            .gutter_geometry(b, Area::new(0, 0, 20, 12).unwrap())
            .unwrap();
        let marker = usize::from(geometry.marker().unwrap().column());
        assert_eq!(frame.rows()[1].text().chars().nth(marker), Some('●'));
        assert_eq!(frame.cursor().unwrap().column, geometry.text().column() + 4);
        let rows = session
            .view()
            .wrapped_rows(b, geometry.text().width(), 12)
            .unwrap();
        for (row, fragment) in rows
            .iter()
            .enumerate()
            .filter(|(_, f)| f.line == 1 && f.columns.start > 0)
        {
            assert_eq!(fragment.margin, 4);
            assert_eq!(frame.rows()[row].text().chars().nth(marker), Some(' '));
        }
        session.execute(Command::Gutter(G::ToggleSide));
    }
    goto(&mut session, 1);
    session.execute(Command::Org(OrgCommand::HideSubtree));
    settle(&mut session);
    let frame = session.frame().unwrap();
    assert!(!frame.rows().iter().any(|row| row.text().contains('●')));
    assert_eq!(
        session.bookmarks().get("body").unwrap().point().position,
        Scalar(5)
    );
}
