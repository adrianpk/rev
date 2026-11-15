use super::editor::*;
use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, ModalCommand, Split};
use std::time::{Duration, Instant};

fn editor(profile: &str, bytes: usize) -> Editor {
    let config = format!("editor.interaction_profile='{profile}'\ncompletion.automatic=false");
    let mut e = Editor::new(&config, "plain", &format!("{}|", " ".repeat(bytes)));
    if profile == "modal" {
        e.session.execute(Command::Modal(ModalCommand::Insert));
    }
    e.session.execute(Command::Split(Split::Right));
    e
}

fn drain(e: &mut Editor) -> (usize, Duration) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut turns = 0;
    let mut maximum = Duration::ZERO;
    while e.session.remaining_input(Instant::now()) == Some(Duration::ZERO) {
        assert!(Instant::now() < deadline, "queued pair input deadline");
        let before = Instant::now();
        e.session.expire_input(before);
        maximum = maximum.max(before.elapsed());
        turns += 1;
    }
    (turns, maximum)
}

// Pending physical keys must be released in FIFO order in either profile.
// A split shares one canonical source owner; no speculative mate is published
// while the first context request still needs cooperative turns.
#[test]
fn queued_release() {
    for profile in ["non-modal", "modal"] {
        for bytes in [1024, 65_536, 262_144, 1_048_576] {
            let mut e = editor(profile, bytes);
            let stamp = e.session.buffer().unwrap().stamp();
            let before = Instant::now();
            for c in ['(', 'q', ')'] {
                e.session.handle(character(c));
            }
            assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
            let (turns, maximum) = drain(&mut e);
            e.expect(&format!("{}(q)|", " ".repeat(bytes)));
            shared(&e);
            println!(
                "PAIR_INTEGRATED profile={profile} source={bytes} views=2 queue_released=true turns={turns} elapsed_us={} max_host_turn_us={}",
                before.elapsed().as_micros(),
                maximum.as_micros()
            );
        }
    }
}

// Counts share context work and publish all generated ends once, or refuse
// atomically. The one-MiB prefix cannot authorize even a partial counted pair.
#[test]
fn shared_expansion() {
    for profile in ["non-modal", "modal"] {
        for bytes in [1024, 65_536, 262_144, 1_048_576] {
            let mut e = editor(profile, bytes);
            let stamp = e.session.buffer().unwrap().stamp();
            let before = Instant::now();
            e.counted("3", '(');
            let admitted = bytes <= rev::pairs::MAX_BYTES;
            expansion(&e, bytes, admitted);
            assert_eq!(e.session.buffer().unwrap().stamp() != stamp, admitted);
            shared(&e);
            println!(
                "PAIR_EXPANSION profile={profile} source={bytes} count=3 views=2 admitted={admitted} elapsed_us={}",
                before.elapsed().as_micros()
            );
        }
    }
}

fn expansion(e: &Editor, bytes: usize, admitted: bool) {
    let suffix = if admitted { "(((|)))" } else { "|" };
    e.expect(&format!("{}{suffix}", " ".repeat(bytes)));
    assert_eq!(
        e.session.buffer().unwrap().history().len(),
        usize::from(admitted)
    );
}

fn shared(e: &Editor) {
    let stamp = e.session.buffer().unwrap().stamp();
    assert_eq!(e.session.windows().iter().count(), 2);
    assert!(
        e.session
            .windows()
            .iter()
            .all(|(_, v)| v.point.stamp == stamp)
    );
}

// Source read-only admission precedes pairing, including navigation-like skip
// and paired deletion. Prompt focus owns its literal input in both profiles.
#[test]
fn guarded_input() {
    for profile in ["non-modal", "modal"] {
        let config = format!("editor.interaction_profile='{profile}'");
        let mut e = Editor::build(&config, "plain", "(|)", true);
        if profile == "modal" {
            e.session.execute(Command::Modal(ModalCommand::Insert));
        }
        let stamp = e.session.buffer().unwrap().stamp();
        for event in [
            character('('),
            character(')'),
            key(KeyCode::Backspace, KeyModifiers::NONE),
        ] {
            e.session.handle(event);
            settle(&mut e.session);
            e.expect("(|)");
        }
        e.session.execute(Command::FindFile);
        e.session.handle(character('('));
        e.session
            .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
        e.expect("(|)");
        assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
}
