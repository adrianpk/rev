use super::headings::goto;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

// Child toggles and all parent/header statistics form one checked transaction.
// Partial, complete and unchecked parents remain observably distinct states.
#[test]
fn checkbox_hierarchy() {
    let source = "* TODO Root [/]\n- [ ] parent [%]\n  - [ ] alpha\n  - [X] beta\n- [ ] sibling\n* Other [9/9]\n- [ ] unrelated\n";
    for (name, line, expected) in [
        (
            "complete",
            3,
            "* TODO Root [1/2]\n- [X] parent [100%]\n  - [X] alpha\n  - [X] beta\n- [ ] sibling\n* Other [9/9]\n- [ ] unrelated\n",
        ),
        (
            "partial",
            2,
            "* TODO Root [0/2]\n- [-] parent [50%]\n  - [ ] alpha\n  - [X] beta\n- [ ] sibling\n* Other [9/9]\n- [ ] unrelated\n",
        ),
        (
            "unchecked",
            4,
            "* TODO Root [0/2]\n- [ ] parent [0%]\n  - [ ] alpha\n  - [ ] beta\n- [ ] sibling\n* Other [9/9]\n- [ ] unrelated\n",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        goto(&mut session, line);
        action(&mut session, OrgCommand::ToggleCheckbox);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected, "{name}");
    }
}

// Explicit radio and the affiliated data-only radio attribute clear siblings at
// the current level, without affecting nested or unrelated source entries.
#[test]
fn checkbox_radio() {
    for (attribute, command) in [
        ("", OrgCommand::ToggleRadioButton),
        ("#+ATTR_ORG: :radio t\n", OrgCommand::CtrlCCtrlC),
    ] {
        let source = format!("* Root [1/3]\n{attribute}- [X] one\n- [ ] two\n- [-] three\n");
        let (_fixture, mut session) = editor("", "notes.org", &source);
        goto(&mut session, if attribute.is_empty() { 3 } else { 4 });
        action(&mut session, command);
        assert_eq!(
            text(&session),
            format!("* Root [1/3]\n{attribute}- [ ] one\n- [X] two\n- [ ] three\n"),
            "{}",
            session.echo()
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// A local ORDERED property blocks completion before any text/history change.
// Completing the predecessor first admits the next sibling normally.
#[test]
fn checkbox_ordered() {
    let source = "* Root [0/2]\n:PROPERTIES:\n:ORDERED: t\n:END:\n- [ ] first\n- [ ] second\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 6);
    let point = session.view().point;
    action(&mut session, OrgCommand::ToggleCheckbox);
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point, point);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(session.echo().contains("Earlier Org checkbox"));
    goto(&mut session, 5);
    action(&mut session, OrgCommand::ToggleCheckbox);
    goto(&mut session, 6);
    action(&mut session, OrgCommand::ToggleCheckbox);
    assert_eq!(
        text(&session),
        source.replace("[0/2]", "[2/2]").replace("[ ]", "[X]")
    );
}

// Presence is an explicit finite operation; descriptions reject the factory
// checkbox form. Insertion counts the new unchecked item in the same edit.
#[test]
fn checkbox_presence() {
    let source = "* Root [0/0]\n- plain\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 2);
    action(&mut session, OrgCommand::ToggleCheckboxPresence);
    assert_eq!(
        text(&session),
        "* Root [0/1]\n- [ ] plain\n",
        "{}",
        session.echo()
    );
    action(&mut session, OrgCommand::ToggleCheckboxPresence);
    assert_eq!(text(&session), source);
    session.execute(Command::Motion(rev::command::Motion::LineEnd));
    action(&mut session, OrgCommand::InsertCheckboxItem);
    let expected = "* Root [0/1]\n- plain\n- [ ] \n";
    assert_eq!(text(&session), expected, "{}", session.echo());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    let (_fixture, mut description) = editor("", "notes.org", "- term :: value\n");
    action(&mut description, OrgCommand::ToggleCheckboxPresence);
    assert_eq!(text(&description), "- term :: value\n");
    assert!(description.echo().contains("Description lists"));
}

// Heading cookies count direct task children when requested, retaining both
// fraction and percentage shapes and excluding grandchildren or ordinary text.
#[test]
fn task_statistics() {
    let source = "* Parent [/] [%]\n:PROPERTIES:\n:COOKIE_DATA: todo\n:END:\n- [ ] box\n** TODO first\n*** DONE grandchild\n** DONE second\n** NOTE ordinary\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::UpdateStatisticsCookies);
    assert_eq!(
        text(&session),
        source.replacen("[/] [%]", "[1/2] [50%]", 1),
        "{}",
        session.echo()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}

// Structural list edits update direct-child statistics before publication.
// One undo restores both the source hierarchy and its former progress spelling.
#[test]
fn structural_statistics() {
    let source = "* Root [2/2]\n- [X] first\n- [X] second\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 3);
    action(&mut session, OrgCommand::MetaRight);
    assert_eq!(
        text(&session),
        "* Root [1/1]\n- [X] first\n  - [X] second\n",
        "{}",
        session.echo()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::Motion(rev::command::Motion::LineEnd));
    action(&mut session, OrgCommand::MetaReturn);
    assert_eq!(
        text(&session),
        "* Root [2/3]\n- [X] first\n- [X] second\n- [ ] \n",
        "{}",
        session.echo()
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}
