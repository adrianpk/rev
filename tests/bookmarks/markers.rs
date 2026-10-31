use super::*;
use rev::buffer::Buffers;
use rev::coordinates::{Byte, Scalar};
use rev::display::{Frame, Position};
use rev::gutter::{Geometry, Mode, Policy, Settings, Side};
use rev::org::Source;
use rev::view::View;
use rev::window::Area;

fn marked_view(buffer: &rev::buffer::Buffer) -> View {
    let mut view = View::new(buffer);
    view.bookmark_markers = true;
    view
}

fn editor() -> Session {
    let buffers = Buffers::new().unwrap();
    let view = marked_view(buffers.get(buffers.initial()).unwrap());
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

fn configured(f: &fixture::Fixture, extra: &str) -> Session {
    let root = f.write(
        "markers.toml",
        &format!(
            "[bookmarks]\nfile='{}'\nsave='manual'\n{extra}",
            f.0.join("bookmarks.toml").display()
        ),
    );
    let startup = rev::config::prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let view = View::with_settings(buffers.get(buffers.initial()).unwrap(), &startup.snapshot);
    Session::new(
        buffers,
        view,
        Size {
            width: 24,
            height: 12,
        },
    )
    .unwrap()
}

fn area(width: u16, height: u16) -> Area {
    Area::new(0, 0, width, height).unwrap()
}

fn project(s: &Session, view: &View, width: u16, height: u16) -> Frame {
    Frame::project(s.buffer().unwrap(), view, Size { width, height }, "").unwrap()
}

fn rows(frame: &Frame) -> Vec<usize> {
    frame
        .rows()
        .iter()
        .enumerate()
        .filter_map(|(row, line)| line.text().contains('●').then_some(row))
        .collect()
}

fn goto(s: &mut Session, line: usize) {
    s.execute(Command::GotoLine);
    s.handle(Event::Paste(line.to_string()));
    key(s, KeyCode::Enter);
}

// Numbers retain their field and priority; markers use one independent cell
// only when the protected three text columns survive, on either gutter side.
#[test]
fn narrow_geometry() {
    for side in [Side::Left, Side::Right] {
        for visible in [false, true] {
            let settings = Settings {
                side,
                visible,
                ..Settings::default()
            };
            for width in 0..=20 {
                for height in [0, 1, 8] {
                    let content = area(width, height);
                    let numbers = Geometry::new(content, 999, settings).unwrap();
                    let actual = Geometry::with_marker(content, 999, settings, true).unwrap();
                    assert_eq!(actual.numbers(), numbers.numbers());
                    let expected = height > 0 && numbers.text().width() >= 4;
                    assert_eq!(
                        actual.marker().is_some(),
                        expected,
                        "{side:?}/{visible}/{width}/{height}"
                    );
                    assert_eq!(
                        actual.text().width(),
                        numbers.text().width() - u16::from(expected)
                    );
                    if let Some(marker) = actual.marker() {
                        assert_eq!(marker.width(), 1);
                        assert!(actual.text().width() >= 3);
                        assert!(
                            actual
                                .text()
                                .local(Position {
                                    column: marker.column(),
                                    row: marker.row()
                                })
                                .is_none()
                        );
                    }
                }
            }
        }
    }
}

// Two names coalesce without losing record identity. Both number modes, hidden
// numbers, filler and disabled markers share the same canonical source rows.
#[test]
fn independent_fields() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "first\nsecond\n");
    let mut s = editor();
    s.load(&path).unwrap();
    goto(&mut s, 2);
    argument(&mut s, BookmarkCommand::Set, "A");
    argument(&mut s, BookmarkCommand::Set, "B");
    let records = s.bookmarks().iter().cloned().collect::<Vec<_>>();
    for side in [Side::Left, Side::Right] {
        for mode in [Mode::Absolute, Mode::Relative] {
            for visible in [false, true] {
                let mut view = s.view().clone();
                view.gutter = Policy::new(Settings {
                    side,
                    mode,
                    visible,
                    ..Settings::default()
                });
                let frame = project(&s, &view, 24, 8);
                assert_eq!(rows(&frame), [1]);
                let geometry = view
                    .gutter_geometry(s.buffer().unwrap(), area(24, 6))
                    .unwrap();
                let marker = geometry.marker().unwrap();
                assert_eq!(
                    frame.rows()[1]
                        .text()
                        .chars()
                        .nth(usize::from(marker.column())),
                    Some('●')
                );
                assert_eq!(
                    view.hit(
                        s.buffer().unwrap(),
                        area(24, 6),
                        Position {
                            column: marker.column(),
                            row: 1
                        }
                    )
                    .unwrap(),
                    None
                );
                let text = geometry.text();
                let hit = view
                    .hit(
                        s.buffer().unwrap(),
                        area(24, 6),
                        Position {
                            column: text.column(),
                            row: 1,
                        },
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(hit.position, Scalar(6));
                view.bookmark_markers = false;
                assert!(rows(&project(&s, &view, 24, 8)).is_empty());
                assert_eq!(s.bookmarks().iter().cloned().collect::<Vec<_>>(), records);
            }
        }
    }
    assert_eq!(std::fs::read_to_string(path).unwrap(), "first\nsecond\n");
    assert!(!s.buffer().unwrap().is_modified());
    assert!(s.buffer().unwrap().history().is_empty());
}

