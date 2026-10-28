use super::repeaters::clock;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};

// Native completion recognizes a local habit and keeps schedule, reset, actual
// history and LAST_REPEAT in the existing single source/history transaction.
#[test]
fn habit_completion() {
    for (name, repeat, next) in [
        ("restart", ".+2d/3d", "2024-03-03 Sun"),
        ("backlog", "+2d/3d", "2024-03-01 Fri"),
        ("catchup", "++2d/3d", "2024-03-03 Sun"),
    ] {
        let source = format!(
            "* TODO Habit\nSCHEDULED: <2024-02-28 Wed {repeat}>\n:PROPERTIES:\n:STYLE: habit\n:END:\n"
        );
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::NextTodo);
        let result = text(&session);
        assert!(
            result.contains(&format!("<{next} {repeat}>")),
            "{name}: {}",
            session.echo()
        );
        assert!(result.starts_with("* TODO Habit\n"));
        assert!(result.contains(":STYLE: habit\n:LAST_REPEAT: [2024-03-01 Fri 14:30]"));
        assert!(result.contains("- State \"DONE\" from \"TODO\" [2024-03-01 Fri 14:30]"));
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), result);
    }
}

// An invalid habit cannot fall through to an ordinary finished task or publish
// a partial date/property/log edit, including a finished reset-state override.
#[test]
fn habit_rejections() {
    for (name, scheduled, property) in [
        ("missing", "", ""),
        ("ordinary", "SCHEDULED: <2024-02-28 Wed>\n", ""),
        ("hourly", "SCHEDULED: <2024-02-28 Wed 14:00 .+24h>\n", ""),
        ("equal", "SCHEDULED: <2024-02-28 Wed .+2d/2d>\n", ""),
        (
            "calendar range",
            "SCHEDULED: <2024-02-28 Wed .+1m/30d>\n",
            "",
        ),
        (
            "finished reset",
            "SCHEDULED: <2024-02-28 Wed .+2d/3d>\n",
            ":REPEAT_TO_STATE: DONE\n",
        ),
    ] {
        let source =
            format!("* TODO Habit\n{scheduled}:PROPERTIES:\n:STYLE: habit\n{property}:END:\n");
        let (_fixture, session) = editor("", "notes.org", &source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::NextTodo);
        assert!(
            session.echo().contains("habit"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}
