use super::*;
#[path = "../support/language_assets.rs"]
mod assets;

// The actual loader admits an unbundled Python parser, not a renamed bundled
// parser. Query declarations, literal suppression, selection and one-step undo
// all travel through the ordinary native completion worker.
#[test]
fn python_workflow() {
    let libraries = assets::libraries();
    let owner = fixture::Fixture::new();
    let highlights = owner.write(
        "highlights.scm",
        "(identifier) @function\n(comment) @comment\n(string) @string",
    );
    let completions = owner.write("completions.scm", "(function_definition name: (identifier) @function)\n(assignment left: (identifier) @binding)\n(block) @scope\n(comment) @comment\n(string) @string");
    let config = format!(
        "[language.python.external]\n{}",
        assets::registration(
            libraries.parser("python"),
            "tree_sitter_python",
            &highlights,
            &completions
        )
    );
    let (_fixture, mut editor) = named_source(
        "sample.py",
        "def configuration():\n    pass\nconf|\n",
        &config,
    );
    assets::settle(&mut editor);
    assert_eq!(editor.buffer().unwrap().mode(), "External");
    assert_eq!(
        editor.buffer().unwrap().external_grammar_status(),
        Some(Ok(()))
    );
    let original = text(&editor);
    complete(&mut editor);
    assert_eq!(
        editor.completion().unwrap().candidates()[0].text(),
        "configuration"
    );
    assert_eq!(text(&editor), original);
    editor.execute(Command::Completion(C::Accept));
    assert!(text(&editor).ends_with("configuration\n"));
    editor.execute(Command::History(History::Undo));
    assert_eq!(text(&editor), original);
    for document in [
        "def configuration():\n    pass\n# con|\n",
        "def configuration():\n    pass\nvalue = 'con|'\n",
    ] {
        let (_fixture, mut editor) = named_source("sample.py", document, &config);
        assets::settle(&mut editor);
        key(&mut editor, KeyCode::Char('f'), KeyModifiers::NONE);
        assets::settle(&mut editor);
        assert!(editor.completion().is_none());
    }
}

// A failed registration leaves the original declaration source usable, and a
// new grammar generation invalidates candidates captured before admission.
#[test]
fn rust_fallback_completion() {
    let owner = fixture::Fixture::new();
    let highlights = owner.write("highlights.scm", "(identifier) @function");
    let completions = owner.write(
        "completions.scm",
        "(function_item name: (identifier) @function)",
    );
    let config = format!(
        "[language.rust.external]\n{}",
        assets::registration(
            &owner.0.join("missing.so"),
            "tree_sitter_rust",
            &highlights,
            &completions
        )
    );
    let (_fixture, mut editor) = named_source("sample.rs", SOURCE, &config);
    assets::settle(&mut editor);
    assert_eq!(editor.buffer().unwrap().mode(), "Rust");
    complete(&mut editor);
    assert_eq!(
        editor.completion().unwrap().candidates()[0].text(),
        "cfg_value"
    );
    assert!(
        editor
            .buffer()
            .unwrap()
            .external_grammar_status()
            .unwrap()
            .is_err()
    );
}
