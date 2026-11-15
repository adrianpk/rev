use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Split};
use rev::config::PairHighlight;
use rev::coordinates::Scalar;
use rev::session::Session;
use rev::view::{Size, View};

use super::{Fixture, settings};

const SIZE: Size = Size {
    width: 100,
    height: 28,
};

fn editor(config: &str) -> Session {
    let snapshot = settings(config);
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.create("a界\r\n(x)").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(1))).unwrap();
    buffer.set_readonly(true);
    let mut view = View::with_settings(buffer, &snapshot);
    view.point.position = Scalar(5);
    view.top = 1;
    Session::new(buffers, view, SIZE).unwrap()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn named(session: &mut Session, name: &str) {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

fn choose(session: &mut Session, value: &str) {
    named(session, "pair-highlight-style");
    session.handle(Event::Paste(value.into()));
    session.handle(key(KeyCode::Enter));
}

fn style(session: &Session) -> PairHighlight {
    session
        .view()
        .pairs
        .style(session.buffer().unwrap().pair_policy())
}

// Named commands change only view policy on read-only source with editing off.
// Off restores the previous style and never publishes source, mark or history.
#[test]
fn named_styles_and_toggle() {
    let mut session = editor("pairs.enabled=false");
    let viewport = (session.view().top, session.view().left);
    let original = (
        session.view().point,
        session.buffer().unwrap().mark(),
        session.buffer().unwrap().state(),
    );
    for (value, expected) in [
        ("match", PairHighlight::Match),
        ("pulse", PairHighlight::Pulse),
        ("depth", PairHighlight::Depth),
    ] {
        choose(&mut session, value);
        assert_eq!(style(&session), expected);
        named(&mut session, "pair-highlight-mode");
        assert_eq!(style(&session), PairHighlight::Off);
        named(&mut session, "pair-highlight-mode");
        assert_eq!(style(&session), expected);
        choose(&mut session, "off");
        named(&mut session, "pair-highlight-mode");
        assert_eq!(style(&session), expected);
    }
    assert_eq!(
        (
            session.view().point,
            session.buffer().unwrap().mark(),
            session.buffer().unwrap().state()
        ),
        original
    );
    assert_eq!((session.view().top, session.view().left), viewport);
    let buffer = session.buffer().unwrap();
    assert!(!buffer.pair_policy().enabled);
    assert!(buffer.is_readonly());
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        "a界\r\n(x)"
    );
}

// Style prompts share native scalar bounds, validation, Tab feedback and C-g.
// Invalid/canceled drafts and supplied numeric prefixes cannot change the view.
#[test]
fn prompt_admission_and_cancel() {
    let mut session = editor(
        "minibuffer.input_limit=5\npairs.highlight='match'\nkeys.editor.'C-c h'='pair-highlight-mode'",
    );
    session.execute(Command::PairHighlightStyle);
    session.handle(Event::Paste("界é😀abc".into()));
    assert_eq!(session.echo(), "Prompt input limit exceeded");
    session.handle(key(KeyCode::Tab));
    assert!(session.echo().contains("Choose depth"));
    session.handle(Event::Paste("pulse".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(style(&session), PairHighlight::Match);
    for value in ["", "bad", "Depth"] {
        // The short selector fits the same configured scalar limit.
        session.execute(Command::PairHighlightStyle);
        session.handle(Event::Paste(value.into()));
        session.handle(key(KeyCode::Enter));
        assert_eq!(style(&session), PairHighlight::Match, "{value}");
    }
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('u'),
        KeyModifiers::CONTROL,
    )));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
    session.handle(key(KeyCode::Char('h')));
    assert!(session.echo().contains("do not support a prefix"));
    assert_eq!(style(&session), PairHighlight::Match);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A split starts from resolved defaults even when its sibling has an override.
// Subsequent choices and last-active styles stay independent in the shared source.
#[test]
fn independent_views() {
    let mut session = editor("pairs.highlight='match'\npairs.pulse_ms=50");
    let first = session.windows().selected_id();
    choose(&mut session, "pulse");
    session.execute(Command::Split(Split::Right));
    let second = session
        .windows()
        .iter()
        .map(|(id, _)| id)
        .find(|id| *id != first)
        .unwrap();
    session.select_view(second).unwrap();
    assert_eq!(style(&session), PairHighlight::Match);
    choose(&mut session, "off");
    session.select_view(first).unwrap();
    assert_eq!(style(&session), PairHighlight::Pulse);
    named(&mut session, "pair-highlight-mode");
    assert_eq!(style(&session), PairHighlight::Off);
    session.select_view(second).unwrap();
    named(&mut session, "pair-highlight-mode");
    assert_eq!(style(&session), PairHighlight::Match);
    assert_eq!(
        session
            .windows()
            .get(first)
            .unwrap()
            .pairs
            .last_non_off(session.buffer().unwrap().pair_policy()),
        PairHighlight::Pulse
    );
    assert_eq!(session.buffer().unwrap().pair_policy().pulse_ms, 50);
    assert!(session.buffer().unwrap().history().is_empty());
}

