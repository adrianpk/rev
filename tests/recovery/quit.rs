use super::*;
use rev::command::{BufferCommand, Command};

fn switch(session: &mut Session, name: &str) {
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name.into()));
    session.handle(press(KeyCode::Enter));
}

// Save/discard decisions cover hidden named/unnamed owners once, preserving
// all buffers while earlier completed files and later destinations are distinct.
#[test]
fn sequential_exit() {
    let fixture = Fixture::new();
    let named = fixture.file("named.txt", b"base");
    let unnamed = fixture.0.join("new.txt");
    let mut buffers = Buffers::new().unwrap();
    let scratch = buffers.initial();
    edit(buffers.get_mut(scratch).unwrap(), "discard");
    let named_id = buffers.visit(&named).unwrap();
    edit(buffers.get_mut(named_id).unwrap(), "saved ");
    let unnamed_id = buffers.create("").unwrap();
    edit(buffers.get_mut(unnamed_id).unwrap(), "界\r\nlast");
    let mut session = session(buffers);
    assert_eq!(session.execute(Command::Quit), Action::Redraw);
    assert!(session.echo().contains("*scratch*"));
    assert_eq!(answer(&mut session, 'n'), Action::Redraw);
    assert!(session.echo().contains("named.txt"));
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert_eq!(fs::read(&named).unwrap(), b"saved base");
    assert_eq!(answer(&mut session, 'y'), Action::Redraw);
    assert_eq!(path(&mut session, &unnamed), Action::Exit);
    assert_eq!(fs::read(&unnamed).unwrap(), "界\r\nlast".as_bytes());
    assert_eq!(text(&session), "discard");
    assert!(session.buffer().unwrap().is_modified());
}

// Cancellation at either the per-buffer question or nested unnamed path leaves
// earlier saves effective but neither consumes discard authority nor loses views.
#[test]
fn canceled_progress() {
    for stage in ["question", "path"] {
        let fixture = Fixture::new();
        let named = fixture.file("earlier.txt", b"base");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&named).unwrap();
        edit(buffers.get_mut(id).unwrap(), "saved ");
        let hidden = buffers.create("").unwrap();
        edit(buffers.get_mut(hidden).unwrap(), "retained");
        let hidden_name = buffers.get(hidden).unwrap().name().to_owned();
        let mut session = session(buffers);
        let view = session.view().clone();
        session.execute(Command::Quit);
        answer(&mut session, 'y');
        assert_eq!(fs::read(&named).unwrap(), b"saved base");
        if stage == "path" {
            answer(&mut session, 'y');
        }
        cancel(&mut session);
        assert_eq!(session.view(), &view, "{stage}");
        switch(&mut session, &hidden_name);
        assert_eq!(text(&session), "retained");
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(session.execute(Command::Quit), Action::Redraw);
        assert!(session.echo().contains(&hidden_name));
        assert_eq!(answer(&mut session, 'n'), Action::Exit);
    }
}

// C-g after an earlier discard does not mark its buffer clean; retry asks again.
// Return, repeat and paste cannot silently answer a quit decision.
#[test]
fn decision_authority() {
    use crossterm::event::{KeyEventKind, KeyEventState};
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.initial();
    edit(buffers.get_mut(first).unwrap(), "first");
    let second = buffers.create("").unwrap();
    edit(buffers.get_mut(second).unwrap(), "second");
    let mut session = session(buffers);
    session.execute(Command::Quit);
    let question = session.echo().to_owned();
    for event in [
        press(KeyCode::Enter),
        Event::Paste("n".into()),
        Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        }),
    ] {
        assert_ne!(session.handle(event), Action::Exit);
        assert!(session.echo().starts_with(&question));
        assert_eq!(text(&session), "first");
    }
    answer(&mut session, 'n');
    cancel(&mut session);
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::Quit);
    assert_eq!(session.echo(), question);
    assert_eq!(answer(&mut session, 'n'), Action::Redraw);
    assert_eq!(answer(&mut session, 'n'), Action::Exit);
}

// Required save failure returns to editing; read-only and missing-parent
// rejection cannot grant the next buffer or an automatic later exit.
#[test]
fn failed_saves() {
    for failure in ["readonly", "missing parent"] {
        let fixture = Fixture::new();
        let target = fixture.0.join("missing/target.txt");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.initial();
        edit(buffers.get_mut(id).unwrap(), "retained");
        buffers
            .get_mut(id)
            .unwrap()
            .set_read_only(failure == "readonly");
        let mut session = session(buffers);
        session.execute(Command::Quit);
        answer(&mut session, 'y');
        assert_eq!(path(&mut session, &target), Action::Redraw);
        assert!(session.echo().starts_with("save: "), "{}", session.echo());
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(text(&session), "retained");
        assert!(!target.exists());
        if failure == "missing parent" {
            fs::create_dir(target.parent().unwrap()).unwrap();
            assert_eq!(session.execute(Command::SaveBuffer), Action::Redraw);
            assert_eq!(path(&mut session, &target), Action::Redraw);
            assert_eq!(fs::read(&target).unwrap(), b"retained");
        }
        assert_ne!(answer(&mut session, 'n'), Action::Exit);
    }
}

// Occupied save-as and external disk changes require their own fresh consent;
// declining or canceling that nested decision aborts exit without discarding.
#[test]
fn declined_conflicts() {
    for named in [false, true] {
        for canceling in [false, true] {
            let fixture = Fixture::new();
            let target = fixture.file("target.txt", b"original");
            let mut buffers = Buffers::new().unwrap();
            let id = if named {
                buffers.visit(&target).unwrap()
            } else {
                buffers.initial()
            };
            edit(buffers.get_mut(id).unwrap(), "retained");
            if named {
                fs::write(&target, b"external").unwrap();
            }
            let before = fs::read(&target).unwrap();
            let mut session = session(buffers);
            session.execute(Command::Quit);
            answer(&mut session, 'y');
            if !named {
                path(&mut session, &target);
            }
            assert!(session.echo().contains(target.to_str().unwrap()));
            if canceling {
                cancel(&mut session);
            } else {
                assert_eq!(answer(&mut session, 'n'), Action::Redraw);
            }
            assert_eq!(fs::read(&target).unwrap(), before);
            assert_eq!(session.execute(Command::Quit), Action::Redraw);
            assert_eq!(answer(&mut session, 'n'), Action::Exit);
        }
    }
}
