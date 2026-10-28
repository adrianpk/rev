use super::headings::goto;
use super::visibility::action;
use super::*;
use rev::command::{History, Motion, OrgCommand, RegionCommand};

fn rejected(session: &mut Session, command: OrgCommand, message: &str) {
    let source = text(session);
    let view = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    let mark = session.buffer().unwrap().mark();
    let history = session.buffer().unwrap().history().len();
    action(session, command);
    assert_eq!(text(session), source, "{}", session.echo());
    assert_eq!(session.view(), &view);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().mark(), mark);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert!(session.echo().contains(message), "{}", session.echo());
}

// Precise source admission cannot reinterpret a literal/table as an item
// or mutate a containing heading. Failed boundaries retain all canonical state.
#[test]
fn list_rejections() {
    for (name, source, line, command, message) in [
        (
            "first",
            "- first\n- last\n",
            1,
            OrgCommand::MetaUp,
            "sibling",
        ),
        (
            "last",
            "- first\n- last\n",
            2,
            OrgCommand::MetaDown,
            "sibling",
        ),
        (
            "indent first",
            "- first\n",
            1,
            OrgCommand::MetaRight,
            "previous",
        ),
        (
            "outermost",
            "- first\n",
            1,
            OrgCommand::OutdentItemTree,
            "outermost",
        ),
        (
            "literal",
            "* Root\n#+begin_example\n- [ ] literal\n#+end_example\n",
            3,
            OrgCommand::ToggleCheckbox,
            "No Org checkbox",
        ),
        (
            "table",
            "* Root\n| - [ ] cell |\n",
            2,
            OrgCommand::InsertItem,
            "No editable",
        ),
        (
            "unclosed",
            "* Root [0/1]\n- [ ] item\n  #+begin_src text\n  - [ ] fake\n",
            2,
            OrgCommand::ToggleCheckbox,
            "unclosed",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        rejected(&mut session, command, message);
        assert_eq!(text(&session), source, "{name}");
    }
}

// Native timer continuation retains the original source term, starts a new
// session timer when needed, and publishes its new description in one step.
#[test]
fn timed_description() {
    let source = "- 0:01:02 :: note\n  continuation\n- ordinary :: next\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.execute(Command::Motion(Motion::LineEnd));
    action(&mut session, OrgCommand::MetaReturn);
    let expected = "- 0:01:02 :: note\n- 0:00:00 :: \n  continuation\n- ordinary :: next\n";
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Both linear region directions use the first selected checkbox's new state,
// update all selected entries coherently and retain one history operation.
#[test]
fn checkbox_regions() {
    let source = "* Root [1/3]\n- [ ] first\n- [X] second\n- [-] third\n* Peer\n";
    for reverse in [false, true] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, 2);
        session.execute(Command::Region(RegionCommand::Set));
        goto(&mut session, 5);
        if reverse {
            session.execute(Command::Region(RegionCommand::Exchange));
        }
        action(&mut session, OrgCommand::ToggleCheckbox);
        let expected = "* Root [3/3]\n- [X] first\n- [X] second\n- [X] third\n* Peer\n";
        assert_eq!(text(&session), expected, "{}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected);
    }
}

// Ordinary drawers contain parsed structure. Existing checkbox source in a
// description item also remains actionable, matching native Org dispatch.
#[test]
fn parsed_checkbox_contexts() {
    for (source, line) in [
        ("* Root\n:LOGBOOK:\n- [ ] inside\n:END:\n", 3),
        ("- [ ] term :: value\n", 1),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        action(&mut session, OrgCommand::CtrlCCtrlC);
        assert_eq!(text(&session), source.replace("[ ]", "[X]"));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
    let source = "* Root [1/1]\n- [X] term :: value\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    session.execute(Command::Motion(Motion::LineEnd));
    action(&mut session, OrgCommand::MetaReturn);
    assert_eq!(
        text(&session),
        "* Root [1/1]\n- [X] term :: value\n-  :: \n"
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Read-only source and incomplete cookie observations reject before changing
// text, points, mark or history. Literal content is never promoted to a target.
#[test]
fn list_admission() {
    let (mut buffers, id) = super::folds::buffer("- [ ] first\n- [ ] last\n");
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
        OrgCommand::MetaReturn,
        OrgCommand::MetaDown,
        OrgCommand::ToggleCheckbox,
        OrgCommand::ToggleRadioButton,
        OrgCommand::InsertCheckboxItem,
    ] {
        rejected(&mut session, command, "read-only");
    }
    let source = format!("* Root {}\n- [ ] item\n", "[/] ".repeat(4097));
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", &source);
    goto(&mut session, 2);
    rejected(&mut session, OrgCommand::ToggleCheckbox, "incomplete");
}

// Decimal resets retain their exact source declaration without a machine-word
// overflow. Generated counters remain subject to the native source byte budget.
#[test]
fn decimal_counters() {
    let source = "1. [@999999999999999999999999] first\n2. second\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::RenumberList);
    let expected = "999999999999999999999999. [@999999999999999999999999] first\n1000000000000000000000000. second\n";
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Peer points and stored marks outside a reordered list map through its total
// counter-width change. Replay uses each peer's current source position.
#[test]
fn list_peer_anchors() {
    let source = "* Root\n20. [@20] first\n21. second\n* Peer\noutside";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 5);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::KeyboardQuit);
    let mark = session.buffer().unwrap().mark().unwrap();
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Window(WindowCommand::Other));
    let peer = session.windows().selected_id();
    session.execute(Command::Window(WindowCommand::Other));
    goto(&mut session, 2);
    action(&mut session, OrgCommand::MetaDown);
    let expected = "* Root\n1. second\n20. [@20] first\n* Peer\noutside";
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        mark.0 - 1
    );
    assert_eq!(session.buffer().unwrap().mark().unwrap().0, mark.0 - 1);
    session.execute(Command::Window(WindowCommand::Other));
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        source.chars().count()
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(mark));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        expected.chars().count()
    );
}

