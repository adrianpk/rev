use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// Explicit tag entry cannot escape literal/deeper or malformed source contexts.
// Invalid accepted names never change canonical text, point or history.
#[test]
fn tag_admission() {
    for (name, source, line) in [
        ("literal", "* H\n#+begin_example\ntext\n#+end_example\n", 3),
        ("drawer", "* H\n:LOGBOOK:\n# comment\n:END:\n", 3),
        ("list", "* H\n- item\n", 2),
        ("malformed", "* H :a::b:\n", 1),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        let before = session.view().clone();
        action(&mut session, OrgCommand::SetTags);
        assert_eq!(text(&session), source, "{name}: {}", session.echo());
        assert_eq!(session.view(), &before);
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(
            session.echo().contains("unavailable") || session.echo().contains("Malformed"),
            "{}",
            session.echo()
        );
    }
    for value in ["invalid-tag".into(), "x".repeat(129)] {
        let (_fixture, mut session) = editor("", "notes.org", "* H\n");
        action(&mut session, OrgCommand::SetTags);
        argument(&mut session, &value);
        assert_eq!(text(&session), "* H\n");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Malformed expressions reject before publication and do not change inherited
// owners, unrelated bytes or the qualified view; source-local Lisp remains inert.
#[test]
fn malformed_tag_queries() {
    let source = "#+BIND: org-use-tag-inheritance nil\n* TODO H :Work:\n:PROPERTIES:\n:OWNER-ROLE: lead\n:END:\n** Child\nunrelated\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    for query in [
        "Work|",
        "OWNER\\-ROLE=\"\\q\"",
        "(lambda () t)",
        "Work/(TODO",
    ] {
        let before = session.view().clone();
        action(&mut session, OrgCommand::MatchTags);
        argument(&mut session, query);
        assert!(
            session.echo().contains("Org match at byte"),
            "{query}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source);
        assert_eq!(session.view(), &before);
        assert!(session.buffer().unwrap().history().is_empty());
    }
    goto(&mut session, 6);
    action(&mut session, OrgCommand::MatchTags);
    argument(&mut session, "Work");
    assert_eq!(session.echo(), "Org match: yes");
}

// Read-only buffers admit actual inspection/matching, including derived absent
// tags and properties, while local mutation rejects before a prompt or edit.
#[test]
fn readonly_tags() {
    let source = "* TODO H :Work:\n** Child\n";
    let (mut buffers, id) = super::folds::buffer(source);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    for command in [OrgCommand::SetTags, OrgCommand::RemoveTag] {
        let before = session.view().clone();
        action(&mut session, command);
        assert!(session.echo().contains("read-only"));
        assert_eq!(session.view(), &before);
    }
    action(&mut session, OrgCommand::MatchTags);
    argument(&mut session, "Work/TODO");
    assert_eq!(session.echo(), "Org match: yes");
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MatchTags);
    argument(&mut session, "TAGS=\"\"+Work");
    assert_eq!(session.echo(), "Org match: yes");
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Drafts retain their original source/view qualifier across observation and
// completion. Revisions invalidate both mutation and query; C-g drops the draft.
#[test]
fn stale_tag_drafts() {
    for command in [OrgCommand::SetTags, OrgCommand::MatchTags] {
        let (_fixture, mut session) = editor("", "notes.org", "* H :tag:\n");
        action(&mut session, command);
        session.replace(0..0, "prefix\n").unwrap();
        argument(&mut session, "tag");
        assert!(session.echo().contains("source or view changed"));
        assert_eq!(text(&session), "prefix\n* H :tag:\n");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "* H :tag:\n");
        action(&mut session, command);
        session.execute(Command::KeyboardQuit);
        session.handle(Event::Paste("ordinary".into()));
        assert_eq!(text(&session), "ordinary* H :tag:\n");
    }
}

// Suffix edits map live peer points through canonical source; undo/redo restore
// their anchors and retain all other heading, drawer, Unicode and EOF bytes.
#[test]
fn shared_tag_views() {
    let source = "* Head :old:\nbody\n* Peer\nmarker 界";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 4);
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != session.windows().selected_id())
        .unwrap();
    goto(&mut session, 1);
    action(&mut session, OrgCommand::SetTags);
    argument(&mut session, "longer 界");
    let expected = source.replace(":old:", ":longer:界:");
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected[..expected.find("marker").unwrap()].chars().count()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        source.find("marker").unwrap()
    );
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Candidate admission accepts its exact observation limit. An extra declaration
// cannot authorize operations from an incomplete prefix of the document.
#[test]
fn tag_observation_limit() {
    for (count, complete) in [(8192, true), (8193, false)] {
        let words = (0..count)
            .map(|i| format!("tag{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let source = format!("#+TAGS: {words}\n* H\n");
        let (_fixture, mut session) = editor("", "notes.org", &source);
        goto(&mut session, 2);
        action(&mut session, OrgCommand::SetTags);
        if complete {
            argument(&mut session, "added");
            assert_eq!(text(&session), source.replace("* H", "* H  :added:"));
        } else {
            assert!(session.echo().contains("incomplete"));
            assert_eq!(text(&session), source);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}
