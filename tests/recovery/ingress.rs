use std::fs::File;
use std::io::{self, Read};

use rev::file::Loaded;

use super::*;

struct Interrupted {
    file: File,
    remaining: usize,
    kind: io::ErrorKind,
}

impl Read for Interrupted {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::new(
                self.kind,
                "read interrupted after partial file",
            ));
        }
        let length = bytes.len().min(self.remaining).min(127);
        let count = self.file.read(&mut bytes[..length])?;
        self.remaining -= count;
        Ok(count)
    }
}

// A real file streams across BOM, CRLF, rope chunks and multibyte seams.
// Inject only a late read error; no incomplete candidate can be installed.
#[test]
fn interrupted_ingress() {
    let fixture = Fixture::new();
    let source = format!(
        "\u{feff}{}",
        "fn main() { let text = \"界🙂\"; }\r\n".repeat(300)
    );
    let path = fixture.file("large.rs", source.as_bytes());
    for (after, kind) in [
        (1, io::ErrorKind::UnexpectedEof),
        (4097, io::ErrorKind::PermissionDenied),
        (source.len() - 1, io::ErrorKind::Other),
    ] {
        let reader = Interrupted {
            file: File::open(&path).unwrap(),
            remaining: after,
            kind,
        };
        let error = Loaded::read(reader).err().expect("no partial ingress");
        assert_eq!(error.kind(), kind);
        assert!(error.to_string().contains("read interrupted"));
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    }
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .load(Loaded::read(File::open(&path).unwrap()).unwrap())
        .unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut saved = Vec::new();
    rev::file::write(buffer, &mut saved).unwrap();
    assert_eq!(saved, source.as_bytes());
    assert!(!buffer.is_modified());
}
