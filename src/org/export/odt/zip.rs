//! Fixed five-member stored ZIP profile: no extras, compression or ZIP64.
use super::*;
struct Zip<'a> {
    bytes: Vec<u8>,
    limit: usize,
    cancel: &'a Cancellation,
}
struct Entry {
    name: &'static str,
    offset: u32,
    size: u32,
    crc: u32,
}
pub(super) fn package(
    members: [(&'static str, &[u8]); 5],
    limit: usize,
    cancel: &Cancellation,
) -> Result<Vec<u8>, String> {
    let mut zip = Zip {
        bytes: Vec::new(),
        limit,
        cancel,
    };
    let mut entries = Vec::with_capacity(5);
    for (name, data) in members {
        entries.push(zip.member(name, data)?);
    }
    let directory = zip.position()?;
    for entry in &entries {
        zip.directory(entry)?;
    }
    let size = zip.position()? - directory;
    zip.end(directory, size)?;
    Ok(zip.bytes)
}
impl Zip<'_> {
    fn emit(&mut self, bytes: &[u8]) -> Result<(), String> {
        document::check(self.cancel)?;
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err("ODT package output byte limit reached".into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn word(&mut self, value: u16) -> Result<(), String> {
        self.emit(&value.to_le_bytes())
    }
    fn integer(&mut self, value: u32) -> Result<(), String> {
        self.emit(&value.to_le_bytes())
    }
    fn position(&self) -> Result<u32, String> {
        u32::try_from(self.bytes.len()).map_err(|_| "ODT ZIP32 size limit reached".into())
    }
    fn profile(&mut self) -> Result<(), String> {
        // Needed version, flags, stored method, DOS time and 1980-01-01 date.
        for word in [20, 0, 0, 0, 0x21] {
            self.word(word)?;
        }
        Ok(())
    }
    fn dimensions(&mut self, e: &Entry) -> Result<(), String> {
        for value in [e.crc, e.size, e.size] {
            self.integer(value)?;
        }
        self.word(e.name.len() as u16)?;
        self.word(0)
    }
    fn member(&mut self, name: &'static str, data: &[u8]) -> Result<Entry, String> {
        let entry = Entry {
            name,
            offset: self.position()?,
            size: u32::try_from(data.len()).map_err(|_| "ODT ZIP member exceeds ZIP32")?,
            crc: crc(data, self.cancel)?,
        };
        self.integer(0x04034b50)?;
        self.profile()?;
        self.dimensions(&entry)?;
        self.emit(name.as_bytes())?;
        for bytes in data.chunks(8192) {
            self.emit(bytes)?;
        }
        Ok(entry)
    }
    fn directory(&mut self, entry: &Entry) -> Result<(), String> {
        self.integer(0x02014b50)?;
        self.word(20)?;
        self.profile()?;
        self.dimensions(entry)?;
        // Comment length, disk start, internal attributes, external attributes.
        self.emit(&[0; 10])?;
        self.integer(entry.offset)?;
        self.emit(entry.name.as_bytes())
    }
    fn end(&mut self, offset: u32, size: u32) -> Result<(), String> {
        self.integer(0x06054b50)?;
        for word in [0, 0, 5, 5] {
            self.word(word)?;
        }
        self.integer(size)?;
        self.integer(offset)?;
        self.word(0)
    }
}
fn crc(bytes: &[u8], cancel: &Cancellation) -> Result<u32, String> {
    const TABLE: [u32; 256] = table();
    let mut crc = u32::MAX;
    for chunk in bytes.chunks(4096) {
        document::check(cancel)?;
        for b in chunk {
            crc = TABLE[((crc ^ u32::from(*b)) & 255) as usize] ^ (crc >> 8);
        }
    }
    Ok(!crc)
}
const fn table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut at = 0;
    while at < 256 {
        let mut value = at as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 1 {
                0xedb88320 ^ (value >> 1)
            } else {
                value >> 1
            };
            bit += 1;
        }
        table[at] = value;
        at += 1;
    }
    table
}
