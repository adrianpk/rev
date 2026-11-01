use crate::coordinates::{Coordinates, Error, Scalar};
use std::ops::Range;

impl Coordinates<'_> {
    pub(super) fn object_paragraph(
        &self,
        point: Scalar,
        around: bool,
        mut count: usize,
    ) -> Result<Range<usize>, Error> {
        let row = self.to_line(point)?.row;
        let blank = self.blank_row(row);
        let mut first = row;
        let mut last = row;
        while first > 0 && self.blank_row(first - 1) == blank {
            first -= 1;
        }
        while last + 1 < self.text.len_lines() && self.blank_row(last + 1) == blank {
            last += 1;
        }
        count -= 1;
        while count > 0 && last + 1 < self.text.len_lines() {
            last = self.following_paragraph(last);
            count -= 1;
        }
        if around {
            self.paragraph_space(&mut first, &mut last);
        }
        let start = self.text.line_to_char(first);
        let end = if last + 1 < self.text.len_lines() {
            self.text.line_to_char(last + 1)
        } else {
            self.text.len_chars()
        };
        Ok(start..end)
    }

    fn blank_row(&self, row: usize) -> bool {
        self.text.line(row).chars().all(char::is_whitespace)
    }

    fn following_paragraph(&self, mut last: usize) -> usize {
        while last + 1 < self.text.len_lines() && self.blank_row(last + 1) {
            last += 1;
        }
        while last + 1 < self.text.len_lines() && !self.blank_row(last + 1) {
            last += 1;
        }
        last
    }

    fn paragraph_space(&self, first: &mut usize, last: &mut usize) {
        let original = *last;
        while *last + 1 < self.text.len_lines() && self.blank_row(*last + 1) {
            *last += 1;
        }
        if *last == original {
            while *first > 0 && self.blank_row(*first - 1) {
                *first -= 1;
            }
        }
    }
}
