use rev::command::OrgCommand;
use rev::coordinates::{Cell, Coordinates};

use super::*;

pub(super) const OUTLINE: &str = "* Root\nbody\n** Child\nchild body\n*** Grand\ngrand body\n** Sibling\nsib body\n* Peer\npeer body\n";

pub(super) fn visible(session: &Session) -> Vec<usize> {
    let buffer = session.buffer().unwrap();
    (0..session.view().visible_lines(buffer))
        .map(|row| session.view().source_row(buffer, row).unwrap())
        .collect()
}
pub(super) fn action(session: &mut Session, command: OrgCommand) {
    session.execute(Command::Org(command));
    settle(session);
}

// Native TAB cycles heading-only, direct child headings and the entire subtree.
// S-TAB cycles overview, every heading and all source; the source stays intact.
#[test]
fn native_cycles() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    settle(&mut session);
    let stamp = session.buffer().unwrap().stamp();
    for expected in [
        vec![0, 8, 9, 10],
        vec![0, 2, 6, 8, 9, 10],
        (0..11).collect(),
    ] {
        session.handle(key(KeyCode::Tab));
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{}", session.echo());
    }
    for expected in [vec![0, 8, 10], vec![0, 2, 4, 6, 8, 10], (0..11).collect()] {
        session.handle(key(KeyCode::BackTab));
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{}", session.echo());
    }
    assert_eq!(text(&session), OUTLINE);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Each direct heading choice reaches the same source projection as native keys.
// Drawer commands hide only the admitted body, keeping both delimiters visible.
#[test]
fn direct_choices() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    for (command, expected) in [
        (OrgCommand::Overview, vec![0, 8, 10]),
        (OrgCommand::Contents, vec![0, 2, 4, 6, 8, 10]),
        (OrgCommand::ShowAll, (0..11).collect()),
        (OrgCommand::HideSubtree, vec![0, 8, 9, 10]),
        (OrgCommand::ShowChildren, vec![0, 2, 6, 8, 9, 10]),
        (OrgCommand::ShowSubtree, (0..11).collect()),
    ] {
        action(&mut session, command);
        assert_eq!(
            visible(&session),
            expected,
            "{command:?}: {}",
            session.echo()
        );
    }
    let source = "* Root\n:LOGBOOK:\nentry\n:END:\ntext\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("2".into()));
    session.handle(key(KeyCode::Enter));
    action(&mut session, OrgCommand::HideDrawer);
    assert_eq!(visible(&session), [0, 1, 3, 4, 5]);
    action(&mut session, OrgCommand::ShowDrawer);
    assert_eq!(visible(&session), (0..6).collect::<Vec<_>>());
    action(&mut session, OrgCommand::ToggleDrawer);
    assert_eq!(visible(&session), [0, 1, 3, 4, 5]);
    assert_eq!(text(&session), source);
}

// Requests can start analysis with coloring disabled; C-g retires the queued
// intent. A new explicit request after cancellation can reuse the one owned slot.
#[test]
fn pending_controls() {
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", OUTLINE);
    session.execute(Command::Org(OrgCommand::Overview));
    assert!(session.remaining_input(Instant::now()).is_some());
    session.execute(Command::KeyboardQuit);
    action(&mut session, OrgCommand::Overview);
    assert_eq!(visible(&session), [0, 8, 10]);
    action(&mut session, OrgCommand::ShowAll);
    assert_eq!(visible(&session), (0..11).collect::<Vec<_>>());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.syntax().is_empty())
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// TAB never folds a containing ancestor when the actual source is plain body,
// malformed syntax or a literal block. Source immutability is the rejection.
#[test]
fn exact_targets() {
    let source = "* Root\n#+begin_example\n,* Fake\n#+end_example\ntext\n";
    let (_fixture, mut session) = editor("", "notes.org", source);
    for line in [2, 3, 5] {
        session.execute(Command::GotoLine);
        session.handle(Event::Paste(line.to_string()));
        session.handle(key(KeyCode::Enter));
        action(&mut session, OrgCommand::Cycle);
        assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 6);
        assert_eq!(text(&session), source);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(
        session.view().cell(buffer).unwrap(),
        Coordinates::new(buffer)
            .unwrap()
            .to_cell(session.view().point.position)
            .unwrap()
    );
    assert_eq!(
        session.view().cell(buffer).unwrap(),
        Cell { row: 4, column: 0 }
    );
}

