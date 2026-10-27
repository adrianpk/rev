use super::*;
use rev::command::{Split, TreeCommand};
use unicode_width::UnicodeWidthStr;

// All admitted row limits apply to a catalog larger than one screen, including
// an empty query. Opening the list never dispatches its highlighted first row.
#[test]
fn candidate_limits() {
    for limit in [1, 8, 32] {
        let mut session = configured(
            &format!("minibuffer.candidate_limit = {limit}"),
            Size {
                width: 80,
                height: 64,
            },
        );
        let stamp = session.buffer().unwrap().stamp();
        query(&mut session, "");
        let frame = session.frame().unwrap();
        assert_eq!(names(&frame).len(), limit);
        assert_eq!(names(&frame)[0], "backward-char");
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Nested panes, retained tree ownership and the command surface remain bounded
// through suspension. Candidate allocation cannot alter view identities/points.
#[test]
fn workspace_resize() {
    let mut session = configured(
        "tree.visible = true\nminibuffer.candidate_limit = 32",
        size(),
    );
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Split(Split::Below));
    session.execute(Command::Tree(TreeCommand::Focus));
    let views = session.windows().order();
    let points: Vec<_> = session
        .windows()
        .iter()
        .map(|(_, view)| view.point)
        .collect();
    query(&mut session, "kill");
    for (width, height) in [(80, 14), (31, 8), (7, 3), (1, 1), (0, 0), (80, 14)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows().len(), usize::from(height));
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        assert!(
            frame
                .cursor()
                .is_none_or(|cursor| cursor.column < width && cursor.row < height)
        );
        assert_eq!(session.windows().order(), views);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(_, view)| view.point)
                .collect::<Vec<_>>(),
            points
        );
    }
    let frame = session.frame().unwrap();
    assert!(!names(&frame).is_empty());
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.band() == Band::Echo && row.text().starts_with("M-x kill"))
    );
    session.execute(Command::KeyboardQuit);
    assert!(names(&session.frame().unwrap()).is_empty());
}

// Clipping keeps full graphemes/cell width while the cursor follows the editable
// query. A Unicode no-match draft survives tiny frames and returns exactly.
#[test]
fn unicode_draft() {
    let mut session = configured("", size());
    let draft = "界e\u{301}".repeat(30);
    query(&mut session, &draft);
    for width in [0, 1, 2, 3, 7, 16, 200] {
        session.handle(Event::Resize(width, 14));
        let frame = session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        assert!(
            frame
                .cursor()
                .is_none_or(|cursor| cursor.column < width && cursor.row < 14)
        );
        assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    }
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.band() == Band::Echo && row.text().starts_with(&format!("M-x {draft}")))
    );
    assert_eq!(names(&frame), ["No matching commands"]);
    assert_eq!(session.buffer().unwrap().len_chars(), 0);
}
