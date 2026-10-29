use super::{lex::Lexer, *};
impl Lexer<'_> {
    pub(super) fn reference(&mut self) -> Result<Reference, Error> {
        let start = self.at;
        let row = self.component(b'@')?;
        let column = self.component(b'$')?;
        Ok(Reference {
            row,
            column,
            source: start..self.at,
        })
    }
    fn component(&mut self, delimiter: u8) -> Result<Option<Component>, Error> {
        if self.byte() != Some(delimiter) {
            return Ok(None);
        }
        self.advance()?;
        let start = self.at;
        let relative = matches!(self.byte(), Some(b'+' | b'-'));
        if relative {
            self.advance()?;
        }
        let digits = self.at;
        self.digits()?;
        if self.at == digits {
            return Err(self.error(
                ErrorKind::Unsupported,
                "expected a numeric absolute/current/relative coordinate",
            ));
        }
        let source = start..self.at;
        let coordinate = self.coordinate(source.clone(), relative)?;
        Ok(Some(Component { coordinate, source }))
    }
    fn coordinate(&self, source: Range<usize>, relative: bool) -> Result<Coordinate, Error> {
        let text = &self.text[source.clone()];
        let coordinate = if relative {
            Coordinate::Relative(
                text.parse()
                    .map_err(|_| self.error(ErrorKind::Syntax, "relative coordinate overflow"))?,
            )
        } else {
            let value = text
                .parse::<usize>()
                .map_err(|_| self.error(ErrorKind::Syntax, "absolute coordinate overflow"))?;
            if value == 0 {
                Coordinate::Current
            } else {
                Coordinate::Absolute(value)
            }
        };
        Ok(coordinate)
    }
}
