use super::*;
use rev::command::{Command, RegionCommand, Scroll, Split, TreeCommand};
use rev::file::{DeleteDecision, Deletion};

// Quit focus spans shared views and zero geometry without resetting their
// navigation, mark, history or retained tree focus on cancellation.
#[test]
fn shared_cancel() {
    for tree in [false, true] {
        let fixture = Fixture::new();
        let target = fixture.file("shared.txt", b"first\nshort\nlast");
        let mut session = session(Buffers::new().unwrap());
        session.load(&target).unwrap();
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Split(Split::Below));
        session.insert('界');
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Scroll(Scroll::NextLine));
        if tree {
            session.execute(Command::Tree(TreeCommand::Focus));
        }
        let views: Vec<_> = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect();
        let selected = session.windows().selected_id();
        let buffer = session.buffer().unwrap();
        let mark = buffer.mark();
        let stamp = buffer.stamp();
        let state = buffer.state();
        let history = buffer.history().len();
        let view = session.view().clone();
        session.execute(Command::Quit);
        session.handle(Event::Resize(0, 0));
        assert!(session.frame().unwrap().cursor().is_none());
        session.handle(Event::Resize(240, 12));
        // Zero width may release tree focus; cancel must preserve the focus
        // actually retained after geometry returns.
        let focused = session.tree().focused();
        cancel(&mut session);
        assert_eq!(session.tree().focused(), focused);
        assert_eq!(session.view(), &view);
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, view)| (id, view.clone()))
                .collect::<Vec<_>>(),
            views
        );
        let buffer = session.buffer().unwrap();
        assert_eq!(buffer.mark(), mark);
        assert!(buffer.mark_active());
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.state(), state);
        assert_eq!(buffer.history().len(), history);
        assert_eq!(session.execute(Command::Quit), Action::Redraw);
        // Three editing windows share one dirty owner, hence one decision.
        assert_eq!(answer(&mut session, 'n'), Action::Exit);
    }
}

// Removed destinations require new explicit consent. A tree-deletion warning
// guards even unedited retained text and can recreate its file only on save.
#[test]
fn removed_destination() {
    for deletion in ["external", "tree"] {
        let fixture = Fixture::new();
        let target = fixture.file("removed.txt", "界\r\nlast".as_bytes());
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&target).unwrap();
        if deletion == "external" {
            edit(buffers.get_mut(id).unwrap(), "saved ");
            fs::remove_file(&target).unwrap();
        } else {
            let DeleteDecision::Ready(confirmed) =
                Deletion::capture(&target).unwrap().confirm().unwrap()
            else {
                panic!("file confirmation")
            };
            assert!(buffers.delete(confirmed).is_complete());
            assert!(buffers.get(id).unwrap().is_modified());
        }
        let expected = if deletion == "external" {
            "saved 界\r\nlast"
        } else {
            "界\r\nlast"
        };
        let mut session = session(buffers);
        session.execute(Command::Quit);
        let action = answer(&mut session, 'y');
        if deletion == "external" {
            assert_eq!(action, Action::Redraw);
            assert!(session.echo().contains("changed on disk"));
            assert!(!target.exists());
            assert_eq!(answer(&mut session, 'y'), Action::Exit);
        } else {
            assert_eq!(action, Action::Exit);
        }
        assert_eq!(fs::read(&target).unwrap(), expected.as_bytes());
    }
}

// A later required save failure does not roll back earlier successful saves,
// lose hidden owners, or let a later independent save resume the abandoned exit.
#[test]
fn failed_progress() {
    let fixture = Fixture::new();
    let earlier = fixture.file("earlier.txt", b"base");
    let later = fixture.0.join("absent/later.txt");
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.visit(&earlier).unwrap();
    edit(buffers.get_mut(first).unwrap(), "saved ");
    let last = buffers.visit(&later).unwrap();
    edit(buffers.get_mut(last).unwrap(), "retained");
    let name = buffers.get(last).unwrap().name().to_owned();
    let mut session = session(buffers);
    session.execute(Command::Quit);
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert_eq!(fs::read(&earlier).unwrap(), b"saved base");
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert!(session.echo().contains("not published"));
    assert!(!later.exists());
    session.execute(Command::Buffer(rev::command::BufferCommand::Switch));
    session.handle(Event::Paste(name));
    session.handle(press(KeyCode::Enter));
    assert_eq!(text(&session), "retained");
    assert!(session.buffer().unwrap().is_modified());
    fs::create_dir(later.parent().unwrap()).unwrap();
    assert_eq!(session.execute(Command::SaveBuffer), Action::Redraw);
    assert_eq!(fs::read(later).unwrap(), b"retained");
    assert_eq!(session.execute(Command::Quit), Action::Exit);
}

// Named read-only text rejects required saving immediately, retains its disk
// bytes and history, and permits discard only through another explicit attempt.
#[test]
fn readonly_exit() {
    let fixture = Fixture::new();
    let target = fixture.file("readonly.txt", b"base");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&target).unwrap();
    edit(buffers.get_mut(id).unwrap(), "retained");
    buffers.get_mut(id).unwrap().set_read_only(true);
    let mut session = session(buffers);
    session.execute(Command::Quit);
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert!(session.echo().contains("read-only"));
    assert_eq!(fs::read(&target).unwrap(), b"base");
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert_eq!(answer(&mut session, 'n'), Action::Exit);
    assert_eq!(fs::read(target).unwrap(), b"base");
}
