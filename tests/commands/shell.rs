use super::*;
use rev::input::OrgContext;

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

// Exact shell sequences coexist with Org's longer C-u route. Cancellation and
// invalid suffixes retire the argument; a digit remains valid argument input.
#[test]
fn shell_sequences() {
    for org in [None, Some(OrgContext::Editing)] {
        for (prefix, suffix, command) in [
            (false, '!', Command::ShellCommand),
            (false, '|', Command::ShellCommandOnRegion),
            (true, '!', Command::ShellCommandInsert),
            (true, '|', Command::ShellFilterRegion),
        ] {
            let mut input = Input::default();
            input.set_org_context(org);
            if prefix {
                assert_eq!(
                    input.handle(control('u')),
                    Outcome::ArgumentPrefix(Command::PrefixArgument)
                );
            }
            assert_eq!(input.handle(meta(suffix)), Outcome::Command(command));
            assert_eq!(input.prefix(), None);
        }
    }
    let mut input = Input::default();
    input.set_org_context(Some(OrgContext::Editing));
    assert_eq!(
        input.handle(control('u')),
        Outcome::ArgumentPrefix(Command::PrefixArgument)
    );
    for event in [control('c'), control('v')] {
        assert_eq!(
            input.handle(event),
            Outcome::ArgumentPrefix(Command::PrefixArgument)
        );
    }
    assert_eq!(
        input.handle(character('t')),
        Outcome::Command(Command::Org(rev::command::OrgCommand::TangleBlock))
    );
    for suffix in [control('g'), meta('q')] {
        input.handle(control('u'));
        assert!(matches!(
            input.handle(suffix),
            Outcome::Command(Command::KeyboardQuit) | Outcome::Message(_)
        ));
        assert!(matches!(input.handle(character('a')), Outcome::Editing(_)));
    }
    input.handle(control('u'));
    assert_eq!(
        input.handle(character('4')),
        Outcome::ArgumentPrefix(Command::DigitArgument)
    );
    assert_eq!(input.prefix_argument().value(), 4);
    let Outcome::TextInvocation(invocation) = input.handle(character('a')) else {
        panic!("numeric argument did not reach text insertion");
    };
    assert_eq!(invocation.text, "a");
    assert_eq!(invocation.argument.value(), 4);
}

// Effective overrides replace exact defaults while modal C-u remains its native
// motion; a shell prefix cannot turn Normal scrolling into an argument prompt.
#[test]
fn shell_overrides() {
    let f = config_fixture::Fixture::new();
    let startup = rev::config::prepare(&f.write(
        "config.toml",
        "[keys.editor]\n'M-!'='forward-char'\n'C-u M-|'='shell-command-output'",
    ));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut input = Input::with_settings(&startup.snapshot);
    assert_eq!(
        input.handle(meta('!')),
        Outcome::Command(Command::Motion(rev::command::Motion::ForwardChar))
    );
    input.handle(control('u'));
    assert_eq!(
        input.handle(meta('|')),
        Outcome::Command(Command::ShellCommandOutput)
    );
    let modal =
        rev::config::prepare(&f.write("modal.toml", "[editor]\ninteraction_profile='modal'"));
    assert!(modal.diagnostic.is_none(), "{:?}", modal.diagnostic);
    let mut input = Input::with_settings(&modal.snapshot);
    assert!(matches!(
        input.handle(control('u')),
        Outcome::Command(Command::Navigate(_))
    ));
    assert_eq!(input.prefix(), None);
}
