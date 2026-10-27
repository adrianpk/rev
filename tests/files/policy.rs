use rev::config::{IndentStyle, prepare};

use super::*;

fn configured(fixture: &Fixture) -> Buffers {
    let root = fixture.file(
        "config.toml",
        b"editor.tab_width = 4\nfiles.newline = 'lf'\nfiles.bom = false\n[language.go]\ntab_width = 8\nindent_width = 2\nindent_style = 'tabs'",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    // Disk edits after startup cannot change the snapshot used by publication.
    fs::write(root, b"language.go.tab_width = 3").unwrap();
    Buffers::with_settings(startup.snapshot).unwrap()
}

// Reassociation adopts language editing policy without applying new-file
// format defaults. Saved identity, revision and history follow publication.
#[test]
fn save_policy() {
    for source in ["", "a\r\nb", "\u{feff}a\r\nb\nlast"] {
        let fixture = Fixture::new();
        let mut buffers = configured(&fixture);
        let path = fixture.file("old.txt", source.as_bytes());
        let id = buffers.visit(&path).unwrap();
        insert_text(buffers.get_mut(id).unwrap(), "!");
        let buffer = buffers.get(id).unwrap();
        let before = (
            buffer.stamp(),
            buffer.state(),
            buffer.newline().to_owned(),
            buffer.has_bom(),
        );
        let expected = bytes(buffer);
        let target = fixture.0.join("new.go");
        let save = buffers.prepare_save(id, Some(&target)).unwrap();
        assert_eq!(buffers.get(id).unwrap().policy().tab_width, 4);
        assert!(buffers.publish_save(save).is_complete());
        let buffer = buffers.get_mut(id).unwrap();
        assert_eq!(buffer.file().unwrap().path(), target);
        assert_eq!(buffer.policy().tab_width, 8);
        assert_eq!(buffer.policy().indent_width, 2);
        assert_eq!(buffer.policy().indent_style, IndentStyle::Tabs);
        assert_eq!(
            (
                buffer.stamp(),
                buffer.state(),
                buffer.newline().to_owned(),
                buffer.has_bom()
            ),
            before
        );
        assert_eq!(buffer.history().len(), 1);
        assert!(!buffer.is_modified());
        assert_eq!(bytes(buffer), expected);
        buffer.navigate(History::Undo, &mut [], None).unwrap();
        assert!(buffer.is_modified());
        assert_eq!(bytes(buffer), source.as_bytes());
        assert_eq!(buffer.policy().tab_width, 8);
        buffer.navigate(History::Redo, &mut [], None).unwrap();
        assert!(!buffer.is_modified());
        assert_eq!(bytes(buffer), expected);
        assert_eq!(fs::read(&target).unwrap(), expected);
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
        assert_eq!(buffers.visit(&target).unwrap(), id);
    }
}

// Real staged writes distinguish prepublication failures from unresolved
// completion. Policy follows the actual published path, never a requested path.
#[test]
fn publication_policy() {
    for fault in [
        Fault::Write,
        Fault::Replace,
        Fault::DirectorySync,
        Fault::Observe,
        Fault::None,
    ] {
        let fixture = Fixture::new();
        let mut buffers = configured(&fixture);
        let path = fixture.file("old.txt", b"\xef\xbb\xbfa\r\nb");
        let id = buffers.visit(&path).unwrap();
        insert_text(buffers.get_mut(id).unwrap(), "!");
        let buffer = buffers.get(id).unwrap();
        let before = (buffer.stamp(), buffer.state());
        let expected = bytes(buffer);
        let target = fixture.0.join("new.go");
        let save = buffers.prepare_save(id, Some(&target)).unwrap();
        let outcome = buffers.publish_with(save, &mut Failing::new(fault, &fixture.0));
        let published = outcome.publication() != Publication::NotPublished;
        let buffer = buffers.get(id).unwrap();
        assert_eq!(
            buffer.policy().tab_width,
            if published { 8 } else { 4 },
            "{fault:?}"
        );
        assert_eq!(
            buffer.file().unwrap().path(),
            if published { &target } else { &path }
        );
        assert_eq!((buffer.stamp(), buffer.state()), before);
        assert_eq!(buffer.history().len(), 1);
        assert_eq!(buffer.is_modified(), !outcome.is_complete());
        assert_eq!(bytes(buffer), expected);
        assert_eq!(target.exists(), published);
        assert!(temporaries(&fixture.0).is_empty());
    }
}

// A real write-file prompt leaves both shared views untouched on cancel or
// failure. Success reprojects their independent points/goals and preserves
// byte fidelity, one existing edit and the saved identity through undo/redo.
#[test]
fn shared_policy() {
    use rev::command::{History as Replay, Motion, Scroll, Split};
    use rev::coordinates::Cell;

    let fixture = Fixture::new();
    let mut buffers = configured(&fixture);
    let path = fixture.file("old.txt", b"\xef\xbb\xbf\tA\r\n\tB");
    let occupied = fixture.file("occupied.go", b"external");
    let id = buffers.visit(&path).unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "!");
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = rev::coordinates::Scalar(2);
    view.column = Some(4);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 8,
        },
    )
    .unwrap();
    let origin = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Motion(Motion::BackwardChar));
    session.execute(Command::Scroll(Scroll::PreviousLine));
    session.execute(Command::Scroll(Scroll::NextLine));
    session.select_view(origin).unwrap();
    let before = (
        session.view().point,
        session.windows().get(peer).unwrap().point,
    );
    write_path(&mut session, &occupied);
    assert!(session.echo().starts_with("Overwrite occupied destination"));
    session.execute(Command::KeyboardQuit);
    write_path(&mut session, &fixture.0.join("missing/new.go"));
    assert!(session.echo().starts_with("save: not published"));
    assert_eq!(session.buffer().unwrap().policy().tab_width, 4);
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), path);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(
        (
            session.view().point,
            session.windows().get(peer).unwrap().point
        ),
        before
    );
    let target = fixture.0.join("new.go");
    write_path(&mut session, &target);
    assert!(session.echo().starts_with("Saved "));
    session.frame().unwrap();
    assert_eq!(
        (
            session.view().point,
            session.windows().get(peer).unwrap().point
        ),
        before
    );
    for (id, row) in [(origin, 0), (peer, 1)] {
        let view = session.windows().get(id).unwrap();
        assert_eq!(
            view.cell(session.buffer().unwrap()).unwrap(),
            Cell { row, column: 8 }
        );
        assert_eq!(view.column, None);
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(fs::read(&target).unwrap(), b"\xef\xbb\xbf!\tA\r\n\tB");
    assert_eq!(fs::read(&path).unwrap(), b"\xef\xbb\xbf\tA\r\n\tB");
    assert_eq!(fs::read(&occupied).unwrap(), b"external");
    session.select_view(peer).unwrap();
    session.execute(Command::History(Replay::Undo));
    assert_eq!(text(session.buffer().unwrap()), "\tA\r\n\tB");
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::History(Replay::Redo));
    assert_eq!(text(session.buffer().unwrap()), "!\tA\r\n\tB");
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.buffer().unwrap().policy().tab_width, 8);
}

fn write_path(session: &mut Session, path: &Path) {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

    session.execute(Command::WriteFile);
    session.handle(Event::Paste(path.to_str().unwrap().into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}
