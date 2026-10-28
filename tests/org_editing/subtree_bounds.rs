use rev::command::{History, Motion, OrgCommand, RectangleCommand, RegionCommand};

use super::headings::goto;
use super::visibility::{OUTLINE, action};
use super::*;

const OPERATIONS: [OrgCommand; 4] = [
    OrgCommand::PromoteSubtree,
    OrgCommand::DemoteSubtree,
    OrgCommand::MoveSubtreeUp,
    OrgCommand::MoveSubtreeDown,
];

fn rejects(session: &mut Session, command: OrgCommand, message: &str) {
    let before = (
        text(session),
        session.view().clone(),
        session.buffer().unwrap().mark(),
        session.buffer().unwrap().stamp(),
    );
    let count = session.buffer().unwrap().history().len();
    action(session, command);
    assert!(
        session.echo().contains(message),
        "{command:?}: {}",
        session.echo()
    );
    assert_eq!(
        (
            text(session),
            session.view().clone(),
            session.buffer().unwrap().mark(),
            session.buffer().unwrap().stamp()
        ),
        before
    );
    assert_eq!(session.buffer().unwrap().history().len(), count);
}

// Invalid hierarchy, missing siblings and deeper forms cannot authorize an edit.
// Malformed source elsewhere remains intact when unrelated valid siblings move.
#[test]
fn subtree_rejections() {
    for (name, source, line, command, message) in [
        (
            "root promotion",
            "* Root\nbody\n",
            1,
            OrgCommand::PromoteSubtree,
            "top-level",
        ),
        (
            "first demotion",
            "* Root\nbody\n",
            1,
            OrgCommand::DemoteSubtree,
            "previous",
        ),
        (
            "first move",
            "* Root\n* Peer\n",
            1,
            OrgCommand::MoveSubtreeUp,
            "sibling",
        ),
        (
            "last move",
            "* Root\n* Peer\n",
            2,
            OrgCommand::MoveSubtreeDown,
            "sibling",
        ),
        (
            "missing parent",
            "* Root\n*** Gap\nbody\n",
            2,
            OrgCommand::PromoteSubtree,
            "missing parent",
        ),
        (
            "bad parent",
            "* Root\n#+begin_src text\n** Child\nbody\n",
            3,
            OrgCommand::PromoteSubtree,
            "unclosed",
        ),
        (
            "bad peer",
            "* Root\n:LOGBOOK:\nentry\n* Peer\n",
            4,
            OrgCommand::MoveSubtreeUp,
            "unclosed",
        ),
        (
            "table context",
            "* Prev\n* Root\n| a | b |\n* Next\n",
            3,
            OrgCommand::DemoteSubtree,
            "unavailable",
        ),
        (
            "list context",
            "* Prev\n* Root\n- item\n* Next\n",
            3,
            OrgCommand::MoveSubtreeDown,
            "unavailable",
        ),
        (
            "literal context",
            "* Prev\n* Root\n#+begin_example\nliteral\n#+end_example\n* Next\n",
            4,
            OrgCommand::MoveSubtreeUp,
            "unavailable",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        rejects(&mut session, command, message);
        assert_eq!(text(&session), source, "{name}");
    }
    let source = "* Bad\n#+begin_src text\n* Good\n#+FUTURE: keep\nbody\n* Last\nlast\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 3);
    action(&mut session, OrgCommand::MoveSubtreeDown);
    assert_eq!(
        text(&session),
        "* Bad\n#+begin_src text\n* Last\nlast\n* Good\n#+FUTURE: keep\nbody\n"
    );
}

// Permissions, inactive mode and incomplete/depth-limited observations preserve
// all canonical state. No partial operation escapes any admission failure.
#[test]
fn subtree_admission() {
    let (mut buffers, id) = super::folds::buffer(OUTLINE);
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
    for command in OPERATIONS {
        rejects(&mut session, command, "read-only");
    }
    let (_fixture, mut session) = editor("", "notes.txt", OUTLINE);
    for command in OPERATIONS {
        rejects(&mut session, command, "not active");
    }
    let source = "* Entry\nbody\n".repeat(4097);
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", &source);
    for command in OPERATIONS {
        rejects(&mut session, command, "incomplete");
    }
    let mut source = (1..=254)
        .map(|level| format!("{} Ancestor\n", "*".repeat(level)))
        .collect::<String>();
    source.push_str(&format!(
        "{} Previous\n{} Target\n{} Child\n",
        "*".repeat(255),
        "*".repeat(255),
        "*".repeat(256)
    ));
    let (_fixture, mut session) = editor("", "notes.org", &source);
    goto(&mut session, 256);
    rejects(&mut session, OrgCommand::DemoteSubtree, "depth limit");
}

fn select(session: &mut Session, first: usize, end: usize, reverse: bool) {
    goto(session, first);
    session.execute(Command::Region(RegionCommand::Set));
    goto(session, end);
    if reverse {
        session.execute(Command::Region(RegionCommand::Exchange));
    }
}

// A linear region must cover complete adjacent sibling subtrees. Both endpoint
// directions change the selected group once, preserving descendants and history.
#[test]
fn subtree_regions() {
    let source = "* Outer\n** A\n*** Child\nalpha\n** B\nbeta\n** C\ngamma\n* Last\n";
    for reverse in [false, true] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        select(&mut session, 2, 7, reverse);
        action(&mut session, OrgCommand::PromoteSubtree);
        let promoted = "* Outer\n* A\n** Child\nalpha\n* B\nbeta\n** C\ngamma\n* Last\n";
        assert_eq!(text(&session), promoted);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        select(&mut session, 2, 7, reverse);
        action(&mut session, OrgCommand::MoveSubtreeDown);
        let moved = "* Outer\n** C\ngamma\n** A\n*** Child\nalpha\n** B\nbeta\n* Last\n";
        assert_eq!(text(&session), moved, "{}", session.echo());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), moved);
    }
    let (_fixture, mut session) = editor("", "notes.org", source);
    select(&mut session, 2, 4, false);
    rejects(&mut session, OrgCommand::PromoteSubtree, "complete");
    session.execute(Command::KeyboardQuit);
    goto(&mut session, 2);
    session.execute(Command::Motion(Motion::ForwardChar));
    session.execute(Command::Region(RegionCommand::Set));
    goto(&mut session, 7);
    rejects(&mut session, OrgCommand::MoveSubtreeDown, "start");
    session.execute(Command::KeyboardQuit);
    goto(&mut session, 2);
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    rejects(&mut session, OrgCommand::PromoteSubtree, "linear");
}

// Every reachable grapheme boundary, including the EOF/separator pair, survives
// forward, inverse and repeated history mapping in an independent shared view.
#[test]
fn subtree_boundary_roundtrip() {
    let source = "* 界e\u{301}\r\nbody\r\n\r\n* Last\r\nend";
    for steps in 0..=source.chars().count() {
        let (_fixture, mut session) = editor("", "notes.org", source);
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Window(WindowCommand::Other));
        let peer = session.windows().selected_id();
        for _ in 0..steps {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        let before = session.view().point.position;
        session.execute(Command::Window(WindowCommand::Other));
        action(&mut session, OrgCommand::MoveSubtreeDown);
        let after = session.windows().get(peer).unwrap().point.position;
        for _ in 0..2 {
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source);
            assert_eq!(
                session.windows().get(peer).unwrap().point.position,
                before,
                "step {steps}"
            );
            session.execute(Command::History(History::Redo));
            assert_eq!(
                session.windows().get(peer).unwrap().point.position,
                after,
                "step {steps}"
            );
        }
    }
}
