use super::workflows::{answer, key, named, session, text};
use crate::fixture::Fixture;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::config::{Roots, prepare};
use rev::lsp::{
    Status,
    install::{Registry, State},
};
use rev::session::{Action, Session};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn wait(s: &mut Session, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(12);
    while !s.echo().contains(expected) {
        assert!(
            Instant::now() < deadline,
            "expected {expected}: {}",
            s.echo()
        );
        s.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn held(directory: &Path) -> Vec<u32> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let path = directory.join("held-pids");
    loop {
        if let Ok(text) = fs::read_to_string(&path) {
            let pids: Vec<_> = text
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if pids.len() == 2 {
                return pids;
            }
        }
        assert!(
            Instant::now() < deadline,
            "complete owned helper PID record missing"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn stopped(pids: &[u32]) -> bool {
    pids.iter().all(
        |pid| match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => matches!(
                stat.rsplit_once(')').unwrap().1.split_whitespace().next(),
                Some("Z" | "X" | "x")
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => panic!("cannot observe owned fixture process {pid}: {error}"),
        },
    )
}

fn begin_update(s: &mut Session, id: &str, modal: bool) {
    named(s, "lsp-update-server", modal);
    answer(s, id);
    assert!(s.echo().contains("selected profiles:"), "{}", s.echo());
    answer(s, "yes");
    assert!(s.echo().contains("started"), "{}", s.echo());
}

// Native M-x/colon management in both profiles exercises confirmation/cancel,
// busy-slot ownership, replacement, incomplete removal and confirmed editor exit.
// Controlled acquisition peers prove process/settings contracts, not Rust semantics.
#[test]
fn native_lifecycle() {
    let Some(profile) = std::env::var_os("REV_LIFECYCLE_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            super::acquisition::fixtures(&f.0);
            let curl = f.0.join("bin/curl");
            let script = fs::read_to_string(&curl).unwrap();
            // Hold the asset transfer after the preparing record exists, rather
            // than hold metadata resolution before any candidate is allocated.
            let script=script.replace("if test -f \"$REV_RUST_FIXTURE/hold\"; then", "asset=false\nfor arg in \"$@\"; do if test \"$arg\" = --output; then asset=true; fi; done\nif test -f \"$REV_RUST_FIXTURE/hold\" && test \"$asset\" = true; then");
            fs::write(curl, script).unwrap();
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "lifecycle_native::native_lifecycle",
                    "--nocapture",
                ])
                .env("REV_LIFECYCLE_PROFILE", profile)
                .env("REV_RUST_FIXTURE", &f.0)
                .env(
                    "PATH",
                    format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
                )
                .env("XDG_CONFIG_HOME", f.0.join("config"))
                .env("XDG_DATA_HOME", f.0.join("data"))
                .env("XDG_STATE_HOME", f.0.join("state"))
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
    let f = Fixture::new();
    let roots = Roots::from_env().unwrap();
    let fixture = PathBuf::from(std::env::var_os("REV_RUST_FIXTURE").unwrap());
    let root = f.write(
        "selected.toml",
        &format!(
            "# retain root\n[editor]\ninteraction_profile='{}'\ntab_width=5\n",
            profile.to_str().unwrap()
        ),
    );
    let file = f.write("sample.rs", "fn main() {}\n");
    let mut s = session(&root, &file);
    named(&mut s, "lsp-cancel-installation", modal);
    assert!(s.echo().contains("No server management job"));
    named(&mut s, "lsp-install-server", modal);
    answer(&mut s, "rust-analyzer");
    wait(&mut s, "selected and saved");
    let old = Registry::load(&roots.data).unwrap().installations[0].clone();
    let saved = fs::read(&root).unwrap();
    let calls = fs::read(fixture.join("helper-cwd")).unwrap();
    for choose in [false, true] {
        named(&mut s, "lsp-update-server", modal);
        if choose {
            answer(&mut s, &old.id);
        }
        key(&mut s, KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert!(s.echo().contains("cancelled"));
        assert_eq!(fs::read(&root).unwrap(), saved);
        assert_eq!(fs::read(fixture.join("helper-cwd")).unwrap(), calls);
    }
    if modal {
        named(&mut s, "lsp-remove-server", modal);
        key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
        s.expire_input(Instant::now() + Duration::from_secs(1));
        assert!(s.echo().contains("cancelled"));
    }
    fs::write(fixture.join("hold"), "").unwrap();
    begin_update(&mut s, &old.id, modal);
    let pids = held(&fixture);
    assert_eq!(pids.len(), 2);
    if modal {
        key(&mut s, KeyCode::Char('i'), KeyModifiers::NONE);
    }
    key(&mut s, KeyCode::Char('x'), KeyModifiers::NONE);
    if modal {
        key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
        s.expire_input(Instant::now() + Duration::from_secs(1));
    }
    let edited = text(&s);
    let point = s.view().point;
    named(&mut s, "lsp-cancel-installation", modal);
    assert!(s.echo().contains("cancellation requested"));
    named(&mut s, "lsp-update-server", modal);
    assert!(s.echo().contains("still owned"));
    wait(&mut s, "cancelled");
    assert!(stopped(&pids));
    assert!(
        !Path::new(&format!("/proc/{}", pids[0])).exists(),
        "owned leader was not reaped"
    );
    assert_eq!(fs::read(&root).unwrap(), saved);
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    assert_eq!(s.lsp_status(), Status::Disabled);
    let registry = Registry::load(&roots.data).unwrap();
    assert_eq!(registry.installations[0], old);
    let incomplete = registry
        .installations
        .iter()
        .find(|i| i.status == State::Incomplete)
        .expect("cancelled candidate is incomplete");
    named(&mut s, "lsp-remove-server", modal);
    answer(&mut s, &incomplete.id);
    answer(&mut s, "yes");
    wait(&mut s, "Removed installation");
    assert_eq!(Registry::load(&roots.data).unwrap().installations.len(), 1);
    fs::remove_file(fixture.join("hold")).unwrap();
    fs::remove_file(fixture.join("held-pids")).unwrap();
    begin_update(&mut s, &old.id, modal);
    wait(&mut s, "updated 1 selected profiles");
    let current = Registry::load(&roots.data)
        .unwrap()
        .installations
        .into_iter()
        .find(|i| i.id != old.id)
        .unwrap();
    assert!(
        prepare(&root)
            .snapshot
            .settings()
            .lsp
            .selected("rust")
            .unwrap()
            .1
            .command
            .contains(&current.id)
    );
    named(&mut s, "lsp-remove-server", modal);
    answer(&mut s, &old.id);
    answer(&mut s, "yes");
    wait(&mut s, "Removed installation");
    assert!(!roots.data.join(&old.payload).exists());
    assert!(roots.data.join(&current.payload).is_dir());
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(prepare(&root).snapshot.settings().editor.tab_width, 5);
    let saved = fs::read(&root).unwrap();
    fs::write(fixture.join("hold"), "").unwrap();
    begin_update(&mut s, &current.id, modal);
    let pids = held(&fixture);
    key(&mut s, KeyCode::Char('x'), KeyModifiers::CONTROL);
    key(&mut s, KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(&mut s, KeyCode::Char('g'), KeyModifiers::CONTROL);
    assert!(!stopped(&pids));
    key(&mut s, KeyCode::Char('x'), KeyModifiers::CONTROL);
    key(&mut s, KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE
        ))),
        Action::Exit
    );
    let deadline = Instant::now() + Duration::from_secs(7);
    while !stopped(&pids) {
        assert!(
            Instant::now() < deadline,
            "confirmed exit did not cancel its owned helper"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(s);
    assert!(
        !Path::new(&format!("/proc/{}", pids[0])).exists(),
        "exit did not reap owned leader"
    );
    assert_eq!(fs::read(&root).unwrap(), saved);
    assert_eq!(fs::read_to_string(file).unwrap(), "fn main() {}\n");
    let registry = Registry::load(&roots.data).unwrap();
    assert_eq!(
        registry
            .installations
            .iter()
            .filter(|i| i.status == State::Ready)
            .count(),
        1
    );
    assert!(registry.installations.iter().any(|i| i == &current));
    assert!(
        registry
            .installations
            .iter()
            .any(|i| i.status == State::Incomplete)
    );
    assert!(
        fs::read_dir(roots.cache.join("lsp"))
            .unwrap()
            .next()
            .is_none()
    );
}
