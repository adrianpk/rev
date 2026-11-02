use super::*;

// Kernel notifications drive one native edit for both shared panes under each
// startup profile, and undo remains dirty without an unchanged-file reload loop.
#[test]
fn shared_kernel_reload() {
    for profile in ["non-modal", "modal"] {
        for replacement in [false, true] {
            let fixture = Fixture::new();
            let path = fixture.file("source", b"prefix old tail");
            let mut editor = editor(&fixture, &path, profile, "reload");
            editor.execute(Command::Motion(Motion::ForwardChar));
            editor.execute(Command::Split(Split::Below));
            let focus = editor.windows().selected_id();
            if replacement {
                let new = fixture.file("replacement", b"prefix fresh tail");
                fs::rename(new, &path).unwrap();
            } else {
                fs::write(&path, "prefix fresh tail").unwrap();
            }
            wait(&mut editor, |s| {
                bytes(s.buffer().unwrap()) == b"prefix fresh tail"
            });
            assert_eq!(editor.windows().selected_id(), focus);
            assert_eq!(editor.buffer().unwrap().history().len(), 1);
            assert!(!editor.buffer().unwrap().is_modified());
            for (_, view) in editor.windows().iter() {
                assert_eq!(view.point.stamp, editor.buffer().unwrap().stamp());
                assert_eq!(view.point.position, Scalar(1));
            }
            editor.execute(Command::History(History::Undo));
            assert_eq!(bytes(editor.buffer().unwrap()), b"prefix old tail");
            assert!(editor.buffer().unwrap().is_modified());
            // A second external update stays observable after rename-over and
            // becomes a passive dirty conflict rather than erasing this undo.
            fs::write(&path, "next external version").unwrap();
            wait(&mut editor, |s| {
                s.buffer().unwrap().external_change_notice().is_some()
            });
            assert_eq!(bytes(editor.buffer().unwrap()), b"prefix old tail");
        }
    }
}

// Ask and dirty policies retain text, input focus and the overwrite guard. A
// later disk version re-notifies even after the earlier echo was dismissed.
#[test]
fn pending_versions() {
    for (policy, dirty) in [("ask", false), ("ask", true), ("reload", true)] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"old");
        let mut editor = editor(&fixture, &path, "non-modal", policy);
        if dirty {
            editor.replace(0..0, "local ").unwrap();
        }
        let prior = editor.buffer().unwrap().file().unwrap().clone();
        let expected = bytes(editor.buffer().unwrap());
        fs::write(&path, "first version").unwrap();
        wait(&mut editor, |s| {
            s.buffer().unwrap().external_change_notice().is_some()
        });
        assert_eq!(bytes(editor.buffer().unwrap()), expected);
        assert_eq!(editor.buffer().unwrap().file(), Some(&prior));
        assert!(editor.echo().contains("revert-buffer"));
        editor.execute(Command::KeyboardQuit);
        fs::write(&path, "second external version").unwrap();
        wait(&mut editor, |s| s.echo().contains("external change"));
        assert_eq!(bytes(editor.buffer().unwrap()), expected);
        let mut typed = String::from_utf8(bytes(editor.buffer().unwrap())).unwrap();
        typed.insert_str(editor.view().point.position.0, "typed ");
        editor.handle(Event::Paste("typed ".into()));
        assert_eq!(bytes(editor.buffer().unwrap()), typed.as_bytes());
        editor.execute(Command::SaveBuffer);
        assert!(
            editor.echo().contains("Destination changed"),
            "{}",
            editor.echo()
        );
        editor.execute(Command::KeyboardQuit);
        editor.execute(Command::RevertBuffer);
        wait(&mut editor, |s| s.echo().starts_with("Discard unsaved"));
        editor.handle(key(KeyCode::Char('y')));
        wait(&mut editor, |s| {
            bytes(s.buffer().unwrap()) == b"second external version"
        });
        assert!(editor.buffer().unwrap().external_change_notice().is_none());
    }
}

