use super::*;
use rev::input::{Input, OrgContext, Outcome};

// Clock reports retain named access and explicit old user assignments after
// their former factory chord becomes a read-only column projection.
#[test]
fn clock_access() {
    for state in [None, Some(ModalCommand::Normal)] {
        let config = format!(
            "{}[keys.org]\n'C-c C-x C-c' = 'org-clock-report'\n",
            profile(state)
        );
        let snapshot = settings(&config);
        let source = "* Task\n\n";
        let (fixture, mut session) = org_editor(&snapshot, source, source.len() - 1, false);
        keys(&mut session, "C-c C-x C-c");
        settle(&mut session);
        let report = text(&session);
        assert!(report.contains("#+BEGIN: clocktable"), "{}", session.echo());
        assert!(session.query_agenda().is_none());
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        named(&mut session, "org-clock-report");
        assert_eq!(text(&session), report);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
            source
        );
    }
}

// Removing the h factory leaf does not invalidate an explicit old assignment.
// The retained owner still validates the header and publishes one source edit.
#[test]
fn old_header_override() {
    for state in [None, Some(ModalCommand::Normal)] {
        let config = format!(
            "{}[keys.org]\n'C-c C-v h' = 'org-babel-insert-header-arg'\n",
            profile(state)
        );
        let snapshot = settings(&config);
        let source = "#+BEGIN_SRC sh\ntrue\n#+END_SRC\n";
        let (_fixture, mut session) = org_editor(&snapshot, source, 0, false);
        header_key(&mut session, "h");
        assert!(prompt(&mut session, "Source header argument"));
        session.handle(Event::Paste(":exports code".into()));
        keys(&mut session, "RET");
        settle(&mut session);
        assert_eq!(
            text(&session),
            source.replace("BEGIN_SRC sh", "BEGIN_SRC sh :exports code")
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        keys(&mut session, "C-_");
        assert_eq!(text(&session), source);
    }
}

// Org-local additions cannot leak into another editing or projection owner.
// An argument prompt consumes the same input without emitting an Org command.
#[test]
fn role_boundaries() {
    for snapshot in profiles() {
        for role in [
            None,
            Some(OrgContext::Capture),
            Some(OrgContext::SourceEdit),
            Some(OrgContext::Agenda),
            Some(OrgContext::Calendar),
        ] {
            for (sequence, name) in [
                ("C-c C-x E", "org-inc-effort"),
                ("C-c C-x C-c", "org-columns"),
                ("C-c C-v j", "org-babel-insert-header-arg"),
                ("C-c C-v RET", "org-babel-insert-header-arg"),
            ] {
                let mut input = Input::with_settings(&snapshot);
                input.set_org_context(role);
                let expected = Outcome::Command(Command::named(name).unwrap());
                assert_ne!(feed(&mut input, sequence), expected, "{role:?}/{sequence}");
                input.argument("Retained: ", "").unwrap();
                assert_ne!(feed(&mut input, sequence), expected);
            }
        }
    }
}

fn feed(input: &mut Input, sequence: &str) -> Outcome {
    let mut last = Outcome::Ignored;
    for key in Sequence::parse(sequence).unwrap().chords() {
        last = input.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
    }
    last
}

// Discovery follows the accepted Org map in either profile. The displaced
// report has no invented alias, while both normalized header routes stay live.
#[test]
fn effective_hints() {
    for snapshot in profiles() {
        let mut input = Input::with_settings(&snapshot);
        input.set_org_context(Some(OrgContext::Editing));
        input.commands().unwrap();
        let palette = input.palette().unwrap();
        for (command, hint) in [
            (OrgCommand::IncEffort, Some("C-c C-x E")),
            (OrgCommand::Columns, Some("C-c C-x C-c")),
            (OrgCommand::ClockReport, None),
        ] {
            assert_eq!(palette.hint(Command::Org(command)), hint);
        }
        let header = palette
            .hint(Command::Org(OrgCommand::InsertHeaderArg))
            .unwrap();
        assert!(["C-c C-v j", "C-c C-v RET"].contains(&header));
    }
}

// Existing contextual checkbox/table dispatch and the radio-button neighbor
// remain distinct from columns. Shift-Meta-Return does not invent TODO headings.
#[test]
fn retained_dispatch() {
    for snapshot in profiles() {
        for (name, source, point, sequence, expected) in [
            ("checkbox", "- [ ] task\n", 0, "C-c C-c", "- [X] task\n"),
            ("table", "| a|b |\n", 2, "C-c C-c", "| a | b |\n"),
            ("plain item", "- item\n", 6, "M-S-RET", "- item\n- [ ] \n"),
            ("heading", "* Task\n", 2, "M-S-RET", "* Task\n"),
            (
                "radio",
                "- [X] first\n- [ ] second\n",
                14,
                "C-c C-x C-r",
                "- [ ] first\n- [X] second\n",
            ),
        ] {
            let (_fixture, mut session) = org_editor(&snapshot, source, point, false);
            keys(&mut session, sequence);
            settle(&mut session);
            assert_eq!(text(&session), expected, "{name}: {}", session.echo());
            assert!(session.query_agenda().is_none());
            assert_eq!(
                session.buffer().unwrap().history().len(),
                usize::from(source != expected)
            );
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source);
        }
    }
}
