use super::{Posix, Transition, Zone};

struct Header {
    version: u8,
    utc: usize,
    standard: usize,
    times: usize,
    types: usize,
    chars: usize,
}
impl Header {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        let bytes = bytes.get(..44).ok_or("Truncated TZif header")?;
        if &bytes[..4] != b"TZif" || !matches!(bytes[4], 0 | b'2' | b'3' | b'4') {
            return Err("Invalid TZif magic or version".into());
        }
        if bytes[5..20].iter().any(|b| *b != 0) {
            return Err("Nonzero TZif reserved header bytes".into());
        }
        let counts: Vec<_> = bytes[20..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| u32::from_be_bytes(*v) as usize)
            .collect();
        checked_counts(&counts)?;
        Ok(Self {
            version: bytes[4],
            utc: counts[0],
            standard: counts[1],
            times: counts[3],
            types: counts[4],
            chars: counts[5],
        })
    }
    fn size(&self, width: usize) -> usize {
        self.times * (width + 1) + self.types * 6 + self.chars + self.standard + self.utc
    }
}
fn checked_counts(v: &[usize]) -> Result<(), String> {
    if v[2] != 0 {
        return Err("Leap-second TZif data is not admitted by Org Unix time".into());
    }
    if v[3] > 65536 || !(1..=256).contains(&v[4]) || !(1..=4096).contains(&v[5]) {
        return Err("TZif counts exceed Org observation bounds".into());
    }
    if ![0, v[4]].contains(&v[0]) || ![0, v[4]].contains(&v[1]) {
        return Err("Invalid TZif indicator counts".into());
    }
    Ok(())
}
pub(super) fn parse(bytes: &[u8]) -> Result<Zone, String> {
    if bytes.len() > 1048576 {
        return Err("Org TZif observation exceeds 1 MiB".into());
    }
    let first = Header::parse(bytes)?;
    let (bytes, width) = block(bytes, &first)?;
    let header = Header::parse(bytes)?;
    let end = 44 + header.size(width);
    let data = bytes.get(44..end).ok_or("Truncated TZif data")?;
    let offsets = offsets(data, &header, width)?;
    let transitions = transitions(data, &header, width)?;
    let future = footer(&bytes[end..], header.version)?;
    let zone = Zone {
        offsets,
        transitions,
        future,
    };
    consistent_footer(&zone, data, &header, width)?;
    Ok(zone)
}
fn block<'a>(bytes: &'a [u8], first: &Header) -> Result<(&'a [u8], usize), String> {
    if first.version == 0 {
        return Ok((bytes, 4));
    }
    let bytes = bytes
        .get(44 + first.size(4)..)
        .ok_or("Truncated TZif compatibility block")?;
    let header = Header::parse(bytes)?;
    if header.version != first.version {
        return Err("Mismatched TZif block versions".into());
    }
    Ok((bytes, 8))
}
fn footer(bytes: &[u8], version: u8) -> Result<Option<Posix>, String> {
    if version == 0 {
        if !bytes.is_empty() {
            return Err("Trailing TZif version 1 data".into());
        }
        return Ok(None);
    }
    let text = bytes
        .strip_prefix(b"\n")
        .and_then(|b| b.strip_suffix(b"\n"))
        .ok_or("TZif footer requires surrounding newlines")?;
    if text.is_empty() {
        return Ok(None);
    }
    let text = std::str::from_utf8(text).map_err(|_| "Invalid TZif footer text")?;
    Ok(Some(Posix::parse(text, version >= b'3')?))
}
fn offsets(data: &[u8], h: &Header, width: usize) -> Result<Vec<Option<i32>>, String> {
    let start = h.times * (width + 1);
    let end = start + h.types * 6;
    let names = &data[end..end + h.chars];
    indicators(&data[end + h.chars..], h)?;
    data[start..end]
        .as_chunks::<6>()
        .0
        .iter()
        .map(|record| offset(record, names))
        .collect()
}
fn offset(record: &[u8], names: &[u8]) -> Result<Option<i32>, String> {
    let seconds = i32::from_be_bytes(record[..4].try_into().expect("four bytes"));
    super::posix::checked_offset(seconds)?;
    if record[4] > 1 {
        return Err("Invalid TZif DST indicator".into());
    }
    let name = designation(names, record[5])?;
    Ok(if name == b"-00" { None } else { Some(seconds) })
}
fn designation(names: &[u8], index: u8) -> Result<&[u8], String> {
    let name = names
        .get(usize::from(index)..)
        .ok_or("Invalid TZif designation index")?;
    let end = name
        .iter()
        .position(|b| *b == 0)
        .ok_or("Unterminated TZif designation")?;
    let name = &name[..end];
    if name.len() > 64 || !name.iter().all(|b| (32..=126).contains(b)) {
        return Err("TZif designation requires at most 64 printable ASCII bytes".into());
    }
    Ok(name)
}

fn indicators(data: &[u8], h: &Header) -> Result<(), String> {
    let (standard, utc) = data.split_at(h.standard);
    for i in 0..h.types {
        let s = standard.get(i).copied().unwrap_or(0);
        let u = utc.get(i).copied().unwrap_or(0);
        if s > 1 || u > s {
            return Err("Invalid TZif standard/UT indicator".into());
        }
    }
    Ok(())
}
fn transitions(data: &[u8], h: &Header, width: usize) -> Result<Vec<Transition>, String> {
    let times = &data[..h.times * width];
    let kinds = &data[h.times * width..h.times * (width + 1)];
    let mut result: Vec<Transition> = Vec::with_capacity(h.times);
    for (raw, kind) in times.chunks_exact(width).zip(kinds) {
        let at = instant(raw);
        if result.last().is_some_and(|t| t.at >= at) {
            return Err("Unordered TZif transitions".into());
        }
        let kind = usize::from(*kind);
        if kind >= h.types {
            return Err("Invalid TZif transition type".into());
        }
        result.push(Transition { at, kind });
    }
    Ok(result)
}
fn instant(bytes: &[u8]) -> i64 {
    if bytes.len() == 4 {
        i64::from(i32::from_be_bytes(bytes.try_into().expect("four bytes")))
    } else {
        i64::from_be_bytes(bytes.try_into().expect("eight bytes"))
    }
}
fn consistent_footer(zone: &Zone, data: &[u8], h: &Header, width: usize) -> Result<(), String> {
    let (Some(last), Some(future)) = (zone.transitions.last(), &zone.future) else {
        return Ok(());
    };
    let record_start = h.times * (width + 1);
    let record = &data[record_start + last.kind * 6..record_start + (last.kind + 1) * 6];
    let names_start = record_start + h.types * 6;
    let name = designation(&data[names_start..names_start + h.chars], record[5])?;
    let expected = future.type_at(last.at)?;
    if Some(expected.offset) != zone.offsets[last.kind]
        || expected.daylight != (record[4] == 1)
        || expected.name.as_bytes() != name
    {
        return Err("TZif footer disagrees with its last time type".into());
    }
    Ok(())
}
