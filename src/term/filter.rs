use super::limits::CONTROL_BYTES;

#[derive(Default)]
enum State {
    #[default]
    Ground,
    Escape,
    Csi,
    Osc,
    String,
    StringEscape,
}

/// Hold complete control strings before admission. Oversized strings are
/// discarded through their terminator; no truncated OSC becomes a host effect.
#[derive(Default)]
pub(super) struct Filter {
    state: State,
    bytes: Vec<u8>,
    oversized: bool,
    osc: bool,
    pub rejected: bool,
}
impl Filter {
    pub fn byte(&mut self, byte: u8) -> Option<&[u8]> {
        match self.state {
            State::Ground => self.ground(byte),
            State::Escape => self.escape(byte),
            State::Csi => self.csi(byte),
            State::Osc | State::String => self.string(byte),
            State::StringEscape => self.string_escape(byte),
        }
    }
    fn ground(&mut self, byte: u8) -> Option<&[u8]> {
        self.bytes.clear();
        self.bytes.push(byte);
        self.oversized = false;
        if byte == 0x1b {
            self.state = State::Escape;
            None
        } else {
            Some(&self.bytes)
        }
    }
    fn escape(&mut self, byte: u8) -> Option<&[u8]> {
        self.push(byte);
        self.state = match byte {
            b'[' => State::Csi,
            b']' => {
                self.osc = true;
                State::Osc
            }
            b'P' | b'^' | b'_' | b'X' => {
                self.osc = false;
                State::String
            }
            0x20..=0x2f => State::Escape,
            0x1b => {
                self.bytes.clear();
                self.bytes.push(byte);
                State::Escape
            }
            _ => return self.finish(),
        };
        None
    }
    fn csi(&mut self, byte: u8) -> Option<&[u8]> {
        if byte == 0x1b {
            return self.ground(byte);
        }
        self.push(byte);
        if (0x40..=0x7e).contains(&byte) || matches!(byte, 0x18 | 0x1a) {
            self.finish()
        } else {
            None
        }
    }
    fn string(&mut self, byte: u8) -> Option<&[u8]> {
        if matches!(byte, 0x18 | 0x1a) {
            self.state = State::Ground;
            return None;
        }
        self.push(byte);
        if byte == 0x1b {
            self.state = State::StringEscape;
        }
        if self.osc && byte == 7 {
            return self.finish_string();
        }
        None
    }
    fn string_escape(&mut self, byte: u8) -> Option<&[u8]> {
        if byte == b'\\' {
            self.push(byte);
            return self.finish_string();
        }
        // ESC aborts a control string and begins the next escape sequence.
        self.bytes.clear();
        self.bytes.push(0x1b);
        self.oversized = false;
        self.state = State::Escape;
        self.escape(byte)
    }
    fn push(&mut self, byte: u8) {
        if self.bytes.len() < CONTROL_BYTES {
            self.bytes.push(byte);
        } else {
            self.oversized = true;
            self.rejected = true;
        }
    }
    fn finish(&mut self) -> Option<&[u8]> {
        self.state = State::Ground;
        if self.oversized {
            None
        } else {
            Some(&self.bytes)
        }
    }
    fn finish_string(&mut self) -> Option<&[u8]> {
        if self.osc {
            self.finish()
        } else {
            self.state = State::Ground;
            None
        }
    }
}
