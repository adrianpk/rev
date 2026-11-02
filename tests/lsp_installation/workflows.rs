use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, LspCommand};
use rev::config::{InteractionProfile, prepare};
use rev::lsp::Status;
use rev::session::Session;
use rev::view::{Size, View};

use crate::fixture::Fixture;

pub(super) fn session(root: &Path, file: &Path) -> Session {
    let startup = prepare(root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    session.load(file).unwrap();
    session
}

pub(super) fn key(session: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    session.handle(Event::Key(KeyEvent::new(code, modifiers)));
}

fn answer(session: &mut Session, text: &str) {
    for c in text.chars() {
        key(session, KeyCode::Char(c), KeyModifiers::NONE);
    }
    key(session, KeyCode::Enter, KeyModifiers::NONE);
}

fn named(session: &mut Session, command: &str, modal: bool) {
    if modal {
        key(session, KeyCode::Char(':'), KeyModifiers::NONE);
    } else {
        key(session, KeyCode::Char('x'), KeyModifiers::ALT);
    }
    answer(session, command);
}

pub(super) fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Each profile uses its actual command entry and native prompts. The subprocess
// gives settings publication owned XDG roots without changing the test process's
// environment or touching the user's configuration/state.
#[test]
fn guided_registration() {
    let Some(profile) = std::env::var_os("REV_LSP_GUIDED_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "workflows::guided_registration", "--nocapture"])
                .env("REV_LSP_GUIDED_PROFILE", profile)
                .env("REV_LSP_GUIDED_ROOT", &f.0)
                .env("XDG_STATE_HOME", f.0.join("state"))
                .env("XDG_DATA_HOME", f.0.join("data"))
                .env("XDG_CONFIG_HOME", f.0.join("config"))
                .env("XDG_CACHE_HOME", f.0.join("cache"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{profile}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    };
    let modal = profile == "modal";
    let directory = std::path::PathBuf::from(std::env::var_os("REV_LSP_GUIDED_ROOT").unwrap());
    let root = directory.join("selected.toml");
    let initial = format!(
        "# retain root comment\n[editor]\ninteraction_profile = '{}'\ntab_width = 5\n",
        profile.to_str().unwrap()
    );
    std::fs::write(&root, &initial).unwrap();
    let file = directory.join("sample.ex");
    std::fs::write(&file, "unchanged source\n").unwrap();
    let mut s = session(&root, &file);
    let stamp = s.buffer().unwrap().stamp();
    let point = s.view().point;
    let answers = [
        "elixir",
        "elixir-custom",
        "/bin/sh",
        "[\"-s\"]",
        "[\"mix.exs\"]",
        "[\"*.ex\", \"*.exs\"]",
    ];
    for cancelled_at in 0..answers.len() {
        named(&mut s, "lsp-setup", modal);
        for answer_text in &answers[..cancelled_at] {
            answer(&mut s, answer_text);
        }
        if modal {
            key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
            s.expire_input(Instant::now() + Duration::from_secs(1));
        } else {
            key(&mut s, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        assert_eq!(
            std::fs::read_to_string(&root).unwrap(),
            initial,
            "cancel stage {cancelled_at}"
        );
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
    }
    named(&mut s, "lsp-setup", modal);
    for argument in answers {
        answer(&mut s, argument);
    }
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
    let admitted = prepare(&root);
    assert!(admitted.diagnostic.is_none(), "{:?}", admitted.diagnostic);
    assert_eq!(
        admitted
            .snapshot
            .settings()
            .lsp
            .selected("elixir")
            .unwrap()
            .0,
        "elixir-custom"
    );
    assert_eq!(s.buffer().unwrap().language(&admitted.snapshot), "elixir");
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(
        s.interaction_profile(),
        if modal {
            InteractionProfile::Modal
        } else {
            InteractionProfile::NonModal
        }
    );
    assert_eq!(text(&s), "unchanged source\n");
    assert_eq!(s.view().point, point);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(s.buffer().unwrap().history().is_empty());
    // An existing compatible choice reuses the saved associations and ends
    // without asking for executable, arguments or patterns again.
    named(&mut s, "lsp-setup", modal);
    answer(&mut s, "elixir");
    answer(&mut s, "elixir-custom");
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
}

// Explicit reload cannot change the running profile, editing settings, global
// LSP toggle or exclusions. Invalid on-disk settings leave that state intact.
#[test]
fn scoped_reload() {
    let f = Fixture::new();
    let root = f.write(
        "config.toml",
        "[editor]\ntab_width = 5\n[lsp.servers.rust]\ncommand = '/missing/server'\n",
    );
    let file = f.write("source.rs", "fn main() {}\n");
    let mut s = session(&root, &file);
    s.insert('x');
    s.execute(Command::Region(rev::command::RegionCommand::Set));
    let point = s.view().point;
    let stamp = s.buffer().unwrap().stamp();
    let mark = s.buffer().unwrap().mark();
    let history = s.buffer().unwrap().history().len();
    f.write("config.toml", "[editor]\ninteraction_profile = 'modal'\ntab_width = 9\n[lsp]\nenabled = true\n[lsp.servers.rust]\ncommand = '/missing/server'\n");
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(s.interaction_profile(), InteractionProfile::NonModal);
    assert_eq!(s.buffer().unwrap().policy().tab_width, 5);
    s.execute(Command::Lsp(LspCommand::Mode));
    s.execute(Command::Lsp(LspCommand::Disconnect));
    assert_eq!(s.lsp_status(), Status::Excluded);
    f.write(
        "config.toml",
        "lsp.enabled = false\nlsp.preferred.rust = false\n",
    );
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    assert_eq!(s.lsp_status(), Status::Excluded);
    s.execute(Command::Lsp(LspCommand::Connect));
    assert_eq!(s.lsp_status(), Status::Unavailable);
    s.execute(Command::Lsp(LspCommand::Disconnect));
    f.write("config.toml", "lsp.preferred.rust = true\n");
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    assert!(s.echo().contains("native-only must be false"));
    assert_eq!(s.lsp_status(), Status::Excluded);
    assert_eq!(s.view().point, point);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&s), "xfn main() {}\n");
    assert_eq!(s.buffer().unwrap().mark(), mark);
    assert!(s.buffer().unwrap().mark_active());
    assert_eq!(s.buffer().unwrap().history().len(), history);
    s.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&s), "fn main() {}\n");
}
