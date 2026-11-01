#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "modal/keymaps.rs"]
mod keymaps;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use fixture::Fixture;
use rev::buffer::Buffers;
use rev::command::{Command, History, ModalCommand};
use rev::config::{InteractionProfile, prepare};
use rev::coordinates::Scalar;
use rev::input::EditingState;
use rev::session::Session;
use rev::view::{Size, View};

fn session(config: &str, text: &str, position: usize) -> Session {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", config);
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.create(text).unwrap();
    let mut view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    view.point.position = Scalar(position);
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

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

const VIM: &str = "[editor]\ninteraction_profile = 'vim'\n[tree]\nvisible = false";

// Every state-entry form resolves a native logical/grapheme destination. Open
// forms publish exactly one reversible edit; other transitions create none.
#[test]
fn insertion_entry() {
    for (name, source, point, entry, expected, cursor, edits) in [
        ("at point", "e\u{301}界!", 0, 'i', "e\u{301}界!", 0, 0),
        ("after grapheme", "e\u{301}界!", 0, 'a', "e\u{301}界!", 2, 0),
        ("first text", " \t界!", 3, 'I', " \t界!", 2, 0),
        ("logical end", "one\r\ntwo", 0, 'A', "one\r\ntwo", 3, 0),
        ("below", "one\ntwo", 1, 'o', "one\n\ntwo", 4, 1),
        ("above", "one\ntwo", 5, 'O', "one\n\ntwo", 4, 1),
        ("eof below", "界", 0, 'o', "界\n", 2, 1),
        ("empty above", "", 0, 'O', "\n", 0, 1),
        ("line end after", "one\r\ntwo", 3, 'a', "one\r\ntwo", 3, 0),
    ] {
        let mut session = session(VIM, source, point);
        session.handle(key(KeyCode::Char(entry)));
        assert_eq!(
            session.editing_state(),
            Some(EditingState::Insert),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.view().point.position, Scalar(cursor), "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), edits, "{name}");
        session.handle(key(KeyCode::Esc));
        assert_eq!(
            session.editing_state(),
            Some(EditingState::Normal),
            "{name}"
        );
        assert_eq!(session.view().point.position, Scalar(cursor), "{name}");
        if edits == 1 {
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source, "{name}");
        }
    }
}

// Modal letters and paste cannot silently enter text in Normal; inserted Unicode
// and its history still use the shared transaction owner. Emacs stays direct.
#[test]
fn text_admission() {
    let mut vim = session(VIM, "", 0);
    vim.handle(key(KeyCode::Char('z')));
    vim.handle(Event::Paste("unwanted".into()));
    assert_eq!(text(&vim), "");
    assert!(vim.buffer().unwrap().history().is_empty());
    vim.handle(key(KeyCode::Char('i')));
    vim.handle(Event::Paste("e\u{301}界".into()));
    let after = vim.view().point.position;
    vim.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('['),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(vim.editing_state(), Some(EditingState::Normal));
    assert_eq!(vim.view().point.position, after);
    assert_eq!(text(&vim), "e\u{301}界");
    vim.execute(Command::History(History::Undo));
    assert_eq!(text(&vim), "");
    let mut emacs = session("[tree]\nvisible = false", "", 0);
    emacs.handle(key(KeyCode::Char('i')));
    assert_eq!(text(&emacs), "i");
    emacs.execute(Command::Modal(ModalCommand::Normal));
    assert_eq!(emacs.interaction_profile(), InteractionProfile::Emacs);
    assert_eq!(emacs.editing_state(), None);
}

// Configuration resolves one immutable startup selection; subsequent file writes
// affect only a new preparation and cannot retarget an admitted snapshot.
#[test]
fn startup_profile() {
    let fixture = Fixture::new();
    let included = fixture.write("base.toml", "[editor]\ninteraction_profile = 'vim'");
    let root = fixture.write("config.toml", "include = ['base.toml']");
    let vim = prepare(&root);
    assert!(vim.diagnostic.is_none());
    assert_eq!(
        vim.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Vim
    );
    assert_eq!(
        vim.snapshot.origin(&["editor", "interaction_profile"]),
        Some(included.as_path())
    );
    fixture.write(
        "config.toml",
        "include = ['base.toml']\n[editor]\ninteraction_profile = 'emacs'",
    );
    let emacs = prepare(&root);
    assert!(emacs.diagnostic.is_none());
    assert_eq!(
        emacs.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Emacs
    );
    assert_eq!(
        vim.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Vim
    );
}

// An invalid profile or forbidden scoped override rejects every candidate leaf,
// including otherwise valid editor changes, through ordinary startup fallback.
#[test]
fn profile_fallback() {
    let fixture = Fixture::new();
    for (name, text, rejected) in [
        ("omitted", "[editor]\nindent_width = 2", false),
        (
            "unknown",
            "[editor]\nindent_width = 2\ninteraction_profile = 'evil'",
            true,
        ),
        (
            "language",
            "[editor]\nindent_width = 2\n[language.rust]\ninteraction_profile = 'vim'",
            true,
        ),
    ] {
        let root = fixture.write("config.toml", text);
        let startup = prepare(&root);
        assert_eq!(startup.diagnostic.is_some(), rejected, "{name}");
        assert_eq!(
            startup.snapshot.settings().editor.interaction_profile,
            InteractionProfile::Emacs,
            "{name}"
        );
        assert_eq!(
            startup.snapshot.settings().editor.indent_width,
            if rejected { 4 } else { 2 },
            "{name}"
        );
        if name == "unknown" {
            assert_eq!(
                startup.diagnostic.unwrap().key,
                ["editor", "interaction_profile"]
            );
        }
    }
}
