use super::*;
use rev::buffer::Buffers;
use rev::command::{RegionCommand, Split};
use rev::coordinates::Scalar;
use rev::pairs::{Query, Status};

// Both literal source ingress paths preserve unmatched punctuation and exact
// UTF-8/CRLF bytes, map shared views and retain native single-step history.
#[test]
fn ingress() {
    let payload = "([{\"界e\u{301}\r\n";
    for (entry, prefix) in [("paste", "original"), ("replacement", "")] {
        let mut e = Editor::new("", "plain", "|original");
        e.session.execute(Command::Split(Split::Right));
        e.session.execute(Command::Region(RegionCommand::Set));
        e.session.execute(Command::Motion(Motion::BufferEnd));
        settle(&mut e.session);
        let before = e.session.view().point;
        if entry == "paste" {
            e.session.handle(Event::Paste(payload.into()));
            settle(&mut e.session);
        } else {
            e.session.replace(0..8, payload).unwrap();
        }
        e.expect(&format!("{prefix}{payload}|"));
        assert_eq!(e.session.buffer().unwrap().mark(), Some(Scalar(0)));
        assert_eq!(e.session.buffer().unwrap().history().len(), 1, "{entry}");
        let stamp = e.session.buffer().unwrap().stamp();
        assert_ne!(stamp, before.stamp);
        assert!(
            e.session
                .windows()
                .iter()
                .all(|(_, v)| v.point.stamp == stamp)
        );
        e.session.execute(Command::History(History::Undo));
        settle(&mut e.session);
        e.expect("original|");
        e.session.execute(Command::History(History::Redo));
        settle(&mut e.session);
        e.expect(&format!("{prefix}{payload}|"));
    }
}

// A real visiting-file reload publishes literal bytes and retires retained pair
// facts. Later physical typing qualifies only the new source and profile.
#[test]
fn reload() {
    let mut e = Editor::new("", "plain", "(|)");
    e.type_key(')');
    let path = e
        .session
        .buffer()
        .unwrap()
        .file()
        .unwrap()
        .path()
        .to_owned();
    let original = e.session.buffer().unwrap().stamp();
    let replacement = "([{\"界e\u{301}\r\n";
    std::fs::write(&path, replacement).unwrap();
    e.session.execute(Command::RevertBuffer);
    observe(&mut e.session);
    assert_eq!(e.text(), replacement);
    assert_ne!(e.session.buffer().unwrap().stamp(), original);
    assert!(!e.session.buffer().unwrap().is_modified());
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    e.session.execute(Command::History(History::Undo));
    settle(&mut e.session);
    e.expect("()|");
    e.session.execute(Command::History(History::Redo));
    settle(&mut e.session);
    assert_eq!(e.text(), replacement);
    e.session.execute(Command::Motion(Motion::BufferEnd));
    settle(&mut e.session);
    e.type_key('(');
    e.expect(&format!("{replacement}(|"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), replacement);
}

// A programmatic replacement during deferred physical typing invalidates the
// captured ticket and point. No old pair plan can publish into the new revision.
#[test]
fn stale_work() {
    let source = " ".repeat(200_000);
    let mut e = Editor::new("", "plain", &format!("{source}|"));
    e.session.handle(character('('));
    assert_eq!(
        e.session.remaining_input(Instant::now()),
        Some(Duration::ZERO)
    );
    e.session.replace(0..0, "[").unwrap();
    settle(&mut e.session);
    e.expect(&format!("[|{source}"));
    assert_eq!(e.session.view().point.position, Scalar(1));
    assert_eq!(e.session.buffer().unwrap().history().len(), 1);
    assert_eq!(e.session.remaining_input(Instant::now()), None);
}

// Save-as changes the canonical language policy for both views. The buffer's
// runtime override survives; re-enabling pairs keeps the new quote preference.
#[test]
fn language_change() {
    let mut e = Editor::new("language.go.pairs.quotes=false", "rust", "|");
    let path = e
        .session
        .buffer()
        .unwrap()
        .file()
        .unwrap()
        .path()
        .with_file_name("renamed.go");
    e.session.execute(Command::Split(Split::Right));
    e.session.execute(Command::AutoPairMode);
    let stamp = e.session.buffer().unwrap().stamp();
    e.session.execute(Command::WriteFile);
    e.session
        .handle(Event::Paste(path.to_str().unwrap().into()));
    e.session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    observe(&mut e.session);
    assert_eq!(e.session.buffer().unwrap().stamp(), stamp);
    let policy = e.session.buffer().unwrap().pair_policy();
    assert!(!policy.enabled);
    assert!(!policy.quotes);
    e.session.execute(Command::AutoPairMode);
    e.type_key('(');
    e.expect("(|)");
    let stamp = e.session.buffer().unwrap().stamp();
    assert!(
        e.session
            .windows()
            .iter()
            .all(|(_, v)| v.point.stamp == stamp)
    );
    e.session.execute(Command::History(History::Undo));
    settle(&mut e.session);
    e.type_key('\'');
    e.expect("'|");
    assert_eq!(std::fs::read_to_string(path).unwrap(), "");
}

// Source replacement, mode changes and buffer-local toggles retire canonical
// requests. A stale proof cannot be observed again through another view.
#[test]
fn retirement() {
    use rev::buffer::Mode;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("()").unwrap();
    let b = buffers.get_mut(id).unwrap();
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    b.select_mode(Mode::Org).unwrap();
    assert_eq!(b.poll_pair_request(ticket, Instant::now()), Status::Retired);
    assert!(b.pair_answer(ticket).is_none());
    let ticket = b.start_pair_request(Query::At(Scalar(0))).unwrap();
    b.toggle_auto_pairs();
    assert_eq!(b.poll_pair_request(ticket, Instant::now()), Status::Retired);
    assert!(b.pair_answer(ticket).is_none());
}
