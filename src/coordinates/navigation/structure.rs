use crate::search::Direction;

use super::super::{Coordinates, Error, Scalar};

impl Coordinates<'_> {
    pub(super) fn paragraph(
        &self,
        point: Scalar,
        direction: Direction,
        mut count: usize,
    ) -> Result<Scalar, Error> {
        let mut row = self.to_line(point)?.row;
        let backward = direction == Direction::Backward;
        let last = self.text.len_lines() - 1;
        let mut body = !self.text.line(row).chars().all(char::is_whitespace);
        while let Some(next) = neighbor(row, last, backward) {
            row = next;
            let blank = self.text.line(row).chars().all(char::is_whitespace);
            if body && blank {
                count -= 1;
                if count == 0 {
                    return Ok(Scalar(self.text.line_to_char(row)));
                }
                body = false;
            } else if !blank {
                body = true;
            }
        }
        Ok(if backward { Scalar(0) } else { self.end() })
    }

    pub(super) fn matching_pair(&self, point: Scalar) -> Result<Scalar, Error> {
        let end = self.line_end(point)?;
        let Some((offset, character)) = self
            .text
            .slice(point.0..end.0)
            .chars()
            .enumerate()
            .find(|(_, c)| pair(*c).is_some())
        else {
            return Ok(point);
        };
        let at = Scalar(point.0 + offset);
        let (mate, backward) = pair(character).expect("selected a delimiter");
        let source = if backward {
            self.text.slice(..at.0)
        } else {
            self.text.slice(at.0 + 1..)
        };
        let mut chars = source.chars();
        if backward {
            chars = source.chars_at(source.len_chars()).reversed();
        }
        let mut depth = 1usize;
        for (offset, current) in chars.enumerate() {
            if current == character {
                depth += 1;
            }
            if current == mate {
                depth -= 1;
            }
            if depth == 0 {
                let target = if backward {
                    at.0 - offset - 1
                } else {
                    at.0 + offset + 1
                };
                return self.delimiter_boundary(Scalar(target));
            }
        }
        Ok(point)
    }

    fn delimiter_boundary(&self, point: Scalar) -> Result<Scalar, Error> {
        if self.is_boundary(point)? {
            Ok(point)
        } else {
            self.previous(point)
        }
    }
}

fn neighbor(row: usize, last: usize, backward: bool) -> Option<usize> {
    if backward {
        row.checked_sub(1)
    } else {
        row.checked_add(1).filter(|next| *next <= last)
    }
}

fn pair(character: char) -> Option<(char, bool)> {
    match character {
        '(' => Some((')', false)),
        ')' => Some(('(', true)),
        '[' => Some((']', false)),
        ']' => Some(('[', true)),
        '{' => Some(('}', false)),
        '}' => Some(('{', true)),
        _ => None,
    }
}
