use crate::support::Result;
use std::{fs, path::Path};

pub const SOURCE: &str = "Body (outer [inner]) tail\n";
pub const SAVED: &str = "Body (outer [inner]) tail!\n";

pub fn navigation() -> String {
    (0..1000)
        .map(|row| format!("Line {row:04} (value)\n"))
        .collect()
}

pub fn prepare(root: &Path, profile: &str) -> Result<String> {
    assert!(matches!(profile, "non-modal" | "modal"));
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        fs::create_dir_all(root.join(directory))?;
    }
    let context = if profile == "modal" {
        "vim_insert"
    } else {
        "editor"
    };
    // Omit highlight to prove the default. Disable unrelated asynchronous work
    // and automatic editing; a custom user binding proves effective hint lookup.
    let config = format!(
        "editor.interaction_profile='{profile}'\npairs.enabled=false\npairs.pulse_ms=250\nminibuffer.input_limit=64\ntree.visible=false\ngutter.visible=false\nbookmarks.markers=false\ncompletion.automatic=false\nlsp.enabled=false\nsyntax.enabled=false\npalette.foreground='#a1b2c3'\npalette.pairs.depth_colors=['#102030','#405060']\npalette.pairs.match_text='#abcdef'\npalette.pairs.match_background='#203040'\nkeys.{context}.'C-c h'='pair-highlight-mode'\n"
    );
    fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    fs::write(root.join("workspace/notes.txt"), SOURCE)?;
    fs::write(root.join("workspace/navigation.txt"), navigation())?;
    Ok(config)
}

pub fn verify(root: &Path, config: &str) -> Result<()> {
    assert_eq!(fs::read_to_string(root.join("workspace/notes.txt"))?, SAVED);
    assert_eq!(
        fs::read_to_string(root.join("workspace/navigation.txt"))?,
        navigation()
    );
    assert_eq!(
        fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    Ok(())
}
