use super::{
    Buffer, Error, Permutation, Piece, Point, Prepared, Replacement, end_points, partition,
};

struct Assembly {
    text: String,
    pieces: Vec<Piece>,
    position: usize,
}

impl Buffer {
    // List movement can change counter widths. Construct both the text and its
    // anchor map from the same checked source partition and counter edits.
    pub(crate) fn prepare_reorder_edited(
        &self,
        sources: &[crate::buffer::Span],
        edits: &[Replacement],
        points: &[Point],
    ) -> Result<Prepared, Error> {
        if edits.is_empty() {
            return self.prepare_reorder(sources, points);
        }
        self.admit(self.stamp())?;
        let ranges = partition(self, sources)?;
        let union = ranges[0].start..ranges.last().ok_or(Error::Order)?.end;
        admit(self, edits, &union)?;
        let mut assembly = Assembly {
            text: String::new(),
            pieces: Vec::new(),
            position: union.start,
        };
        for source in sources {
            assembly.fragment(self, source, edits)?;
        }
        let order = Permutation {
            endpoints: end_points(self, &assembly.pieces)?,
            pieces: assembly.pieces,
            sparse: false,
        };
        let edit = Replacement {
            source: self.span(union)?,
            text: assembly.text,
        };
        self.prepare_ordered(&[edit], points, None, Some(order))
    }
}

fn admit(
    buffer: &Buffer,
    edits: &[Replacement],
    union: &std::ops::Range<usize>,
) -> Result<(), Error> {
    buffer.changes(edits)?;
    for edit in edits {
        if edit.source.range.is_empty()
            || edit.source.range.start < union.start
            || edit.source.range.end > union.end
        {
            return Err(Error::Order);
        }
    }
    Ok(())
}

impl Assembly {
    fn fragment(
        &mut self,
        buffer: &Buffer,
        source: &crate::buffer::Span,
        edits: &[Replacement],
    ) -> Result<(), Error> {
        let mut start = source.range.start;
        for edit in edits
            .iter()
            .filter(|edit| source.range.contains(&edit.source.range.start))
        {
            if edit.source.range.end > source.range.end {
                return Err(Error::Order);
            }
            self.original(buffer, start..edit.source.range.start)?;
            self.append(edit.source.range.clone(), &edit.text, false);
            start = edit.source.range.end;
        }
        self.original(buffer, start..source.range.end)
    }

    fn original(&mut self, buffer: &Buffer, range: std::ops::Range<usize>) -> Result<(), Error> {
        if range.is_empty() {
            return Ok(());
        }
        let source = buffer.span(range.clone())?;
        self.append(range, &buffer.slice(&source)?.to_string(), true);
        Ok(())
    }

    fn append(&mut self, before: std::ops::Range<usize>, text: &str, retained: bool) {
        let after = self.position..self.position + text.chars().count();
        self.position = after.end;
        self.text.push_str(text);
        self.pieces.push(Piece {
            before,
            after,
            retained,
        });
    }
}
