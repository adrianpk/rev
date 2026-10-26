use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, History, Motion, Split};
use rev::coordinates::{Coordinates, Scalar};
use rev::session::Session;

use super::*;

fn enter(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

fn query(session: &mut Session, command: Command, value: &str) {
    session.execute(command);
    session.handle(Event::Paste(value.into()));
}

fn coherent(session: &mut Session) {
    session.frame().unwrap();
    let buffer = session.buffer().unwrap();
    let reference = syntax(buffer, Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    assert_eq!(buffer.mode(), "Rust");
    assert_eq!(
        buffer.syntax().captures(buffer, 0..buffer.len_bytes()),
        reference.captures(buffer, 0..buffer.len_bytes())
    );
    let coordinates = Coordinates::new(buffer).unwrap();
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, buffer.stamp());
        assert!(coordinates.is_boundary(view.point.position).unwrap());
        coordinates.to_cell(view.point.position).unwrap();
    }
}

// The oracle is the actual adjacent chunks, not an assumed Ropey chunk size.
// A long comment lets a unique literal cross their seam after accepted edits.
fn chunk_search(session: &mut Session) {
    let buffer = session.buffer().unwrap();
    let span = buffer.span(0..buffer.len_chars()).unwrap();
    let whole = buffer.slice(&span).unwrap();
    let mut chunks = whole.chunks();
    let first = chunks.next().unwrap();
    if chunks.next().is_none() {
        return;
    }
    let seam = first.chars().count();
    let range = seam - 3..seam + 5;
    let needle = whole.slice(range.clone()).to_string();
    let source = whole.to_string();
    let expected = source[..source.find(&needle).unwrap()].chars().count();
    assert_eq!(
        expected, range.start,
        "seam query must have a unique first occurrence"
    );
    session.execute(Command::Motion(Motion::BufferStart));
    query(session, Command::SearchForward, &needle);
    let found = session.search_match().unwrap();
    assert_eq!(found.range, expected..expected + needle.chars().count());
    assert!(range.start < seam && range.end > seam);
    session.execute(Command::KeyboardQuit);
    assert_eq!(session.view().point.position, Scalar(0));
}

// Save/reopen fidelity is checked after shared Unicode paste, literal search,
// replacement and whole-grapheme deletion, then through every history state.
// Long single lines span actual rope chunks; BOM, separators and EOF are bytes.
#[test]
fn unicode_roundtrip() {
    let long = (0..900)
        .map(|n| format!(" {n}界🙂e\u{301}\t"))
        .collect::<String>();
    for (name, bom, source, newline) in [
        ("empty", "", String::new(), "\n"),
        (
            "one-line",
            "",
            "fn base(){let s=\"界🙂e\u{301}\t\";}".into(),
            "\n",
        ),
        ("lf", "", "fn base() {}\n".into(), "\n"),
        (
            "crlf-bom",
            "\u{feff}",
            "// 界e\u{301}🙂\r\nfn base() {}".into(),
            "\r\n",
        ),
        (
            "mixed-bom",
            "\u{feff}",
            "// 界\nfn base(){\r\n\tlet s=\"🙂\";\r\n}\n".into(),
            "\r\n",
        ),
        (
            "long-bom",
            "\u{feff}",
            format!("/*{long}*/\r\nfn base() {{}}"),
            "\r\n",
        ),
    ] {
        let fixture = Fixture::new();
        let path = fixture.file(&format!("{name}.rs"), &format!("{bom}{source}"));
        let mut session = Session::empty(Size {
            width: 61,
            height: 10,
        })
        .unwrap();
        session.load(&path).unwrap();
        coherent(&mut session);
        session.execute(Command::Split(Split::Right));
        let selected = session.windows().selected_id();
        let peer = session.windows().order()[1];
        session.select_view(peer).unwrap();
        session.execute(Command::Motion(Motion::BufferEnd));
        session.select_view(selected).unwrap();
        let prefix = format!("// needle 👩\u{200d}💻e\u{301}\t界{newline}");
        session.handle(Event::Paste(prefix.clone()));
        assert_eq!(
            text(session.buffer().unwrap()),
            format!("{prefix}{source}"),
            "{name}"
        );
        assert_eq!(
            session.windows().get(peer).unwrap().point.position.0,
            prefix.chars().count() + source.chars().count()
        );
        coherent(&mut session);
        if name == "long-bom" {
            let buffer = session.buffer().unwrap();
            assert!(
                buffer
                    .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                    .unwrap()
                    .chunks()
                    .count()
                    > 1
            );
        }
        chunk_search(&mut session);
        session.execute(Command::Motion(Motion::BufferStart));
        query(&mut session, Command::SearchForward, "needle");
        assert_eq!(session.search_match().unwrap().range, 3..9);
        enter(&mut session);
        let origin = session.view().clone();
        query(&mut session, Command::SearchBackward, "NEEDLE");
        assert!(session.search_match().is_none());
        session.execute(Command::KeyboardQuit);
        assert_eq!(session.view(), &origin);
        session.execute(Command::Motion(Motion::BufferStart));
        query(&mut session, Command::QueryReplace, "needle");
        enter(&mut session);
        session.handle(Event::Paste("界e\u{301}🙂".into()));
        enter(&mut session);
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('y'),
            KeyModifiers::NONE,
        )));
        let replaced = format!("{}{}", prefix.replacen("needle", "界e\u{301}🙂", 1), source);
        assert_eq!(text(session.buffer().unwrap()), replaced);
        session.execute(Command::Motion(Motion::BufferStart));
        query(&mut session, Command::SearchForward, "e\u{301}");
        enter(&mut session);
        session.execute(Command::BackwardDeleteChar);
        let expected = replaced.replacen("e\u{301}", "", 1);
        assert_eq!(text(session.buffer().unwrap()), expected);
        assert_eq!(session.buffer().unwrap().history().len(), 3);
        coherent(&mut session);
        session.execute(Command::SaveBuffer);
        assert!(
            !session.buffer().unwrap().is_modified(),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            format!("{bom}{expected}").as_bytes()
        );
        let saved = session.buffer().unwrap().state();
        for previous in [&replaced, &format!("{prefix}{source}"), &source] {
            session.execute(Command::History(History::Undo));
            assert_eq!(text(session.buffer().unwrap()), *previous, "{name}");
            assert!(session.buffer().unwrap().is_modified());
            coherent(&mut session);
        }
        for next in [&format!("{prefix}{source}"), &replaced, &expected] {
            session.execute(Command::History(History::Redo));
            assert_eq!(text(session.buffer().unwrap()), *next, "{name}");
            coherent(&mut session);
        }
        assert_eq!(session.buffer().unwrap().state(), saved);
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(
            fs::read(&path).unwrap(),
            format!("{bom}{expected}").as_bytes()
        );
        let mut reopened = Session::empty(Size {
            width: 61,
            height: 10,
        })
        .unwrap();
        reopened.load(&path).unwrap();
        assert_eq!(text(reopened.buffer().unwrap()), expected, "{name}");
        assert_eq!(reopened.buffer().unwrap().len_bytes(), expected.len());
        assert!(reopened.buffer().unwrap().history().is_empty());
        assert!(!reopened.buffer().unwrap().is_modified());
        coherent(&mut reopened);
    }
}
