use super::*;
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
pub(super) fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709303400)),
    )
}
pub(super) const SOURCE: &str = "* TODO Due habit\nSCHEDULED: <2024-02-28 Wed .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n- State \"DONE\" from \"TODO\" [2024-02-25 Sun]\n* TODO Future habit\nSCHEDULED: <2024-03-03 Sun .+2d/3d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n";
fn visible(s: &mut Session, needle: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(needle))
}
// Native controls operate on one derived source projection, with meaningful
// today-only/all-today/hidden states, typed graph entry and return to source.
#[test]
fn habit_agenda_controls() {
    let (_f, session) = editor("", "notes.org", SOURCE);
    let mut s = session.with_org_clock(clock);
    let view = s.view().clone();
    let stamp = s.buffer().unwrap().stamp();
    action(&mut s, OrgCommand::HabitGraph);
    assert!(visible(&mut s, "Habit agenda 2024-03-01"), "{}", s.echo());
    assert!(visible(&mut s, "Due habit"));
    assert!(!visible(&mut s, "Future habit"));
    s.handle(key(KeyCode::Char('A')));
    assert!(visible(&mut s, "Future habit"));
    s.handle(key(KeyCode::Char('K')));
    assert!(!visible(&mut s, "Due habit"));
    s.handle(key(KeyCode::Char('K')));
    assert!(visible(&mut s, "Due habit"));
    action(&mut s, OrgCommand::HabitDate);
    argument(&mut s, "2024-03-03");
    assert!(!visible(&mut s, "Due habit"));
    action(&mut s, OrgCommand::HabitTodayOnly);
    assert!(visible(&mut s, "Future habit"));
    action(&mut s, OrgCommand::HabitWindow);
    argument(&mut s, "0 0 4");
    assert!(visible(&mut s, "!"));
    s.handle(Event::Paste("must not edit".into()));
    s.execute(Command::Newline);
    assert_eq!(text(&s), SOURCE);
    assert_eq!(s.view(), &view);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(s.buffer().unwrap().history().is_empty());
    s.handle(control('g'));
    assert!(!visible(&mut s, "Habit agenda"));
    assert_eq!(s.view(), &view);
}
// Refresh alone observes canonical edits and current time. Painting or waiting
// retains the immutable graph and creates no new analysis deadlines after settle.
#[test]
fn habit_agenda_refresh() {
    let (_f, session) = editor("", "notes.org", SOURCE);
    let mut s = session.with_org_clock(clock);
    action(&mut s, OrgCommand::HabitGraph);
    s.replace(0..0, "# unsaved\n").unwrap();
    settle(&mut s);
    assert!(visible(&mut s, "stale: g refresh"));
    let snapshot = text(&s);
    let history = s.buffer().unwrap().history().len();
    for _ in 0..3 {
        s.frame().unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
    }
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert!(!visible(&mut s, "stale: g refresh"));
    assert_eq!(text(&s), snapshot);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    action(&mut s, OrgCommand::HabitWindow);
    argument(&mut s, "367 0 0");
    assert!(s.echo().contains("366"));
    assert_eq!(text(&s), snapshot);
    s.handle(control('g'));
    assert!(visible(&mut s, "Habit agenda"));
}
