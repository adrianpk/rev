//! Finite PNG container admission. Pixel decoding belongs to the terminal.
use super::check;
use crate::org::Cancellation;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Png {
    pub width: u32,
    pub height: u32,
    pub(crate) bytes: Vec<u8>,
}
impl Png {
    /// Admit one complete static PNG container, CRCs and bounded pixel extent.
    /// Compressed pixels are retained verbatim for the terminal's PNG decoder.
    pub fn parse(bytes: Vec<u8>, cancel: &Cancellation) -> Result<Self, String> {
        if bytes.len() > 16777216 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("Expected a bounded PNG resource".into());
        }
        let mut parser = Parser {
            bytes: &bytes,
            at: 8,
            count: 0,
            idat: false,
            ended: false,
            after_data: false,
            size: None,
        };
        while parser.at < bytes.len() {
            check(cancel)?;
            parser.chunk(cancel)?;
        }
        let (width, height) = parser.size.ok_or("PNG has no IHDR")?;
        if !parser.ended || !parser.idat {
            return Err("Incomplete PNG resource".into());
        }
        Ok(Self {
            width,
            height,
            bytes,
        })
    }
}
struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    count: usize,
    idat: bool,
    ended: bool,
    after_data: bool,
    size: Option<(u32, u32)>,
}
impl Parser<'_> {
    fn chunk(&mut self, cancel: &Cancellation) -> Result<(), String> {
        self.count += 1;
        if self.count > 4096 || self.ended {
            return Err("PNG chunk limit or trailing data".into());
        }
        let (body, end) = chunk(self.bytes, self.at, cancel)?;
        let kind = &body[..4];
        if self.count == 1 && kind != b"IHDR" {
            return Err("PNG must start with IHDR".into());
        }
        self.admit(kind, &body[4..])?;
        self.at = end;
        Ok(())
    }
    fn admit(&mut self, kind: &[u8], data: &[u8]) -> Result<(), String> {
        match kind {
            b"IHDR" => self.header(data),
            b"IDAT" => self.data(),
            b"IEND" => self.end(data),
            b"PLTE" => self.palette(data),
            _ => self.ancillary(kind),
        }
    }
    fn header(&mut self, data: &[u8]) -> Result<(), String> {
        if self.size.is_some() {
            return Err("Duplicate PNG IHDR".into());
        }
        self.size = Some(header(data)?);
        Ok(())
    }
    fn data(&mut self) -> Result<(), String> {
        if self.after_data {
            return Err("PNG IDAT chunks must be consecutive".into());
        }
        self.idat = true;
        Ok(())
    }
    fn end(&mut self, data: &[u8]) -> Result<(), String> {
        if !data.is_empty() {
            return Err("PNG IEND must be empty".into());
        }
        self.ended = true;
        Ok(())
    }
    fn palette(&mut self, data: &[u8]) -> Result<(), String> {
        if self.idat || data.is_empty() || data.len() > 768 || !data.len().is_multiple_of(3) {
            return Err("Invalid PNG palette".into());
        }
        Ok(())
    }
    fn ancillary(&mut self, kind: &[u8]) -> Result<(), String> {
        if kind[0].is_ascii_uppercase() || matches!(kind, b"acTL" | b"fcTL" | b"fdAT") {
            return Err("Unsupported critical or animated PNG chunk".into());
        }
        self.after_data |= self.idat;
        Ok(())
    }
}
fn word(bytes: &[u8], at: usize) -> Result<u32, String> {
    let word = bytes.get(at..at + 4).ok_or("Truncated PNG integer")?;
    Ok(u32::from_be_bytes(
        word.try_into().expect("four admitted bytes"),
    ))
}
fn header(data: &[u8]) -> Result<(u32, u32), String> {
    if data.len() != 13 {
        return Err("Invalid PNG IHDR length".into());
    }
    let size = (word(data, 0)?, word(data, 4)?);
    extent(size)?;
    encoding(data)?;
    Ok(size)
}
fn extent(size: (u32, u32)) -> Result<(), String> {
    if size.0 == 0
        || size.1 == 0
        || size.0 > 8192
        || size.1 > 8192
        || u64::from(size.0) * u64::from(size.1) > 16777216
    {
        return Err("PNG pixel extent exceeds native limit".into());
    }
    Ok(())
}
fn encoding(data: &[u8]) -> Result<(), String> {
    let depths: &[u8] = match data[9] {
        0 => &[1, 2, 4, 8, 16],
        2 | 4 | 6 => &[8, 16],
        3 => &[1, 2, 4, 8],
        _ => &[],
    };
    if !depths.contains(&data[8]) || data[10] != 0 || data[11] != 0 || data[12] > 1 {
        return Err("Unsupported PNG encoding".into());
    }
    Ok(())
}
pub(super) fn crc(bytes: &[u8], cancel: &Cancellation) -> Result<u32, String> {
    let mut value = !0u32;
    for chunk in bytes.chunks(8192) {
        check(cancel)?;
        for byte in chunk {
            value = crc_byte(value, *byte);
        }
    }
    Ok(!value)
}
fn crc_byte(mut value: u32, byte: u8) -> u32 {
    value ^= u32::from(byte);
    for _ in 0..8 {
        value = (value >> 1) ^ ((0u32.wrapping_sub(value & 1)) & 0xedb88320);
    }
    value
}

fn chunk<'a>(
    bytes: &'a [u8],
    at: usize,
    cancel: &Cancellation,
) -> Result<(&'a [u8], usize), String> {
    let length = word(bytes, at)? as usize;
    let end = at.checked_add(length + 12).ok_or("PNG size overflow")?;
    let chunk = bytes.get(at + 4..end).ok_or("Truncated PNG chunk")?;
    let (body, checksum) = chunk.split_at(chunk.len() - 4);
    if crc(body, cancel)? != word(checksum, 0)? {
        return Err("PNG chunk CRC mismatch".into());
    }
    let kind = &body[..4];
    if !kind.iter().all(u8::is_ascii_alphabetic) || kind[2].is_ascii_lowercase() {
        return Err("Invalid PNG chunk name".into());
    }
    Ok((body, end))
}
