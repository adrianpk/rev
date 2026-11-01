#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;

use fixture::Fixture;
use rev::config::{InteractionProfile, prepare};

// Configuration resolves one immutable startup selection; subsequent file writes
// affect only a new preparation and cannot retarget an admitted snapshot.
#[test]
fn startup_profile() {
    let fixture = Fixture::new();
    let included = fixture.write("base.toml", "[editor]\ninteraction_profile = 'vim'");
    let root = fixture.write("config.toml", "include = ['base.toml']");
    let vim = prepare(&root);
    assert!(vim.diagnostic.is_none());
    assert_eq!(
        vim.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Vim
    );
    assert_eq!(
        vim.snapshot.origin(&["editor", "interaction_profile"]),
        Some(included.as_path())
    );
    fixture.write(
        "config.toml",
        "include = ['base.toml']\n[editor]\ninteraction_profile = 'emacs'",
    );
    let emacs = prepare(&root);
    assert!(emacs.diagnostic.is_none());
    assert_eq!(
        emacs.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Emacs
    );
    assert_eq!(
        vim.snapshot.settings().editor.interaction_profile,
        InteractionProfile::Vim
    );
}

// An invalid profile or forbidden scoped override rejects every candidate leaf,
// including otherwise valid editor changes, through ordinary startup fallback.
#[test]
fn profile_fallback() {
    let fixture = Fixture::new();
    for (name, text, rejected) in [
        ("omitted", "[editor]\nindent_width = 2", false),
        (
            "unknown",
            "[editor]\nindent_width = 2\ninteraction_profile = 'evil'",
            true,
        ),
        (
            "language",
            "[editor]\nindent_width = 2\n[language.rust]\ninteraction_profile = 'vim'",
            true,
        ),
    ] {
        let root = fixture.write("config.toml", text);
        let startup = prepare(&root);
        assert_eq!(startup.diagnostic.is_some(), rejected, "{name}");
        assert_eq!(
            startup.snapshot.settings().editor.interaction_profile,
            InteractionProfile::Emacs,
            "{name}"
        );
        assert_eq!(
            startup.snapshot.settings().editor.indent_width,
            if rejected { 4 } else { 2 },
            "{name}"
        );
        if name == "unknown" {
            assert_eq!(
                startup.diagnostic.unwrap().key,
                ["editor", "interaction_profile"]
            );
        }
    }
}
