use rev::buffer::edit::History;
use rev::command::{Command, KillCommand, LinearCommand};
use rev::config::{Snapshot, prepare};
use rev::session::Session;
use rev::view::{Size, View};

use super::{Buffers, point, replacement, text, type_at};

fn settings(config: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{config}");
    startup.snapshot
}

// Configured count and byte targets prune oldest whole typing groups. Equality
// admits two groups, while saved-state identity survives complete undo/redo.
#[test]
fn group_retention() {
    for (name, count, bytes, retained) in [
        ("count", 2, 1024, 2),
        ("payload equality", 100, 10, 2),
        ("payload exceeded", 100, 9, 1),
    ] {
        let config =
            format!("history.group_limit = {count}\nhistory.payload_target_bytes = {bytes}");
        let mut buffers = Buffers::with_settings(settings(&config)).unwrap();
        let buffer = buffers.get_mut(buffers.initial()).unwrap();
        let mut points = [point(buffer, 0)];
        for _ in 0..4 {
            buffer.break_typing();
            type_at(buffer, &mut points, 'é', 0);
            type_at(buffer, &mut points, '界', 0);
        }
        assert_eq!(buffer.history().len(), retained, "{name}");
        buffer.mark_saved(buffer.stamp()).unwrap();
        for _ in 0..retained {
            buffer
                .navigate(History::Undo, &mut points, Some(0))
                .unwrap();
        }
        assert_eq!(text(buffer), "é界".repeat(4 - retained), "{name}");
        assert!(buffer.is_modified(), "{name}");
        assert!(
            buffer
                .navigate(History::Undo, &mut points, Some(0))
                .is_err()
        );
        for _ in 0..retained {
            buffer
                .navigate(History::Redo, &mut points, Some(0))
                .unwrap();
        }
        assert_eq!(text(buffer), "é界é界é界é界", "{name}");
        assert!(!buffer.is_modified(), "{name}");
    }
}

// A lower policy leaves already published traversal intact. The next completed
// branch discards redo and prunes atomically, retaining the actual saved state.
#[test]
fn deferred_retention() {
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    for value in ["a", "b", "c"] {
        let end = buffer.len_chars();
        let edit = replacement(buffer, end..end, value);
        let prepared = buffer.prepare(&[edit], &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
    }
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    buffer.mark_saved(buffer.stamp()).unwrap();
    let stamp = buffer.stamp();
    buffer.apply_settings(&settings("history.group_limit = 1"));
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(
        (buffer.history().len(), buffer.redo_history().len()),
        (2, 1)
    );
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "abc");
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    let edit = replacement(buffer, 2..2, "z");
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(
        (buffer.history().len(), buffer.redo_history().len()),
        (1, 0)
    );
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "ab");
    assert!(!buffer.is_modified());
    assert!(buffer.navigate(History::Undo, &mut [], None).is_err());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "abz");
    assert!(buffer.is_modified());
}

fn editor(config: &str, source: &str, position: usize) -> Session {
    let mut buffers = Buffers::with_settings(settings(config)).unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = rev::coordinates::Scalar(position);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

fn ring(session: &Session) -> Vec<String> {
    session.kill_ring().map(ToString::to_string).collect()
}

// Capacity evicts the oldest entry and yank-pop wraps over only retained text;
// one-entry rings remain usable rather than disabling yank or rotating to evictees.
#[test]
fn ring_eviction() {
    for (capacity, expected) in [(1, vec![" c"]), (2, vec![" c", " b"])] {
        let config = format!("kill_ring.capacity = {capacity}\nkill_ring.coalesce = false");
        let mut session = editor(&config, "a b c", 0);
        for _ in 0..3 {
            session.execute(Command::Kill(KillCommand::Word));
        }
        assert_eq!(ring(&session), expected);
        assert_eq!(text(session.buffer().unwrap()), "");
        session.execute(Command::Linear(LinearCommand::Yank));
        assert_eq!(text(session.buffer().unwrap()), " c");
        session.execute(Command::Linear(LinearCommand::Pop));
        assert_eq!(text(session.buffer().unwrap()), *expected.last().unwrap());
        session.execute(Command::Linear(LinearCommand::Pop));
        assert_eq!(text(session.buffer().unwrap()), " c");
    }
}

// Coalescing follows the existing forward/append and backward/prepend policy;
// disabling it keeps each adjacent kill as a separate newest-first ring entry.
#[test]
fn directional_coalescing() {
    for (coalesce, backward, expected) in [
        (true, false, vec!["a b"]),
        (true, true, vec!["a b"]),
        (false, false, vec![" b", "a"]),
        (false, true, vec!["a ", "b"]),
    ] {
        let mut session = editor(
            &format!(
                "kill_ring.coalesce = {coalesce}\nkill_ring.capacity = {}",
                if coalesce { 1 } else { 2 }
            ),
            "a b",
            if backward { 3 } else { 0 },
        );
        let command = if backward {
            KillCommand::BackwardWord
        } else {
            KillCommand::Word
        };
        session.execute(Command::Kill(command));
        session.execute(Command::Kill(command));
        assert_eq!(ring(&session), expected);
        assert_eq!(session.buffer().unwrap().history().len(), 2);
    }
}
