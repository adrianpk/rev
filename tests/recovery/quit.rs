use super::*;
use rev::command::{BufferCommand, Command};

fn switch(session: &mut Session, name: &str) {
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name.into()));
    session.handle(press(KeyCode::Enter));
}

// Both names and the existing binding enter the same exit coordinator. Named
// entry works in both profiles; save/discard/cancel preserve their authority.
#[test]
fn quit_alias_decisions() {
    for profile in ["non-modal", "modal"] {
        for entry in ["quit", "save-buffers-kill-terminal", "C-x C-c"] {
            for (choice, response) in [
                ("clean", None),
                ("save", Some('y')),
                ("discard", Some('n')),
                ("discard-alias", Some('d')),
                ("discard-uppercase", Some('D')),
                ("cancel", None),
            ] {
                check_quit_decision(profile, entry, choice, response);
            }
        }
    }
}

fn check_quit_decision(profile: &str, entry: &str, choice: &str, response: Option<char>) {
    let fixture = Fixture::new();
    let settings = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false"
    );
    let startup = rev::config::prepare(&fixture.file("config.toml", settings.as_bytes()));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let target = fixture.file("notes.txt", b"base");
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&target).unwrap();
    if choice != "clean" {
        edit(buffers.get_mut(id).unwrap(), "changed ");
    }
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap();
    let action = if entry == "C-x C-c" {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
        )));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )))
    } else {
        let open = if profile == "modal" {
            press(KeyCode::Char(':'))
        } else {
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
        };
        session.handle(open);
        session.handle(Event::Paste(entry.into()));
        session.handle(press(KeyCode::Enter))
    };
    if choice == "clean" {
        assert_eq!(action, Action::Exit, "{profile}: {entry}");
    } else {
        assert_eq!(action, Action::Redraw, "{profile}: {entry}: {choice}");
        assert!(session.echo().starts_with("Quit:"), "{}", session.echo());
        assert!(session.echo().ends_with("y=save, n=discard, C-g=cancel"));
        match response {
            Some(response) => assert_eq!(answer(&mut session, response), Action::Exit),
            None => {
                if profile == "modal" {
                    let now = std::time::Instant::now();
                    session.handle_at(press(KeyCode::Esc), now);
                    session.expire_input(now + rev::input::META_TIMEOUT);
                } else {
                    cancel(&mut session);
                }
                assert_eq!(session.echo(), "Quit");
                assert!(session.buffer().unwrap().is_modified());
                assert_eq!(text(&session), "changed base");
            }
        }
    }
    assert_eq!(
        fs::read(&target).unwrap(),
        if choice == "save" {
            b"changed base".as_slice()
        } else {
            b"base".as_slice()
        }
    );
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
    for response in ['n', 'd', 'D'] {
        check_decision_authority(response);
    }
}

fn check_decision_authority(response: char) {
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
        Event::Paste(response.to_string()),
        Event::Key(KeyEvent {
            code: KeyCode::Char(response),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        }),
        Event::Key(KeyEvent::new(
            KeyCode::Char(response),
            KeyModifiers::CONTROL,
        )),
    ] {
        assert_ne!(session.handle(event), Action::Exit);
        assert!(session.echo().starts_with(&question));
        assert_eq!(text(&session), "first");
    }
    answer(&mut session, response);
    let second = session.echo().to_owned();
    let repeat = Event::Key(KeyEvent {
        code: KeyCode::Char(response),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Repeat,
        state: KeyEventState::NONE,
    });
    assert_ne!(session.handle(repeat), Action::Exit);
    assert!(session.echo().starts_with(&second));
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
            .set_readonly(failure == "readonly");
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
                let question = session.echo().to_owned();
                assert_eq!(answer(&mut session, 'd'), Action::Redraw);
                assert!(session.echo().starts_with(&question));
                assert_eq!(answer(&mut session, 'n'), Action::Redraw);
            }
            assert_eq!(fs::read(&target).unwrap(), before);
            assert_eq!(session.execute(Command::Quit), Action::Redraw);
            assert_eq!(answer(&mut session, 'n'), Action::Exit);
        }
    }
}
