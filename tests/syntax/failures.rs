use std::ops::ControlFlow;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Motion, RectangleCommand, RegionCommand, Scroll, Split};
use rev::coordinates::Scalar;
use tree_sitter::{ParseOptions, ParseState};

use super::*;

fn compare_session(session: &mut Session) {
    session.frame().unwrap();
    let buffer = session.buffer().unwrap();
    let reference = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        reference.captures(buffer, 0..buffer.len_bytes())
    );
    assert!(buffer.syntax().is_incremental());
}

// A real Tree-sitter progress cancellation occurs after shared text and points
// are accepted. Both views fall back without stale colors; undo recovers a full
// current tree, and a subsequent edit resumes incremental reuse.
#[test]
fn canceled_parse() {
    let fixture = Fixture::new();
    let original = "fn base() {}\n";
    let path = fixture.file("main.rs", original);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    let mut views = [View::new(buffer), View::new(buffer)];
    views[1].point.position = Scalar(buffer.len_chars());
    let mut points = [views[0].point, views[1].point];
    let prefix = "fn inserted() { let text = \"🙂\"; }\n".repeat(2000);
    let prepared = buffer
        .prepare(
            &[Replacement {
                source: buffer.span(0..0).unwrap(),
                text: prefix.clone(),
            }],
            &points,
            None,
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    let stamp = buffer.stamp();
    let state = buffer.state();
    let mark = buffer.mark();
    let mut called = false;
    let mut cancel = |_: &ParseState| {
        called = true;
        ControlFlow::Break(())
    };
    buffer.refresh_syntax_with(ParseOptions::new().progress_callback(&mut cancel));
    assert!(
        called,
        "parser cancellation must reach the actual progress boundary"
    );
    assert_eq!(buffer.mode(), "Text");
    assert!(!buffer.syntax().is_incremental());
    assert!(
        buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes())
            .is_empty()
    );
    for (view, point) in views.iter_mut().zip(points) {
        view.point = point;
        view.cell(buffer).unwrap();
        let frame = Frame::project(
            buffer,
            view,
            Size {
                width: 80,
                height: 8,
            },
            "",
        )
        .unwrap();
        assert!(frame.rows().iter().all(|row| row.syntax().is_empty()));
    }
    buffer.refresh_syntax();
    assert_eq!(buffer.mode(), "Text");
    assert_eq!(text(buffer), format!("{prefix}{original}"));
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.state(), state);
    assert_eq!(buffer.mark(), mark);
    assert_eq!(buffer.history().len(), 1);
    buffer.navigate(History::Undo, &mut points, None).unwrap();
    compare(buffer);
    assert_eq!(text(buffer), original);
    assert!(!buffer.syntax().is_incremental());
    apply(buffer, 0..0, "// recovered\n");
    compare(buffer);
    assert!(buffer.syntax().is_incremental());
    let expected = text(buffer);
    let history = buffer.history().len();
    assert!(buffers.save(id).unwrap().is_complete());
    assert_eq!(std::fs::read(&path).unwrap(), expected.as_bytes());
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.history().len(), history);
    assert!(!buffer.is_modified());
    assert!(
        !buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes().min(64))
            .is_empty()
    );
}

// Exhausting the actual query result bound drops all partial styles for that
// request. The accepted document/history and a later small visible query survive.
#[test]
fn query_limit() {
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn base() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let source = "fn item() {}\n".repeat(12_000);
    apply(buffer, 0..buffer.len_chars(), &source);
    buffer.refresh_syntax();
    assert_eq!(buffer.mode(), "Rust");
    assert!(buffer.syntax().is_incremental());
    let stamp = buffer.stamp();
    let state = buffer.state();
    assert!(
        buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes())
            .is_empty()
    );
    assert!(!buffer.syntax().captures(buffer, 0..12).is_empty());
    assert_eq!(text(buffer), source);
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.state(), state);
    assert_eq!(buffer.history().len(), 1);
    assert!(buffers.save(id).unwrap().is_complete());
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    let buffer = buffers.get(id).unwrap();
    assert!(!buffer.is_modified());
    assert!(
        buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes())
            .is_empty()
    );
    assert!(!buffer.syntax().captures(buffer, 0..12).is_empty());
}