// New views resolve language defaults, explicit overrides survive a rebind and
// toggling a startup-off view has a deterministic depth style to restore.
#[test]
fn language_defaults_and_rebind() {
    let f = Fixture::new();
    let source = "pairs.highlight='off'\nlanguage.rust.pairs.highlight='pulse'\nlanguage.rust.pairs.pulse_ms=1000";
    let root = f.write("config.toml", source);
    let snapshot = rev::config::prepare(&root).snapshot;
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&f.write("source.rs", "()")).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::with_settings(buffer, &snapshot);
    assert_eq!(view.pairs.style(buffer.pair_policy()), PairHighlight::Pulse);
    view.pairs.choose(PairHighlight::Off, buffer.pair_policy());
    let plain = buffers.create("").unwrap();
    view.rebind(buffers.get(plain).unwrap());
    let defaults = buffers.get(plain).unwrap().pair_policy();
    assert_eq!(view.pairs.style(defaults), PairHighlight::Off);
    view.pairs.toggle(defaults);
    assert_eq!(view.pairs.style(defaults), PairHighlight::Pulse);
    let mut fresh = View::new(buffers.get(plain).unwrap());
    fresh.pairs.toggle(defaults);
    assert_eq!(fresh.pairs.style(defaults), PairHighlight::Depth);
    assert_eq!(std::fs::read_to_string(root).unwrap(), source);
}

// The two literal catalog additions are unbound in every factory context.
// Existing commands remain covered by the independent complete-map acceptance.
#[test]
fn catalog() {
    assert_eq!(Command::ALL.len(), 719);
    for (name, command) in [
        ("pair-highlight-mode", Command::PairHighlightMode),
        ("pair-highlight-style", Command::PairHighlightStyle),
    ] {
        assert_eq!(Command::named(name), Some(command));
        assert!(command.description().is_some());
        for context in rev::input::keymap::Context::ALL {
            assert!(context.defaults().iter().all(|(_, bound)| *bound != name));
        }
    }
}

// A view preference cannot discard existing undo/redo entries or an active mark.
// Verify a real source publication and both history directions around commands.
#[test]
fn retained_source_history() {
    use rev::buffer::edit::{History, Point, Replacement};

    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [Point {
        stamp: buffer.stamp(),
        position: Scalar(1),
    }];
    let edits = [Replacement {
        source: buffer.span(1..1).unwrap(),
        text: "()".into(),
    }];
    let prepared = buffer.prepare(&edits, &points, None).unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    buffer.navigate(History::Undo, &mut points, None).unwrap();
    buffer.set_mark(Some(Scalar(1))).unwrap();
    let mut view = View::new(buffer);
    view.point = points[0];
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    let original = (
        session.view().point,
        session.buffer().unwrap().mark(),
        session.buffer().unwrap().state(),
    );
    choose(&mut session, "pulse");
    named(&mut session, "pair-highlight-mode");
    assert_eq!(
        (
            session.view().point,
            session.buffer().unwrap().mark(),
            session.buffer().unwrap().state()
        ),
        original
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.buffer().unwrap().redo_history().len(), 1);
    session.execute(Command::History(History::Redo));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(session.buffer().unwrap().len_chars(), 6);
    choose(&mut session, "off");
    session.execute(Command::History(History::Undo));
    assert_eq!(session.buffer().unwrap().len_chars(), 4);
}

// A command issued while another prompt owns focus cannot alter the underlying
// view. Cancellation then restores normal command admission without source edits.
#[test]
fn focus_admission() {
    for focus in [
        Command::FindFile,
        Command::SearchForward,
        Command::ExecuteExtended,
    ] {
        let mut session = editor("");
        session.execute(focus);
        for command in [Command::PairHighlightMode, Command::PairHighlightStyle] {
            session.execute(command);
            assert_eq!(style(&session), PairHighlight::Depth);
        }
        session.execute(Command::KeyboardQuit);
        named(&mut session, "pair-highlight-mode");
        assert_eq!(style(&session), PairHighlight::Off);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
