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
