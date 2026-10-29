use super::*;
use rev::{command::History, session::RefileStatus};
// Archive tagging is reversible source metadata. Sibling movement reuses one
// checked same-buffer history group and preserves planning, drawers and children.
#[test]
fn archive_internal() {
    let source = "* Parent :work:\n** TODO Note :local:\nSCHEDULED: <2024-03-01 Fri>\n:PROPERTIES:\n:ID: keep\n:END:\n*** Child\n** Stay\n";
    let (fixture, mut session) = refile::editor(source, "* Target\n");
    session.execute(Command::GotoLine);
    argument(&mut session, "2");
    action(&mut session, C::ToggleArchiveTag);
    assert!(
        text(&session).contains(":local:ARCHIVE:"),
        "{}",
        text(&session)
    );
    assert_eq!(
        session.view().screen_row(session.buffer().unwrap(), 2),
        None
    );
    action(&mut session, C::ShowSubtree);
    assert!(
        session
            .view()
            .screen_row(session.buffer().unwrap(), 2)
            .is_some()
    );
    action(&mut session, C::Cycle);
    assert_eq!(
        session.view().screen_row(session.buffer().unwrap(), 2),
        None
    );
    action(&mut session, C::ShowAll);
    assert!(
        session
            .view()
            .screen_row(session.buffer().unwrap(), 2)
            .is_some()
    );
    action(&mut session, C::ToggleArchiveTag);
    assert_eq!(text(&session), source);
    action(&mut session, C::ArchiveSibling);
    assert!(
        session.echo().contains("Archive result"),
        "{}",
        session.echo()
    );
    assert_eq!(
        text(&session),
        "* Parent :work:\n** Archive :ARCHIVE:\n*** TODO Note :local:\nSCHEDULED: <2024-03-01 Fri>\n:PROPERTIES:\n:ID: keep\n:END:\n**** Child\n** Stay\n"
    );
    assert_eq!(session.refile_results().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        source
    );
}
// External archiving retains navigable origin and task metadata in the real
// destination buffer; a missing archive file is created only by ordinary Save.
#[test]
fn archive_external() {
    let source = "\u{feff}* Parent :work:\r\n** TODO Note :local:\r\nSCHEDULED: <2024-03-01 Fri>\r\n:PROPERTIES:\r\n:ID: keep\r\n:END:\r\n*** Child\r\n** Stay\r\n";
    let (fixture, session) = refile::editor(source, "* Target\n");
    let mut session = session.with_org_clock(clock);
    session.execute(Command::GotoLine);
    argument(&mut session, "2");
    action(&mut session, C::ArchiveSubtree);
    assert!(
        session.echo().contains("Archive result"),
        "{}",
        session.echo()
    );
    assert_eq!(text(&session), "* Parent :work:\r\n** Stay\r\n");
    let path = fixture.0.join("source.org_archive");
    assert!(!path.exists());
    assert_eq!(
        session
            .refile_results()
            .iter()
            .map(|r| r.status)
            .collect::<Vec<_>>(),
        [RefileStatus::ChangedInBuffer; 2]
    );
    session.load(&path).unwrap();
    assert!(session.buffer().unwrap().is_org());
    let archived = text(&session);
    assert!(
        archived.starts_with("* TODO Note :local:\nSCHEDULED:"),
        "{archived}"
    );
    for expected in [
        ":ID: keep",
        ":ARCHIVE_TIME: [2024-03-01 Fri 14:30]",
        ":ARCHIVE_OLPATH: Parent",
        ":ARCHIVE_TODO: TODO",
        "** Child",
        ":ARCHIVE_CATEGORY: source",
    ] {
        assert!(
            archived.contains(expected),
            "missing {expected}: {archived}"
        );
    }
    assert!(archived.contains(&format!(
        ":ARCHIVE_FILE: {}",
        fixture.0.join("source.org").display()
    )));
    assert_eq!(archived.matches(":PROPERTIES:").count(), 1);
    session.execute(Command::SaveBuffer);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), archived);
    assert_eq!(session.refile_results()[1].status, RefileStatus::Saved);
    assert_eq!(
        session.refile_results()[0].status,
        RefileStatus::ChangedInBuffer
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        source
    );
}
// A source-local archive override uses exact checked heading resolution. An
// unavailable directory or ambiguous destination rejects before source removal.
#[test]
fn archive_destinations() {
    for (name, directive, destination, success) in [
        ("heading", "destination.org::* Done", "* Done\n", true),
        (
            "create heading",
            "destination.org::* Done",
            "* Other\n",
            true,
        ),
        (
            "duplicate",
            "destination.org::* Done",
            "* Done\n* Done\n",
            false,
        ),
        ("missing parent", "absent/archive.org::", "* Done\n", false),
        ("same source", "source.org::", "* Done\n", false),
    ] {
        let source = format!("#+ARCHIVE: {directive}\n* Note\n* Stay\n");
        let (fixture, mut session) = refile::editor(&source, destination);
        session.execute(Command::GotoLine);
        argument(&mut session, "2");
        action(&mut session, C::ArchiveSubtree);
        if success {
            assert!(
                session.echo().contains("Archive result"),
                "{name}: {}",
                session.echo()
            );
            session.load(&fixture.0.join("destination.org")).unwrap();
            assert!(
                text(&session).contains("** Note"),
                "{name}: {}",
                text(&session)
            );
        } else {
            assert_eq!(text(&session), source, "{name}: {}", session.echo());
            assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        }
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
            source,
            "{name}"
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
            destination,
            "{name}"
        );
    }
}

