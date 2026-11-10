use super::*;

struct Coordinates {
    original: usize,
    final_line: usize,
}
struct Header {
    object: ObjectId,
    coordinates: Coordinates,
}
struct Metadata<'a> {
    author: Option<&'a [u8]>,
    email: Option<&'a [u8]>,
    time: Option<&'a [u8]>,
    subject: Option<&'a [u8]>,
    filename: Option<PathBuf>,
    text: Option<&'a [u8]>,
}
pub(super) struct Attributions {
    pub(super) lines: Vec<Attribution>,
    pub(super) complete: bool,
    path_bytes: usize,
}
impl Header {
    fn parse(header: &[u8], format: ObjectFormat, expected: usize) -> Result<Self, String> {
        let mut parts = header.split(|b| *b == b' ');
        let object = ObjectId::parse(parts.next().ok_or("Blame lacks an object")?, format)?;
        let coordinates = Coordinates::parse(&mut parts, expected)?;
        Ok(Self {
            object,
            coordinates,
        })
    }
}
impl Coordinates {
    fn parse<'a>(
        parts: &mut impl Iterator<Item = &'a [u8]>,
        expected: usize,
    ) -> Result<Self, String> {
        let original = number(parts.next().ok_or("Blame lacks an original line")?)?;
        let final_line = number(parts.next().ok_or("Blame lacks a final line")?)?;
        if final_line != expected {
            return Err("Blame source lines are not a complete ordered prefix".into());
        }
        if let Some(count) = parts.next() {
            number(count)?;
        }
        if parts.next().is_some() {
            return Err("Malformed blame header".into());
        }
        Ok(Self {
            original,
            final_line,
        })
    }
}
impl<'a> Metadata<'a> {
    fn read(records: &mut impl Iterator<Item = &'a [u8]>) -> Result<Self, String> {
        let mut metadata = Self {
            author: None,
            email: None,
            time: None,
            subject: None,
            filename: None,
            text: None,
        };
        for record in records.by_ref() {
            if let Some(value) = record.strip_prefix(b"\t") {
                metadata.text = Some(value);
                break;
            }
            metadata.record(record)?;
        }
        Ok(metadata)
    }
    fn record(&mut self, record: &'a [u8]) -> Result<(), String> {
        let Some((key, value)) = record
            .iter()
            .position(|b| *b == b' ')
            .map(|at| (&record[..at], &record[at + 1..]))
        else {
            if record == b"boundary" {
                return Ok(());
            }
            return Err("Malformed blame metadata".into());
        };
        if key == b"filename" {
            self.filename = Some(unquote_path(value)?);
        } else {
            metadata_field(value)?;
            match key {
                b"author" => self.author = Some(value),
                b"author-mail" => self.email = Some(value),
                b"author-time" => self.time = Some(value),
                b"summary" => self.subject = Some(value),
                _ => {}
            }
        }
        Ok(())
    }
}
impl Attributions {
    pub(super) fn read(
        bytes: &[u8],
        format: ObjectFormat,
        policy: &config::Git,
        cancel: &Cancellation,
        truncated: bool,
    ) -> Result<Self, String> {
        let mut records = bytes
            .split_inclusive(|b| *b == b'\n')
            .filter_map(|l| l.strip_suffix(b"\n"));
        let mut parsed = Self {
            lines: Vec::new(),
            complete: !truncated,
            path_bytes: 0,
        };
        while let Some(header) = records.next() {
            if cancel.is_cancelled() {
                return Err("Blame parsing cancelled".into());
            }
            let header = Header::parse(header, format, parsed.lines.len() + 1)?;
            let metadata = Metadata::read(&mut records)?;
            if !parsed.admit(&header, metadata, truncated, policy.max_path_bytes)? {
                break;
            }
        }
        Ok(parsed)
    }
    fn admit(
        &mut self,
        header: &Header,
        metadata: Metadata<'_>,
        truncated: bool,
        path_limit: usize,
    ) -> Result<bool, String> {
        let Some(text) = metadata.text else {
            if truncated {
                return Ok(false);
            }
            return Err("Incomplete blame source record".into());
        };
        let Some(filename) = metadata.filename else {
            return Err("Blame lacks an exact original filename".into());
        };
        self.path_bytes += filename.capacity();
        if self.path_bytes > path_limit {
            self.complete = false;
            return Ok(false);
        }
        self.lines.push(Attribution {
            object: header.object,
            path: filename,
            original_line: header.coordinates.original,
            final_line: header.coordinates.final_line,
            author: metadata
                .author
                .ok_or("Blame lacks author metadata")?
                .to_vec(),
            email: metadata.email.ok_or("Blame lacks author mail")?.to_vec(),
            time: metadata.time.ok_or("Blame lacks author time")?.to_vec(),
            subject: metadata.subject.ok_or("Blame lacks summary")?.to_vec(),
            text: text.to_vec(),
        });
        Ok(true)
    }
}