// Canceled, read-only and stale preparations cannot advance a valid parse tree.
// An independently held old syntax snapshot also rejects newer source stamps.
#[test]
fn rejected_edits() {
    let fixture = Fixture::new();
    let path = fixture.file("main.rs", "fn main() {}\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let replacement = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "// canceled\n".into(),
    };
    drop(
        buffer
            .prepare(std::slice::from_ref(&replacement), &[], None)
            .unwrap(),
    );
    buffer.set_read_only(true);
    assert!(
        buffer
            .prepare(std::slice::from_ref(&replacement), &[], None)
            .is_err()
    );
    buffer.set_read_only(false);
    assert_eq!(buffer.mode(), "Rust");
    assert_eq!(text(buffer), "fn main() {}\n");
    assert!(buffer.history().is_empty());
    let stale = buffer.prepare(&[replacement], &[], None).unwrap();
    let old = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    apply(buffer, 0..0, "// accepted\n");
    compare(buffer);
    let stamp = buffer.stamp();
    let captures = buffer.syntax().captures(buffer, 0..buffer.len_bytes());
    assert!(buffer.publish(stale, &mut []).is_err());
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.mode(), "Rust");
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        captures
    );
    assert!(old.captures(buffer, 0..buffer.len_bytes()).is_empty());
    assert_eq!(buffer.history().len(), 1);
}

// Real multi-row rectangle kill/yank and their history replay use exact row
// edits. Missing function names cannot retain the old function captures.
#[test]
fn rectangle_history() {
    let fixture = Fixture::new();
    let source = "fn ab(){}\nfn cd(){}";
    let path = fixture.file("main.rs", source);
    let mut session = Session::empty(Size {
        width: 80,
        height: 8,
    })
    .unwrap();
    session.load(&path).unwrap();
    for _ in 0..3 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    session.execute(Command::Scroll(Scroll::NextLine));
    for _ in 0..2 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.echo(), "Rectangle killed");
    assert_eq!(text(session.buffer().unwrap()), "fn (){}\nfn (){}");
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
    compare_session(&mut session);
    let buffer = session.buffer().unwrap();
    assert!(
        buffer
            .syntax()
            .captures(buffer, 0..buffer.len_bytes())
            .iter()
            .all(|capture| capture.style != Style::Function)
    );
    session.execute(Command::History(Direction::Undo));
    compare_session(&mut session);
    assert_eq!(text(session.buffer().unwrap()), source);
    session.execute(Command::History(Direction::Redo));
    compare_session(&mut session);
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    assert_eq!(session.echo(), "Rectangle yanked");
    compare_session(&mut session);
    assert_eq!(text(session.buffer().unwrap()), source);
    session.execute(Command::History(Direction::Undo));
    compare_session(&mut session);
    assert_eq!(text(session.buffer().unwrap()), "fn (){}\nfn (){}");
}

// Visible byte queries stay within requested rows. Horizontal projection after
// an incremental Unicode insertion keeps bounded cell ranges and no stale rows.
#[test]
fn visible_queries() {
    let fixture = Fixture::new();
    let source = (0..1000)
        .map(|index| format!("fn item_{index}() {{}}\n"))
        .collect::<String>();
    let path = fixture.file("main.rs", &source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    apply(buffer, 0..0, "// 🙂\n");
    buffer.refresh_syntax();
    assert!(buffer.syntax().is_incremental());
    let whole = buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap();
    let bytes = whole.line_to_byte(550)..whole.line_to_byte(553);
    let captures = buffer.syntax().captures(buffer, bytes.clone());
    assert!(!captures.is_empty() && captures.len() <= 32);
    assert!(
        captures
            .iter()
            .all(|capture| capture.bytes.start >= bytes.start && capture.bytes.end <= bytes.end)
    );
    let mut view = View::new(buffer);
    view.top = 550;
    view.left = 3;
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 16,
            height: 4,
        },
        "",
    )
    .unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .flat_map(|row| row.syntax())
            .all(|paint| paint.columns.end <= 16)
    );
    assert!(frame.rows()[0].text().contains("item_549"));
}

// A multibyte paste maps every shared view before new foregrounds appear.
// Region/search/cursor styling and subsequent undo use that same current revision.
#[test]
fn shared_surfaces() {
    let fixture = Fixture::new();
    let source = "fn alpha() { let text = \"界\"; }\n";
    let path = fixture.file("main.rs", source);
    let mut session = Session::empty(Size {
        width: 80,
        height: 12,
    })
    .unwrap();
    session.load(&path).unwrap();
    session.execute(Command::Split(Split::Right));
    session.handle(Event::Paste("// 🙂\r\n".into()));
    compare_session(&mut session);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::ForwardWord));
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.selection().is_some() && !row.syntax().is_empty())
    );
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("alpha".into()));
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.search().is_some() && !row.syntax().is_empty())
    );
    let cursor = frame.cursor().unwrap();
    assert!(cursor.column < 80 && cursor.row < 12);
    let buffer = session.buffer().unwrap();
    for (_, view) in session.windows().iter() {
        view.cell(buffer).unwrap();
        assert_eq!(view.point.stamp, buffer.stamp());
    }
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.execute(Command::History(Direction::Undo));
    compare_session(&mut session);
    assert_eq!(text(session.buffer().unwrap()), source);
    assert!(!session.buffer().unwrap().is_modified());
}
