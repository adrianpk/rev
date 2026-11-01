use std::io::{self, Write};

use crossterm::{ExecutableCommand, cursor::SetCursorStyle};

/// Retains a reset obligation even after a partial cursor-shape write fails.
#[derive(Default)]
pub(super) struct Style {
    current: Option<SetCursorStyle>,
    owned: bool,
}

impl Style {
    pub(super) fn set(&mut self, output: &mut impl Write, style: SetCursorStyle) -> io::Result<()> {
        if self.current == Some(style) {
            return Ok(());
        }
        self.owned = true;
        output.execute(style)?;
        self.current = Some(style);
        Ok(())
    }

    pub(super) fn restore(&mut self, output: &mut impl Write) -> io::Result<()> {
        if self.owned {
            output.execute(SetCursorStyle::DefaultUserShape)?;
            self.current = None;
            self.owned = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Presentation changes once per transition. Every attempted shape owns a
    // reset, including a failed write; an unused owner emits no restoration.
    #[test]
    fn restoration() {
        let mut style = Style::default();
        let mut bytes = Vec::new();
        style.restore(&mut bytes).unwrap();
        assert!(bytes.is_empty());
        style.set(&mut bytes, SetCursorStyle::SteadyBlock).unwrap();
        style.set(&mut bytes, SetCursorStyle::SteadyBlock).unwrap();
        style.set(&mut bytes, SetCursorStyle::SteadyBar).unwrap();
        style.restore(&mut bytes).unwrap();
        assert_eq!(bytes, b"\x1b[2 q\x1b[6 q\x1b[0 q");
        assert!(
            style
                .set(&mut Failing, SetCursorStyle::SteadyBlock)
                .is_err()
        );
        assert!(style.restore(&mut Failing).is_err());
        assert!(style.owned);
        style.restore(&mut bytes).unwrap();
        assert!(!style.owned);
        assert!(bytes.ends_with(b"\x1b[0 q"));
    }

    struct Failing;
    impl Write for Failing {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("closed output"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