// Publication, history and deletion invalidate the row index even when the
// source stamp stays unchanged for collection-only mutations.
#[test]
fn current_index() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "first\nsecond\n");
    let mut s = editor();
    s.load(&path).unwrap();
    goto(&mut s, 2);
    argument(&mut s, BookmarkCommand::Set, "A");
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [1]
    );
    let anchor = s.bookmarks().get("A").unwrap().point().position.0;
    s.replace(anchor..anchor + 1, "?").unwrap();
    assert_eq!(
        s.bookmarks().get("A").unwrap().resolution(),
        rev::bookmark::Resolution::Stale
    );
    assert!(rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)).is_empty());
    s.execute(Command::History(History::Undo));
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [1]
    );
    s.execute(Command::Motion(Motion::BufferStart));
    s.execute(Command::Newline);
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [2]
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [1]
    );
    s.execute(Command::History(History::Redo));
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [2]
    );
    argument(&mut s, BookmarkCommand::Delete, "A");
    assert!(rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)).is_empty());
    argument(&mut s, BookmarkCommand::Set, "A");
    assert_eq!(
        rows(&project(&s, &marked_view(s.buffer().unwrap()), 24, 8)),
        [1]
    );
}

// Wrapped continuations and folded bodies never inherit a surrogate marker;
// a checked jump reveals the actual bookmarked logical source line.
#[test]
fn fragment_identity() {
    let f = fixture::Fixture::new();
    let path = f.write("source.org", "* Parent\none two three four\n* Last\n");
    let mut s = editor();
    s.load(&path).unwrap();
    goto(&mut s, 2);
    argument(&mut s, BookmarkCommand::Set, "body");
    goto(&mut s, 3);
    argument(&mut s, BookmarkCommand::Set, "heading");
    let mut view = s.view().clone();
    view.point.position = Scalar(0);
    let wrapped = project(&s, &view, 15, 14);
    assert_eq!(rows(&wrapped), [2, 5]);
    let mut continuation = view.clone();
    continuation.point.position = Scalar(19);
    continuation
        .reveal(
            s.buffer().unwrap(),
            Size {
                width: 15,
                height: 3,
            },
        )
        .unwrap();
    let clipped = project(&s, &continuation, 15, 3);
    assert!(clipped.cursor().is_some());
    assert!(rows(&clipped).is_empty());
    let buffer = s.buffer().unwrap();
    let hidden = Source::new(buffer).range(Byte(9)..Byte(28)).unwrap();
    view.set_folds(buffer, &[hidden]).unwrap();
    let folded = project(&s, &view, 15, 14);
    assert_eq!(rows(&folded), [2]);
    assert!(folded.rows()[2].text().contains("Last"));
    s.execute(Command::Motion(Motion::BufferStart));
    s.execute(Command::Org(rev::command::OrgCommand::HideSubtree));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(std::time::Instant::now()).is_none() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "owned fold work did not finish"
        );
        s.expire_input(std::time::Instant::now() + std::time::Duration::from_millis(10));
        std::thread::yield_now();
    }
    assert!(
        !s.frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains("one two"))
    );
    argument(&mut s, BookmarkCommand::Jump, "body");
    assert_eq!(s.view().point.position, Scalar(9));
    assert!(rows(&project(&s, s.view(), 15, 14)).len() >= 2);
}

