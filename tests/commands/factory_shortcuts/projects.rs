use super::*;
use rev::context::{self, Context, Store};
use rev::input::EditingState;
use std::time::{Duration, Instant};

fn seed(fixture: &Fixture) -> context::ProjectId {
    let source = fixture.write("root/file", "a界e\u{301}\nsecond\n");
    let root = context::Root {
        id: context::RootId::new().unwrap(),
        path: context::NativePath::absolute(source.parent().unwrap()).unwrap(),
        label: context::Name::new("Source").unwrap(),
        origin_project_id: None,
    };
    let id = context::ProjectId::new().unwrap();
    let reference = context::SessionRef {
        owner: context::Owner::Project(id.clone()),
        revision: context::Revision::INITIAL,
    };
    let file = context::FileReference::capture(&source, std::slice::from_ref(&root)).unwrap();
    let (store, empty) = Store::open(&fixture.0.join("state/contexts.toml")).unwrap();
    let mut candidate = empty.clone();
    candidate.projects.push(context::Project {
        id: id.clone(),
        revision: context::Revision::INITIAL,
        name: context::Name::new("Source").unwrap(),
        roots_revision: context::Revision::INITIAL,
        root,
        session_ref: reference.clone(),
    });
    candidate.sessions.push(context::SavedSession::new(
        reference,
        context::Revision::INITIAL,
        context::Snapshot {
            files: vec![context::FileEntry {
                file: file.clone(),
                cursor: context::SourcePosition { line: 1, column: 3 },
                viewport: None,
            }],
            active_file: Some(file),
        },
    ));
    let published = store.publish(&empty.prepare(candidate).unwrap()).unwrap();
    assert!(published.durability_warning.is_none());
    id
}

fn ready(session: &mut Session, mut condition: impl FnMut(&mut Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition(session) {
        assert!(
            Instant::now() < deadline,
            "Project operation did not settle: {}",
            session.echo()
        );
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn prompt(session: &mut Session, label: &str) -> bool {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .any(|row| row.text().contains(label))
}

fn choose(session: &mut Session) {
    keys(session, "C-x p p");
    ready(session, |s| prompt(s, "Project:"));
    session.handle(Event::Paste("Source".into()));
    keys(session, "TAB RET");
}

// Every editing state enters the real Project picker. Cancellation preserves
// unsaved text and metadata; restoration still requires the dirty-source choice
// and loads the registered ID, root, file and saved Unicode scalar position.
#[test]
fn project_restore() {
    for state in STATES {
        let fixture = Fixture::new();
        let project = seed(&fixture);
        let path = fixture.0.join("state/contexts.toml");
        let catalog = std::fs::read(&path).unwrap();
        let mut session = editor(profile(state), "retained", Mode::Text);
        session.set_context_storage(path.clone()).unwrap();
        session.insert('!');
        enter(&mut session, state);
        let source = text(&session);
        let stamp = session.buffer().unwrap().stamp();
        // Native quit retires Visual selection to Normal, including in prompts.
        let cancelled_state = match state {
            Some(
                ModalCommand::VisualCharacter
                | ModalCommand::VisualLine
                | ModalCommand::VisualBlock,
            ) => Some(EditingState::Normal),
            _ => session.editing_state(),
        };
        let before = (
            session.view().point,
            cancelled_state,
            session.buffer().unwrap().history().len(),
        );
        keys(&mut session, "C-x p p");
        ready(&mut session, |s| prompt(s, "Project:"));
        keys(&mut session, "C-g");
        assert_eq!(text(&session), source);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(
            (
                session.view().point,
                session.editing_state(),
                session.buffer().unwrap().history().len()
            ),
            before
        );
        assert_eq!(std::fs::read(&path).unwrap(), catalog);
        choose(&mut session);
        ready(&mut session, |s| s.echo().contains("Leave context"));
        keys(&mut session, "C-g");
        assert_eq!(text(&session), source);
        assert_eq!(session.context(), &Context::Standalone);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        choose(&mut session);
        ready(&mut session, |s| s.echo().contains("Leave context"));
        keys(&mut session, "n");
        ready(&mut session, |s| {
            s.context() == &Context::Project(project.clone())
        });
        assert_eq!(text(&session), "a界e\u{301}\nsecond\n");
        assert_eq!(session.view().point.position, Scalar(2));
        assert_eq!(
            session.tree().root(),
            Some(fixture.0.join("root").as_path())
        );
        assert_eq!(std::fs::read(&path).unwrap(), catalog);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("root/file")).unwrap(),
            "a界e\u{301}\nsecond\n"
        );
    }
}
