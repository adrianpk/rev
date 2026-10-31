use crossterm::event::{Event, KeyCode};
use rev::command::Command;
use rev::display::Band;

use super::acceptance::{editor, key, query, start, text};
use super::fixture::Fixture;

// Both native prompts complete a selected ambiguous directory, then another
// child after zero-row clipping. Tab preserves the root and dirty source until
// explicit acceptance; cancellation restores the same editing view and history.
#[test]
fn selected_directory() {
    for extended in [false, true] {
        for accept in [false, true] {
            let fixture = Fixture::new();
            let mut session = editor(&fixture);
            fixture.write("root/alpha/unused", "not visited");
            fixture.write("root/alpine/first.txt", "first child");
            let target = fixture.write("root/alpine/界 name.txt", "selected child");
            let source = fixture.0.join("root/source.txt");
            session.insert('!');
            session.frame().unwrap();
            let view = session.view().clone();
            let stamp = session.buffer().unwrap().stamp();
            let history = session.buffer().unwrap().history().len();
            let root = session.tree().root().map(std::path::Path::to_owned);
            start(&mut session, extended);
            session.handle(Event::Paste("al".into()));
            assert_eq!(selected(&mut session), "alpha/");
            session.handle(key(KeyCode::Down));
            assert_eq!(selected(&mut session), "alpine/");
            session.handle(key(KeyCode::Tab));
            assert_eq!(query(&session.frame().unwrap()), "Find file: alpine/");
            assert_eq!(selected(&mut session), "first.txt");
            session.handle(key(KeyCode::Down));
            assert_eq!(selected(&mut session), "界 name.txt");
            session.handle(Event::Resize(80, 4));
            assert!(
                !session
                    .frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .any(|row| row.band() == Band::Completion)
            );
            session.handle(key(KeyCode::Tab));
            session.handle(Event::Resize(80, 14));
            assert_eq!(
                query(&session.frame().unwrap()),
                "Find file: alpine/界 name.txt"
            );
            assert_eq!(session.tree().root().map(std::path::Path::to_owned), root);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().history().len(), history);
            assert_eq!(text(&session), "!retained\ntext");
            if accept {
                session.handle(key(KeyCode::Enter));
                assert_eq!(session.buffer().unwrap().file().unwrap().path(), target);
                assert_eq!(text(&session), "selected child");
                session.load(&source).unwrap();
            } else {
                session.execute(Command::KeyboardQuit);
                session.frame().unwrap();
                assert_eq!(*session.view(), view);
            }
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().history().len(), history);
            assert_eq!(text(&session), "!retained\ntext");
            assert_eq!(std::fs::read(source).unwrap(), b"retained\ntext");
            assert_eq!(std::fs::read(target).unwrap(), b"selected child");
        }
    }
}

fn selected(session: &mut rev::session::Session) -> String {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.selection().is_some())
        .unwrap()
        .text()
        .trim()
        .to_owned()
}
