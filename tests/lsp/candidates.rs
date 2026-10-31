use super::fixture::Fixture;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, CompletionCommand as C, History, LspCommand, Split};
use rev::coordinates::Scalar;
use rev::display::Size;
use rev::session::{CompletionFallback as Fallback, CompletionSource as Source, Session};
use rev::view::View;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const RUST: &str = "fn cfg_native() {}\nfn main() { cfg| }\n";
fn session(
    fixture: &Fixture,
    mode: &str,
    source: &str,
    name: &str,
    result: Value,
    automatic: bool,
) -> Session {
    fixture.write("peer.result", &result.to_string());
    let script = fixture.write("candidates.sh", include_str!("candidates.sh"));
    let args = [
        script.to_str().unwrap(),
        mode,
        fixture.0.join("peer").to_str().unwrap(),
    ]
    .map(str::to_owned);
    let config = format!(
        "[completion]\nautomatic = {automatic}\ndelay_ms = 1\n[lsp]\nenabled = true\nrequest_timeout_ms = 150\nshutdown_timeout_ms = 200\n[lsp.servers.rust]\ncommand = '/bin/sh'\nargs = {args:?}\nlanguages = ['rust', 'org', 'unbundled']\n[files.associations]\n'*.custom' = 'unbundled'\n"
    );
    let path = fixture.write(name, &source.replace('|', ""));
    let settings = rev::config::prepare(&fixture.write("config.toml", &config));
    assert!(settings.diagnostic.is_none(), "{:?}", settings.diagnostic);
    let mut buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(source[..source.find('|').unwrap()].chars().count());
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
fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
fn frames(f: &Fixture) -> Vec<Value> {
    std::fs::read_to_string(f.0.join("peer.frames"))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}
fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "bounded completion wait expired");
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
fn complete(s: &mut Session) {
    s.execute(Command::Completion(C::AtPoint));
    settle(s);
}
fn key(s: &mut Session, code: KeyCode) {
    s.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}
fn labels(s: &Session) -> Vec<String> {
    s.completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|c| c.label().to_owned())
        .collect()
}

// All admitted primary forms use the shared popup and canonical history; merely
// selecting never writes. Defaults apply before per-item overrides and snippets
// and command-dependent items cannot become literal placeholder operations.
#[test]
fn primary_forms() {
    let range = json!({"start":{"line":1,"character":12},"end":{"line":1,"character":15}});
    let forms = [
        json!([{"label":"cfg server","insertText":"cfg_server"}]),
        json!([{"label":"cfg server","textEdit":{"range":range,"newText":"cfg_server"}}]),
        json!([{"label":"cfg server","textEdit":{"insert":range,"replace":range,"newText":"cfg_server"}}]),
        json!({"isIncomplete":false,"itemDefaults":{"editRange":range,"insertTextFormat":1,"insertTextMode":1},"items":[{"label":"cfg server","textEditText":"cfg_server"}]}),
    ];
    for result in forms {
        let f = Fixture::new();
        let mut s = session(&f, "ready", RUST, "main.rs", result, false);
        let original = text(&s);
        let point = s.view().point;
        complete(&mut s);
        assert_eq!(
            s.completion_source(),
            Some(Source::Semantic { incomplete: false })
        );
        assert_eq!(labels(&s), ["cfg server"]);
        assert_eq!(text(&s), original);
        assert_eq!(s.view().point, point);
        assert!(s.buffer().unwrap().history().is_empty());
        key(&mut s, KeyCode::Tab);
        assert_eq!(text(&s), original.replace("{ cfg }", "{ cfg_server }"));
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), original);
        s.execute(Command::History(History::Redo));
        assert!(text(&s).contains("cfg_server }"));
        assert_eq!(
            std::fs::read_to_string(f.0.join("main.rs")).unwrap(),
            original
        );
        drop(s);
        let pid = std::fs::read_to_string(f.0.join("peer.pid")).unwrap();
        assert!(!std::path::Path::new(&format!("/proc/{}", pid.trim())).exists());
    }
}

// Prefix classes win before server sortText. Equal/absent sort keys preserve
// the server's order. filterText is distinct from display label and insertion.
#[test]
fn shared_filter() {
    let f = Fixture::new();
    let result = json!([
        {"label":"subsequence", "filterText":"c_f_g", "insertText":"sub", "sortText":"0"},
        {"label":"cfg_z", "sortText":"2"}, {"label":"cfg_b", "sortText":"1"},
        {"label":"cfg_a", "sortText":"1", "preselect":true},
        {"label":"cfg snippet", "insertTextFormat":2,"insertText":"${1:x}"},
        {"label":"cfg command", "command":{"command":"run"}},
        {"label":"cfg import", "additionalTextEdits":[{"range":{},"newText":"import"}]}
    ]);
    let mut s = session(&f, "ready", RUST, "main.rs", result, false);
    complete(&mut s);
    assert_eq!(labels(&s), ["cfg_b", "cfg_a", "cfg_z", "subsequence"]);
    assert_eq!(s.completion().unwrap().selected(), Some(0));
    key(&mut s, KeyCode::Esc);
    assert!(s.buffer().unwrap().history().is_empty());
}

