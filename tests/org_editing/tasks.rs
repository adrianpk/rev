use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand};

const STAMP: &str = "[2026-10-28 Wed 17:05]";
pub(super) fn clock() -> Result<String, String> {
    Ok(STAMP.into())
}
pub(super) fn choose(session: &mut Session, state: &str) {
    action(session, OrgCommand::SetTodo);
    argument(session, state);
}

// Native command, default chord and configured alias share one transition and
// preserve priority, tags, line endings, EOF and ordinary unknown title words.
#[test]
fn task_paths() {
    for path in ["key", "name", "alias"] {
        let source = "* [#A] 界 title :tag:\r\nbody";
        let (_fixture, mut session) = editor("[keys.org]\n'M-j' = 'org-todo'", "notes.org", source);
        match path {
            "key" => {
                session.handle(control('c'));
                session.handle(control('t'));
            }
            "name" => named(&mut session, "org-todo"),
            _ => {
                session.handle(meta('j'));
            }
        }
        settle(&mut session);
        assert_eq!(
            text(&session),
            source.replacen("* ", "* TODO ", 1),
            "{path}: {}",
            session.echo()
        );
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(text(&session), source.replacen("* ", "* DONE ", 1));
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(text(&session), source);
        action(&mut session, OrgCommand::PreviousTodo);
        assert_eq!(text(&session), source.replacen("* ", "* DONE ", 1));
    }
    let (_fixture, mut session) = editor("", "notes.org", "* UNKNOWN title");
    action(&mut session, OrgCommand::Todo);
    assert_eq!(text(&session), "* TODO UNKNOWN title");
}

// Distinct file sequences and category declarations override configured defaults;
// annotations select exact fast keys without persisting declaration syntax.
#[test]
fn task_declarations() {
    for kind in ["TODO", "SEQ_TODO", "TYP_TODO"] {
        let source = format!(
            "#+{kind}: NEXT(n) WAIT(w) | FINISHED(f)\n#+TODO: BUG(b) FIXED(x)\n* WAIT Title\n"
        );
        let (_fixture, mut session) = editor(
            "org.todo.sequence=['OPEN','CLOSED']\norg.todo.finished=['CLOSED']",
            "notes.org",
            &source,
        );
        goto(&mut session, 3);
        action(&mut session, OrgCommand::Todo);
        assert_eq!(text(&session), source);
        argument(&mut session, "b");
        assert_eq!(
            text(&session),
            source.replace("* WAIT", "* BUG"),
            "{}",
            session.echo()
        );
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(text(&session), source.replace("* WAIT", "* FIXED"));
        action(&mut session, OrgCommand::NextTodo);
        assert_eq!(text(&session), source.replace("* WAIT ", "* "));
    }
}

// One completion publishes the keyword, CLOSED, state note and parent cookie.
// Undo/redo restore every byte in one history step, including owned drawers.
#[test]
fn task_transaction() {
    let source = "#+TODO: TODO(t) | DONE(d@)\r\n* Parent [0/1]\r\n** TODO [#A] Child :work:\r\nSCHEDULED: <2026-10-29 Thu>\r\n:PROPERTIES:\r\n:OWNER: keep\r\n:END:\r\nbody";
    let (_fixture, session) = editor(
        "org.todo.log_done=true\norg.todo.log_drawer='LOGBOOK'",
        "notes.org",
        source,
    );
    let mut session = session.with_task_clock(clock);
    goto(&mut session, 3);
    choose(&mut session, "DONE");
    assert_eq!(text(&session), source);
    argument(&mut session, "completed 界");
    let expected = source.replace("[0/1]", "[1/1]").replace("** TODO", "** DONE")
        .replace("SCHEDULED:", &format!("CLOSED: {STAMP} SCHEDULED:"))
        .replace(":END:\r\n", &format!(":END:\r\n:LOGBOOK:\r\n- State \"DONE\" from \"TODO\" {STAMP} \\\\\r\n  completed 界\r\n:END:\r\n"));
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
}

