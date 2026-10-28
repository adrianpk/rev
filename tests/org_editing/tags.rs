use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

fn set(session: &mut Session, tags: &str) {
    action(session, OrgCommand::SetTags);
    argument(session, tags);
}

// Tag edits replace only the local suffix, preserving heading spelling, title,
// planning, unrelated text, CRLF/UTF-8 and EOF. A whole edit undoes in one step.
#[test]
fn local_tags() {
    for (name, source, tags, expected) in [
        (
            "new",
            "* TODO [#A] Head\r\nbody",
            "work @home",
            "* TODO [#A] Head  :work:@home:\r\nbody",
        ),
        (
            "replace",
            "* 界 title    :Work:work:  \nbody\n",
            ":New:New:#tag:",
            "* 界 title    :New:#tag:  \nbody\n",
        ),
        ("clear", "* H    :keep:\n", "", "* H    \n"),
        ("eof", "* H", "tag", "* H  :tag:"),
    ] {
        let (_fixture, mut session) = editor("", "notes.org", source);
        set(&mut session, tags);
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source, "{name}");
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected);
    }
}

// Inspection preserves local/inherited/file provenance and case. Removing an
// inherited name never rewrites its actual owner; factory inheritance is on.
#[test]
fn inherited_tags() {
    let source = "#+FILETAGS: :File:\n* Parent :Work:\n** Child :work:local:\nbody\n";
    for (config, included, excluded) in [
        ("", "inherited [L2] Work", "missing"),
        ("org.tags.inherit = false", "local [L3] work", "inherited"),
    ] {
        let (_fixture, mut session) = editor(config, "notes.org", source);
        goto(&mut session, 4);
        action(&mut session, OrgCommand::InspectTags);
        assert!(session.echo().contains(included), "{}", session.echo());
        assert!(!session.echo().contains(excluded));
        action(&mut session, OrgCommand::RemoveTag);
        argument(&mut session, "Work");
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
        action(&mut session, OrgCommand::RemoveTag);
        argument(&mut session, "work");
        assert_eq!(text(&session), source.replace(":work:local:", ":local:"));
    }
}

// Completion is case-sensitive and prompt-owned, using actual local/file tags
// and TAGS candidates. C-g retires the draft and leaves source untouched.
#[test]
fn tag_completion() {
    let source =
        "#+TAGS: project(p) proposed\n#+FILETAGS: :File:\n* Parent :Work:\n** Child :local:\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    goto(&mut session, 4);
    action(&mut session, OrgCommand::SetTags);
    session.handle(Event::Paste(":local:W".into()));
    session.handle(key(KeyCode::Tab));
    assert!(session.echo().contains("Work"));
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), source.replace(":local:", ":local:Work:"));
    session.execute(Command::History(History::Undo));
    action(&mut session, OrgCommand::SetTags);
    session.handle(Event::Paste("proj".into()));
    session.handle(key(KeyCode::Tab));
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), source.replace(":local:", ":project:"));
    session.execute(Command::History(History::Undo));
    action(&mut session, OrgCommand::SetTags);
    session.handle(Event::Paste("work".into()));
    session.handle(key(KeyCode::Tab));
    assert!(session.echo().contains("No matching"));
    session.execute(Command::KeyboardQuit);
    assert_eq!(text(&session), source);
}

// Binding, named action, exact heading C-c C-c and a validated alias select the
// same local edit. A deeper literal C-c C-c retains its own rejection behavior.
#[test]
fn tag_command_paths() {
    for path in ["key", "name", "context", "alias"] {
        let (_fixture, mut session) = editor(
            "[keys.org]\n'M-j' = 'org-set-tags-command'",
            "notes.org",
            "* H\n",
        );
        match path {
            "key" => {
                session.handle(control('c'));
                session.handle(control('q'));
            }
            "context" => {
                session.handle(control('c'));
                session.handle(control('c'));
            }
            "name" => named(&mut session, "org-set-tags-command"),
            _ => {
                session.handle(meta('j'));
            }
        }
        settle(&mut session);
        argument(&mut session, "tag");
        assert_eq!(text(&session), "* H  :tag:\n", "{path}: {}", session.echo());
    }
}

// Native matching projects the actual current heading, its configured inherited
// metadata and source TODO declarations without editing or materializing a task.
#[test]
fn tag_property_matching() {
    let source = "#+TODO: WAITING NEXT | FINISHED\n#+FILETAGS: :File:\n* Parent :Work:\n:PROPERTIES:\n:CUSTOM: inherited\n:END:\n** WAITING [#A] Child :local:\n:PROPERTIES:\n:EFFORT: 2.5\n:OWNER-ROLE: lead\n:END:\nbody\n";
    let (_fixture, mut session) =
        editor("org.properties.inherit = ['CUSTOM']", "notes.org", source);
    goto(&mut session, 7);
    for (query, expected) in [
        ("(Work|absent)+local+File", "yes"),
        ("work", "no"),
        ("local/WAITING", "yes"),
        ("local/FINISHED", "no"),
        ("EFFORT>=2+CUSTOM=\"inherited\"", "yes"),
        (
            "OWNER\\-ROLE=\"lead\"+LEVEL=2+TODO=\"WAITING\"+PRIORITY=\"A\"",
            "yes",
        ),
        ("ITEM=\"Child\"", "yes"),
    ] {
        action(&mut session, OrgCommand::MatchTags);
        argument(&mut session, query);
        assert_eq!(session.echo(), format!("Org match: {expected}"), "{query}");
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
