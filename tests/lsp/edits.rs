use super::candidates::{RUST, complete, frames, key, session, settle, text, wait};
use super::fixture::Fixture;
use crossterm::event::KeyCode;
use rev::command::{Command, History};
use serde_json::{Value, json};

fn import() -> Value {
    json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"use demo::Thing;\n"})
}
fn resolved(f: &Fixture, extra: Value) {
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
    resolved(&f, json!([import()]));
    let items =
        json!([{"label":"cfg_other","data":{"index":1}},{"label":"cfg_server","data":{"index":2}}]);
    let mut s = session(&f, "resolve", RUST, "main.rs", items, false);
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
    let mut s = session(&f, "ready", RUST, "main.rs", items, false);
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
        resolved(&f, extra);
        let mut s = session(&f, mode, &source, "main.rs", defaults, false);
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
