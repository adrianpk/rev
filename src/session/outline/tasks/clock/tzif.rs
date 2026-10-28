// Only the current, explicitly covered UTC instant is needed here. Recurrence,
// civil-time ambiguity and future POSIX rule expansion belong to temporal editing.
struct Header {
    times: usize,
    types: usize,
    chars: usize,
    leaps: usize,
    standard: usize,
    utc: usize,
}
impl Header {
    fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.get(..4) != Some(b"TZif") {
            return Err("Local task zone is not TZif".into());
        }
        let values: Vec<_> = (0..6)
            .map(|i| number(bytes, 20 + i * 4))
            .collect::<Result<_, _>>()?;
        if values[2] != 0
            || values[4] == 0
            || values[4] > 256
            || values.iter().any(|n| *n > 1048576)
        {
            return Err("Invalid TZif task zone counts".into());
        }
        Ok(Self {
            utc: values[0],
            standard: values[1],
            leaps: values[2],
            times: values[3],
            types: values[4],
            chars: values[5],
        })
    }
    fn size(&self, width: usize) -> usize {
        self.times * (width + 1)
            + self.types * 6
            + self.chars
            + self.leaps * (width + 4)
            + self.standard
            + self.utc
    }
}
pub(super) fn offset(bytes: &[u8], seconds: i64) -> Result<i32, String> {
    let (bytes, width) = block(bytes)?;
    let header = Header::parse(bytes)?;
    let data = bytes
        .get(44..44 + header.size(width))
        .ok_or("Truncated TZif task zone")?;
    let index = selected(data, &header, width, seconds)?;
    let at = header.times * (width + 1) + index * 6;
    let raw = data.get(at..at + 4).ok_or("Invalid TZif type")?;
    let offset = i32::from_be_bytes(raw.try_into().expect("four bytes"));
    if !(-86400..=86400).contains(&offset) {
        return Err("Unsupported TZif task offset".into());
    }
    Ok(offset)
}
fn selected(data: &[u8], header: &Header, width: usize, seconds: i64) -> Result<usize, String> {
    let mut selected = 0;
    let mut previous = i64::MIN;
    for i in 0..header.times {
        let value = instant(&data[i * width..(i + 1) * width]);
        if value <= previous {
            return Err("Unordered TZif task transitions".into());
        }
        previous = value;
        let index = usize::from(data[header.times * width + i]);
        if index >= header.types {
            return Err("Invalid TZif task transition type".into());
        }
        if value <= seconds {
            selected = index;
        }
    }
    if header.times > 0 && seconds > previous {
        return Err("Current task time exceeds explicit TZif coverage".into());
    }
    Ok(selected)
}
fn instant(bytes: &[u8]) -> i64 {
    if bytes.len() == 4 {
        i64::from(i32::from_be_bytes(bytes.try_into().expect("four bytes")))
    } else {
        i64::from_be_bytes(bytes.try_into().expect("eight bytes"))
    }
}
fn number(bytes: &[u8], start: usize) -> Result<usize, String> {
    let raw = bytes.get(start..start + 4).ok_or("Truncated TZif header")?;
    Ok(u32::from_be_bytes(raw.try_into().expect("four bytes")) as usize)
}

fn block(bytes: &[u8]) -> Result<(&[u8], usize), String> {
    let first = Header::parse(bytes)?;
    let version = bytes.get(4).ok_or("Truncated TZif")?;
    let (bytes, width) = match version {
        b'2' | b'3' | b'4' => (
            bytes
                .get(44 + first.size(4)..)
                .ok_or("Truncated TZif block")?,
            8,
        ),
        0 => (bytes, 4),
        _ => return Err("Unsupported TZif task zone".into()),
    };
    Ok((bytes, width))
}
