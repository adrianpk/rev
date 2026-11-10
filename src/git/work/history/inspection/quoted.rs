use super::*;

// Git's C-quoted filename grammar preserves whitespace, quotes, backslashes
// and non-UTF-8 bytes. Escaped display strings never become path arguments.
pub(in crate::git::work) fn unquote_path(value: &[u8]) -> Result<PathBuf, String> {
    let bytes = if let Some(inner) = value.strip_prefix(b"\"") {
        let inner = inner
            .strip_suffix(b"\"")
            .ok_or("Unterminated quoted blame path")?;
        QuotedPath::decode(inner)?
    } else {
        if value.len() > 4096 {
            return Err("Blame path exceeds 4096 bytes".into());
        }
        value.to_vec()
    };
    validate_path(&bytes)?;
    Ok(PathBuf::from(OsString::from_vec(bytes)))
}
struct QuotedPath<'a> {
    input: &'a [u8],
    at: usize,
}
impl QuotedPath<'_> {
    fn decode(input: &[u8]) -> Result<Vec<u8>, String> {
        if input.len() > 16384 {
            return Err("Quoted blame path exceeds its bound".into());
        }
        let mut parser = QuotedPath { input, at: 0 };
        let mut bytes = Vec::with_capacity(input.len().min(4096));
        while parser.at < input.len() {
            let mut b = input[parser.at];
            parser.at += 1;
            if b == b'\\' {
                b = parser.escape()?;
            }
            if bytes.len() >= 4096 {
                return Err("Blame path exceeds 4096 bytes".into());
            }
            bytes.push(b);
        }
        Ok(bytes)
    }
    fn escape(&mut self) -> Result<u8, String> {
        let b = *self
            .input
            .get(self.at)
            .ok_or("Incomplete blame path escape")?;
        self.at += 1;
        match b {
            b'\\' | b'"' => Ok(b),
            b'0'..=b'7' => self.octal(b),
            _ => control(b).ok_or_else(|| "Unsupported blame path escape".into()),
        }
    }
    fn octal(&mut self, first: u8) -> Result<u8, String> {
        let mut value = u16::from(first - b'0');
        for _ in 0..2 {
            if let Some(c @ b'0'..=b'7') = self.input.get(self.at) {
                value = value * 8 + u16::from(c - b'0');
                self.at += 1;
            } else {
                break;
            }
        }
        u8::try_from(value).map_err(|_| "Invalid blame path octal escape".into())
    }
}
fn control(byte: u8) -> Option<u8> {
    match byte {
        b'a' => Some(7),
        b'b' => Some(8),
        b't' => Some(b'\t'),
        b'n' => Some(b'\n'),
        b'v' => Some(11),
        b'f' => Some(12),
        b'r' => Some(b'\r'),
        _ => None,
    }
}
