use super::candidates::{
    Document, RUST, complete, frames, key, labels, session, settle, text, wait,
};
use super::fixture::Fixture;
use crossterm::event::KeyCode;
use rev::command::{Command, CompletionCommand as C, History, LspCommand, Split};
use rev::session::{CompletionFallback as Fallback, CompletionSource as Source, Session};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn import() -> Value {
    json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"use demo::Thing;\n"})
}
fn resolved(f: &Fixture, extra: &Value) {
    f.write(
        "peer.resolved",
        &json!({"label":"cfg_server","additionalTextEdits":extra}).to_string(),
    );
}
fn methods(f: &Fixture, method: &str) -> Vec<Value> {
    frames(f)
        .into_iter()
        .filter(|v| v["method"] == method)
        .collect()
}

// Only acceptance of the chosen item resolves it. The primary and import are
// one publication, one version advance and one undo/redo transaction; point
// includes the import's shift. Selection and navigation do not send resolves.
#[test]
fn resolved_import() {
    let f = Fixture::new();
    resolved(&f, &json!([import()]));
    let items =
        json!([{"label":"cfg_other","data":{"index":1}},{"label":"cfg_server","data":{"index":2}}]);
    let mut s = session(
        &f,
        "resolve",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &items,
        false,
    );
    complete(&mut s);
    assert!(methods(&f, "completionItem/resolve").is_empty());
    key(&mut s, KeyCode::Down);
    assert!(methods(&f, "completionItem/resolve").is_empty());
    let original = text(&s);
    let version = s.lsp_document_version().unwrap().version;
    key(&mut s, KeyCode::Tab);
    settle(&mut s);
    assert_eq!(methods(&f, "completionItem/resolve").len(), 1);
    assert_eq!(
        methods(&f, "completionItem/resolve")[0]["params"]["data"],
        json!({"index":2})
    );
    let expected = "use demo::Thing;\n".to_owned() + &original.replace("{ cfg }", "{ cfg_server }");
    assert_eq!(text(&s), expected);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(
        s.view().point.position.0,
        expected.find("cfg_server }").unwrap() + "cfg_server".len()
    );
    wait(|| s.lsp_document_version().is_some());
    assert_eq!(s.lsp_document_version().unwrap().version, version + 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), original);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), expected);
    assert_eq!(
        std::fs::read_to_string(f.0.join("main.rs")).unwrap(),
        original
    );
    let capabilities =
        frames(&f)[0]["params"]["capabilities"]["textDocument"]["completion"].clone();
    assert_eq!(capabilities["completionItem"]["snippetSupport"], false);
    assert_eq!(
        capabilities["completionItem"]["resolveSupport"]["properties"],
        json!(["additionalTextEdits"])
    );
}

// Additional edits already in the list need no resolve when unavailable. They
// use the same atomic edit path, without resolving every item on presentation.
#[test]
fn immediate_import() {
    let f = Fixture::new();
    let items = json!([{"label":"cfg_server","additionalTextEdits":[import()]}]);
    let mut s = session(
        &f,
        "ready",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &items,
        false,
    );
    complete(&mut s);
    key(&mut s, KeyCode::Enter);
    settle(&mut s);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert!(text(&s).starts_with("use demo::Thing;\n"));
    assert!(methods(&f, "completionItem/resolve").is_empty());
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), RUST.replace('|', ""));
}

// Materialized list defaults preserve the selected item's opaque data and
// primary replace extent. UTF-8/UTF-16 additional edits use the original base,
// including non-BMP/combining text and CRLF across one atomic undo.
#[test]
fn resolved_defaults() {
    let before = "fn main() { let _ = \"😀e\u{301}\"; ";
    let source = format!("// header\r\n{before}cfg|Suffix }}\r\n");
    for (mode, start) in [
        ("resolve-utf8", before.len()),
        ("resolve", before.encode_utf16().count()),
    ] {
        let f = Fixture::new();
        let insert =
            json!({"start":{"line":1,"character":start},"end":{"line":1,"character":start+3}});
        let replace =
            json!({"start":{"line":1,"character":start},"end":{"line":1,"character":start+9}});
        let defaults = json!({"isIncomplete":false,"itemDefaults":{"data":{"shared":true},"insertTextFormat":1,"editRange":{"insert":insert,"replace":replace}},
            "items":[{"label":"cfg_server","textEditText":"cfg_server"}]});
        let end = if mode == "resolve-utf8" {
            before.len() - 2
        } else {
            before.encode_utf16().count() - 2
        };
        let extra = json!([{"range":{"start":{"line":1,"character":20},"end":{"line":1,"character":end}},"newText":"\"x\""},
            {"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":9}},"newText":"// import"}]);
        resolved(&f, &extra);
        let mut s = session(
            &f,
            mode,
            &Document {
                source: &source,
                name: "main.rs",
            },
            &defaults,
            false,
        );
        complete(&mut s);
        key(&mut s, KeyCode::Tab);
        settle(&mut s);
        let requests = methods(&f, "completionItem/resolve");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["params"]["data"], json!({"shared":true}));
        assert_eq!(requests[0]["params"]["textEdit"]["replace"], replace);
        assert_eq!(
            text(&s),
            "// import\r\nfn main() { let _ = \"x\"; cfg_server }\r\n"
        );
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source.replace('|', ""));
    }
}

