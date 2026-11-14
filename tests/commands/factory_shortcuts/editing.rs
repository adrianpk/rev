use super::*;
use rev::command::{RegionCommand, Split};

// Real Meta input consumes its count once, moves to a Unicode scalar boundary
// on this logical line, and retains active selection, modal state and history.
#[test]
fn first_text() {
    for state in STATES {
        let snapshot = settings(profile(state));
        for mode in [Mode::Text, Mode::Org, Mode::Markdown] {
            first_text_cases(&snapshot, mode, state);
        }
    }
}

fn first_text_cases(snapshot: &Snapshot, mode: Mode, state: Option<ModalCommand>) {
    for (name, source, expected) in [
        ("spaces", "    x", 4),
        ("tab", "\tx", 1),
        ("mixed", " \t x", 3),
        ("blank", " \t ", 3),
        ("empty", "", 0),
        ("unicode space", "\u{a0}x", 1),
        ("wide", "  界", 2),
        ("combining", "  e\u{301}", 2),
        ("second line", "other\n  界", 8),
    ] {
        let mut session = source_editor(snapshot, source, mode, false);
        session.execute(Command::Region(RegionCommand::Set));
        enter(&mut session, state);
        let before = (
            session.editing_state(),
            session.buffer().unwrap().mark(),
            session.buffer().unwrap().mark_active(),
            session.buffer().unwrap().visual(),
            session.buffer().unwrap().stamp(),
        );
        keys(&mut session, "M-4 M-m");
        assert_eq!(session.view().point.position, Scalar(expected), "{name}");
        assert_eq!(
            (
                session.editing_state(),
                session.buffer().unwrap().mark(),
                session.buffer().unwrap().mark_active(),
                session.buffer().unwrap().visual(),
                session.buffer().unwrap().stamp(),
            ),
            before,
            "{name}: {mode:?} {state:?}"
        );
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// First-text consumes a supplied argument even though this destination ignores
// its magnitude; the next ordinary motion still advances only one grapheme.
#[test]
fn count_consumed() {
    for state in STATES {
        let mut session = editor(profile(state), "  abcd", Mode::Text);
        enter(&mut session, state);
        keys(&mut session, "M-4 M-m Right");
        assert_eq!(session.view().point.position, Scalar(3), "{state:?}");
        assert_eq!(text(&session), "  abcd");
    }
}

// The new pane chord balances an intentionally asymmetric layout without
// selecting a different pane or changing source, view state or split policy.
#[test]
fn pane_balance() {
    for state in STATES {
        let config = format!("{}[windows]\nsplit_ratio = 0.3\n", profile(state));
        let mut session = editor(&config, "retained", Mode::Text);
        for _ in 0..2 {
            session.execute(Command::Split(Split::Right));
        }
        enter(&mut session, state);
        let selected = session.windows().selected_id();
        let before: Vec<_> = session
            .windows()
            .iter()
            .map(|(id, v)| (id, v.point))
            .collect();
        let state_before = session.editing_state();
        let stamp = session.buffer().unwrap().stamp();
        let widths_before = widths(&session);
        assert!(widths_before.iter().max().unwrap() - widths_before.iter().min().unwrap() > 1);
        keys(&mut session, "C-x +");
        let widths = widths(&session);
        assert!(widths.iter().max().unwrap() - widths.iter().min().unwrap() <= 1);
        assert_eq!(session.windows().selected_id(), selected);
        assert_eq!(session.editing_state(), state_before);
        assert_eq!(
            session
                .windows()
                .iter()
                .map(|(id, v)| (id, v.point))
                .collect::<Vec<_>>(),
            before
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&session), "retained");
        assert!(session.buffer().unwrap().history().is_empty());
        // The next split still uses the configured 0.3 ratio, not 0.5.
        session.execute(Command::Split(Split::Right));
        assert_eq!(session.windows().area(selected, SIZE).unwrap().width(), 9);
    }
}

fn widths(session: &Session) -> Vec<u16> {
    session
        .windows()
        .geometry(SIZE)
        .unwrap()
        .panes()
        .iter()
        .map(|(_, area)| area.width())
        .collect()
}

// Redo restores two separately accepted paste groups through native history;
// the same count is available in nonmodal, Insert and Replace editing.
#[test]
fn redo_groups() {
    for state in [
        None,
        Some(ModalCommand::Insert),
        Some(ModalCommand::Replace),
    ] {
        let mut session = editor(profile(state), "", Mode::Text);
        enter(&mut session, state);
        session.handle(Event::Paste("界".into()));
        keys(&mut session, "M-m");
        session.handle(Event::Paste("e\u{301}".into()));
        let source = text(&session);
        let point = session.view().point.position;
        keys(&mut session, "M-2 C-_");
        assert_eq!(text(&session), "");
        keys(&mut session, "M-2 C-M-_");
        assert_eq!(text(&session), source);
        assert_eq!(session.view().point.position, point);
        let before = session.buffer().unwrap().stamp();
        keys(&mut session, "C-g");
        assert_eq!(session.buffer().unwrap().stamp(), before);
    }
}

// Normal retains C-r for redo; Visual states receive no new Control-Meta
// assignment. Nonmodal C-r still acquires backward search rather than history.
#[test]
fn redo_boundaries() {
    for state in STATES
        .into_iter()
        .filter(|s| !matches!(s, Some(ModalCommand::Insert | ModalCommand::Replace)))
    {
        let mut session = editor(profile(state), "", Mode::Text);
        session.execute(Command::Modal(ModalCommand::Insert));
        session.handle(Event::Paste("retained".into()));
        session.execute(Command::named("undo").unwrap());
        enter(&mut session, state);
        if state.is_some() {
            keys(&mut session, "C-M-_");
            assert_eq!(text(&session), "");
        }
        if state == Some(ModalCommand::Normal) {
            keys(&mut session, "C-r");
            assert_eq!(text(&session), "retained");
        }
        if state.is_none() {
            keys(&mut session, "C-r");
            assert!(
                session
                    .frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .any(|r| r.text().contains("backward"))
            );
            keys(&mut session, "C-g");
            assert_eq!(text(&session), "");
        }
    }
}

// User assignments replace each new leaf; local Org maps win over editor maps,
// and editor overrides do not leak into independent modal-state owners.
#[test]
fn overrides() {
    for chord in ["M-m", "C-x +", "C-M-_", "C-x p p"] {
        let config = format!("[keys.editor]\n'{chord}' = 'backward-char'\n");
        let mut session = editor(&config, "  xy", Mode::Text);
        keys(&mut session, chord);
        assert_eq!(session.view().point.position, Scalar(3), "{chord}");
        assert_eq!(text(&session), "  xy");
    }
    let mut local = editor(
        "[keys.editor]\n'M-m' = 'backward-char'\n[keys.org]\n'M-m' = 'beginning-of-buffer'",
        "  xy",
        Mode::Org,
    );
    keys(&mut local, "M-m");
    assert_eq!(local.view().point.position, Scalar(0));
    let mut modal = editor(
        "[editor]\ninteraction_profile = 'modal'\n[keys.editor]\n'M-m' = 'backward-char'\n[keys.vim_normal]\n'C-x +' = 'beginning-of-buffer'",
        "  xy",
        Mode::Text,
    );
    keys(&mut modal, "M-m");
    assert_eq!(modal.view().point.position, Scalar(2));
    keys(&mut modal, "C-x +");
    assert_eq!(modal.view().point.position, Scalar(0));
}

// Prompt input owns these chords, and quitting an incomplete Project prefix
// consumes it before the next ordinary source character can be interpreted.
#[test]
fn prompt_isolation() {
    let mut session = editor("", "retained", Mode::Text);
    session.execute(Command::Split(Split::Right));
    let geometry = session.windows().geometry(SIZE).unwrap();
    session.execute(Command::FindFile);
    keys(&mut session, "C-x + C-g");
    assert_eq!(session.windows().geometry(SIZE).unwrap(), geometry);
    keys(&mut session, "C-x p C-g");
    session.handle(Event::Paste("!".into()));
    assert_eq!(text(&session), "retained!");
}

// First-text remains a source-only motion in a read-only buffer; no profile
// needs a write grant or replacement history group to reach its destination.
#[test]
fn readonly_motion() {
    for state in STATES {
        let mut session = source_editor(&settings(profile(state)), "  界", Mode::Text, true);
        enter(&mut session, state);
        let stamp = session.buffer().unwrap().stamp();
        keys(&mut session, "M-m");
        assert_eq!(session.view().point.position, Scalar(2));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&session), "  界");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Discovery reads the same effective map as dispatch. Reassigning the new pane
// chord's target to a shorter alias updates its displayed hint immediately.
#[test]
fn effective_hints() {
    for (config, hint) in [
        ("", "C-x +"),
        ("[keys.editor]\n'M-m' = 'balance-windows'", "M-m"),
    ] {
        let mut input = rev::input::Input::with_settings(&settings(config));
        input.commands().unwrap();
        assert_eq!(
            input
                .palette()
                .unwrap()
                .hint(Command::named("balance-windows").unwrap()),
            Some(hint)
        );
    }
}
