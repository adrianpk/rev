use super::Fixture;
use std::path::Path;
use std::time::Instant;

// Reuse the established bounded util-linux PTY owner and measurements. These
// modules also serve broader journeys; only their terminal boundary is used here.
#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/terminal.rs"]
mod terminal;

// Actual kernel events must wake an otherwise idle native terminal without a
// keystroke in both profiles. The shared owner enforces eight seconds, 1 MiB,
// owned-child cleanup, idle CPU/output admission and terminal restoration.
#[test]
fn idle_native_wake() {
    for profile in ["non-modal", "modal"] {
        let fixture = Fixture::new();
        for path in [
            "pty/home",
            "pty/config/rev",
            "pty/data",
            "pty/state",
            "pty/cache",
            "workspace",
        ] {
            std::fs::create_dir_all(fixture.0.join(path)).unwrap();
        }
        fixture.file("pty/config/rev/config.toml", format!("editor.interaction_profile = '{profile}'\ntree.visible = false\ncompletion.automatic = false\nfiles.external_change_debounce_ms = 10\n").as_bytes());
        let source = fixture.file("workspace/terminal.rs", b"fn original() {}\n");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut terminal = terminal::Terminal::start(
            &fixture.0,
            Path::new(env!("CARGO_BIN_EXE_rev")),
            &root.join("scripts/check-completion.sh"),
        )
        .unwrap();
        terminal.frame(|frame| frame.contains("original")).unwrap();
        let started = Instant::now();
        let replacement = fixture.file("workspace/replacement", b"fn external_value() {}\n");
        std::fs::rename(replacement, &source).unwrap();
        terminal
            .frame(|frame| frame.contains("external_value"))
            .unwrap();
        println!(
            "MEASURE file notification profile={profile} reload_ms={}",
            started.elapsed().as_millis()
        );
        terminal.idle().unwrap();
        terminal.finish().unwrap();
        assert!(terminal.contains("REV_TTY_RESTORED"));
        assert_eq!(std::fs::read(&source).unwrap(), b"fn external_value() {}\n");
    }
}