// Invalid additional edits reject the complete resolved item. No valid prefix
// of its batch, primary edit, point or history can publish before rejection.
#[test]
fn rejected_resolves() {
    let overlap = json!({"range":{"start":{"line":1,"character":14},"end":{"line":1,"character":15}},"newText":"bad"});
    let reversed = json!({"range":{"start":{"line":0,"character":2},"end":{"line":0,"character":1}},"newText":"bad"});
    let outside = json!({"range":{"start":{"line":90,"character":0},"end":{"line":90,"character":0}},"newText":"bad"});
    let mut cross = import();
    cross["uri"] = json!("file:///other.rs");
    let cases = [
        json!({"label":"cfg_server","additionalTextEdits":[import(),overlap]}),
        json!({"label":"cfg_server","additionalTextEdits":[import(),import()]}),
        json!({"label":"cfg_server","additionalTextEdits":[reversed]}),
        json!({"label":"cfg_server","additionalTextEdits":[outside]}),
        json!({"label":"cfg_server","additionalTextEdits":[cross]}),
        json!({"label":"cfg_server","additionalTextEdits":[{"range":{},"newText":"bad"}]}),
        json!({"label":"cfg_server","command":{"command":"execute"}}),
        json!({"label":"cfg_server","insertTextFormat":2,"insertText":"${1:bad}"}),
        json!({"label":"cfg_server","insertText":"changed_primary"}),
        json!({"label":"cfg_server","workspaceEdit":{"changes":{"file:///other.rs":[]}}}),
        json!({"label":"cfg_server","additionalTextEdits":"malformed"}),
        json!({}),
        json!(null),
    ];
    for result in cases {
        let f = Fixture::new();
        f.write("peer.resolved", &result.to_string());
        let mut s = session(
            &f,
            "resolve",
            &Document {
                source: RUST,
                name: "main.rs",
            },
            &json!([{"label":"cfg_server"}]),
            false,
        );
        complete(&mut s);
        let point = s.view().point;
        key(&mut s, KeyCode::Tab);
        settle(&mut s);
        assert_eq!(
            s.completion_source(),
            Some(Source::Native(Fallback::NoAdmitted)),
            "{result}"
        );
        assert!(
            labels(&s)
                .iter()
                .any(|label| label.starts_with("cfg_native"))
        );
        assert_eq!(text(&s), RUST.replace('|', ""));
        assert_eq!(s.view().point, point);
        assert!(s.buffer().unwrap().history().is_empty());
        assert_eq!(methods(&f, "completionItem/resolve").len(), 1);
    }
}

// The same invalid batch is omitted at list time; truthful default admission
// cannot present a cross-document range as a functioning same-document item.
#[test]
fn rejected_initial_edits() {
    let f = Fixture::new();
    let mut cross = import();
    cross["textDocument"] = json!({"uri":"file:///other.rs"});
    let items = json!([{"label":"cfg_server","additionalTextEdits":[cross]}]);
    let mut s = session(
        &f,
        "ready",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &items,
        false,
    );
    complete(&mut s);
    assert_eq!(
        s.completion_source(),
        Some(Source::Native(Fallback::NoAdmitted))
    );
    assert!(s.buffer().unwrap().history().is_empty());
    let f = Fixture::new();
    let defaults = json!({"isIncomplete":false,"itemDefaults":{"editRange":{"start":{"line":1,"character":12},"end":{"line":1,"character":15},"uri":"file:///other.rs"}},"items":[{"label":"cfg_server"}]});
    let mut s = session(
        &f,
        "resolve",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &defaults,
        false,
    );
    complete(&mut s);
    assert_eq!(
        s.completion_source(),
        Some(Source::Native(Fallback::NoAdmitted))
    );
    assert!(methods(&f, "completionItem/resolve").is_empty());
}