// Each startup policy has a literal projection oracle. Coloring and pretty
// headings cannot disable initialization; CRLF, Unicode and EOF keep source identity.
#[test]
fn startup_visibility() {
    let source = "* Røøt\r\n:PROPERTIES:\r\n:ID: café\r\n:END:\r\nprose\r\n** Child\r\nchild body\r\n* Peer\r\n:REQUEST:\r\ndetail 👩‍💻\r\n:END:";
    for (name, policy, expected) in [
        ("overview", "overview", vec![0, 7]),
        ("content", "content", vec![0, 5, 7]),
        ("showall", "showall", vec![0, 1, 3, 4, 5, 6, 7, 8, 10]),
        ("showeverything", "showeverything", (0..11).collect()),
        ("omitted", "", (0..11).collect()),
    ] {
        let config = format!(
            "syntax.enabled=false\norg.headings.pretty=false\n{}",
            if policy.is_empty() {
                String::new()
            } else {
                format!("org.startup_folded='{policy}'")
            }
        );
        let (fixture, mut session) = editor(&config, "notes.org", source);
        let stamp = session.buffer().unwrap().stamp();
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{name}: {}", session.echo());
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("notes.org")).unwrap(),
            source
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(!session.buffer().unwrap().is_modified());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Startup content shows every heading, including archived descendants. The
// existing runtime archive cycling rule remains a separate command behavior.
#[test]
fn archived_startup_visibility() {
    let source = "* Root :ARCHIVE:\nbody\n** Child\nbody";
    let (_fixture, mut session) = editor("org.startup_folded='content'", "notes.org", source);
    settle(&mut session);
    assert_eq!(visible(&session), [0, 2]);
    assert_eq!(text(&session), source);
}

// Only structurally recognized metadata can override configuration. Source
// order includes multiple tokens and body keywords; unknown words never erase a choice.
#[test]
fn startup_precedence() {
    for (name, metadata, hidden) in [
        ("mixed case", "#+sTaRtUp: OVERVIEW ShOwAlL odd", true),
        (
            "last token",
            "#+STARTUP: overview content showeverything",
            false,
        ),
        ("unknown fallback", "#+STARTUP: fold odd", false),
        (
            "unknown retains",
            "#+STARTUP: showall\n#+STARTUP: fold odd",
            true,
        ),
        (
            "body order",
            "#+STARTUP: showeverything\n* Child\n#+STARTUP: showall",
            true,
        ),
        (
            "source literal",
            "#+begin_src org\n#+STARTUP: overview\n#+end_src",
            false,
        ),
        (
            "example literal",
            "#+begin_example\n#+STARTUP: overview\n#+end_example",
            false,
        ),
        (
            "drawer literal",
            ":REQUEST:\n#+STARTUP: overview\n:END:",
            false,
        ),
        ("fixed width", ": #+STARTUP: overview", false),
    ] {
        let source = format!("* Root\n{metadata}\n:LOGBOOK:\nentry\n:END:\nprose");
        let (_fixture, mut session) = editor("", "notes.org", &source);
        settle(&mut session);
        let row = source.lines().position(|line| line == "entry").unwrap();
        assert_eq!(
            visible(&session).contains(&row),
            !hidden,
            "{name}: {}",
            session.echo()
        );
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("prose")),
            "{name}"
        );
        assert_eq!(text(&session), source);
    }
}

// Whole-view commands preserve outline choices, literal examples and source.
// Mixed state closes all first, then opens all without revealing the subtree.
#[test]
fn all_drawers_visibility() {
    let source = "* Root\n:PROPERTIES:\n:ID: one\n:END:\nprose\n:REQUEST:\nrequest\n:END:\n** Child\nbody\n* Peer\nbody\n#+begin_example\n:FAKE:\nliteral\n:END:\n#+end_example\n";
    let (_fixture, mut session) = editor("org.startup_folded='showall'", "notes.org", source);
    settle(&mut session);
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("9".into()));
    session.handle(key(KeyCode::Enter));
    action(&mut session, OrgCommand::HideSubtree);
    action(&mut session, OrgCommand::ShowAllDrawers);
    assert!(visible(&session).contains(&2));
    assert!(visible(&session).contains(&6));
    assert!(!visible(&session).contains(&9));
    action(&mut session, OrgCommand::HideAllDrawers);
    assert!(!visible(&session).contains(&2));
    assert!(visible(&session).contains(&14));
    action(&mut session, OrgCommand::ShowAll);
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("2".into()));
    session.handle(key(KeyCode::Enter));
    action(&mut session, OrgCommand::HideDrawer);
    action(&mut session, OrgCommand::ToggleAllDrawers);
    assert!(!visible(&session).contains(&2));
    assert!(!visible(&session).contains(&6));
    action(&mut session, OrgCommand::ToggleAllDrawers);
    assert_eq!(visible(&session), (0..18).collect::<Vec<_>>());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Return accepts the highlighted recent command without a query or Tab, in
// both selectors. Repeated invocation alternates drawer bodies; cancel is inert.
#[test]
fn repeat_drawer_command() {
    let source = "* Root\n:PROPERTIES:\n:ID: one\n:END:\nprose\n:REQUEST:\nrequest\n:END:\n";
    for profile in ["non-modal", "modal"] {
        let (_fixture, mut session) = editor(
            &format!("editor.interaction_profile='{profile}'"),
            "notes.org",
            source,
        );
        settle(&mut session);
        let stamp = session.buffer().unwrap().stamp();
        let open = if profile == "modal" {
            key(KeyCode::Char(':'))
        } else {
            meta('x')
        };
        for invocation in 0..33 {
            session.handle(open.clone());
            if invocation == 0 {
                session.handle(Event::Paste("org-toggle-all-drawers".into()));
            }
            let frame = session.frame().unwrap();
            let selected = frame
                .rows()
                .iter()
                .find(|row| {
                    row.band() == rev::display::Band::Completion && row.selection().is_some()
                })
                .unwrap();
            assert!(selected.text().starts_with("org-toggle-all-drawers"));
            session.handle(key(KeyCode::Enter));
            settle(&mut session);
            assert_eq!(session.echo(), "org-toggle-all-drawers", "{profile}");
            for row in [2, 6] {
                assert_eq!(
                    visible(&session).contains(&row),
                    invocation % 2 == 1,
                    "{profile}: invocation {invocation}, source row {row}"
                );
            }
        }
        let before = visible(&session);
        session.handle(open);
        session.frame().unwrap();
        session.handle(control('g'));
        settle(&mut session);
        assert_eq!(visible(&session), before, "{profile}");
        assert_eq!(text(&session), source, "{profile}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{profile}");
        assert!(session.buffer().unwrap().history().is_empty(), "{profile}");
    }
}
