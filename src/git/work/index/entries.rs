use super::*;

pub(in crate::git::work) fn entries(
    bytes: &[u8],
    format: ObjectFormat,
    index: bool,
    limit: usize,
) -> Result<Vec<Entry>, String> {
    if bytes.iter().filter(|b| **b == 0).count() > limit {
        return Err("Index entries exceed the exact target count".into());
    }
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err("Index entries lack their NUL boundary".into());
    }
    bytes
        .split(|b| *b == 0)
        .filter(|part| !part.is_empty())
        .map(|record| entry(record, format, index))
        .collect()
}

fn entry(record: &[u8], format: ObjectFormat, index: bool) -> Result<Entry, String> {
    let header = Header::parse(record, index)?;
    let mode = std::str::from_utf8(header.mode)
        .ok()
        .and_then(|s| u32::from_str_radix(s, 8).ok())
        .ok_or("Invalid index mode")?;
    Ok(Entry {
        path: PathBuf::from(OsString::from_vec(header.path.to_vec())),
        mode: Some(mode),
        object: Some(ObjectId::parse(header.object, format)?),
    })
}

/// A complete entry header retains raw byte views until semantic decoding.
struct Header<'a> {
    mode: &'a [u8],
    object: &'a [u8],
    path: &'a [u8],
}

impl<'a> Header<'a> {
    fn parse(record: &'a [u8], index: bool) -> Result<Self, String> {
        let tab = record
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("Index entry lacks its path boundary")?;
        let mut fields = record[..tab].split(|b| *b == b' ');
        let parts = [
            fields.next().ok_or("Missing index mode")?,
            fields.next().ok_or("Missing index object")?,
            fields.next().ok_or("Missing index stage/type")?,
        ];
        if fields.next().is_some() || (index && parts[2] != b"0") {
            return Err("Index entry is malformed or unmerged".into());
        }
        Ok(Self {
            mode: parts[0],
            object: parts[if index { 1 } else { 2 }],
            path: &record[tab + 1..],
        })
    }
}
