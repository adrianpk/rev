mod pairs;
mod paragraphs;
mod words;

use crate::command::editing::Object;
use crate::coordinates::{Coordinates, Error, Scalar};
use std::ops::Range;

impl Coordinates<'_> {
    /// Resolve a finite text object against this exact source. Absent objects
    /// are distinct from empty valid interiors and never publish a partial range.
    pub fn text_object(
        &self,
        point: Scalar,
        object: Object,
        count: usize,
    ) -> Result<Option<Range<usize>>, Error> {
        if !self.is_boundary(point)? {
            return Err(Error::Cluster(point));
        }
        match object {
            Object::Word { big, around } => self.object_word(point, (big, around), count.max(1)),
            Object::Paragraph { around } => {
                self.object_paragraph(point, around, count.max(1)).map(Some)
            }
            Object::Pair { open, around } => self.object_pair(point, (open, around), count.max(1)),
            Object::Quote { quote, around } => {
                self.object_quote(point, (quote, around), count.max(1))
            }
        }
    }
}
