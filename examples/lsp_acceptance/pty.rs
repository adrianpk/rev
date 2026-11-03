use super::support;
use crate::{
    native_pty::{Terminal, row},
    support::Result,
};
use serde_json::json;
use std::path::Path;

pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let peer = root.join("pty-peer");
    let original = configure(root, &peer)?;
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.rs")?;
    assert!(
        !peer.join("peer.pid").exists(),
        "disabled editor launched a server"
    );
    semantic(&mut terminal, &peer)?;
    import_undo(&mut terminal)?;
    exclusion(&mut terminal, &peer)?;
    finish(&mut terminal, &peer)?;
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/terminal.rs"))?,
        original
    );
    println!(
        "PASS Linux LSP PTY: disabled startup, enable, semantic popup, resolved import/one undo, exclusion/native, restore/semantic, empty/native, disable/reap, source unchanged and TTY restored"
    );
    Ok(())
}
fn configure(root: &Path, peer: &Path) -> Result<&'static str> {
    let config = support::peer(peer)?;
    std::fs::write(root.join("pty/config/rev/config.toml"), config)?;
    std::fs::write(
        peer.join("peer.result"),
        json!([{"label":"cfg_server"}]).to_string(),
    )?;
    std::fs::write(peer.join("peer.resolved"),json!({"label":"cfg_server","additionalTextEdits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"use demo::Thing;\n"}]}).to_string())?;
    let original = "fn cfg_native() {}\nfn main() { cfg }\n";
    std::fs::write(root.join("workspace/terminal.rs"), original)?;
    Ok(original)
}
fn finish(terminal: &mut Terminal, peer: &Path) -> Result<()> {
    terminal.send(b"\x07\x1bxlsp-mode\r")?;
    terminal.frame(|frame| frame.contains("LSP: disabled"))?;
    support::reaped(&support::pid(peer)?)?;
    terminal.idle()?;
    terminal.finish()?;
    Ok(())
}
fn semantic(terminal: &mut Terminal, peer: &Path) -> Result<()> {
    terminal.send(b"\x1bxgoto-line\r2\r\x05\x1b[D\x1b[D\x1bxlsp-mode\r")?;
    support::wait(|| support::count(peer, "textDocument/didOpen") == 1)?;
    terminal.send(b"\x1bxcompletion-at-point\r")?;
    terminal.frame(semantic_popup)?;
    Ok(())
}
fn import_undo(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\t")?;
    terminal.frame(|frame| {
        !frame.contains('┌')
            && row(frame, 1).contains("use demo::Thing;")
            && row(frame, 3).contains("cfg_server")
    })?;
    terminal.send(b"\x18u")?;
    terminal
        .frame(|frame| row(frame, 2).contains("{ cfg }") && frame.contains("terminal.rs -- RW"))?;
    Ok(())
}
fn exclusion(terminal: &mut Terminal, peer: &Path) -> Result<()> {
    terminal.send(b"\x1bxlsp-disconnect\r\x1bxcompletion-at-point\r")?;
    terminal.frame(native_popup)?;
    terminal.send(b"\x07\x1bxlsp-connect\r")?;
    support::wait(|| support::count(peer, "textDocument/didOpen") == 2)?;
    terminal.send(b"\x1bxcompletion-at-point\r")?;
    terminal.frame(semantic_popup)?;
    std::fs::write(peer.join("peer.result"), "[]")?;
    terminal.send(b"\x07\x1bxcompletion-at-point\r")?;
    terminal.frame(native_popup)?;
    Ok(())
}

fn native_popup(frame: &str) -> bool {
    frame.contains('┌') && frame.contains("cfg_native") && !frame.contains("cfg_server")
}
fn semantic_popup(frame: &str) -> bool {
    frame.contains('┌') && frame.contains("cfg_server")
}