// Timeout, server errors and EOF release the acceptance wait and offer current
// native fallback without publishing a primary-only fragment or respawn loop.
#[test]
fn resolve_failures() {
    for (mode, reason) in [
        ("resolve-error", Fallback::Unavailable),
        ("resolve-eof", Fallback::Unavailable),
        ("resolve-timeout", Fallback::Timeout),
    ] {
        let f = Fixture::new();
        let mut s = session(
            &f,
            mode,
            &Document {
                source: RUST,
                name: "main.rs",
            },
            &json!([{"label":"cfg_server"}]),
            false,
        );
        complete(&mut s);
        key(&mut s, KeyCode::Tab);
        settle(&mut s);
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
        assert_eq!(text(&s), RUST.replace('|', ""));
        assert!(s.buffer().unwrap().history().is_empty());
        assert_eq!(methods(&f, "completionItem/resolve").len(), 1);
        if mode == "resolve-timeout" {
            wait(|| !methods(&f, "$/cancelRequest").is_empty());
        }
    }
}

// A pending acceptance cannot block editing or resolve twice. Cancellation,
// revision/view/invocation changes and activation/connection retirement invalidate
// the result even if undo restores identical bytes before the peer replies.
#[test]
fn retired_resolves() {
    for action in [
        "cancel", "edit", "undo", "view", "exclude", "restore", "disable", "restart", "invoke",
    ] {
        let f = Fixture::new();
        resolved(&f, &json!([import()]));
        let mut s = session(
            &f,
            "resolve-late",
            &Document {
                source: RUST,
                name: "main.rs",
            },
            &json!([{"label":"cfg_server"}]),
            false,
        );
        complete(&mut s);
        let generation = s.lsp_document_version().unwrap();
        let start = Instant::now();
        key(&mut s, KeyCode::Tab);
        assert!(
            start.elapsed() < Duration::from_millis(200),
            "terminal waited for resolve"
        );
        s.execute(Command::Completion(C::Accept));
        wait(|| {
            s.expire_input(Instant::now());
            !methods(&f, "completionItem/resolve").is_empty()
        });
        assert_eq!(methods(&f, "completionItem/resolve").len(), 1);
        retire_resolve(&mut s, action);
        let expected = text(&s);
        let point = s.view().point;
        let state = s.buffer().unwrap().state();
        settle(&mut s);
        if !matches!(action, "disable" | "restart") {
            wait(|| f.0.join("peer.late").exists());
        }
        assert_eq!(text(&s), expected, "{action}");
        assert_eq!(s.view().point, point, "{action}");
        assert_eq!(s.buffer().unwrap().state(), state, "{action}");
        if action != "invoke" {
            assert!(s.completion().is_none(), "{action}");
        }
        if action == "restart" {
            wait(|| s.lsp_document_version().is_some());
            assert_ne!(
                s.lsp_document_version().unwrap().connection,
                generation.connection
            );
        }
        if action == "restore" {
            wait(|| s.lsp_document_version().is_some());
            assert_ne!(
                s.lsp_document_version().unwrap().generation,
                generation.generation
            );
        }
    }
}

fn retire_resolve(s: &mut Session, action: &str) {
    match action {
        "cancel" => key(s, KeyCode::Esc),
        "edit" => key(s, KeyCode::Char('x')),
        "undo" => {
            s.replace(0..0, " ").unwrap();
            s.execute(Command::History(History::Undo));
        }
        "view" => {
            s.execute(Command::Split(Split::Right));
        }
        "invoke" => {
            s.execute(Command::Completion(C::AtPoint));
        }
        _ => retire_connection(s, action),
    }
}

fn retire_connection(s: &mut Session, action: &str) {
    match action {
        "exclude" => {
            s.execute(Command::Lsp(LspCommand::Disconnect));
        }
        "restore" => {
            s.execute(Command::Lsp(LspCommand::Disconnect));
            s.execute(Command::Lsp(LspCommand::Connect));
        }
        "disable" => {
            s.execute(Command::Lsp(LspCommand::Mode));
        }
        "restart" => {
            s.execute(Command::Lsp(LspCommand::Restart));
        }
        _ => unreachable!(),
    }
}

