use super::fixture::Fixture;
use rev::config::{AiCommand, AiContext, AiInput, AiOutput, SettingsEdit, parse, prepare};
use std::{fs, path::Path};

fn definition(input: &str, context: &str, output: &str) -> String {
    format!(
        "prompt = 'Keep literal $(command) and @source.'\ninput = '{input}'\ncontext = '{context}'\noutput = '{output}'\n"
    )
}
fn admitted() -> AiCommand {
    AiCommand {
        prompt: "Literal instruction".into(),
        input: AiInput::Selection,
        context: AiContext::Input,
        output: AiOutput::Replace,
        model: Some("provider/model".into()),
    }
}

// Typed target combinations cannot silently widen a selection or turn a point
// into replacement. Definitions remain bounded literal records with all fields.
#[test]
fn definition_bounds() {
    for input in ["selection", "point", "buffer"] {
        for context in ["input", "directory"] {
            for output in ["explanation", "insert", "replace"] {
                let text = format!(
                    "[ai.commands.total]\n{}",
                    definition(input, context, output)
                );
                let valid = output == "explanation"
                    || (input == "point" && output == "insert")
                    || (input != "point" && output == "replace");
                assert_eq!(
                    parse(Path::new("commands.toml"), text.as_bytes()).is_ok(),
                    valid,
                    "{input}/{context}/{output}"
                );
            }
        }
    }
    for name in [
        "explain",
        "save-command",
        "prompt",
        "Bad",
        "1bad",
        "bad_name",
        "",
    ] {
        let text = format!(
            "[ai.commands.'{name}']\n{}",
            definition("selection", "input", "replace")
        );
        assert!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_err(),
            "{name}"
        );
    }
    for change in [
        "prompt = ''",
        "input = 'file'",
        "context = 'all'",
        "output = 'write'",
        "model = ''",
        "endpoint = 'elsewhere'",
    ] {
        let field = change.split('=').next().unwrap().trim();
        let text = definition("selection", "input", "replace")
            .lines()
            .filter(|line| !line.starts_with(&format!("{field} =")))
            .collect::<Vec<_>>()
            .join("\n");
        let text = format!("[ai.commands.total]\n{text}\n{change}");
        assert!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_err(),
            "{change}"
        );
    }
    for limit in [16384, 16385] {
        let text = format!(
            "[ai.commands.total]\nprompt = '{}'\ninput='point'\ncontext='input'\noutput='insert'",
            "x".repeat(limit)
        );
        assert_eq!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_ok(),
            limit == 16384
        );
    }
    for limit in [64, 65] {
        let text = (0..limit)
            .map(|n| {
                format!(
                    "[ai.commands.c{n}]\n{}",
                    definition("point", "input", "insert")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_ok(),
            limit == 64
        );
    }
    for length in [1, 64, 65] {
        let name = "a".repeat(length);
        let text = format!(
            "[ai.commands.{name}]\n{}",
            definition("point", "input", "insert")
        );
        assert_eq!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_ok(),
            length <= 64
        );
    }
    for length in [256, 257] {
        let text = format!(
            "[ai.commands.total]\n{}model='{}'",
            definition("selection", "input", "replace"),
            "m".repeat(length)
        );
        assert_eq!(
            parse(Path::new("commands.toml"), text.as_bytes()).is_ok(),
            length == 256
        );
    }
    let missing = "[ai.commands.total]\nprompt='literal'\ninput='selection'\ncontext='input'";
    assert!(parse(Path::new("commands.toml"), missing.as_bytes()).is_err());
}

