use std::fs;
use std::time::{Duration, Instant};

use rev::command::{Command, CompletionCommand, LspCommand};
use rev::lsp::Status;
use rev::session::{CompletionSource, Session};
use serde_json::{Value, json};

use crate::fixture::Fixture;
use crate::workflows::{session, text};

fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "bounded source-switch observation expired"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn settle(s: &mut Session) {
    wait(|| {
        let now = Instant::now();
        s.expire_input(now);
        s.remaining_input(now).is_none()
    });
}

fn ready(s: &mut Session) {
    wait(|| {
        s.expire_input(Instant::now());
        matches!(s.lsp_status(), Status::Ready(_))
    });
}

fn profile(f: &Fixture, name: &str, language: &str, mode: &str) -> String {
    let args = [
        f.0.join("launch.sh").to_str().unwrap().to_owned(),
        f.0.join("peer.sh").to_str().unwrap().to_owned(),
        mode.into(),
        f.0.join(name).to_str().unwrap().to_owned(),
    ];
    format!(
        "[lsp.servers.{name}]\nlanguages = ['{language}']\ncommand = '/bin/sh'\nargs = {args:?}\nroot_markers = []\nenv = {{ REV_LITERAL = 'literal $HOME $(false)', REV_LITERAL_LOG = {:?} }}\n",
        f.0.join(format!("{name}.env")).to_str().unwrap()
    )
}

fn setup(f: &Fixture, first_mode: &str) -> (String, std::path::PathBuf) {
    f.write("peer.sh", include_str!("../lsp/candidates.sh"));
    f.write("launch.sh", "#!/bin/sh\nset -eu\nprintf '%s' \"$REV_LITERAL\" > \"$REV_LITERAL_LOG\"\nexec /bin/sh \"$@\"\n");
    for (name, label) in [
        ("a", "old_semantic"),
        ("b", "new_semantic"),
        ("stable", "stable_semantic"),
    ] {
        f.write(
            &format!("{name}.result"),
            &json!([{"label":label}]).to_string(),
        );
    }
    let configuration = format!(
        "[lsp]\nenabled = true\nshutdown_timeout_ms = 1000\nrequest_timeout_ms = 2000\n[files.associations]\n'*.ex' = 'elixir'\n{}{}{}",
        profile(f, "a", "elixir", first_mode),
        profile(f, "b", "elixir", "ready"),
        profile(f, "stable", "rust", "ready")
    );
    let file = f.write("source.ex", "candidate");
    (configuration, file)
}

fn preferred(configuration: &str, choice: &str) -> String {
    configuration.replacen(
        "[lsp]\n",
        &format!("[lsp]\npreferred = {{ elixir = {choice}, rust = 'stable' }}\n"),
        1,
    )
}

fn complete(s: &mut Session) -> Vec<String> {
    s.execute(Command::Completion(CompletionCommand::AtPoint));
    settle(s);
    s.completion()
        .map(|c| {
            c.candidates()
                .iter()
                .map(|item| item.label().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn frames(f: &Fixture, name: &str) -> Vec<Value> {
    fs::read_to_string(f.0.join(format!("{name}.frames")))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

// A language with no grammar routes semantic completion through its explicit
// source, passes environment overrides literally, and keeps another root's
// unchanged connection alive through switching and native-only selection.
#[test]
fn semantic_source_switch() {
    let f = Fixture::new();
    let (configuration, file) = setup(&f, "ready");
    let root = f.write("config.toml", &preferred(&configuration, "'b'"));
    let stable = f.write("other/source.rs", "fn native() {}\n");
    let mut s = session(&root, &stable);
    ready(&mut s);
    let stable_pid = fs::read_to_string(f.0.join("stable.pid")).unwrap();
    s.load(&file).unwrap();
    ready(&mut s);
    assert_eq!(complete(&mut s), ["new_semantic"]);
    assert_eq!(
        s.completion_source(),
        Some(CompletionSource::Semantic { incomplete: false })
    );
    assert!(!f.0.join("a.pid").exists());
    assert_eq!(
        fs::read_to_string(f.0.join("b.env")).unwrap(),
        "literal $HOME $(false)"
    );
    assert!(
        frames(&f, "b")
            .iter()
            .any(|v| v["method"] == "textDocument/didOpen"
                && v["params"]["textDocument"]["languageId"] == "elixir")
    );
    let old = s.lsp_document_version().unwrap();
    let stamp = s.buffer().unwrap().stamp();
    let point = s.view().point;
    f.write("config.toml", &preferred(&configuration, "'a'"));
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    ready(&mut s);
    assert_ne!(s.lsp_document_version().unwrap(), old);
    assert_eq!(complete(&mut s), ["old_semantic"]);
    assert_eq!(s.view().point, point);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&s), "candidate");
    f.write("config.toml", &preferred(&configuration, "false"));
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    assert_eq!(s.lsp_status(), Status::Unavailable);
    assert!(complete(&mut s).is_empty());
    assert_eq!(s.completion_source(), None);
    assert!(s.echo().contains("Native completion syntax is unavailable"));
    s.load(&stable).unwrap();
    assert!(matches!(s.lsp_status(), Status::Ready(_)));
    assert_eq!(
        fs::read_to_string(f.0.join("stable.pid")).unwrap(),
        stable_pid
    );
    assert!(
        !frames(&f, "stable")
            .iter()
            .any(|v| v["method"] == "shutdown")
    );
    let native = preferred(&configuration, "false").replace("rust = 'stable'", "rust = false");
    f.write("config.toml", &native);
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    let labels = complete(&mut s);
    assert!(labels.iter().any(|label| label == "native"), "{labels:?}");
    assert!(!labels.iter().any(|label| label == "stable_semantic"));
    assert!(matches!(
        s.completion_source(),
        Some(CompletionSource::Native(_))
    ));
}

// The old peer sends its retained response after the preference changes. Its
// retired activation cannot publish that response into the current native popup.
#[test]
fn late_response_retired() {
    let f = Fixture::new();
    let (configuration, file) = setup(&f, "hold");
    let root = f.write("config.toml", &preferred(&configuration, "'a'"));
    let mut s = session(&root, &file);
    ready(&mut s);
    s.execute(Command::Completion(CompletionCommand::AtPoint));
    wait(|| {
        frames(&f, "a")
            .iter()
            .any(|v| v["method"] == "textDocument/completion")
    });
    f.write("config.toml", &preferred(&configuration, "'b'"));
    s.execute(Command::Lsp(LspCommand::ReloadSettings));
    f.write("a.release", "release");
    wait(|| f.0.join("a.sent").exists());
    ready(&mut s);
    assert_eq!(complete(&mut s), ["new_semantic"]);
    assert_eq!(
        s.completion_source(),
        Some(CompletionSource::Semantic { incomplete: false })
    );
    assert_eq!(text(&s), "candidate");
    assert!(s.buffer().unwrap().history().is_empty());
}
