use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::OrgCommand;

// Source beyond the bounded property observation cannot authorize mutation.
// Duplicate bases and malformed metadata are diagnosed before any transaction.
#[test]
fn property_bounds() {
    let excessive = format!("* H\n:PROPERTIES:\n{}:END:\n", ":X+: value\n".repeat(8193));
    for (name, source, expected) in [
        ("records", excessive.as_str(), "incomplete"),
        ("malformed", "* H\n:PROPERTIES:\n:X: value\n", "Malformed"),
        (
            "invalid",
            "* H\n:PROPERTIES:\nnot a property\n:END:\n",
            "Malformed",
        ),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        let before = session.view().clone();
        action(&mut session, OrgCommand::SetProperty);
        assert!(
            session.echo().contains(expected),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view(), &before, "{name}");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    let source = "* H\n:PROPERTIES:\n:X: first\n:X: second\n:END:\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    action(&mut session, OrgCommand::SetProperty);
    argument(&mut session, "X");
    argument(&mut session, "third");
    assert!(session.echo().contains("Duplicate local"));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Read-only source still supports provenance inspection and drawer presentation;
// every property mutation rejects without changing revision, point or history.
#[test]
fn readonly_properties() {
    let source = "* H\n:PROPERTIES:\n:X: kept\n:END:\n";
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
        OrgCommand::SetProperty,
        OrgCommand::DeleteProperty,
        OrgCommand::EditPropertyOwner,
    ] {
        let before = session.view().clone();
        action(&mut session, command);
        assert!(session.echo().contains("read-only"));
        assert_eq!(session.view(), &before);
        assert_eq!(text(&session), source);
    }
    goto(&mut session, 3);
    action(&mut session, OrgCommand::InspectProperty);
    argument(&mut session, "X");
    assert!(session.echo().contains("kept"));
    assert!(session.echo().contains("local X at line 3"));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Only admitted declarations affect data. Literal/nested keywords and executable
// looking unknown values remain inert; source directives cannot enable inheritance.
#[test]
fn inert_property_source() {
    let source = "#+PROPERTY: X file\n#+BIND: org-use-property-inheritance t\n* H\n#+begin_example\n#+PROPERTY: X fake\n#+end_example\n:LOGBOOK:\n#+PROPERTY: X nested\n:END:\nbody\n";
    for (name, config, value) in [
        ("factory", "", ""),
        ("selected", "org.properties.inherit = ['X']", "file"),
    ] {
        let (_fixture, mut session) = editor(config, "notes.org", source);
        goto(&mut session, 10);
        action(&mut session, OrgCommand::InspectProperty);
        argument(&mut session, "X");
        assert!(
            session.echo().contains(&format!("= {value:?}")),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// File-local drawer insertion precedes declarations even when later comments
// exist. Empty fields accept a real value without losing source separators.
#[test]
fn file_drawer_placement() {
    for (source, expected) in [
        (
            "# comment\n\n#+TITLE: title\n# later\n* H\n",
            "# comment\n:PROPERTIES:\n:X: value\n:END:\n\n#+TITLE: title\n# later\n* H\n",
        ),
        (":PROPERTIES:\n:X:\n:END:", ":PROPERTIES:\n:X: value\n:END:"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        action(&mut session, OrgCommand::SetProperty);
        argument(&mut session, "X");
        argument(&mut session, "value");
        assert_eq!(text(&session), expected, "{}", session.echo());
    }
}

// The familiar local binding, its compiled command and a validated alias enter
// identical qualified prompts, while M-x and C-g retain transient ownership.
#[test]
fn property_command_paths() {
    for path in ["key", "name", "alias"] {
        let (_fixture, mut session) = editor(
            "[keys.org]\n'M-j' = 'org-set-property'",
            "notes.org",
            "* H\n",
        );
        match path {
            "key" => {
                session.handle(control('c'));
                session.handle(control('x'));
                session.handle(key(KeyCode::Char('p')));
            }
            "name" => named(&mut session, "org-set-property"),
            _ => {
                session.handle(meta('j'));
            }
        }
        settle(&mut session);
        argument(&mut session, "CUSTOM");
        argument(&mut session, "value");
        assert_eq!(
            text(&session),
            "* H\n:PROPERTIES:\n:CUSTOM: value\n:END:\n",
            "{path}: {}",
            session.echo()
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}