// References resolve after includes and root overrides, but an undefined name,
// bad source record or protected cancellation chord cannot reach a live map.
#[test]
fn definition_graph() {
    let f = Fixture::new();
    let base=f.write("base.toml",&format!("[keys.editor]\n'M-j'='ai-total'\n[keys.vim_normal]\n'g t'='ai-total'\n[ai.commands.base]\n{}",definition("point","input","insert")));
    let root = f.write(
        "config.toml",
        &format!(
            "include=['base.toml']\n[ai.commands.total]\n{}",
            definition("selection", "directory", "replace")
        ),
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let snapshot = startup.snapshot;
    assert_eq!(snapshot.settings().ai.commands.len(), 2);
    assert_eq!(
        snapshot.origin(&["ai", "commands", "base"]),
        Some(base.as_path())
    );
    use rev::input::keymap::{Context, Sequence};
    for (map, key) in [(Context::Editor, "M-j"), (Context::VimNormal, "g t")] {
        assert_eq!(
            snapshot
                .keymaps()
                .map(map)
                .lookup(&Sequence::parse(key).unwrap())
                .unwrap()
                .name(),
            "ai-total"
        );
    }
    f.write("config.toml", "include=['base.toml']");
    assert!(
        prepare(&root)
            .diagnostic
            .unwrap()
            .message
            .contains("existing command")
    );
    f.write(
        "config.toml",
        &format!(
            "[ai.commands.total]\n{}\n[keys.editor]\n'C-g'='ai-total'",
            definition("selection", "input", "replace")
        ),
    );
    assert!(
        prepare(&root)
            .diagnostic
            .unwrap()
            .message
            .contains("reserved")
    );
    f.write(
        "base.toml",
        "[ai.commands.total]\nprompt='missing required fields'",
    );
    f.write(
        "config.toml",
        &format!(
            "include=['base.toml']\n[ai.commands.total]\n{}",
            definition("selection", "input", "replace")
        ),
    );
    assert_eq!(prepare(&root).diagnostic.unwrap().source, base);
    for context in ["minibuffer", "tree"] {
        let text = format!(
            "[ai.commands.total]\n{}\n[keys.{context}]\n'M-j'='ai-total'",
            definition("point", "input", "insert")
        );
        f.write("config.toml", &text);
        assert!(
            prepare(&root)
                .diagnostic
                .unwrap()
                .message
                .contains("unavailable")
        );
    }
    let entries = |range: std::ops::Range<usize>| {
        range
            .map(|n| {
                format!(
                    "[ai.commands.c{n}]\n{}",
                    definition("point", "input", "insert")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    f.write("base.toml", &entries(0..33));
    f.write(
        "config.toml",
        &format!("include=['base.toml']\n{}", entries(33..65)),
    );
    assert!(
        prepare(&root)
            .diagnostic
            .unwrap()
            .message
            .contains("64 definitions")
    );
}

// Root publication adds only the chosen definition. Includes, user comments,
// unrelated values and existing definitions survive a real atomic write.
#[test]
fn definition_publication() {
    let f = Fixture::new();
    let base = f.write(
        "base.toml",
        &format!(
            "[ai.commands.base]\n{}",
            definition("point", "input", "insert")
        ),
    );
    let bytes = fs::read(&base).unwrap();
    let root=f.write("config.toml","# User heading\ninclude = ['base.toml'] # retain graph\n[editor]\ntab_width = 7 # retain comment\n");
    let mut edit = SettingsEdit::capture(&root).unwrap();
    edit.ai_command("total", &admitted()).unwrap();
    let published = edit.publish(&f.0.join("state")).unwrap();
    let text = fs::read_to_string(&root).unwrap();
    assert!(text.starts_with("# User heading\ninclude = ['base.toml'] # retain graph"));
    assert!(text.contains("tab_width = 7 # retain comment"));
    assert!(!text.contains("[ai.commands.base]"));
    assert_eq!(fs::read(&base).unwrap(), bytes);
    assert_eq!(
        published.snapshot.settings().ai.commands["total"],
        admitted()
    );
    assert_eq!(published.snapshot.settings().ai.commands.len(), 2);
    assert_eq!(published.snapshot.settings().editor.tab_width, 7);
    let inline = f.write(
        "inline.toml",
        "# Inline user style\nai = { commands = {} } # keep inline\n",
    );
    let mut edit = SettingsEdit::capture(&inline).unwrap();
    edit.ai_command("total", &admitted()).unwrap();
    let published = edit.publish(&f.0.join("state")).unwrap();
    assert_eq!(
        published.snapshot.settings().ai.commands["total"],
        admitted()
    );
    assert!(
        fs::read_to_string(&inline)
            .unwrap()
            .contains("# keep inline")
    );
}

// Captured root/include changes and a competing publisher reject rather than
// overwriting a newer graph. Invalid graphs never grant edit authority.
#[test]
fn definition_conflicts() {
    for changed in ["config.toml", "base.toml"] {
        let f = Fixture::new();
        f.write("base.toml", "# initial include");
        let root = f.write("config.toml", "include=['base.toml']");
        let mut edit = SettingsEdit::capture(&root).unwrap();
        edit.ai_command("total", &admitted()).unwrap();
        f.write(changed, "# concurrent change");
        let bytes = fs::read(&root).unwrap();
        assert!(
            edit.publish(&f.0.join("state"))
                .err()
                .unwrap()
                .to_string()
                .contains("conflict")
        );
        assert_eq!(fs::read(&root).unwrap(), bytes);
    }
    let f = Fixture::new();
    let root = f.write("config.toml", "# first");
    let mut a = SettingsEdit::capture(&root).unwrap();
    let mut b = SettingsEdit::capture(&root).unwrap();
    a.ai_command("total", &admitted()).unwrap();
    b.ai_command("other", &admitted()).unwrap();
    a.publish(&f.0.join("state")).unwrap();
    let bytes = fs::read(&root).unwrap();
    assert!(b.publish(&f.0.join("state")).is_err());
    assert_eq!(fs::read(&root).unwrap(), bytes);
    f.write("config.toml", "[unknown]\nvalue=true");
    assert!(SettingsEdit::capture(&root).is_err());
}
