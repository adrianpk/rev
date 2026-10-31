use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::config::prepare;
use rev::display::{Band, Frame};
use rev::file::completion::{ENTRY_LIMIT, Navigation};
use rev::session::Session;
use rev::view::{Size, View};

use super::fixture::Fixture;

pub(super) fn configured(fixture: &Fixture, config: &str) -> Session {
    let snapshot = prepare(&fixture.write("config.toml", config));
    assert!(snapshot.diagnostic.is_none());
    let buffers = Buffers::with_settings(snapshot.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 14,
        },
    )
    .unwrap();
    session
        .load(&fixture.write("root/document.txt", "retained\ntext"))
        .unwrap();
    session
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn selected(frame: &Frame) -> &str {
    frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .unwrap()
        .text()
        .trim()
}

fn query(frame: &Frame) -> &str {
    frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Echo)
        .unwrap()
        .text()
}

// Relative, absolute, dot-parent and home spellings retain their own editable
// prefix through completion, including spaces and multibyte directory names.
#[test]
fn path_spellings() {
    let fixture = Fixture::new();
    fixture.write("base/alpha.txt", "retained");
    fixture.write("home/界 dir/solo.txt", "retained");
    let base = fixture.0.join("base");
    let home = fixture.0.join("home");
    for (draft, expected) in [
        ("./al".to_owned(), "./alpha.txt".to_owned()),
        ("../home/界 d".to_owned(), "../home/界 dir/".to_owned()),
        ("~/界 d".to_owned(), "~/界 dir/".to_owned()),
        ("~/界 dir/so".to_owned(), "~/界 dir/solo.txt".to_owned()),
        (
            format!("{}/al", base.display()),
            format!("{}/alpha.txt", base.display()),
        ),
    ] {
        let mut navigation = Navigation::new(base.clone(), Some(home.clone()));
        navigation.update(&draft, false);
        assert_eq!(navigation.complete().unwrap(), Some(expected), "{draft}");
    }
}

// Selection scrolls within the configured rows and survives zero-row suspension.
// Every tiny frame stays bounded; cancel restores the same view and file state.
#[test]
fn resized_selection() {
    let fixture = Fixture::new();
    let mut session = configured(
        &fixture,
        "tree.visible = true\nminibuffer.candidate_limit = 2",
    );
    for index in 0..20 {
        fixture.write(&format!("root/file{index:02}.txt"), "retained");
    }
    session.frame().unwrap();
    let view = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("fi".into()));
    for _ in 0..12 {
        session.handle(key(KeyCode::Down));
        session.frame().unwrap();
    }
    assert_eq!(selected(&session.frame().unwrap()), "file12.txt");
    for (width, height) in [(0, 0), (1, 1), (3, 2), (80, 4), (80, 14)] {
        session.handle(Event::Resize(width, height));
        bounded(&session.frame().unwrap(), width, height);
    }
    let restored = session.frame().unwrap();
    assert_eq!(selected(&restored), "file12.txt");
    assert!(query(&restored).starts_with("Find file: fi"));
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
    assert_eq!(*session.view(), view);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
}

fn bounded(frame: &Frame, width: u16, height: u16) {
    assert_eq!(frame.rows().len(), usize::from(height));
    for row in frame.rows() {
        assert!(row.row() < height);
        assert_eq!(
            unicode_width::UnicodeWidthStr::width(row.text()),
            usize::from(width)
        );
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < width && cursor.row < height);
    }
}

// A bounded subset retains its incomplete warning. Tab fills its visible first
// identity without implying uniqueness or requiring an extra navigation gesture.
#[test]
fn incomplete_surface() {
    let fixture = Fixture::new();
    let mut session = configured(&fixture, "minibuffer.candidate_limit = 2");
    for index in 0..=ENTRY_LIMIT {
        fixture.write(&format!("root/p{index:04}"), "");
    }
    session.execute(Command::FindFile);
    session.handle(Event::Paste("p".into()));
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().starts_with("Incomplete candidate list"))
    );
    let name = selected(&frame).to_owned();
    session.handle(key(KeyCode::Tab));
    let frame = session.frame().unwrap();
    assert!(query(&frame).starts_with(&format!("Find file: {name}")));
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().starts_with("Incomplete candidate list"))
    );
    assert!(!session.buffer().unwrap().is_modified());
}

// Completion and paste use the real configured prompt bound. A refused selected
// name keeps the existing draft, focus, selection, buffer identity and history.
#[test]
fn bounded_completion() {
    let fixture = Fixture::new();
    let mut session = configured(
        &fixture,
        "minibuffer.input_limit = 3\nminibuffer.candidate_limit = 1",
    );
    fixture.write("root/alpha", "retained");
    fixture.write("root/alpine", "retained");
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("al".into()));
    session.handle(key(KeyCode::Tab));
    assert_eq!(session.echo(), "prompt input limit exceeded");
    assert!(query(&session.frame().unwrap()).starts_with("Find file: al  "));
    session.handle(key(KeyCode::Down));
    session.handle(key(KeyCode::Tab));
    assert_eq!(session.echo(), "prompt input limit exceeded");
    let frame = session.frame().unwrap();
    assert!(query(&frame).starts_with("Find file: al  "));
    assert_eq!(selected(&frame), "alpine");
    session.handle(Event::Paste("界é😀a".into()));
    assert!(query(&session.frame().unwrap()).starts_with("Find file: al  "));
    session.execute(Command::KeyboardQuit);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}
