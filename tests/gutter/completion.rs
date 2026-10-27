use std::time::{Duration, Instant};

use crossterm::event::Event;
use rev::command::{Command, Motion};
use rev::display::Band;
use rev::session::Action;
use unicode_width::UnicodeWidthStr;

use super::config_fixture::Fixture;

// Real file candidates retain their own unnumbered region while all editing
// views keep their full navigation/policy state, even through zero suspension.
#[test]
fn file_candidates() {
    let fixture = Fixture::new();
    fixture.write("alpha.txt", "alpha");
    fixture.write("alpine.txt", "alpine");
    let mut session = super::surfaces::editor(&fixture, true, true);
    session.execute(Command::Motion(Motion::BufferEnd));
    session.frame().unwrap();
    let before: Vec<_> = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.clone()))
        .collect();
    let source = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    session.execute(Command::FindFile);
    let draft = format!("{}/al", fixture.0.display());
    session.handle(Event::Paste(draft.clone()));
    for (width, height) in [
        (61, 13),
        (19, 8),
        (0, 0),
        (1, 1),
        (32, 6),
        (120, 24),
        (61, 13),
    ] {
        session.handle(Event::Resize(width, height));
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
                .is_none_or(|p| p.column < width && p.row < height)
        );
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            before
        );
    }
    let frame = session.frame().unwrap();
    let candidates: Vec<_> = frame
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim())
        .collect();
    assert_eq!(candidates, ["alpha.txt", "alpine.txt"]);
    let query = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Echo)
        .unwrap();
    assert!(query.text().starts_with(&format!("Find file: {draft}")));
    assert_eq!(frame.cursor().unwrap().row, query.row());
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        source
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(60)),
        Action::Wait
    );
}
