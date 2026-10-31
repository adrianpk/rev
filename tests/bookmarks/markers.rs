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
    let buffer = s.buffer().unwrap();
    let hidden = Source::new(buffer).range(Byte(9)..Byte(28)).unwrap();
    view.set_folds(buffer, &[hidden]).unwrap();
    let folded = project(&s, &view, 15, 14);
    assert_eq!(rows(&folded), [2]);
    assert!(folded.rows()[2].text().contains("Last"));
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
