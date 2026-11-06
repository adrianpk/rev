use crate::native::{cancel, check_row, editor, send};
use crate::support::{Result, settle, source};
use rev::command::{Command, GitCommand, ModalCommand, Motion, TerminalCommand, TreeCommand};
use rev::coordinates::Scalar;
use std::path::Path;

// State variants compare advertised dispatch with the same named handler in an
// independently owned session, including Visual point/mark and source identity.
pub(super) fn run(root: &Path) -> Result<()> {
    let config = "editor.interaction_profile='modal'\n[keys.vim_normal]\n'M-j'='forward-char'\n[keys.vim_insert]\n'C-f'='backward-char'\n'Right'='backward-char'\n'M-k'='forward-char'\n[keys.vim_replace]\n'C-f'='backward-char'\n'Right'='backward-char'\n'M-l'='forward-char'\n[keys.vim_visual_character]\n'M-a'='forward-char'\n[keys.vim_visual_line]\n'M-b'='forward-char'\n[keys.vim_visual_block]\n'M-c'='forward-char'";
    for (name, state, hint) in [
        ("normal", ModalCommand::Normal, "M-j"),
        ("insert", ModalCommand::Insert, "M-k"),
        ("replace", ModalCommand::Replace, "M-l"),
        ("character", ModalCommand::VisualCharacter, "M-a"),
        ("line", ModalCommand::VisualLine, "M-b"),
        ("block", ModalCommand::VisualBlock, "M-c"),
    ] {
        let mut bound = editor(&root.join(name).join("bound"), config)?;
        let mut named = editor(&root.join(name).join("named"), config)?;
        for owner in [&mut bound, &mut named] {
            owner.execute(Command::Modal(state));
        }
        let before = (
            bound.editing_state(),
            bound.view().point,
            bound.buffer()?.mark(),
        );
        check_row(&mut bound, "forward-char", Some(hint))?;
        assert_eq!(
            (
                bound.editing_state(),
                bound.view().point,
                bound.buffer()?.mark()
            ),
            before,
            "{name}: cancel"
        );
        send(&mut bound, hint)?;
        named.execute(Command::Motion(Motion::ForwardChar));
        assert_eq!(bound.view().point.position, Scalar(2), "{name}");
        assert_eq!(
            bound.view().point.position,
            named.view().point.position,
            "{name}"
        );
        assert_eq!(bound.editing_state(), named.editing_state(), "{name}");
        assert_eq!(bound.buffer()?.mark(), named.buffer()?.mark(), "{name}");
        assert_eq!(bound.buffer()?.visual(), named.buffer()?.visual(), "{name}");
        assert_eq!(source(&bound), "abc\nsecond line\n");
    }
    tree(&root.join("tree"))?;
    git(&root.join("git"))?;
    for profile in ["non-modal", "modal"] {
        terminal(&root.join(profile), profile)?;
    }
    println!(
        "PASS native context dispatch: all six modal states and Visual cancellation, tree aliases/shadowing, local Git ownership, both-profile live terminal consumption, bound/named agreement, unchanged source and owned cleanup"
    );
    Ok(())
}

fn tree(root: &Path) -> Result<()> {
    let config = "[keys.editor]\n'M-j'='forward-char'\n[keys.tree]\n'C-f'='tree-collapse'\n'Right'='tree-collapse'\n'M-j'='forward-char'";
    let mut owner = editor(root, config)?;
    owner.execute(Command::Tree(TreeCommand::Focus));
    assert!(owner.tree().focused());
    check_row(&mut owner, "forward-char", None)?;
    check_row(&mut owner, "insert-tab", None)?;
    check_row(&mut owner, "tree-expand", Some("M-j"))?;
    let point = owner.view().point;
    send(&mut owner, "M-j")?;
    assert!(owner.tree().entries()[0].expanded());
    assert_eq!(owner.view().point, point);
    assert_eq!(source(&owner), "abc\nsecond line\n");
    Ok(())
}

fn git(root: &Path) -> Result<()> {
    let mut owner = editor(root, "[keys.git]\n'q'='git-refresh'\n'M-j'='git-quit'")?;
    let output = std::process::Command::new("git")
        .args(["init", "--quiet"])
        .arg(root.join("workspace"))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    owner.execute(Command::Git(GitCommand::Status));
    settle(&mut owner)?;
    assert!(
        owner.git_visible() && owner.git_observation().is_some(),
        "{}",
        owner.echo()
    );
    check_row(&mut owner, "forward-char", None)?;
    check_row(&mut owner, "git-quit", Some("M-j"))?;
    send(&mut owner, "M-j")?;
    assert!(!owner.git_visible());
    assert_eq!(source(&owner), "abc\nsecond line\n");
    Ok(())
}

fn terminal(root: &Path, profile: &str) -> Result<()> {
    std::fs::create_dir_all(root)?;
    let script = root.join("terminal.sh");
    std::fs::write(
        &script,
        "printf 'READY\\n'\nwhile IFS= read -r line; do printf '%s\\n' \"$line\"; done\n",
    )?;
    let config = format!(
        "editor.interaction_profile='{profile}'\nterminal.program='/bin/sh'\nterminal.args=[{}]\n[keys.editor]\n'C-x z'='terminal-toggle'\n'M-j'='mouse-mode'\n'C-c s'='mouse-mode'\n[keys.vim_normal]\n'C-x z'='terminal-toggle'\n'M-j'='mouse-mode'\n'C-c s'='mouse-mode'",
        serde_json::to_string(&script)?
    );
    let mut owner = editor(root, &config)?;
    owner.execute(Command::Terminal(TerminalCommand::Open));
    assert!(owner.terminal_focused(), "{}", owner.echo());
    check_row(&mut owner, "newline", None)?;
    check_row(&mut owner, "mouse-mode", None)?;
    check_row(&mut owner, "terminal-toggle", Some("C-x z"))?;
    assert!(owner.terminal_focused());
    send(&mut owner, "C-x z")?;
    assert!(!owner.terminal_focused());
    assert_eq!(source(&owner), "abc\nsecond line\n");
    cancel(&mut owner)?;
    owner.shutdown_terminals()?;
    Ok(())
}
