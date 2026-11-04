use super::*;

// Actual pending bytes, rather than assumed shell editing, grant replacement.
// Size rejection preserves the preceding paste; accepted later input revokes it.
#[test]
fn queued_paste() {
    let script = r"stty raw -echo; printf '\033[?2004hREADY'; dd bs=1 count=16 of=input.bin 2>/dev/null; printf RECEIVED; while :; do sleep 60; done";
    let mut terminal = Terminal::new(
        script,
        Limits {
            max_pending_input_bytes: 4096,
            ..Limits::default()
        },
    );
    terminal.wait(|s| text(s).contains("READY"));
    assert!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .screen()
            .mode()
            .contains(alacritty_terminal::term::TermMode::BRACKETED_PASTE)
    );
    let id = terminal.id;
    terminal.sessions.queue(id, b"P").unwrap();
    let paste = terminal
        .sessions
        .paste(id, &"x".repeat(4083), None)
        .unwrap();
    assert_eq!(
        terminal
            .sessions
            .get(id)
            .unwrap()
            .storage()
            .pending_input_bytes,
        4096
    );
    assert!(
        terminal
            .sessions
            .paste(id, &"x".repeat(4084), Some(paste))
            .is_err()
    );
    assert!(terminal.sessions.paste(id, "\x1bbad", Some(paste)).is_err());
    assert_eq!(
        terminal
            .sessions
            .get(id)
            .unwrap()
            .storage()
            .pending_input_bytes,
        4096
    );
    let paste = terminal.sessions.paste(id, "ok", Some(paste)).unwrap();
    terminal.sessions.queue(id, b"!").unwrap();
    assert!(terminal.sessions.paste(id, "bad", Some(paste)).is_err());
    terminal.wait(|s| text(s).contains("RECEIVED"));
    assert_eq!(
        std::fs::read(terminal._fixture.0.join("input.bin")).unwrap(),
        b"P\x1b[200~ok\x1b[201~!"
    );
    assert!(terminal.sessions.paste(id, "late", Some(paste)).is_err());
    terminal.close();
}
