use super::*;
use std::time::{Duration, Instant};

fn type_at(session: &mut Session, character: char, now: Instant) {
    session.handle_at(
        Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE)),
        now,
    );
}
fn ready(session: &mut Session, now: Instant) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        session.expire_input(now);
        if session.remaining_input(now).is_none() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "completion remained busy: {}",
            session.echo()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

// A native identifier edit arms one exact debounce deadline; no work/list is
// published early, and a completed request leaves no idle wake or source edit.
#[test]
fn debounce_and_idle() {
    let (_fixture, mut session) = source("fn cfg_value() {}\nfn main() { c| }");
    let now = Instant::now();
    type_at(&mut session, 'f', now);
    assert_eq!(
        session.remaining_input(now),
        Some(Duration::from_millis(150))
    );
    session.expire_input(now + Duration::from_millis(149));
    assert!(session.completion().is_none());
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    ready(&mut session, now + Duration::from_millis(150));
    assert_eq!(
        session.completion().unwrap().candidates()[0].text(),
        "cfg_value"
    );
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert!(
        session
            .remaining_input(now + Duration::from_secs(1))
            .is_none()
    );
}

// Rapid edits replace the debounce context, and edits during owned work retire
// its result. Only the latest current prefix can become an accepted edit.
#[test]
fn newest_edit_wins() {
    let (_fixture, mut session) = source("fn cfg_value() {}\nfn main() { c| }");
    let now = Instant::now();
    type_at(&mut session, 'f', now);
    type_at(&mut session, 'g', now + Duration::from_millis(50));
    assert_eq!(
        session.remaining_input(now),
        Some(Duration::from_millis(200))
    );
    session.expire_input(now + Duration::from_millis(200));
    type_at(&mut session, 'x', now + Duration::from_millis(201));
    session.handle_at(
        Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        now + Duration::from_millis(202),
    );
    ready(&mut session, now + Duration::from_millis(352));
    assert_eq!(
        session.completion().unwrap().candidates()[0].text(),
        "cfg_value"
    );
    session.execute(Command::Completion(C::Accept));
    assert!(text(&session).ends_with("{ cfg_value }"));
    session.execute(Command::History(History::Undo));
    assert!(text(&session).ends_with("{ cfg }"));
}

// Cancellation suppresses reopening without a new text edit; bracketed paste
// never schedules a per-character request. A later eligible key can recover.
#[test]
fn cancellation_and_paste() {
    let (_fixture, mut session) = source("fn cfg_value() {}\nfn main() { c| }");
    let now = Instant::now();
    type_at(&mut session, 'f', now);
    ready(&mut session, now + Duration::from_millis(150));
    key(&mut session, KeyCode::Esc, KeyModifiers::NONE);
    session.expire_input(now + Duration::from_secs(10));
    assert!(session.completion().is_none());
    assert!(session.remaining_input(now).is_none());
    session.handle(Event::Paste("g".into()));
    assert!(session.remaining_input(now).is_none());
    type_at(&mut session, '_', now + Duration::from_secs(11));
    ready(&mut session, now + Duration::from_millis(11150));
    assert_eq!(
        session.completion().unwrap().candidates()[0].text(),
        "cfg_value"
    );
}

// Context suppression uses the actual Rust tree. Comments/strings stay quiet
// by default, explicitly enabled contexts work, and manual invocation bypasses
// automatic suppression without changing its accepted declaration coverage.
#[test]
fn literal_contexts() {
    for document in [
        "fn sample_value() {}\n// sa|",
        "fn sample_value() {}\nfn main() { let value = \"sa|\"; }",
    ] {
        for enabled in [false, true] {
            let config = format!("[completion]\nin_comments = {enabled}\nin_strings = {enabled}\n");
            let (_fixture, mut session) = configured_source(document, &config);
            let now = Instant::now();
            type_at(&mut session, 'm', now);
            ready(&mut session, now + Duration::from_millis(150));
            assert_eq!(session.completion().is_some(), enabled, "{document}");
            complete(&mut session);
            assert_eq!(
                session.completion().unwrap().candidates()[0].text(),
                "sample_value"
            );
        }
    }
}

