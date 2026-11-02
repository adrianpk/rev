use std::io::{self, BufRead, Read};

pub(super) const RECORD: usize = 2 * 1024 * 1024;
pub(super) const QUEUE: usize = 4 * 1024 * 1024;

// Byte-oriented LF framing preserves Unicode separators inside JSON strings.
// take bounds allocation even when the peer never terminates a record.
pub(super) fn read(input: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    let n = input
        .take((RECORD + 1) as u64)
        .read_until(b'\n', &mut bytes)?;
    if n == 0 {
        return Ok(None);
    }
    if n > RECORD || bytes.last() != Some(&b'\n') {
        return Err(io::Error::other(
            "Pi record exceeds 2 MiB or lacks final LF",
        ));
    }
    bytes.pop();
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    Ok(Some(bytes))
}

pub(super) fn encode(value: &serde_json::Value) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| "Pi request cannot be encoded")?;
    if bytes.len() >= RECORD {
        return Err("Pi encoded request exceeds 2 MiB".into());
    }
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    // LF alone separates records; Unicode separators and CRLF remain valid
    // payload/framing, while unterminated and excessive records fail finitely.
    #[test]
    fn bounded_lf() {
        let mut input = std::io::Cursor::new("{\"text\":\"a\u{2028}b\u{2029}c\"}\r\n{}\n");
        let value: serde_json::Value =
            serde_json::from_slice(&read(&mut input).unwrap().unwrap()).unwrap();
        assert_eq!(value["text"], "a\u{2028}b\u{2029}c");
        assert_eq!(read(&mut input).unwrap().unwrap(), b"{}");
        assert!(read(&mut input).unwrap().is_none());
        for bytes in [b"{}".to_vec(), vec![b'x'; RECORD + 1]] {
            assert!(read(&mut std::io::Cursor::new(bytes)).is_err());
        }
        assert!(encode(&serde_json::json!({"text":"x".repeat(RECORD)})).is_err());
    }
}
