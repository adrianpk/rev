use std::process::Command;

use rev::config::{
    Editor, Files, History, IndentStyle, KillRing, Match, Minibuffer, Newline, Palette, Rgb, Roots,
    Scroll, Settings, Syntax, Tree, TreeRoot, Windows, prepare,
};

use crate::fixture::Fixture;

// Every global scalar is projected from the admitted catalog. Nondefault values
// detect an omitted projection or a mistaken assignment to another concern.
#[test]
fn scalar_projection() {
    let fixture = Fixture::new();
    let root = fixture.write(
        "config.toml",
        r#"
editor.indent_style = 'tabs'
editor.indent_width = 2
editor.tab_width = 8
scroll.page_overlap = 0
windows.split_ratio = 0.7
minibuffer.candidate_limit = 12
minibuffer.match = 'substring'
minibuffer.history_limit = 64
minibuffer.input_limit = 2000
history.group_limit = 2
history.payload_target_bytes = 1
kill_ring.capacity = 4
kill_ring.coalesce = false
tree.visible = true
tree.width = 31
tree.max_fraction = 0.4
tree.root = 'file-parent'
tree.show_hidden = false
tree.directories_first = false
syntax.enabled = false
syntax.max_bytes = 16
files.newline = 'crlf'
files.bom = true
"#,
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let expected = Settings {
        editor: Editor {
            indent_style: IndentStyle::Tabs,
            indent_width: 2,
            tab_width: 8,
        },
        scroll: Scroll { page_overlap: 0 },
        windows: Windows { split_ratio: 0.7 },
        minibuffer: Minibuffer {
            candidate_limit: 12,
            r#match: Match::Substring,
            history_limit: 64,
            input_limit: 2000,
        },
        history: History {
            group_limit: 2,
            payload_target_bytes: 1,
        },
        kill_ring: KillRing {
            capacity: 4,
            coalesce: false,
        },
        tree: Tree {
            visible: true,
            width: 31,
            max_fraction: 0.4,
            root: TreeRoot::FileParent,
            show_hidden: false,
            directories_first: false,
        },
        syntax: Syntax {
            enabled: false,
            max_bytes: 16,
        },
        files: Files {
            newline: Newline::Crlf,
            bom: true,
            associations: Default::default(),
        },
        ..Settings::default()
    };
    assert_eq!(startup.snapshot.settings(), &expected);
    assert_eq!(startup.snapshot.editor("unknown"), expected.editor);
}

fn colors(palette: &Palette) -> [(&str, Rgb); 25] {
    [
        ("background", palette.background),
        ("foreground", palette.foreground),
        ("cursor", palette.cursor),
        ("selection", palette.selection),
        ("selection_text", palette.selection_text),
        ("search", palette.search),
        ("search_text", palette.search_text),
        ("status", palette.status),
        ("status_text", palette.status_text),
        ("status_inactive", palette.status_inactive),
        ("border", palette.border),
        ("border_active", palette.border_active),
        ("tree", palette.tree),
        ("tree_selected", palette.tree_selected),
        ("tree_selected_text", palette.tree_selected_text),
        ("message", palette.message),
        ("warning", palette.warning),
        ("error", palette.error),
        ("syntax.keyword", palette.syntax.keyword),
        ("syntax.string", palette.syntax.string),
        ("syntax.comment", palette.syntax.comment),
        ("syntax.type", palette.syntax.r#type),
        ("syntax.function", palette.syntax.function),
        ("syntax.number", palette.syntax.number),
        ("syntax.punctuation", palette.syntax.punctuation),
    ]
}

// Unique channel values for all 25 roles detect swapped or forgotten leaves
// during both partial merging and typed projection. Root changes one role only.
#[test]
fn palette_projection() {
    let fixture = Fixture::new();
    let defaults = Palette::default();
    let mut text = String::new();
    for (index, (role, _)) in colors(&defaults).iter().enumerate() {
        text.push_str(&format!(
            "palette.{role} = '#{:02x}{:02x}{:02x}'\n",
            index + 1,
            64 + index,
            128 + index
        ));
    }
    let included = fixture.write("colors.toml", &text);
    let root = fixture.write(
        "config.toml",
        "include = ['colors.toml']\npalette.background = '#A0b1C2'",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    for (index, (role, color)) in colors(&startup.snapshot.settings().palette)
        .iter()
        .enumerate()
    {
        let expected = if index == 0 {
            Rgb(0xa0, 0xb1, 0xc2)
        } else {
            Rgb((index + 1) as u8, (64 + index) as u8, (128 + index) as u8)
        };
        assert_eq!(*color, expected, "{role}");
        let key: Vec<_> = std::iter::once("palette").chain(role.split('.')).collect();
        let origin = if index == 0 { &root } else { &included };
        assert_eq!(
            startup.snapshot.origin(&key),
            Some(origin.as_path()),
            "{role}"
        );
    }
}

// Each leaf must replace just itself, even when its built-in value is shared
// by another role. This catches accidental defaulting of omitted siblings.
#[test]
fn single_palette_leaf() {
    let fixture = Fixture::new();
    let defaults = Palette::default();
    for (role, _) in colors(&defaults) {
        let path = fixture.write("config.toml", &format!("palette.{role} = '#012345'"));
        let startup = prepare(&path);
        assert!(startup.diagnostic.is_none(), "{role}");
        let actual = colors(&startup.snapshot.settings().palette);
        let expected = colors(&defaults).map(|(name, inherited)| {
            (
                name,
                if name == role {
                    Rgb(1, 35, 69)
                } else {
                    inherited
                },
            )
        });
        assert_eq!(actual, expected, "{role}");
    }
}

// False and zero override earlier positive values. Empty source-local include
// arrays do not erase previous layers, while association and key tables merge.
#[test]
fn resolved_presence() {
    let fixture = Fixture::new();
    let included = fixture.write("first.toml", "syntax.enabled = true\nsyntax.max_bytes = 64\nfiles.bom = true\nkill_ring.coalesce = true\nlanguage.go.indent_width = 2\nlanguage.go.tab_width = 8\n[files.associations]\n'*.h' = 'c'\n'*.rs' = 'rust'\n[keys.editor]\n'M-j' = 'next-line'\n'C-x C-s' = 'save-buffer'");
    fixture.write("empty.toml", "include = []");
    let root = fixture.write("config.toml", "include = ['first.toml', 'empty.toml']\nsyntax.enabled = false\nsyntax.max_bytes = 0\nfiles.bom = false\nkill_ring.coalesce = false\neditor.tab_width = 4\nlanguage.go.indent_width = 3\n[files.associations]\n'*.h' = 'cpp'\n[keys.editor]\n'M-j' = 'previous-line'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let settings = startup.snapshot.settings();
    assert_eq!(
        (settings.syntax.enabled, settings.syntax.max_bytes),
        (false, 0)
    );
    assert!(!settings.files.bom);
    assert!(!settings.kill_ring.coalesce);
    assert_eq!(settings.files.associations["*.h"], "cpp");
    assert_eq!(settings.files.associations["*.rs"], "rust");
    assert_eq!(settings.keys.editor["M-j"], "previous-line");
    assert_eq!(settings.keys.editor["C-x C-s"], "save-buffer");
    assert_eq!(
        (
            startup.snapshot.editor("go").indent_width,
            startup.snapshot.editor("go").tab_width
        ),
        (3, 8)
    );
    assert_eq!(
        startup.snapshot.origin(&["keys", "editor", "M-j"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["keys", "editor", "C-x C-s"]),
        Some(included.as_path())
    );
}

// A child process supplies a real project working directory without changing
// the parent test process's CWD or environment. Only user-root explicit sources
// participate; a variable-looking path remains a literal local filename.
#[test]
fn explicit_user_sources() {
    if let Some(root) = std::env::var_os("REV_CONFIG_RESOLUTION_CHILD") {
        selected_user_sources(std::path::Path::new(&root));
        return;
    }
    let fixture = Fixture::new();
    let root = fixture.write(
        "user/rev/config.toml",
        "include = ['$HOME.toml']\neditor.indent_width = 6",
    );
    fixture.write("user/rev/$HOME.toml", "editor.tab_width = 8");
    fixture.write("project/config.toml", "malformed project config");
    fixture.write(
        "project/.rev/config.toml",
        "malformed hidden project config",
    );
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "resolution_contract::explicit_user_sources",
            "--nocapture",
        ])
        .current_dir(fixture.0.join("project"))
        .env("REV_CONFIG_RESOLUTION_CHILD", root)
        .env("HOME", fixture.0.join("home"))
        .env("XDG_CONFIG_HOME", fixture.0.join("user"))
        .env("XDG_DATA_HOME", fixture.0.join("data"))
        .env("XDG_STATE_HOME", fixture.0.join("state"))
        .env("XDG_CACHE_HOME", fixture.0.join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.0.join("home").exists());
    assert!(!fixture.0.join("data").exists());
    assert!(!fixture.0.join("state").exists());
    assert!(!fixture.0.join("cache").exists());
}

fn selected_user_sources(root: &std::path::Path) {
    let roots = Roots::from_env().unwrap();
    assert_eq!(roots.config_file(), root);
    let startup = prepare(&roots.config_file());
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        (
            startup.snapshot.settings().editor.indent_width,
            startup.snapshot.settings().editor.tab_width
        ),
        (6, 8)
    );
    let literal = root.parent().unwrap().join("$HOME.toml");
    assert_eq!(
        startup.snapshot.origin(&["editor", "tab_width"]),
        Some(literal.as_path())
    );
}
