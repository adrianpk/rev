use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};
use rev::input::{Input, OrgContext, Outcome};

// The agenda role admits the same working source actions as editing, consumes
// unrelated text and preserves prompt/cancellation precedence over those keys.
#[test]
fn estimate_contexts() {
    let mut input = Input::default();
    input.set_org_context(Some(OrgContext::Agenda));
    for (letter, command) in [('e', OrgCommand::SetEffort), (',', OrgCommand::Priority)] {
        assert_eq!(
            input.handle(key(KeyCode::Char(letter))),
            Outcome::Command(Command::Org(command))
        );
    }
    input.argument("Estimate: ", "").unwrap();
    input.handle(key(KeyCode::Char('e')));
    assert_eq!(input.prompt().unwrap().text(), "e");
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    input.set_org_context(Some(OrgContext::Calendar));
    assert_ne!(
        input.handle(key(KeyCode::Char('e'))),
        Outcome::Command(Command::Org(OrgCommand::SetEffort))
    );
}

// Derived matching reads the same explicit/default priority and preserves the
// actual title, while a typed custom workflow retains its own task keyword.
#[test]
fn estimate_matching() {
    let source = "* OPEN Task :work:\n";
    let config = "org.todo.sequence=['OPEN','CLOSED']\norg.todo.finished=['CLOSED']\norg.priority.default='C'";
    let (_fixture, mut session) = editor(config, "notes.org", source);
    for query in ["PRIORITY=\"C\"+ITEM=\"Task\"", "TODO=\"OPEN\"+work"] {
        action(&mut session, OrgCommand::MatchTags);
        argument(&mut session, query);
        assert_eq!(session.echo(), "Org match: yes");
    }
    action(&mut session, OrgCommand::Priority);
    argument(&mut session, "A");
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "1h");
    let expected = "* OPEN [#A] Task :work:\n:PROPERTIES:\n:EFFORT: 1h\n:END:\n";
    assert_eq!(text(&session), expected);
    action(&mut session, OrgCommand::NextTodo);
    assert_eq!(text(&session), expected.replace("OPEN", "CLOSED"));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), expected);
    action(&mut session, OrgCommand::MatchTags);
    argument(
        &mut session,
        "PRIORITY=\"A\"+ITEM=\"Task\"+EFFORT=\"1h\"+TODO=\"OPEN\"",
    );
    assert_eq!(session.echo(), "Org match: yes");
}

// Selective inheritance remains explicit. Setting a child estimate never edits
// the inherited owner, and its spelling survives a later task/log transaction.
#[test]
fn effort_ownership() {
    let source = "* Parent\n:PROPERTIES:\n:EFFORT: 2h\n:END:\n** TODO Child\n";
    let (_fixture, mut session) = editor("org.properties.inherit=['EFFORT']", "notes.org", source);
    goto(&mut session, 5);
    action(&mut session, OrgCommand::InspectEffort);
    assert_eq!(session.echo(), "Org effort: 2h = 120 minutes");
    action(&mut session, OrgCommand::SetEffort);
    argument(&mut session, "0:30");
    assert_eq!(
        text(&session),
        format!("{source}:PROPERTIES:\n:EFFORT: 0:30\n:END:\n")
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}