// Missing/invalid images preserve live text and history; a valid recreation at
// the same watched filename recovers without reopening or re-registering it.
#[test]
fn recreation_and_invalid_ingress() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"keep");
    let mut editor = editor(&fixture, &path, "non-modal", "reload");
    fs::remove_file(&path).unwrap();
    wait(&mut editor, |s| {
        s.buffer().unwrap().external_change_notice().is_some()
    });
    assert_eq!(bytes(editor.buffer().unwrap()), b"keep");
    assert!(editor.buffer().unwrap().history().is_empty());
    fs::write(&path, [0xff]).unwrap();
    wait(&mut editor, |s| {
        s.buffer()
            .unwrap()
            .external_change_notice()
            .is_some_and(|n| n.contains("UTF-8") || n.contains("utf-8"))
    });
    assert_eq!(bytes(editor.buffer().unwrap()), b"keep");
    fs::write(&path, "recreated").unwrap();
    wait(&mut editor, |s| bytes(s.buffer().unwrap()) == b"recreated");
    assert!(editor.buffer().unwrap().external_change_notice().is_none());
}

// Identical-byte rename-over refreshes disk evidence under both policies while
// retaining point, revision and history. Completed editor saves also stay single.
#[test]
fn identical_and_own_save() {
    for policy in ["ask", "reload"] {
        let fixture = Fixture::new();
        let path = fixture.file("source", b"same bytes");
        let mut editor = editor(&fixture, &path, "non-modal", policy);
        editor.execute(Command::Motion(Motion::ForwardChar));
        let stamp = editor.buffer().unwrap().stamp();
        let replacement = fixture.file("new", b"same bytes");
        fs::rename(replacement, &path).unwrap();
        let observed = file::observe(&path).unwrap();
        wait(&mut editor, |s| {
            s.buffer().unwrap().file().unwrap().observation() == observed.as_ref()
        });
        assert_eq!(editor.buffer().unwrap().stamp(), stamp);
        assert_eq!(editor.view().point.position, Scalar(1));
        assert!(editor.buffer().unwrap().history().is_empty());
        editor.replace(0..0, "local ").unwrap();
        editor.execute(Command::SaveBuffer);
        let saved = editor.buffer().unwrap().stamp();
        // An unrelated watched-file update provides an observable second worker
        // round, rather than a fixed sleep pretending that own-save events ran.
        let other = fixture.file("other", b"other");
        editor.load(&other).unwrap();
        fs::write(&other, "fresh other").unwrap();
        if policy == "ask" {
            wait(&mut editor, |s| {
                s.buffer().unwrap().external_change_notice().is_some()
            });
        } else {
            wait(&mut editor, |s| {
                bytes(s.buffer().unwrap()) == b"fresh other"
            });
        }
        editor.load(&path).unwrap();
        assert_eq!(editor.buffer().unwrap().stamp(), saved);
        assert_eq!(editor.buffer().unwrap().history().len(), 1);
        assert!(!editor.buffer().unwrap().is_modified());
    }
}

// Hidden sources are observed and read-only visiting views refresh through the
// native transaction without acquiring write permission or stealing focus.
#[test]
fn hidden_and_readonly() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"old");
    let other = fixture.file("other", b"selected");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap();
    editor.load(&other).unwrap();
    let focus = editor.windows().selected_id();
    fs::write(&path, "fresh readonly").unwrap();
    wait(&mut editor, |s| s.echo().starts_with("Reloaded source"));
    assert_eq!(bytes(editor.buffer().unwrap()), b"selected");
    assert_eq!(editor.windows().selected_id(), focus);
    editor.load(&path).unwrap();
    assert_eq!(bytes(editor.buffer().unwrap()), b"fresh readonly");
    assert!(editor.buffer().unwrap().is_readonly());
    assert!(!editor.buffer().unwrap().is_modified());
}

// The registration cap admits the first live file, reports unavailable coverage
// for later files and leaves the explicit native reload usable on those files.
#[test]
fn registration_limit() {
    let fixture = Fixture::new();
    let config = fixture.file("config.toml", b"files.external_change_max_buffers = 1\nfiles.external_change_debounce_ms = 0\ntree.visible = false");
    let first = fixture.file("first", b"first");
    let second = fixture.file("second", b"second");
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&first).unwrap();
    buffers.visit(&second).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap();
    assert!(editor.echo().contains("limit reached"));
    fs::write(&first, "new first").unwrap();
    wait(&mut editor, |s| bytes(s.buffer().unwrap()) == b"new first");
    editor.load(&second).unwrap();
    fs::write(&second, "new second").unwrap();
    editor.execute(Command::RevertBuffer);
    wait(&mut editor, |s| bytes(s.buffer().unwrap()) == b"new second");
    assert!(!editor.buffer().unwrap().is_modified());
}

