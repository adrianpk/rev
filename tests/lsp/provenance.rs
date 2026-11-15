use super::candidates::*;
use super::fixture::Fixture;
use rev::command::{Command, CompletionCommand as C, History};
use rev::macros::Status;
use serde_json::json;

// Semantic completion returns unmatched punctuation through an owned local
// child. Accepted macro replay stays literal with completion disabled and cannot
// ask that provider again or synthesize pairing keys from its accepted text.
#[test]
fn literal_completion() {
    let f = Fixture::new();
    let payload = "([{\"界e\u{301}";
    let mut s = session(
        &f,
        "ready",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &json!([{"label":"cfg literal", "filterText":"cfg", "insertText":payload}]),
        false,
    );
    let point = s.view().point.position;
    let original = text(&s);
    key(&mut s, crossterm::event::KeyCode::F(3));
    assert_eq!(s.macro_status(), Status::Recording);
    s.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::ALT,
        ),
    ));
    settle(&mut s);
    assert!(
        s.completion().is_some(),
        "{}: {:?}",
        s.echo(),
        s.macro_stop()
    );
    assert_eq!(labels(&s), ["cfg literal"]);
    key(&mut s, crossterm::event::KeyCode::Enter);
    settle(&mut s);
    key(&mut s, crossterm::event::KeyCode::F(4));
    let expected = RUST
        .replace('|', "")
        .replace("{ cfg }", &format!("{{ {payload} }}"));
    assert_eq!(text(&s), expected);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    let requests = frames(&f)
        .iter()
        .filter(|v| v["method"] == "textDocument/completion")
        .count();
    assert_eq!(requests, 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), original);
    assert_eq!(s.view().point.position, point);
    s.execute(Command::Completion(C::Mode));
    key(&mut s, crossterm::event::KeyCode::F(4));
    settle(&mut s);
    assert_eq!(s.macro_status(), Status::Idle);
    assert_eq!(s.macro_stop(), None);
    assert_eq!(text(&s), expected);
    assert_eq!(
        frames(&f)
            .iter()
            .filter(|v| v["method"] == "textDocument/completion")
            .count(),
        requests
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), original);
}
