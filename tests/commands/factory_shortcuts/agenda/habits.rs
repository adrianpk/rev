use super::*;

const HABITS: &str = "* TODO Due habit\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n* TODO Future habit\nSCHEDULED: <2024-03-03 Sun .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n";

fn graph(session: &mut Session) {
    session.execute(Command::Org(OrgCommand::HabitGraph));
    settle(session);
    assert!(prompt(session, "Habit agenda 2024-03-01"));
    assert!(prompt(session, "Due habit"));
    assert!(!prompt(session, "Future habit"));
}

fn graph_row(session: &mut Session) -> String {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.text().contains("Due habit"))
        .unwrap()
        .text()
        .to_string()
}

// Habit controls keep native display policy and bounded graph-window input.
// d cannot open the displaced date prompt; view changes never edit source.
#[test]
fn habit_views() {
    for snapshot in profiles() {
        let (fixture, session) = org_editor(&snapshot, HABITS, 0, true);
        let mut session = session.with_org_clock(clock);
        let stamp = session.buffer().unwrap().stamp();
        let origin = session.view().clone();
        graph(&mut session);
        for retired in ["A", "G", "d"] {
            keys(&mut session, retired);
            assert!(!prompt(&mut session, "Future habit"));
            assert!(session.frame().unwrap().cursor().is_none());
        }
        keys(&mut session, "v A");
        assert!(prompt(&mut session, "Future habit"));
        let before = graph_row(&mut session);
        keys(&mut session, "v G");
        assert!(prompt(&mut session, "Habit graph preceding"));
        keys(&mut session, "C-g");
        assert_eq!(graph_row(&mut session), before);
        keys(&mut session, "v G");
        argument(&mut session, "367 0 0");
        assert!(session.echo().contains("366"));
        keys(&mut session, "C-g");
        assert_eq!(graph_row(&mut session), before);
        keys(&mut session, "v G");
        argument(&mut session, "0 0 40");
        let after = graph_row(&mut session);
        assert_ne!(after, before);
        assert_eq!(after.trim_end().chars().count(), 41);
        keys(&mut session, "K");
        assert!(!prompt(&mut session, "Due habit"));
        keys(&mut session, "K C-u K");
        assert!(prompt(&mut session, "Due habit"));
        session.handle(Event::Paste("blocked".into()));
        keys(&mut session, "C-g");
        assert_eq!(session.view(), &origin);
        unchanged(&fixture, &session, HABITS, stamp);
    }
}

fn old_settings(modal: bool) -> Snapshot {
    settings(&format!(
        "{}\n[keys.org_agenda]\n'd'='org-habit-graph-date'\n'M'='org-tags-view'\n's'='org-search-view'\n'A'='org-habit-toggle-show-all-today'\n'G'='org-habit-set-graph-window'\n'S'='org-agenda-sort'\n",
        profile(modal.then_some(ModalCommand::Normal))
    ))
}

fn invoke(session: &mut Session, route: &str) {
    if route.starts_with("org-") {
        named(session, route);
    } else {
        keys(session, route);
        settle(session);
    }
}

// Explicit legacy query/sort leaves and canonical names remain equivalent.
// They retain native effects instead of acquiring unrelated reference features.
#[test]
fn old_queries() {
    for modal in [false, true] {
        let snapshot = old_settings(modal);
        for (route, draft, title) in [
            ("M", "work", "Zebra"),
            ("org-tags-view", "work", "Zebra"),
            ("s", "Alpha", "Alpha 界"),
            ("org-search-view", "Alpha", "Alpha 界"),
            ("S", "title-up", "Alpha 界"),
            ("org-agenda-sort", "title-up", "Alpha 界"),
        ] {
            let (fixture, session) = org_editor(&snapshot, SOURCE, 0, false);
            let mut session = session.with_org_clock(clock);
            let stamp = session.buffer().unwrap().stamp();
            todo(&mut session);
            invoke(&mut session, route);
            assert!(session.frame().unwrap().cursor().is_some(), "{route}");
            argument(&mut session, draft);
            assert_eq!(
                session.query_agenda().unwrap().rows[0].title.as_ref(),
                title
            );
            keys(&mut session, "q");
            unchanged(&fixture, &session, SOURCE, stamp);
        }
    }
}

// Displaced habit-date and relocated display actions retain old user access
// and named invocation, including cancellation without closing the graph.
#[test]
fn old_habits() {
    for modal in [false, true] {
        let snapshot = old_settings(modal);
        for routes in [
            ["A", "G", "d"],
            [
                "org-habit-toggle-show-all-today",
                "org-habit-set-graph-window",
                "org-habit-graph-date",
            ],
        ] {
            let (fixture, session) = org_editor(&snapshot, HABITS, 0, false);
            let mut session = session.with_org_clock(clock);
            let stamp = session.buffer().unwrap().stamp();
            graph(&mut session);
            invoke(&mut session, routes[0]);
            assert!(prompt(&mut session, "Future habit"));
            invoke(&mut session, routes[1]);
            assert!(prompt(&mut session, "Habit graph preceding"));
            argument(&mut session, "0 0 4");
            invoke(&mut session, routes[2]);
            assert!(prompt(&mut session, "Habit agenda date"));
            keys(&mut session, "C-g");
            assert!(prompt(&mut session, "Habit agenda 2024-03-01"));
            invoke(&mut session, routes[2]);
            argument(&mut session, "2024-03-03");
            assert!(prompt(&mut session, "Habit agenda 2024-03-03"));
            keys(&mut session, "C-g");
            unchanged(&fixture, &session, HABITS, stamp);
        }
    }
}