// The marker field is present for a scratch view but cannot invent targets.
// A suspended surface exposes neither cursor nor source/marker cells.
#[test]
fn empty_surface() {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let view = marked_view(buffer);
    assert!(
        view.gutter_geometry(buffer, area(24, 6))
            .unwrap()
            .marker()
            .is_some()
    );
    for size in [
        Size {
            width: 0,
            height: 0,
        },
        Size {
            width: 24,
            height: 0,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        assert!(frame.rows().is_empty());
        assert!(frame.cursor().is_none());
    }
}

// Startup leaves are copied into each view. M-x changes only the selected
// effective policy; splits and file rebinds retain it without changing records.
#[test]
fn view_policy() {
    use rev::command::{BufferCommand, Split, WindowCommand};
    let f = fixture::Fixture::new();
    let path = f.write("source", "first\nsecond\n");
    let other = f.write("other", "other\n");
    let mut s = configured(
        &f,
        "markers=false\nmarker='*'\n[palette.gutter]\nbookmark='#123456'\n",
    );
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "A");
    let record = s.bookmarks().get("A").unwrap().clone();
    assert!(!s.view().bookmark_markers);
    assert!(
        s.view()
            .gutter_geometry(s.buffer().unwrap(), area(24, 10))
            .unwrap()
            .marker()
            .is_none()
    );
    s.execute(Command::ExecuteExtended);
    s.handle(Event::Paste("bookmark-markers-mode".into()));
    key(&mut s, KeyCode::Enter);
    assert!(s.view().bookmark_markers);
    let frame = s.frame().unwrap();
    let marker = s
        .view()
        .gutter_geometry(s.buffer().unwrap(), area(24, 10))
        .unwrap()
        .marker()
        .unwrap();
    let cells = surface::draw(&frame);
    let cell = cells[&(0, usize::from(marker.column()))];
    assert_eq!(cell.character, '*');
    assert_eq!(cell.foreground, Some(rev::config::Rgb(0x12, 0x34, 0x56)));
    s.execute(Command::Split(Split::Below));
    let original = s.windows().selected_id();
    let peer = s.windows().next_view().unwrap();
    assert!(s.windows().get(peer).unwrap().bookmark_markers);
    s.execute(Command::Bookmark(BookmarkCommand::Markers));
    assert!(!s.view().bookmark_markers);
    assert!(s.windows().get(peer).unwrap().bookmark_markers);
    s.select_view(peer).unwrap();
    s.load(&other).unwrap();
    assert!(s.view().bookmark_markers);
    s.load(&path).unwrap();
    s.handle(Event::Resize(9, 6));
    assert!(
        s.view()
            .gutter_geometry(s.buffer().unwrap(), area(9, 4))
            .unwrap()
            .marker()
            .is_none()
    );
    s.handle(Event::Resize(24, 12));
    assert!(
        s.view()
            .gutter_geometry(s.buffer().unwrap(), area(24, 10))
            .unwrap()
            .marker()
            .is_some()
    );
    assert_eq!(s.bookmarks().get("A"), Some(&record));
    s.select_view(original).unwrap();
    s.execute(Command::Window(WindowCommand::Delete));
    assert!(s.view().bookmark_markers);
    s.execute(Command::Buffer(BufferCommand::Kill));
    s.handle(Event::Paste(String::new()));
    key(&mut s, KeyCode::Enter);
    assert_eq!(
        s.bookmarks().get("A").unwrap().resolution(),
        rev::bookmark::Resolution::Unresolved
    );
    argument(&mut s, BookmarkCommand::Jump, "A");
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), path);
    assert!(s.view().bookmark_markers);
    assert!(
        s.frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().contains('*'))
    );
    assert_eq!(s.bookmarks().get("A").unwrap().id(), record.id());
    assert!(!s.buffer().unwrap().is_modified());
    assert!(s.buffer().unwrap().history().is_empty());
}

