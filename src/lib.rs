#![forbid(unsafe_code)]

pub mod buffer;
pub mod coordinates;
pub mod terminal;

use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use terminal::Terminal;

/// Runs the terminal session and reports errors after attempting restoration.
pub fn run() -> io::Result<()> {
    let mut terminal = Terminal::acquire(io::stdout())?;
    let result = terminal.enter().and_then(|()| session(&mut terminal));
    let cleanup = terminal.restore();
    match (result, cleanup) {
        (Err(error), Err(cleanup)) => Err(io::Error::new(
            error.kind(),
            format!("{error}; terminal restoration: {cleanup}"),
        )),
        (result, cleanup) => result.and(cleanup),
    }
}

fn session(terminal: &mut Terminal<impl Write>) -> io::Result<()> {
    terminal
        .writer()
        .write_all(b"\x1b[2J\x1b[Hrev\r\nC-x C-c: exit\r\n")?;
    terminal.writer().flush()?;
    let mut prefix = false;
    loop {
        if let Event::Key(key) = event::read()?
            && quit(key, &mut prefix)
        {
            return Ok(());
        }
    }
}

fn quit(key: KeyEvent, prefix: &mut bool) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }
    let control = key.modifiers == KeyModifiers::CONTROL;
    let exit = *prefix && control && key.code == KeyCode::Char('c');
    *prefix = control && key.code == KeyCode::Char('x');
    exit
}

#[cfg(test)]
mod tests {
    use super::*;

    // Prefix cancellation and release reports must not trigger an exit.
    #[test]
    fn quit_sequence() {
        let control = |letter| KeyEvent::new(KeyCode::Char(letter), KeyModifiers::CONTROL);
        let cases = [
            ("exit", vec![control('x'), control('c')], true),
            (
                "cancel",
                vec![control('x'), control('g'), control('c')],
                false,
            ),
            ("bare control", vec![control('c')], false),
            ("old shortcut", vec![control('q')], false),
            (
                "release",
                vec![
                    control('x'),
                    KeyEvent::new_with_kind(
                        KeyCode::Char('x'),
                        KeyModifiers::CONTROL,
                        KeyEventKind::Release,
                    ),
                    control('c'),
                ],
                true,
            ),
        ];
        for (name, keys, expected) in cases {
            let mut prefix = false;
            let result = keys.into_iter().any(|key| quit(key, &mut prefix));
            assert_eq!(result, expected, "{name}");
        }
    }
}
