use super::*;
use alacritty_terminal::{
    index::{Column, Line, Point},
    term::{TermMode, cell::Flags},
    vte::ansi::Color,
};
use rev::term::State;

// A real shell reads input from its controlling PTY; screen state is parsed,
// rather than an output log. Resize is observed by the child before it exits.
#[test]
fn shell_screen() {
    let script = r"stty -echo; printf '\033[2J\033[H\033[31;1mREADY\033[0m'; read -r action; printf '\033[?1049h\033[2J\033[HALTERNATE'; read -r action; printf '\033[?1049l'; stty size; exit 7";
    let start = Instant::now();
    let mut terminal = Terminal::new(script, Limits::default());
    terminal.wait(|session| text(session).contains("READY"));
    let startup = start.elapsed();
    let pid = terminal
        .sessions
        .get(terminal.id)
        .unwrap()
        .process_id()
        .unwrap();
    let cell = &terminal.sessions.get(terminal.id).unwrap().screen().grid()
        [Point::new(Line(0), Column(0))];
    assert_eq!(cell.c, 'R');
    assert_eq!(
        cell.fg,
        Color::Named(alacritty_terminal::vte::ansi::NamedColor::Red)
    );
    assert!(cell.flags.contains(Flags::BOLD));
    assert_eq!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .screen()
            .grid()
            .cursor
            .point
            .column
            .0,
        5
    );
    terminal.send(b"alternate\n");
    terminal.wait(|session| text(session).contains("ALTERNATE"));
    assert!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .screen()
            .mode()
            .contains(TermMode::ALT_SCREEN)
    );
    terminal
        .sessions
        .resize(
            terminal.id,
            Geometry {
                columns: 100,
                rows: 30,
            },
        )
        .unwrap();
    assert_eq!(
        terminal.sessions.get(terminal.id).unwrap().process_id(),
        Some(pid)
    );
    terminal.send(b"finish\n");
    terminal.wait(|session| matches!(session.state(), State::Exited(_)));
    let session = terminal.sessions.get(terminal.id).unwrap();
    assert!(!session.screen().mode().contains(TermMode::ALT_SCREEN));
    assert!(terminal.text().contains("READY"));
    assert!(terminal.text().contains("30 100"), "{}", terminal.text());
    assert!(matches!(session.state(), State::Exited(status) if status.code() == Some(7)));
    assert_eq!(session.process_id(), None);
    assert!(terminal.sessions.queue(terminal.id, b"late").is_err());
    println!(
        "native shell startup_ms={} cell_bytes={} storage={:?}",
        startup.as_millis(),
        size_of::<alacritty_terminal::term::cell::Cell>(),
        terminal.sessions.get(terminal.id).unwrap().storage()
    );
}

// UTF-8 and escape sequences can cross individual read boundaries. Oversized
// title requests are ignored whole, while ordinary output following them survives.
#[test]
fn bounded_child_effects() {
    let script = r"stty -echo; printf '\033]2;safe title\007\033[2J\033[H界e'; i=0; while [ $i -lt 100 ]; do printf '\314\201'; i=$((i+1)); done; printf '\033]52;c;Y2hpbGQ=\007'; printf '\033]2;'; i=0; while [ $i -lt 5000 ]; do printf x; i=$((i+1)); done; printf '\007\r\nAFTER'; read -r action";
    let mut terminal = Terminal::new(script, Limits::default());
    terminal.wait(|session| text(session).contains("AFTER"));
    let session = terminal.sessions.get(terminal.id).unwrap();
    assert_eq!(session.title().as_deref(), Some("safe title"));
    let cell = &session.screen().grid()[Point::new(Line(0), Column(2))];
    assert_eq!(cell.c, 'e');
    assert_eq!(
        cell.zerowidth().unwrap().len(),
        rev::term::COMBINING_SCALARS
    );
    assert!(session.storage().extra_bytes > 0);
    assert!(session.storage().extra_bytes <= rev::term::EXTRA_BYTES);
    assert!(session.diagnostic().unwrap().contains("Oversized"));
    terminal.sessions.close(terminal.id).unwrap();
}

// Protocol replies use the same bounded input owner and cannot invoke host
// effects. Incomplete controls consume at most the admitted string storage.
#[test]
fn protocol_reply() {
    let script = r"stty raw -echo; printf '\033[6n'; value=$(dd bs=1 count=6 2>/dev/null); printf '\033[2J\033[HREPLIED'; read -r action";
    let mut terminal = Terminal::new(script, Limits::default());
    terminal.wait(|session| text(session).contains("REPLIED"));
    assert_eq!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .storage()
            .pending_input_bytes,
        0
    );
    terminal.sessions.close(terminal.id).unwrap();
}
