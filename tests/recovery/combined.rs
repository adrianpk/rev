use std::fs::File;
use std::io;
use std::ops::ControlFlow;
use std::path::Path;

use rev::buffer::edit::{History, Point};
use rev::command::{Command, Split};
use rev::coordinates::Scalar;
use rev::file::{Observation, Output, Publication};
use tree_sitter::{ParseOptions, ParseState};

use super::*;

struct Uncertain {
    calls: Vec<&'static str>,
}

impl Output for Uncertain {
    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        self.calls.push("replace");
        fs::rename(temporary, target)
    }

    fn sync_directory(&mut self, _directory: &File) -> io::Result<()> {
        self.calls.push("sync directory");
        Err(io::Error::other("directory completion unavailable"))
    }

    fn observe(&mut self, _path: &Path) -> io::Result<Observation> {
        self.calls.push("observe");
        Err(io::Error::other("observation unavailable"))
    }

    fn remove(&mut self, _path: &Path) -> io::Result<()> {
        panic!("published output must not be removed during cleanup")
    }
}

// Parser cancellation and two later filesystem failures cannot undo accepted
// text or a real replacement. Retry needs fresh consent and cannot resume quit.
#[test]
fn parser_publication() {
    let fixture = Fixture::new();
    let target = fixture.file("main.rs", b"fn base() {}\r\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&target).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    let mut points = [
        Point {
            stamp: buffer.stamp(),
            position: Scalar(0),
        },
        Point {
            stamp: buffer.stamp(),
            position: Scalar(buffer.len_chars()),
        },
    ];
    let inserted = "fn item() { let text = \"界🙂\"; }\r\n".repeat(2000);
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: inserted.clone(),
    };
    let prepared = buffer.prepare(&[edit], &points, None).unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    let before = snapshot(buffer);
    let mut called = false;
    let mut stop = |_: &ParseState| {
        called = true;
        ControlFlow::Break(())
    };
    buffer.parse_with(ParseOptions::new().progress_callback(&mut stop));
    assert!(called);
    assert_eq!(buffer.mode(), "Text");
    assert_eq!(snapshot(buffer), before);
    let positions = points;
    let save = buffers.prepare_save(id, None).unwrap();
    let mut output = Uncertain { calls: Vec::new() };
    let result = buffers.publish_with(save, &mut output);
    assert_eq!(result.publication(), Publication::Uncertain);
    assert_eq!(output.calls, ["replace", "sync directory", "observe"]);
    let error = result.error().unwrap().to_string();
    assert!(
        error.contains("directory completion unavailable")
            && error.contains("observation unavailable")
    );
    assert_eq!(
        fs::read(&target).unwrap(),
        format!("{inserted}fn base() {{}}\r\n").as_bytes()
    );
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.stamp(), before.stamp);
    assert_eq!(buffer.state(), before.state);
    assert_eq!(buffer.mark(), before.mark);
    assert_eq!(buffer.history().len(), before.history);
    assert_eq!(points, positions);
    assert!(buffer.is_modified() && buffer.save_warning().is_some());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 1);
    let mut view = View::new(buffer);
    view.point = points[0];
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 240,
            height: 12,
        },
    )
    .unwrap();
    session.execute(Command::Quit);
    cancel(&mut session);
    assert_eq!(session.view().point, points[0]);
    session.execute(Command::SaveBuffer);
    assert!(session.echo().contains("changed on disk"));
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "fn base() {}\r\n");
    assert!(session.buffer().unwrap().is_modified());
}

// Failed ingress while an interaction owns focus retains its exact source,
// shared views and pending argument. A later rejected replacement changes none.
#[test]
fn ingress_interaction() {
    for bytes in [b"\xff".as_slice(), b"valid\0binary".as_slice()] {
        let fixture = Fixture::new();
        let target = fixture.file("main.rs", b"fn main() {}\r\n");
        let invalid = fixture.file("invalid.txt", bytes);
        let mut session = session(Buffers::new().unwrap());
        session.load(&target).unwrap();
        session.execute(Command::Split(Split::Right));
        session.insert('界');
        let before = snapshot(session.buffer().unwrap());
        let views: Vec<_> = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect();
        session.execute(Command::QueryReplace);
        assert!(session.load(&invalid).is_err());
        assert_eq!(snapshot(session.buffer().unwrap()), before);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().contains("Query replace: "))
        );
        session.handle(Event::Paste("main".into()));
        session.handle(press(KeyCode::Enter));
        session.handle(Event::Paste("bad\0replacement".into()));
        session.handle(press(KeyCode::Enter));
        assert!(session.echo().contains("NUL"));
        assert_eq!(snapshot(session.buffer().unwrap()), before);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            views
        );
        session.execute(Command::WriteFile);
        session.handle(Event::Paste(
            fixture.0.join("canceled.txt").to_str().unwrap().into(),
        ));
        cancel(&mut session);
        assert!(!fixture.0.join("canceled.txt").exists());
        assert_eq!(snapshot(session.buffer().unwrap()), before);
    }
}

// Late read-only or peer-position changes reject a whole disjoint mutation,
// preserving the saved state, redo future, rectangle mark and valid captures.
#[test]
fn rejected_compound() {
    for failure in ["readonly", "peer"] {
        let fixture = Fixture::new();
        let target = fixture.file("main.rs", b"fn a() {}\r\nfn b() {}\r\n");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&target).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        edit(buffer, "// redo\r\n");
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        let mut points = [
            Point {
                stamp: buffer.stamp(),
                position: Scalar(2),
            },
            Point {
                stamp: buffer.stamp(),
                position: Scalar(14),
            },
        ];
        buffer.toggle_rectangle(points[0]).unwrap();
        buffer.refresh_syntax();
        let captures = buffer.syntax().captures(buffer, 0..buffer.len_bytes());
        let edits: Vec<_> = [3..4, 14..15]
            .into_iter()
            .map(|range| Replacement {
                source: buffer.span(range).unwrap(),
                text: "changed".into(),
            })
            .collect();
        let prepared = buffer.prepare(&edits, &points, None).unwrap();
        if failure == "readonly" {
            buffer.set_readonly(true);
        } else {
            points[1].position = Scalar(15);
        }
        let before = snapshot(buffer);
        let positions = points;
        assert!(buffer.publish(prepared, &mut points).is_err());
        assert_eq!(snapshot(buffer), before, "{failure}");
        assert_eq!(points, positions);
        assert_eq!(
            buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
            captures
        );
        buffer.set_readonly(false);
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert!(snapshot(buffer).bytes.starts_with(b"// redo\r\n"));
        assert!(buffer.is_modified());
        assert_eq!(fs::read(&target).unwrap(), b"fn a() {}\r\nfn b() {}\r\n");
    }
}
