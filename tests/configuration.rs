#![forbid(unsafe_code)]
#[path = "configuration/org_export.rs"]
mod org_export;

#[path = "configuration/associations.rs"]
mod associations;
#[path = "configuration/boundaries.rs"]
mod boundaries;
#[path = "configuration/candidate.rs"]
mod candidate;
#[path = "configuration/catalog.rs"]
mod catalog;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "configuration/gutter.rs"]
mod gutter;
#[path = "configuration/include_bounds.rs"]
mod include_bounds;
#[path = "configuration/includes.rs"]
mod includes;
#[path = "configuration/keymaps.rs"]
mod keymaps;
#[path = "configuration/merging.rs"]
mod merging;
#[path = "configuration/mouse.rs"]
mod mouse;
#[path = "configuration/parsing.rs"]
mod parsing;
#[path = "configuration/reference.rs"]
mod reference;
#[path = "configuration/reference_bounds.rs"]
mod reference_bounds;
#[path = "configuration/resolution.rs"]
mod resolution;
#[path = "configuration/resolution_contract.rs"]
mod resolution_contract;
#[path = "configuration/scopes.rs"]
mod scopes;
#[path = "configuration/startup.rs"]
mod startup;
#[path = "configuration/template.rs"]
mod template;
#[path = "configuration/xdg.rs"]
mod xdg;

use rev::config::{Document, Grammar, IndentStyle};

// Input must retain explicit false/zero/empty values, even when they coincide
// with a built-in default; the resolver must still know which source supplied it.
#[test]
fn partial_presence() {
    assert_eq!(toml::from_str::<Document>("").unwrap(), Document::default());
    let cases = [
        ("absent", "", None, None, None),
        (
            "explicit",
            "include = []\n[syntax]\nenabled = false\nmax_bytes = 0",
            Some(false),
            Some(0),
            Some(Vec::new()),
        ),
        (
            "nondefault",
            "include = ['colors.toml']\n[syntax]\nenabled = true\nmax_bytes = 42",
            Some(true),
            Some(42),
            Some(vec!["colors.toml".to_owned()]),
        ),
    ];
    for (name, source, enabled, max_bytes, include) in cases {
        let input: Document = toml::from_str(source).unwrap();
        assert_eq!(input.syntax.enabled, enabled, "{name}");
        assert_eq!(input.syntax.max_bytes, max_bytes, "{name}");
        assert_eq!(input.include, include, "{name}");
        assert_eq!(input.version, None, "{name}");
        assert_eq!(input.editor.indent_width, None, "{name}");
    }
}

// A Go width override must leave all siblings absent. Parsing a later global
// layer must not insert global defaults into the language's partial record.
#[test]
fn language_presence() {
    let cases = [
        ("empty", "[language.go]", None, None, None),
        (
            "width only",
            "[language.go]\nindent_width = 2",
            Some(2),
            None,
            None,
        ),
        (
            "style and grammar",
            "[language.go]\nindent_style = 'tabs'\ngrammar = 'plain'",
            None,
            Some(IndentStyle::Tabs),
            Some(Grammar::Plain),
        ),
    ];
    for (name, source, width, style, grammar) in cases {
        let input: Document = toml::from_str(source).unwrap();
        let language = &input.language["go"];
        assert_eq!(language.indent_width, width, "{name}");
        assert_eq!(language.indent_style, style, "{name}");
        assert_eq!(language.grammar, grammar, "{name}");
        assert_eq!(language.tab_width, None, "{name}");
        assert_eq!(input.editor.indent_style, None, "{name}");
    }
}

// Source records must retain metadata and raw mapping/color values without
// consulting disk or commands; semantic admission belongs to candidate validation.
#[test]
fn source_records() {
    let input: Document = toml::from_str(
        r##"
version = 1
include = ["base.toml", "palette.toml"]
[minibuffer]
match = "substring"
[tree]
root = "file-parent"
[files]
newline = "crlf"
bom = false
[files.associations]
"*.h" = "cpp"
[keys.editor]
"M-j" = "next-line"
[keys.minibuffer]
"C-g" = "keyboard-quit"
[keys.tree]
"RET" = "tree-open"
[palette]
background = ""
[palette.syntax]
type = "#2ac3de"
"##,
    )
    .unwrap();
    assert_eq!(input.version, Some(1));
    assert_eq!(input.include.unwrap(), ["base.toml", "palette.toml"]);
    assert_eq!(
        input.minibuffer.r#match,
        Some(rev::config::Match::Substring)
    );
    assert_eq!(input.tree.root, Some(rev::config::TreeRoot::FileParent));
    assert_eq!(input.files.bom, Some(false));
    assert_eq!(input.files.newline, Some(rev::config::Newline::Crlf));
    assert_eq!(input.files.associations["*.h"], "cpp");
    assert_eq!(input.keys.editor["M-j"], "next-line");
    assert_eq!(input.keys.minibuffer["C-g"], "keyboard-quit");
    assert_eq!(input.keys.tree["RET"], "tree-open");
    assert_eq!(input.palette.background.as_deref(), Some(""));
    assert_eq!(input.palette.syntax.r#type.as_deref(), Some("#2ac3de"));
    assert_eq!(input.palette.foreground, None);
}

#[path = "configuration/org_keymaps.rs"]
mod org_keymaps;

#[path = "configuration/org_palette.rs"]
mod org_palette;

#[path = "configuration/org_properties.rs"]
mod org_properties;

#[path = "configuration/org_estimates.rs"]
mod org_estimates;
#[path = "configuration/org_tags.rs"]
mod org_tags;
#[path = "configuration/org_tasks.rs"]
mod org_tasks;

#[path = "configuration/org_time.rs"]
mod org_time;

#[path = "configuration/org_clocks.rs"]
mod org_clocks;
#[path = "configuration/org_habits.rs"]
mod org_habits;
#[path = "configuration/org_timers.rs"]
mod org_timers;

#[path = "configuration/org_agenda.rs"]
mod org_agenda;

#[path = "configuration/org_capture.rs"]
mod org_capture;

#[path = "configuration/org_refile.rs"]
mod org_refile;

#[path = "configuration/org_archive.rs"]
mod org_archive;

#[path = "configuration/org_links.rs"]
mod org_links;

#[path = "configuration/org_publishing.rs"]
mod org_publishing;

#[path = "configuration/org_source.rs"]
mod org_source;

#[path = "configuration/org_feeds.rs"]
mod org_feeds;