// Existing sibling identity and nearest inherited location overrides use
// native structure, not an arbitrary matching display title or source line.
#[test]
fn archive_context_targets() {
    for (name, source, line, expected, accepted) in [
        (
            "existing sibling",
            "* Archive :ARCHIVE:\n** Old\n* Note\n* Stay\n",
            "3",
            "* Archive :ARCHIVE:\n** Old\n** Note\n* Stay\n",
            true,
        ),
        (
            "later sibling",
            "* Note\n* Stay\n* Archive :ARCHIVE:\n** Old\n",
            "1",
            "* Stay\n* Archive :ARCHIVE:\n** Old\n** Note\n",
            true,
        ),
        (
            "duplicate siblings",
            "* Note\n* Archive :ARCHIVE:\n* Archive :ARCHIVE:\n",
            "1",
            "",
            false,
        ),
        (
            "self sibling",
            "* Archive :ARCHIVE:\n** Old\n",
            "1",
            "",
            false,
        ),
    ] {
        let (_fixture, mut session) = refile::editor(source, "* Target\n");
        session.execute(Command::GotoLine);
        argument(&mut session, line);
        action(&mut session, C::ArchiveSibling);
        assert_eq!(
            text(&session),
            if accepted { expected } else { source },
            "{name}: {}",
            session.echo()
        );
        if accepted {
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source, "{name}");
        } else {
            assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        }
    }
    let source = "#+ARCHIVE: absent/file.org::\n* Parent\n:PROPERTIES:\n:ARCHIVE: destination.org::* Done\n:END:\n** Note\n:PROPERTIES:\n:ARCHIVE_TIME: old\n:ID: stable\n:END:\nbody\n** Stay\n";
    let (fixture, mut session) = refile::editor(source, "\u{feff}* Done\r\nbody EOF");
    session.execute(Command::GotoLine);
    argument(&mut session, "6");
    action(&mut session, C::ArchiveSubtree);
    assert!(
        session.echo().contains("Archive result"),
        "{}",
        session.echo()
    );
    session.load(&fixture.0.join("destination.org")).unwrap();
    assert!(session.buffer().unwrap().has_bom());
    let archived = text(&session);
    assert!(
        archived.starts_with("* Done\r\nbody EOF\r\n** Note\r\n"),
        "{archived}"
    );
    assert_eq!(archived.matches(":ARCHIVE_TIME:").count(), 1);
    assert!(archived.contains(":ID: stable\r\n"));
    assert!(archived.contains(":ARCHIVE_OLPATH: Parent\r\n"));
    assert!(!fixture.0.join("absent").exists());
}
// Rejected read-only files and actual external replacements retain source,
// metadata and history. Archive cancellation reaps the one existing worker.
#[test]
fn archive_rejection() {
    use std::os::unix::fs::PermissionsExt;
    for (name, readonly) in [("read-only archive", true), ("read-only source", false)] {
        let source = "#+ARCHIVE: destination.org::\n* Note\n* Stay\n";
        let (fixture, mut session) = refile::editor(source, "* Old\n");
        let path = fixture.0.join(if readonly {
            "destination.org"
        } else {
            "source.org"
        });
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
        session.execute(Command::GotoLine);
        argument(&mut session, "2");
        action(&mut session, C::ArchiveSubtree);
        assert_eq!(text(&session), source, "{name}: {}", session.echo());
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert!(session.refile_results().is_empty(), "{name}");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
            "* Old\n"
        );
    }
    let (_fixture, mut session) = refile::editor("* Note\n* Stay\n", "* Old\n");
    session.execute(Command::Org(C::ArchiveSubtree));
    cancel(&mut session);
    settle(&mut session);
    assert_eq!(text(&session), "* Note\n* Stay\n");
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert!(session.buffer().unwrap().history().is_empty());
}
