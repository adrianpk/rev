use super::headings::goto;
use super::properties::argument;
use super::tasks::{choose, clock};
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// Returning to an unfinished state removes only CLOSED metadata, including its
// otherwise empty line, so a property drawer remains structurally owned.
#[test]
fn task_reopening() {
    for (source, expected) in [
        (
            "* DONE H\nCLOSED: [2026-10-27 Tue 10:00]\n:PROPERTIES:\n:OWNER: keep\n:END:\n",
            "* TODO H\n:PROPERTIES:\n:OWNER: keep\n:END:\n",
        ),
        (
            "* DONE H\nSCHEDULED: <2026-10-30 Fri> CLOSED: [2026-10-27 Tue 10:00]\n",
            "* TODO H\nSCHEDULED: <2026-10-30 Fri> \n",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        choose(&mut session, "TODO");
        assert_eq!(text(&session), expected, "{}", session.echo());
        action(&mut session, OrgCommand::InspectTodo);
        assert!(session.echo().contains("Org task: TODO"));
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// Invalid source declarations, deeper literal/list contexts and malformed
// metadata reject before a prompt, preserving source, point and history.
#[test]
fn task_admission() {
    for (source, line) in [
        ("#+TODO: TODO | TODO\n* TODO H\n", 2),
        ("#+TODO: TODO(x) | DONE(x)\n* TODO H\n", 2),
        ("#+TODO: TODO | | DONE\n* TODO H\n", 2),
        ("#+TODO: TODO(broken) | DONE\n* TODO H\n", 2),
        ("#+TODO: \n* TODO H\n", 2),
        ("* TODO H\n#+begin_src sh\nTODO body\n#+end_src\n", 3),
        ("* TODO H\n- item\n", 2),
        ("* TODO H\n:PROPERTIES:\n:OWNER: keep\n", 1),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        let before = session.view().clone();
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(text(&session), source, "{}", session.echo());
        assert_eq!(session.view(), &before);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Source-local workflow caps admit exactly 64 states and reject the successor.
// Configured custom states also drive matching and checkbox-owned TODO cookies.
#[test]
fn task_workflow_bounds() {
    for (count, valid) in [(64, true), (65, false)] {
        let words = (0..count)
            .map(|i| format!("S{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let source = format!("#+TODO: {words}\n* S0 H\n");
        let (_fixture, mut session) = editor("", "notes.org", &source);
        goto(&mut session, 2);
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(
            text(&session),
            if valid {
                source.replace("* S0", "* S1")
            } else {
                source
            }
        );
    }
    let source = "* Parent [0/2]\n:PROPERTIES:\n:COOKIE_DATA: todo\n:END:\n- [ ] item\n** OPEN A\n** FINISHED B\n";
    let config = "org.todo.sequence=['OPEN','FINISHED']\norg.todo.finished=['FINISHED']";
    let (_fixture, mut session) = editor(config, "notes.org", source);
    goto(&mut session, 5);
    action(&mut session, OrgCommand::UpdateStatisticsCookies);
    assert_eq!(text(&session), source.replace("[0/2]", "[1/2]"));
    goto(&mut session, 6);
    action(&mut session, OrgCommand::MatchTags);
    argument(&mut session, "TODO=\"OPEN\"+ITEM=\"A\"");
    assert_eq!(session.echo(), "Org match: yes");
    choose(&mut session, "FINISHED");
    assert_eq!(
        text(&session),
        source
            .replace("[0/2]", "[2/2]")
            .replace("** OPEN A", "** FINISHED A")
    );
}

// EOF logging works after a multibyte title. All peer points follow independent
// source anchors through the keyword, metadata and parent-cookie transaction.
#[test]
fn task_shared_eof() {
    let source = "* Parent [0%]\n** TODO 界";
    let (_fixture, session) = editor("org.todo.log_done=true", "notes.org", source);
    let mut session = session.with_task_clock(clock);
    goto(&mut session, 2);
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
    let peer_before = session.windows().get(peer).unwrap().point.position;
    goto(&mut session, 2);
    choose(&mut session, "DONE");
    let result = text(&session);
    assert!(
        result.starts_with("* Parent [100%]\n** DONE 界\nCLOSED: [2026-10-28 Wed 17:05]\n"),
        "{result}: {}",
        session.echo()
    );
    assert_eq!(
        session.windows().get(peer).unwrap().point.position.0,
        peer_before.0 + 2
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        peer_before
    );
}

// Read-only sources still expose real task choices and current state; every
// transition rejects before selection/logging and cannot affect history.
#[test]
fn readonly_tasks() {
    let source = "* TODO H\n";
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
        OrgCommand::Todo,
        OrgCommand::SetTodo,
        OrgCommand::NextTodo,
        OrgCommand::PreviousTodo,
    ] {
        action(&mut session, command);
        assert_eq!(session.echo(), "buffer is read-only");
    }
    action(&mut session, OrgCommand::InspectTodo);
    assert!(session.echo().contains("Org task: TODO"));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}
