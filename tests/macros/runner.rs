use super::*;

// One iteration can edit two native buffers. Undo remains buffer-local;
// traversing panes in the definition does not flatten their histories.
#[test]
fn multiple_buffers() {
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.create("").unwrap();
    let second = buffers.create("").unwrap();
    let name = buffers.get(second).unwrap().name().to_owned();
    let view = View::new(buffers.get(first).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    session.execute(Command::Split(rev::command::Split::Below));
    session.execute(Command::Window(rev::command::WindowCommand::Other));
    session.execute(Command::Buffer(rev::command::BufferCommand::Switch));
    feed(&mut session, [Event::Paste(name), key(KeyCode::Enter)]);
    session.execute(Command::Window(rev::command::WindowCommand::Other));
    record(
        &mut session,
        [
            character('a'),
            control('x'),
            character('o'),
            character('b'),
            control('x'),
            character('o'),
        ],
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "aa");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a");
    session.execute(Command::Window(rev::command::WindowCommand::Other));
    assert_eq!(text(&session), "bb");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "b");
}

// External source publication between host turns revokes the admitted
// continuation, preserving both that change and prior automatic effects.
#[test]
fn source_invalidation() {
    let mut session = editor("", "");
    record(&mut session, [character('x')]);
    feed(&mut session, [meta('2'), key(KeyCode::F(4))]);
    session.replace(1..1, "z").unwrap();
    settle(&mut session);
    assert_eq!(text(&session), "xz");
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::OwnerChanged
    );
    assert_eq!(session.macro_status(), Status::Idle);
}

// Disjoint edits within an outer iteration undo together through native
// records. Adjacent typing, other iterations and redo keep their boundaries.
#[test]
fn undo_iterations() {
    let mut session = editor("", "one\ntwo\n");
    record(
        &mut session,
        [
            control('a'),
            character('>'),
            control('n'),
            control('a'),
            character('!'),
        ],
    );
    session.handle(character('z'));
    feed(
        &mut session,
        [control('u'), character('2'), key(KeyCode::F(4))],
    );
    settle(&mut session);
    let finished = text(&session);
    session.execute(Command::History(History::Undo));
    let once = text(&session);
    assert_ne!(finished, once);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), ">one\n!ztwo\n");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), ">one\n!two\n");
    session.execute(Command::History(History::Redo));
    session.execute(Command::History(History::Redo));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), finished);
}

// C-g runs between finite native batches, preserves partial source effects,
// closes the partial undo unit and releases the runner's immediate deadline.
#[test]
fn cancellation() {
    let mut session = editor("", "");
    record(&mut session, (0..300).map(|_| character('x')));
    feed(
        &mut session,
        [
            control('u'),
            character('1'),
            character('0'),
            character('0'),
            character('0'),
            character('0'),
            key(KeyCode::F(4)),
        ],
    );
    session.expire_input(Instant::now());
    let partial = text(&session);
    assert!(partial.len() > 300 && partial.len() < 600);
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(text(&session), partial);
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::Canceled);
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "x".repeat(300));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), partial);
    session.handle(character('z'));
    assert_eq!(text(&session), format!("{partial}z"));
}

// Startup modal state is checked before effects and between iterations.
// Normal/Insert transitions inside a definition still use the native owner.
#[test]
fn modal_entry() {
    let config = "[editor]\ninteraction_profile = 'modal'\n";
    let mut session = editor(config, "");
    record(
        &mut session,
        [character('i'), character('x'), key(KeyCode::Esc)],
    );
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "xx");
    session.handle(character('i'));
    session.handle(key(KeyCode::F(4)));
    assert_eq!(session.macro_status(), Status::Idle);
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::EntryState);
    assert_eq!(text(&session), "xx");

    let mut session = editor(config, "");
    record(&mut session, [character('i'), character('x')]);
    session.handle(key(KeyCode::Esc));
    feed(&mut session, [meta('2'), key(KeyCode::F(4))]);
    settle(&mut session);
    assert_eq!(text(&session), "xx");
    assert_eq!(session.macro_stop().unwrap().reason, StopReason::EntryState);
}

// Native failure retains completed input and stops subsequent logical steps.
// Saturated successful cursor motion remains an ordinary successful step.
#[test]
fn native_stops() {
    let mut session = editor("", "");
    record(&mut session, [character('x'), control('f'), character('y')]);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "xyxy");
    assert_eq!(session.macro_stop(), None);

    let mut boundary = editor("", "ab");
    record(&mut boundary, [control('d'), character('x')]);
    feed(&mut boundary, [meta('2'), key(KeyCode::F(4))]);
    settle(&mut boundary);
    assert_eq!(text(&boundary), "xx");
    let stopped = boundary.macro_stop().unwrap();
    assert_eq!(stopped.reason, StopReason::NativeFailure);
    assert_eq!((stopped.iteration, stopped.step), (2, 1));
    assert!(boundary.echo().contains("iteration 2, step 1"));

    let mut buffers = Buffers::new().unwrap();
    let source = buffers.create("").unwrap();
    let locked = buffers.create("locked source").unwrap();
    buffers.get_mut(locked).unwrap().set_readonly(true);
    let name = buffers.get(locked).unwrap().name().to_owned();
    let view = View::new(buffers.get(source).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 16,
        },
    )
    .unwrap();
    record(&mut session, [character('x'), character('y')]);
    session.execute(Command::Buffer(rev::command::BufferCommand::Switch));
    feed(&mut session, [Event::Paste(name), key(KeyCode::Enter)]);
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "locked source");
    let stop = session.macro_stop().unwrap();
    assert_eq!(stop.reason, StopReason::NativeFailure);
    assert_eq!((stop.iteration, stop.step), (1, 1));
    assert_eq!(session.macro_status(), Status::Idle);
}

// A native async command is refused before its handler acquires a job or
// prompt; the existing last macro and ordinary editing remain usable.
#[test]
fn unsupported_jobs() {
    let mut session = editor("", "");
    record(&mut session, [character('x')]);
    session.handle(key(KeyCode::F(3)));
    session.execute(Command::ShellCommand);
    assert_eq!(
        session.macro_stop().unwrap().reason,
        StopReason::Unsupported
    );
    session.handle(character('z'));
    assert_eq!(text(&session), "xz");
    session.handle(key(KeyCode::F(4)));
    settle(&mut session);
    assert_eq!(text(&session), "xzx");
}
