use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// The accepted reference duration grammar excludes signed, non-finite, scientific
// and fractional clock components. Rejection never materializes a local drawer.
#[test]
fn duration_rejection() {
    for draft in [
        "-1", "NaN", "inf", "1e3", "1.5:30", "1:2.5", "1:02:3.5", "1:2:3:4", "1H", "one hour",
    ] {
        let source = "* TODO Task\n";
        let (_fixture, mut session) = editor("", "notes.org", source);
        let before = session.view().clone();
        action(&mut session, OrgCommand::SetEffort);
        argument(&mut session, draft);
        assert_eq!(text(&session), source, "{draft}: {}", session.echo());
        assert_eq!(session.view(), &before);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Exact parser bounds and finite arithmetic are enforced independently of prompt
// limits. Work-unit overrides do not change canonical colon-clock arithmetic.
#[test]
fn duration_bounds() {
    let mut units = rev::config::DurationUnits::default();
    for (text, valid) in [
        ("1h".repeat(32), true),
        ("1h".repeat(33), false),
        ("1".repeat(128), true),
        ("1".repeat(129), false),
    ] {
        assert_eq!(rev::org::Duration::parse(&text, &units).is_ok(), valid);
    }
    assert_eq!(
        rev::org::Duration::parse("1 h 2min", &units)
            .unwrap()
            .minutes(),
        62.0
    );
    units.h = 30.0;
    assert_eq!(
        rev::org::Duration::parse("1:30", &units).unwrap().minutes(),
        90.0
    );
    units.y = 1e308;
    assert!(rev::org::Duration::parse("2y", &units).is_err());
}

// Unsupported priority input and deep/literal contexts cannot authorize an
// ancestor rewrite. Existing title/metadata bytes, views and history survive.
#[test]
fn priority_rejection() {
    for draft in ["1", "AA", "a", "Z", "NaN", "界"] {
        let source = "* TODO [#B] Task\n";
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, OrgCommand::Priority);
        argument(&mut session, draft);
        assert_eq!(text(&session), source, "{draft}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    for (source, row) in [
        ("* TODO H\n#+begin_example\nbody\n#+end_example\n", 3),
        ("* TODO H\n- item\n", 2),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, row);
        for command in [OrgCommand::PriorityUp, OrgCommand::SetEffort] {
            action(&mut session, command);
            assert_eq!(text(&session), source);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}

// Cancelled or stale estimate prompts never publish a partial priority/property
// edit. A draft cannot follow another selected pane or buffer revision.
#[test]
fn estimate_drafts() {
    for command in [OrgCommand::Priority, OrgCommand::SetEffort] {
        let source = "* TODO H\n";
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, command);
        session.execute(Command::KeyboardQuit);
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
        action(&mut session, command);
        session.replace(0..0, "prefix\n").unwrap();
        argument(&mut session, "1h");
        assert!(session.echo().contains("source or view changed"));
        assert_eq!(text(&session), format!("prefix\n{source}"));
    }
}

// Read-only source exposes actual priority/effort values while every edit refuses
// before opening an argument prompt; no mark, text or history changes occur.
#[test]
fn readonly_estimates() {
    let source = "* TODO [#A] H\n:PROPERTIES:\n:EFFORT: 1:30\n:END:\n";
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
    for command in [
        OrgCommand::Priority,
        OrgCommand::PriorityUp,
        OrgCommand::SetEffort,
        OrgCommand::IncEffort,
    ] {
        action(&mut session, command);
        assert_eq!(session.echo(), "buffer is read-only");
    }
    action(&mut session, OrgCommand::InspectEffort);
    assert_eq!(session.echo(), "Org effort: 1:30 = 90 minutes");
    action(&mut session, OrgCommand::InspectPriority);
    assert_eq!(session.echo(), "Org priority: A (explicit)");
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Narrow header/property edits preserve unknown leading words, EOF separators
// and peer anchors; one history step restores each whole admitted operation.
#[test]
fn estimate_source_edges() {
    for (source, expected) in [
        ("* TODO", "* TODO [#A]"),
        ("* :tag:", "* [#A] :tag:"),
        ("* TODO :tag:", "* TODO [#A] :tag:"),
        ("* UNKNOWN [#B] title", "* [#A] UNKNOWN [#B] title"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, OrgCommand::Priority);
        argument(&mut session, "A");
        assert_eq!(text(&session), expected, "{source}: {}", session.echo());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
    let source = "* TODO 界";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "1 h 2min");
    assert_eq!(
        text(&session),
        "* TODO 界\n:PROPERTIES:\n:EFFORT: 1 h 2min\n:END:"
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Replacing or removing an existing estimate changes only that value/record;
// source key case, indentation, unknown properties and priorities stay intact.
#[test]
fn effort_replacement() {
    let source = "* TODO [#B] Task\r\n:PROPERTIES:\r\n  :effort:   1h  \r\n:KEEP: x\r\n:END:\r\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "1:30");
    assert_eq!(text(&session), source.replace("1h", "1:30"));
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "");
    assert_eq!(text(&session), source.replace("  :effort:   1h  \r\n", ""));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source.replace("1h", "1:30"));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Peer source anchors map through both estimate owners; undo restores their
// positions independently while retaining the other pane's selected source.
#[test]
fn estimate_shared_views() {
    let source = "* TODO Task\nbody\n* Peer\nmarker 界";
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
    action(&mut session, OrgCommand::Priority);
    argument(&mut session, "A");
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "1h");
    let expected = source
        .replace("TODO ", "TODO [#A] ")
        .replace("\nbody", "\n:PROPERTIES:\n:EFFORT: 1h\n:END:\nbody");
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected.find("marker").unwrap()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source.replace("TODO ", "TODO [#A] "));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        source.find("marker").unwrap()
    );
}