// Configured thresholds count Unicode characters; Rust separators may trigger
// without an identifier. Normal prose and no-match contexts acquire no list.
#[test]
fn thresholds_and_separators() {
    for (document, character, expected) in [
        ("fn café_value() {}\nfn main() { c| }", 'a', true),
        ("fn café_value() {}\nfn main() { | }", 'c', false),
        ("fn sample() {}\nfn main() { target| }", '.', true),
        ("fn sample() {}\nfn main() { module:| }", ':', true),
        ("fn sample() {}\nfn main() { missing| }", 'x', false),
        ("fn é_value() {}\nfn main() { | }", 'é', false),
        ("fn é_value() {}\nfn main() { é| }", '_', true),
    ] {
        let (_fixture, mut session) = source(document);
        let now = Instant::now();
        type_at(&mut session, character, now);
        ready(&mut session, now + Duration::from_millis(150));
        assert_eq!(session.completion().is_some(), expected, "{document}");
    }
}

// Both runtime controls affect only the session. Popup bindings use the admitted
// completion map, including custom prefixes, while ordinary Enter remains intact.
#[test]
fn controls_and_bindings() {
    let config = "[completion]\nmax_candidates = 1\npopup_rows = 1\n[keys.completion]\n\"C-c x\" = \"completion-accept\"\n";
    let (_fixture, mut session) = configured_source(SOURCE, config);
    complete(&mut session);
    assert_eq!(session.completion().unwrap().candidates().len(), 1);
    key(&mut session, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert!(session.completion().is_some());
    key(&mut session, KeyCode::Char('x'), KeyModifiers::NONE);
    assert!(text(&session).contains("{ cfg_value }"));
    session.execute(Command::Completion(C::Mode));
    complete(&mut session);
    assert!(session.completion().is_none());
    session.execute(Command::Completion(C::Mode));
    session.execute(Command::Completion(C::AutoMode));
    type_at(&mut session, 'x', Instant::now());
    assert!(session.remaining_input(Instant::now()).is_none());
    session.execute(Command::Completion(C::AutoMode));
    assert!(session.echo().contains("enabled"));
}

// Startup parsing rejects out-of-bounds policies and unavailable popup actions;
// admitted bounds and false/empty overrides retain normal layering semantics.
#[test]
fn configuration_bounds() {
    for (key, rejected) in [
        ("delay_ms", "2001"),
        ("min_prefix", "0"),
        ("max_candidates", "201"),
        ("popup_rows", "33"),
    ] {
        let config = format!("[completion]\n{key} = {rejected}\n");
        let error =
            rev::config::parse(std::path::Path::new("config.toml"), config.as_bytes()).unwrap_err();
        assert_eq!(error.key, ["completion", key]);
    }
    assert!(
        rev::config::parse(
            std::path::Path::new("config.toml"),
            b"[keys.completion]\nX = 'save-buffer'"
        )
        .is_err()
    );
    assert!(
        rev::config::parse(
            std::path::Path::new("config.toml"),
            b"[keys.completion]\nC-g = 'completion-cancel'"
        )
        .is_err()
    );
}

// Smart case is applied to real extracted names rather than display labels.
// Explicit false/zero startup overrides survive include layering and retain
// independent siblings, with the normal immutable provenance contract.
#[test]
fn matching_and_layers() {
    for (prefix, expected) in [("C", 1), ("c", 2)] {
        let document =
            format!("fn Config_value() {{}}\nfn config_value() {{}}\nfn main() {{ {prefix}| }}");
        let (_fixture, mut session) = source(&document);
        let now = Instant::now();
        type_at(&mut session, 'o', now);
        ready(&mut session, now + Duration::from_millis(150));
        assert_eq!(session.completion().unwrap().candidates().len(), expected);
    }
    let fixture = fixture::Fixture::new();
    fixture.write(
        "base.toml",
        "[completion]\nautomatic = false\nmax_candidates = 1\n",
    );
    let path = fixture.write(
        "root.toml",
        "include = ['base.toml']\n[completion]\nenabled = false\ndelay_ms = 0\n",
    );
    let snapshot = rev::config::resolve(&rev::config::load(&path).unwrap()).unwrap();
    let policy = &snapshot.settings().completion;
    assert!(!policy.enabled && !policy.automatic);
    assert_eq!(policy.delay_ms, 0);
    assert_eq!(policy.max_candidates, 1);
    assert_eq!(policy.min_prefix, 2);
    assert_eq!(
        snapshot.origin(&["completion", "enabled"]),
        Some(path.as_path())
    );
}