// Distinct semantic outcomes all select current native syntax, including Org;
// no failure overwrites the buffer or presents unsupported placeholder edits.
#[test]
fn native_fallbacks() {
    let cases = [
        ("ready", json!([]), Fallback::Empty),
        (
            "ready",
            json!([{"label":"cfg_snip","insertTextFormat":2}]),
            Fallback::NoAdmitted,
        ),
        ("unsupported", json!([]), Fallback::Unsupported),
        ("timeout", json!([]), Fallback::Timeout),
        ("crash", json!([]), Fallback::Unavailable),
        ("ready", json!({"malformed":true}), Fallback::Unavailable),
    ];
    for (mode, result, reason) in cases {
        let f = Fixture::new();
        let mut s = session(&f, mode, RUST, "main.rs", result, false);
        complete(&mut s);
        assert_eq!(
            s.completion_source(),
            Some(Source::Native(reason)),
            "{mode}: {}",
            s.echo()
        );
        assert!(
            labels(&s)
                .iter()
                .any(|label| label.starts_with("cfg_native"))
        );
        if mode == "timeout" {
            wait(|| frames(&f).iter().any(|v| v["method"] == "$/cancelRequest"));
        }
    }
    let f = Fixture::new();
    let mut s = session(&f, "ready", "* |\n", "note.org", json!([]), false);
    complete(&mut s);
    assert_eq!(s.completion_source(), Some(Source::Native(Fallback::Empty)));
    assert!(labels(&s).iter().any(|label| label.starts_with("TODO")));
}

// Semantic availability does not depend on a bundled/native grammar. Unsaved
// state is synchronized before its request, with no project scan or implicit save.
#[test]
fn unbundled_source() {
    let f = Fixture::new();
    let mut s = session(
        &f,
        "ready",
        "cfg|",
        "main.custom",
        json!([{"label":"cfg_server", "filterText":"cfg_unsaved"}]),
        false,
    );
    s.replace(0..3, "cfg_unsaved").unwrap();
    complete(&mut s);
    assert!(matches!(
        s.completion_source(),
        Some(Source::Semantic { .. })
    ));
    let frames = frames(&f);
    let open = frames
        .iter()
        .position(|v| v["method"] == "textDocument/didOpen")
        .unwrap();
    let request = frames
        .iter()
        .position(|v| v["method"] == "textDocument/completion")
        .unwrap();
    assert!(open < request);
    assert_eq!(frames[open]["params"]["textDocument"]["text"], "cfg");
    let change = frames
        .iter()
        .position(|v| v["method"] == "textDocument/didChange")
        .unwrap();
    assert!(open < change && change < request);
    assert_eq!(
        frames[change]["params"]["contentChanges"],
        json!([{
            "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},
            "text":"cfg_unsaved"
        }])
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("main.custom")).unwrap(),
        "cfg"
    );
}

// A retired invocation cannot revive the popup, even if undo restores identical
// bytes or the selected view/exclusion/connection changes before its late reply.
#[test]
fn late_replies() {
    for action in ["cancel", "edit", "view", "exclude", "disable"] {
        let f = Fixture::new();
        let mut s = session(
            &f,
            "late",
            RUST,
            "main.rs",
            json!([{"label":"cfg_server"}]),
            false,
        );
        s.execute(Command::Completion(C::AtPoint));
        wait(|| {
            s.expire_input(Instant::now());
            frames(&f)
                .iter()
                .any(|v| v["method"] == "textDocument/completion")
        });
        match action {
            "cancel" => {
                s.execute(Command::Completion(C::Cancel));
            }
            "edit" => {
                s.replace(0..0, " ").unwrap();
                s.execute(Command::History(History::Undo));
            }
            "view" => {
                s.execute(Command::Split(Split::Right));
            }
            "exclude" => {
                s.execute(Command::Lsp(LspCommand::Disconnect));
            }
            "disable" => {
                s.execute(Command::Lsp(LspCommand::Mode));
            }
            _ => unreachable!(),
        }
        settle(&mut s);
        assert!(s.completion().is_none(), "{action}");
        if action != "disable" {
            wait(|| f.0.join("peer.late").exists());
        }
        assert_eq!(text(&s), RUST.replace('|', ""));
    }
}
