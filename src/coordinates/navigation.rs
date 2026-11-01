mod find;
mod objects;
mod runs;
pub(crate) use find::Found;
mod structure;

use crate::command::navigation::{Target, Word};
use crate::search::Direction;

use super::{Coordinates, Error, Scalar};
use runs::{Class, Runs};

impl Coordinates<'_> {
    /// Resolve one source-qualified target; counts stop at source boundaries.
    pub fn navigation(&self, point: Scalar, target: Target, count: usize) -> Result<Scalar, Error> {
        if !self.is_boundary(point)? {
            return Err(Error::Cluster(point));
        }
        match target {
            Target::Character(direction) => self.characters(point, direction, count),
            Target::Word(word) => self.words(point, word, count),
            Target::LineStart => self.line_start(point),
            Target::LineEnd => self.line_end(point),
            Target::FirstText => self.first_text(point),
            Target::Paragraph(direction) => self.paragraph(point, direction, count),
            Target::Pair => self.matching_pair(point),
            _ => Err(Error::Scalar(point)),
        }
    }

    /// The same Unicode word owner used by native b/e and word-at-point search.
    pub fn word_at(&self, point: Scalar) -> Result<Option<std::ops::Range<Scalar>>, Error> {
        let next = self.next(point)?;
        let cluster = self
            .text
            .get_slice(point.0..next.0)
            .ok_or(Error::Scalar(point))?;
        if !cluster.chars().any(|c| c.is_alphanumeric() || c == '_') {
            return Ok(None);
        }
        let end = self.forward_word(point)?;
        Ok(Some(self.backward_word(end)?..end))
    }

    pub(crate) fn replacement_end(
        &self,
        point: Scalar,
        mut count: usize,
    ) -> Result<Option<Scalar>, Error> {
        let end = self.line_end(point)?;
        let mut runs = Runs::new(self.text, point, false, false)?;
        while let Some(cluster) = runs.cluster()? {
            if cluster.range.end > end {
                break;
            }
            count -= 1;
            if count == 0 {
                return Ok(Some(cluster.range.end));
            }
        }
        Ok(None)
    }

    fn characters(
        &self,
        point: Scalar,
        direction: Direction,
        count: usize,
    ) -> Result<Scalar, Error> {
        let mut runs = Runs::new(self.text, point, direction == Direction::Backward, false)?;
        let mut target = point;
        for _ in 0..count {
            let Some(cluster) = runs.cluster()? else {
                break;
            };
            target = if direction == Direction::Backward {
                cluster.range.start
            } else {
                cluster.range.end
            };
        }
        Ok(target)
    }

    fn first_text(&self, point: Scalar) -> Result<Scalar, Error> {
        let row = self.to_line(point)?.row;
        for glyph in self.glyphs(row)? {
            let glyph = glyph?;
            if !glyph.text.chars().all(char::is_whitespace) {
                return Ok(glyph.source.start);
            }
        }
        self.line_end(point)
    }

    pub(super) fn words(&self, point: Scalar, word: Word, count: usize) -> Result<Scalar, Error> {
        let backward = matches!(
            word,
            Word::PreviousStart | Word::PreviousBigStart | Word::PreviousEnd | Word::PreviousBigEnd
        );
        let big = matches!(
            word,
            Word::NextBigStart | Word::PreviousBigStart | Word::NextBigEnd | Word::PreviousBigEnd
        );
        let mut runs = Runs::new(self.text, point, backward, big)?;
        match word {
            Word::NextStart | Word::NextBigStart => self.next_start(&mut runs, count),
            Word::PreviousEnd | Word::PreviousBigEnd => self.previous_end(&mut runs, point, count),
            _ => self.word_edge(&mut runs, backward, count),
        }
    }

    fn next_start(&self, runs: &mut Runs<'_>, mut count: usize) -> Result<Scalar, Error> {
        // Leaving the current class, including punctuation, is the shared w rule.
        runs.next()?;
        while let Some(run) = runs.next()? {
            if run.class != Class::Space {
                count -= 1;
                if count == 0 {
                    return Ok(run.range.start);
                }
            }
        }
        Ok(self.end())
    }

    fn word_edge(
        &self,
        runs: &mut Runs<'_>,
        backward: bool,
        mut count: usize,
    ) -> Result<Scalar, Error> {
        while let Some(run) = runs.next()? {
            if run.class != Class::Word {
                continue;
            }
            count -= 1;
            if count == 0 {
                return Ok(if backward {
                    run.range.start
                } else {
                    run.range.end
                });
            }
        }
        Ok(if backward { Scalar(0) } else { self.end() })
    }

    fn previous_end(
        &self,
        runs: &mut Runs<'_>,
        point: Scalar,
        mut count: usize,
    ) -> Result<Scalar, Error> {
        while let Some(run) = runs.next()? {
            if run.class == Class::Word && run.range.end < point {
                count -= 1;
                if count == 0 {
                    return Ok(run.range.end);
                }
            }
        }
        Ok(Scalar(0))
    }
}