// Resource boundaries admit the exact configured profile and reject one extra
// edit or byte as a whole item. The successful batch still undoes in one step.
#[test]
fn edit_limits() {
    for count in [32, 33] {
        let f = Fixture::new();
        let source = "// h\n".repeat(33) + RUST;
        let extras:Vec<_>=(0..count).map(|line|json!({"range":{"start":{"line":line,"character":0},"end":{"line":line,"character":0}},"newText":"// import\n"})).collect();
        resolved(&f, &json!(extras));
        let mut s = session(
            &f,
            "resolve",
            &Document {
                source: &source,
                name: "main.rs",
            },
            &json!([{"label":"cfg_server"}]),
            false,
        );
        complete(&mut s);
        key(&mut s, KeyCode::Tab);
        settle(&mut s);
        if count == 32 {
            let base = source.replace('|', "").replace("{ cfg }", "{ cfg_server }");
            let expected: String = base
                .split_inclusive('\n')
                .enumerate()
                .map(|(index, line)| {
                    if index < 32 {
                        format!("// import\n{line}")
                    } else {
                        line.to_owned()
                    }
                })
                .collect();
            assert_eq!(text(&s), expected);
            assert_eq!(s.buffer().unwrap().history().len(), 1);
            s.execute(Command::History(History::Undo));
            assert_eq!(text(&s), source.replace('|', ""));
        } else {
            assert_eq!(
                s.completion_source(),
                Some(Source::Native(Fallback::NoAdmitted))
            );
            assert_eq!(text(&s), source.replace('|', ""));
            assert!(s.buffer().unwrap().history().is_empty());
        }
    }
    for excess in [0, 1] {
        let f = Fixture::new();
        let mut extra = import();
        let inserted = "x".repeat(256 * 1024 - "cfg_server".len() + excess);
        extra["newText"] = json!(inserted);
        resolved(&f, &json!([extra]));
        let mut s = session(
            &f,
            "resolve",
            &Document {
                source: RUST,
                name: "main.rs",
            },
            &json!([{"label":"cfg_server"}]),
            false,
        );
        complete(&mut s);
        key(&mut s, KeyCode::Tab);
        settle(&mut s);
        if excess == 0 {
            assert_eq!(
                text(&s),
                inserted + &RUST.replace('|', "").replace("{ cfg }", "{ cfg_server }")
            );
            assert_eq!(s.buffer().unwrap().history().len(), 1);
            s.execute(Command::History(History::Undo));
        } else {
            assert_eq!(
                s.completion_source(),
                Some(Source::Native(Fallback::NoAdmitted))
            );
        }
        assert_eq!(text(&s), RUST.replace('|', ""));
        assert!(s.buffer().unwrap().history().is_empty());
    }
}

// Even a completed worker reply remains a proposal: replacing its connection
// before the terminal admits it must retire that acceptance generation.
#[test]
fn retired_connection() {
    let f = Fixture::new();
    resolved(&f, &json!([import()]));
    let mut s = session(
        &f,
        "resolve",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &json!([{"label":"cfg_server"}]),
        false,
    );
    complete(&mut s);
    let original = s.lsp_document_version().unwrap();
    key(&mut s, KeyCode::Tab);
    wait(|| f.0.join("peer.resolved-sent").exists());
    assert_eq!(text(&s), RUST.replace('|', ""));
    s.execute(Command::Lsp(LspCommand::Mode));
    s.execute(Command::Lsp(LspCommand::Mode));
    settle(&mut s);
    wait(|| s.lsp_document_version().is_some());
    let current = s.lsp_document_version().unwrap();
    assert_ne!(current.connection, original.connection);
    assert_ne!(current.generation, original.generation);
    assert!(s.completion().is_none());
    assert_eq!(text(&s), RUST.replace('|', ""));
    assert!(s.buffer().unwrap().history().is_empty());
}

// An import cannot legitimize a second edit splitting an encoding unit or CRLF.
// The whole returned batch fails while its original revision remains intact.
#[test]
fn rejected_unicode() {
    let source = "// 😀\r\nfn cfg_native() {}\r\nfn main() { cfg| }\r\n";
    for mode in ["resolve", "resolve-utf8"] {
        for column in [4, if mode == "resolve" { 6 } else { 8 }] {
            let f = Fixture::new();
            let invalid = json!({"range":{"start":{"line":0,"character":column},"end":{"line":0,"character":column}},"newText":"bad"});
            resolved(&f, &json!([import(), invalid]));
            let mut s = session(
                &f,
                mode,
                &Document {
                    source,
                    name: "main.rs",
                },
                &json!([{"label":"cfg_server"}]),
                false,
            );
            complete(&mut s);
            key(&mut s, KeyCode::Tab);
            settle(&mut s);
            assert_eq!(
                s.completion_source(),
                Some(Source::Native(Fallback::NoAdmitted))
            );
            assert_eq!(text(&s), source.replace('|', ""));
            assert!(s.buffer().unwrap().history().is_empty());
        }
    }
}

// A kernel-triggered source revision retires an existing semantic popup and
// its source evidence before any old item can resolve or edit the new image.
#[test]
fn reload_retires_completion() {
    let f = Fixture::new();
    let items = json!([{"label":"cfg_server","data":{"index":1}}]);
    let mut s = session(
        &f,
        "resolve",
        &Document {
            source: RUST,
            name: "main.rs",
        },
        &items,
        false,
    );
    complete(&mut s);
    assert!(s.completion().is_some());
    let external = "fn external_value() {}\nfn main() { external_value(); }\n";
    std::fs::write(f.0.join("main.rs"), external).unwrap();
    wait(|| {
        s.expire_input(Instant::now());
        text(&s) == external
    });
    assert!(s.completion().is_none());
    s.execute(Command::Completion(C::Accept));
    settle(&mut s);
    assert_eq!(text(&s), external);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert!(methods(&f, "completionItem/resolve").is_empty());
}