// Native bookmarks and Org indexes follow the reload's sole revision. Old
// source-qualified spans cannot read the replaced image, and undo maps anchors.
#[test]
fn canonical_derived_state() {
    let fixture = Fixture::new();
    let path = fixture.file("source.org", b"* Root\nbody\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut scan = rev::org::Scan::new(buffer, rev::org::Limits::default()).unwrap();
    let cancel = rev::org::Cancellation::default();
    while scan.advance(&cancel) == rev::org::Coverage::Pending {}
    buffer.install_org_index(scan.into_index()).unwrap();
    let old = buffer.span(7..11).unwrap();
    let view = View::new(buffer);
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )
    .unwrap();
    editor.execute(Command::Motion(Motion::BufferEnd));
    editor.execute(Command::Bookmark(rev::command::BookmarkCommand::Set));
    editor.handle(Event::Paste("end".into()));
    editor.handle(key(KeyCode::Enter));
    assert!(editor.bookmarks().get("end").is_some());
    fs::write(&path, "* Root\nlonger external body\n").unwrap();
    wait(&mut editor, |s| {
        bytes(s.buffer().unwrap()) == b"* Root\nlonger external body\n"
    });
    let buffer = editor.buffer().unwrap();
    assert!(buffer.slice(&old).is_err());
    assert_eq!(buffer.org_index().unwrap().stamp(), buffer.stamp());
    let point = editor.bookmarks().get("end").unwrap().point();
    assert_eq!(point.stamp, buffer.stamp());
    assert_eq!(point.position, Scalar(buffer.len_chars()));
    editor.execute(Command::History(History::Undo));
    assert_eq!(
        editor.bookmarks().get("end").unwrap().point().position,
        Scalar(12)
    );
}

// A real registration failure is visible without hot retries. Explicit reload
// still works and its checked new association evidence re-admits observation.
#[test]
fn unavailable_registration() {
    let fixture = Fixture::new();
    let path = fixture.0.join("late/source");
    let mut editor = editor(&fixture, &path, "non-modal", "reload");
    wait(&mut editor, |s| {
        s.buffer()
            .unwrap()
            .external_change_notice()
            .is_some_and(|n| n.contains("observation unavailable"))
    });
    assert!(bytes(editor.buffer().unwrap()).is_empty());
    std::fs::create_dir(fixture.0.join("late")).unwrap();
    fs::write(&path, "created").unwrap();
    editor.execute(Command::RevertBuffer);
    wait(&mut editor, |s| bytes(s.buffer().unwrap()) == b"created");
    fs::write(&path, "observed again").unwrap();
    wait(&mut editor, |s| {
        bytes(s.buffer().unwrap()) == b"observed again"
    });
}

// Cancellation between the observed read and final publication leaves a passive
// pending condition, not an acknowledged save or a replacement worker series.
#[test]
fn canceled_automatic_reload() {
    let fixture = Fixture::new();
    let path = fixture.file("source", b"old");
    let mut editor = editor(&fixture, &path, "non-modal", "reload");
    fs::write(&path, "external").unwrap();
    wait(&mut editor, |s| s.echo() == "Rechecking reload source");
    editor.execute(Command::KeyboardQuit);
    assert_eq!(bytes(editor.buffer().unwrap()), b"old");
    assert!(
        editor
            .buffer()
            .unwrap()
            .external_change_notice()
            .is_some_and(|n| n.contains("canceled"))
    );
    wait(&mut editor, |s| s.remaining_input(Instant::now()).is_none());
    editor.execute(Command::RevertBuffer);
    wait(&mut editor, |s| bytes(s.buffer().unwrap()) == b"external");
    assert!(editor.buffer().unwrap().external_change_notice().is_none());
}