// A valid store remains untouched by disabled-marker painting, then first
// enabled demand loads it outside paint. Unresolved targets stay blank; painting
// a resolved live target never reopens a removed source or a changed store.
#[test]
fn paint_admission() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "alpha");
    let store = f.write("bookmarks.toml", &format!(
        "version=1\n[[bookmarks]]\nname='restored'\npath='{}'\nposition=0\nbefore=''\nafter='alpha'\nat_start=true\nat_end=true\n", path.display()
    ));
    let mut s = configured(&f, "markers=false\n");
    s.load(&path).unwrap();
    assert!(s.bookmarks().is_empty());
    let mut view = s.view().clone();
    view.bookmark_markers = true;
    assert!(rows(&project(&s, &view, 24, 12)).is_empty());
    s.frame().unwrap();
    assert!(s.bookmarks().is_empty());
    s.execute(Command::Bookmark(BookmarkCommand::Markers));
    assert_eq!(s.bookmarks().len(), 1);
    assert_eq!(
        s.bookmarks().get("restored").unwrap().resolution(),
        rev::bookmark::Resolution::Unresolved
    );
    assert!(rows(&s.frame().unwrap()).is_empty());
    argument(&mut s, BookmarkCommand::Jump, "restored");
    let record = s.bookmarks().get("restored").unwrap().clone();
    assert_eq!(record.resolution(), rev::bookmark::Resolution::Resolved);
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&store, "malformed external replacement").unwrap();
    for _ in 0..3 {
        assert_eq!(rows(&s.frame().unwrap()), [0]);
        assert_eq!(s.bookmarks().get("restored"), Some(&record));
    }
    assert_eq!(
        std::fs::read_to_string(store).unwrap(),
        "malformed external replacement"
    );
    assert!(!s.buffer().unwrap().is_modified());
}

// Actual native pointer selection and search overlays use the combined text
// area on either side, with and without numbers. The marker is never source.
#[test]
fn overlay_geometry() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    for side in ["left", "right"] {
        for visible in [false, true] {
            let f = fixture::Fixture::new();
            let path = f.write("source", "alpha beta gamma delta\nlast\n");
            let mut s = configured(&f, &format!("[gutter]\nside='{side}'\nvisible={visible}\n"));
            s.load(&path).unwrap();
            argument(&mut s, BookmarkCommand::Set, "A");
            let geometry = s
                .view()
                .gutter_geometry(s.buffer().unwrap(), area(24, 10))
                .unwrap();
            let text = geometry.text();
            let marker = geometry.marker().unwrap();
            s.execute(Command::SearchForward);
            s.handle(Event::Paste("beta".into()));
            let frame = s.frame().unwrap();
            assert_eq!(
                frame.rows()[0].search(),
                Some(text.column() + 6..text.column() + 10)
            );
            assert_eq!(rows(&frame), [0]);
            s.execute(Command::KeyboardQuit);
            let before = s.view().point;
            let mouse = |kind, column| {
                Event::Mouse(MouseEvent {
                    kind,
                    column,
                    row: 0,
                    modifiers: KeyModifiers::NONE,
                })
            };
            s.handle(mouse(
                MouseEventKind::Down(MouseButton::Left),
                marker.column(),
            ));
            s.handle(mouse(
                MouseEventKind::Up(MouseButton::Left),
                marker.column(),
            ));
            assert_eq!(s.view().point, before);
            s.handle(mouse(
                MouseEventKind::Down(MouseButton::Left),
                text.column() + 1,
            ));
            s.handle(mouse(
                MouseEventKind::Drag(MouseButton::Left),
                text.column() + 5,
            ));
            s.handle(mouse(
                MouseEventKind::Up(MouseButton::Left),
                text.column() + 5,
            ));
            assert_eq!(s.view().point.position, Scalar(5));
            let frame = s.frame().unwrap();
            assert_eq!(
                frame.rows()[0].selection(),
                Some(text.column() + 1..text.column() + 5)
            );
            assert_eq!(rows(&frame), [0]);
            assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                "alpha beta gamma delta\nlast\n"
            );
            assert!(!s.buffer().unwrap().is_modified());
        }
    }
}
