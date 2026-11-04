use super::support::*;
use alacritty_terminal::term::TermMode;
use crossterm::event::{KeyCode, KeyModifiers};
use rev::{buffer::BufferId, command::TerminalCommand, term::SessionId};
use std::{fs, path::Path};

struct Owned {
    id: SessionId,
    buffer: BufferId,
    shell: u32,
    tool: u32,
}
pub(crate) fn run(root: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, profile)?;
    }
    Ok(())
}
fn journey(root: &Path, profile: &str) -> Result<()> {
    let mut editor = Editor::open(root, profile)?;
    assert_eq!(editor.session.terminals().ids().count(), 0);
    let source = editor.session.buffer()?.stamp();
    let owned = shell(&mut editor, profile)?;
    guards(&mut editor)?;
    relocate(&mut editor, profile, &owned)?;
    quit_tool(&mut editor, owned.tool)?;
    key(
        &mut editor.session,
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    );
    assert_eq!(editor.session.buffer()?.stamp(), source);
    assert_eq!(super::support::source(&editor.session), ORIGINAL);
    println!(
        "PASS native profile={profile}: interactive Bash, cwd, actual lazygit alternate screen/help and identity-preserving panel/buffer relocation"
    );
    Ok(())
}
fn shell(editor: &mut Editor, profile: &str) -> Result<Owned> {
    named(&mut editor.session, profile, "terminal-open");
    editor.wait(|s| screen(s).contains("SHELL_READY>"))?;
    let id = editor.selected();
    let buffer = editor.session.terminal_buffer().unwrap().id();
    let shell = editor
        .session
        .terminals()
        .get(id)
        .unwrap()
        .process_id()
        .unwrap();
    editor.line("printf '\\r\\nSHELL_STATE:%s\\r\\n' $$; mkdir -p kept; cd kept; pwd");
    editor.wait(|s| screen(s).contains(&format!("SHELL_STATE:{shell}")))?;
    editor.wait(|s| screen(s).contains("/kept"))?;
    let tool = start_tool(editor)?;
    Ok(Owned {
        id,
        buffer,
        shell,
        tool,
    })
}
fn start_tool(editor: &mut Editor) -> Result<u32> {
    editor
        .line("cd ..; sh -c 'printf %s $$ >tool.pid; exec lazygit --use-config-file lazygit.yml'");
    editor.wait(|s| {
        s.screen().mode().contains(TermMode::ALT_SCREEN) && screen(s).contains("Files")
    })?;
    let tool = pid_file(&editor.directory.join("tool.pid"))?;
    assert_eq!(
        fs::read_to_string(format!("/proc/{tool}/comm"))?.trim(),
        "lazygit"
    );
    key(&mut editor.session, KeyCode::Char('?'), KeyModifiers::NONE);
    editor.wait(|s| screen(s).contains("Keybindings"))?;
    Ok(tool)
}
fn relocate(editor: &mut Editor, profile: &str, owned: &Owned) -> Result<()> {
    for placement in [TerminalCommand::MoveToBuffer, TerminalCommand::MoveToPanel] {
        named(
            &mut editor.session,
            profile,
            if placement == TerminalCommand::MoveToBuffer {
                "terminal-move-to-buffer"
            } else {
                "terminal-move-to-panel"
            },
        );
        assert_eq!(editor.selected(), owned.id);
        assert_eq!(editor.session.terminal_buffer().unwrap().id(), owned.buffer);
        assert_eq!(
            editor
                .session
                .terminals()
                .get(owned.id)
                .unwrap()
                .process_id(),
            Some(owned.shell)
        );
        assert_eq!(pid_file(&editor.directory.join("tool.pid"))?, owned.tool);
        editor.wait(|s| screen(s).contains("Keybindings"))?;
    }
    Ok(())
}
fn quit_tool(editor: &mut Editor, tool: u32) -> Result<()> {
    key(&mut editor.session, KeyCode::Esc, KeyModifiers::NONE);
    editor.wait(|s| screen(s).contains("Files") && !screen(s).contains("Keybindings"))?;
    key(&mut editor.session, KeyCode::Char('q'), KeyModifiers::NONE);
    editor.wait(|s| {
        !s.screen().mode().contains(TermMode::ALT_SCREEN) && screen(s).contains("SHELL_READY>")
    })?;
    reaped(tool)?;
    Ok(())
}

fn guards(editor: &mut Editor) -> Result<()> {
    let stamp = editor.session.buffer()?.stamp();
    for name in [
        "save-buffer",
        "write-file",
        "revert-buffer",
        "undo",
        "undo-redo",
        "org-mode",
        "completion-at-point",
        "bookmark-set",
        "lsp-connect",
        "org-insert-heading",
        "ai-prompt",
        "shell-command-on-region",
    ] {
        editor
            .session
            .execute(rev::command::Command::named(name).expect("catalog command"));
        assert!(
            editor.session.echo().contains("requires a source buffer"),
            "{name}: {}",
            editor.session.echo()
        );
        assert_eq!(editor.session.buffer()?.stamp(), stamp);
        assert!(editor.session.terminal_focused());
    }
    Ok(())
}
