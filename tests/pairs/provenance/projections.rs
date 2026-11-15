use super::*;
use crate::fixture::Fixture;
use rev::buffer::Buffers;
use rev::command::RegionCommand;
use rev::coordinates::Scalar;
use rev::display::Frame;
use rev::view::{Size, View};

const SIZE: Size = Size {
    width: 80,
    height: 16,
};

fn source(format: &str) -> String {
    let marker = if format == "org" { "*" } else { "#" };
    format!(
        "{marker} Root\nRoot body.\n{marker}{marker} Child\nChild body.\n{marker}{marker}{marker} Deep\nDeep body.\n"
    )
}

fn editor(format: &str, pretty: bool) -> (Fixture, Session) {
    let f = Fixture::new();
    let config = format!("org.indent=true\norg.headings.pretty={pretty}\nmarkdown.pretty={pretty}");
    let startup = rev::config::prepare(&f.write("config.toml", &config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let source = source(format);
    let id = buffers
        .visit(&f.write(&format!("notes.{format}"), &source))
        .unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    view.gutter.apply(rev::gutter::Change::Visible(false));
    view.bookmark_markers = false;
    view.point.position = Scalar(source.find("Child body").unwrap() + 6);
    (f, Session::new(buffers, view, SIZE).unwrap())
}

fn pending(s: &Session) -> Frame {
    Frame::project(s.buffer().unwrap(), s.view(), SIZE, "").unwrap()
}

fn headings(before: &Frame, after: &Frame) {
    for row in [0, 2, 4] {
        assert_eq!(after.rows()[row].text(), before.rows()[row].text());
        assert_eq!(after.rows()[row].syntax(), before.rows()[row].syntax());
    }
}

// Pair publication and its pending lexical refresh retain completed heading
// rows, indentation and source/display cursor mapping during same-line typing.
#[test]
fn continuity() {
    for format in ["org", "md"] {
        for pretty in [true, false] {
            let (_f, mut s) = editor(format, pretty);
            observe(&mut s);
            let before = pending(&s);
            let point = s.view().point.position;
            let cursor = before.cursor().unwrap();
            s.handle(character('('));
            settle(&mut s);
            let after = pending(&s);
            headings(&before, &after);
            assert_eq!(s.view().point.position, Scalar(point.0 + 1));
            assert_eq!(after.cursor().unwrap().row, cursor.row);
            assert_eq!(after.cursor().unwrap().column, cursor.column + 1);
            assert_eq!(s.buffer().unwrap().history().len(), 1);
            observe(&mut s);
            headings(&before, &pending(&s));
            s.execute(Command::History(History::Undo));
            settle(&mut s);
            assert_eq!(s.view().point.position, point);
            s.execute(Command::History(History::Redo));
            observe(&mut s);
            headings(&before, &pending(&s));
        }
    }
}

// Same-line wrapping and unwrapping publish two delimiter edits. Pending frames
// must retain the same completed headings as single-range native body typing.
#[test]
fn region_continuity() {
    for format in ["org", "md"] {
        let (_f, mut s) = editor(format, true);
        observe(&mut s);
        let before = pending(&s);
        let point = s.view().point.position;
        s.execute(Command::Region(RegionCommand::Set));
        for _ in 0..5 {
            s.execute(Command::Motion(Motion::ForwardChar));
        }
        settle(&mut s);
        s.handle(character('['));
        settle(&mut s);
        assert_eq!(s.view().point.position, Scalar(point.0 + 1));
        headings(&before, &pending(&s));
        observe(&mut s);
        s.execute(Command::PairUnwrap);
        settle(&mut s);
        assert_eq!(s.view().point.position, point);
        headings(&before, &pending(&s));
        observe(&mut s);
        headings(&before, &pending(&s));
    }
}