// Blocking is checked before the clock or draft. Descendants, ordered siblings
// and a local exemption use recognized task words rather than arbitrary titles.
#[test]
fn task_blocking() {
    for (source, line, blocked) in [
        ("* TODO Parent\n** TODO Child\n", 1, true),
        ("* TODO Parent\n** DONE Child\n", 1, false),
        (
            "* Parent\n:PROPERTIES:\n:ORDERED: t\n:END:\n** TODO First\n** TODO Second\n",
            6,
            true,
        ),
        (
            "* TODO Parent\n:PROPERTIES:\n:NOBLOCKING: t\n:END:\n** TODO Child\n",
            1,
            false,
        ),
    ] {
        let (_fixture, mut session) = editor("org.todo.block_children=true", "notes.org", source);
        goto(&mut session, line);
        choose(&mut session, "DONE");
        assert_eq!(
            session.buffer().unwrap().history().is_empty(),
            blocked,
            "{}",
            session.echo()
        );
        if blocked {
            assert_eq!(text(&session), source);
            assert!(session.echo().contains("blocked"));
        }
    }
}

// State-entry logging outranks departure logging. Inherited LOGGING resets the
// defaults, nil suppresses them and an existing selected drawer receives newest first.
#[test]
fn task_logging() {
    for (property, next, note, logged, closed) in [
        ("", "WAIT", true, true, false),
        (":LOGGING: nil\n", "WAIT", false, false, false),
        (":LOGGING: logdone\n", "DONE", false, false, true),
        (":LOGGING: WAIT(!)\n", "WAIT", false, true, false),
    ] {
        let source = format!(
            "#+TODO: TODO(t!) WAIT(w@/!) | DONE(d!)\n* Parent\n:PROPERTIES:\n{property}:LOG_INTO_DRAWER: LOGBOOK\n:END:\n** TODO Child\n:LOGBOOK:\n- older\n:END:\n"
        );
        let (_fixture, session) = editor("org.todo.log_done=true", "notes.org", &source);
        let mut session = session.with_task_clock(clock);
        goto(&mut session, 6 + usize::from(!property.is_empty()));
        choose(&mut session, next);
        if note {
            argument(&mut session, "why");
        }
        let result = text(&session);
        assert!(
            result.contains(&format!("** {next} Child")),
            "{property}: {}",
            session.echo()
        );
        assert_eq!(result.contains("- State"), logged);
        assert_eq!(result.contains("CLOSED:"), closed);
        if logged {
            assert!(result.find("- State").unwrap() < result.find("- older").unwrap());
        }
    }
}

// Canceling selection or logging, or accepting a stale prompt, cannot publish
// any part of a transition. A failed operation clock also leaves no history.
#[test]
fn task_drafts() {
    for stale in [false, true] {
        let source = "#+TODO: TODO | DONE(@)\n* TODO H\n";
        let (_fixture, session) = editor("org.todo.log_done=true", "notes.org", source);
        let mut session = session.with_task_clock(clock);
        goto(&mut session, 2);
        choose(&mut session, "DONE");
        assert_eq!(text(&session), source);
        if stale {
            session.replace(0..0, "prefix\n").unwrap();
            argument(&mut session, "note");
            assert!(session.echo().contains("source or view changed"));
            assert_eq!(text(&session), format!("prefix\n{source}"));
        } else {
            session.execute(Command::KeyboardQuit);
            assert_eq!(text(&session), source);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
    let (_fixture, session) = editor("org.todo.log_done=true", "notes.org", "* TODO H\n");
    let mut session = session.with_task_clock(|| Err("clock unavailable".into()));
    action(&mut session, OrgCommand::NextTodo);
    assert_eq!(text(&session), "* TODO H\n");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.echo(), "clock unavailable");
}