// First outermost tree commands shift the whole list by one column. Promoting
// a nested item adopts following siblings and updates the former parent cookie.
#[test]
fn list_indent_rules() {
    for (source, line, command, expected) in [
        (
            "- first\n- second\n  - child\n",
            1,
            OrgCommand::IndentItemTree,
            " - first\n - second\n   - child\n",
        ),
        (
            "  - first\n  - second\n",
            1,
            OrgCommand::OutdentItemTree,
            " - first\n - second\n",
        ),
        (
            "* R [1/1]\n- [X] p [2/2]\n  - [X] a\n  - [X] b\n",
            3,
            OrgCommand::MetaLeft,
            "* R [2/2]\n- [X] p [0/0]\n- [X] a\n  - [X] b\n",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        action(&mut session, command);
        assert_eq!(text(&session), expected, "{}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// Region starts inside a bullet prefix select by the checkbox's actual source
// anchor. Non-checkbox lines cannot determine the state of selected checkboxes.
#[test]
fn region_checkbox_anchor() {
    for (source, line, steps, expected) in [
        (
            "* Root [1/2]\n- plain\n- [ ] first\n- [X] second\n* Peer\n",
            3,
            2,
            "* Root [2/2]\n- plain\n- [X] first\n- [X] second\n* Peer\n",
        ),
        (
            "* Root [1/2]\n- plain\n- [ ] first\n- [X] second\n* Peer\n",
            3,
            3,
            "* Root [0/2]\n- plain\n- [ ] first\n- [ ] second\n* Peer\n",
        ),
        (
            "* Root [1/2]\n- plain\n- [X] first\n- [ ] second\n* Peer\n",
            2,
            0,
            "* Root [0/2]\n- plain\n- [ ] first\n- [ ] second\n* Peer\n",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        for _ in 0..steps {
            session.execute(Command::Motion(Motion::ForwardChar));
        }
        session.execute(Command::Region(RegionCommand::Set));
        goto(&mut session, 5);
        action(&mut session, OrgCommand::ToggleCheckbox);
        assert_eq!(text(&session), expected, "{}", session.echo());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}
