use super::*;
impl Parser<'_> {
    pub(super) fn format(&mut self) -> Result<Option<usize>, Error> {
        if !matches!(self.token().kind, Kind::Format) {
            return Ok(None);
        }
        self.at += 1;
        let source = self.token().source.clone();
        let value = &self.text[source.clone()];
        let digits = value.strip_prefix('f').ok_or_else(|| {
            self.error(
                ErrorKind::Unsupported,
                "only fixed decimal fN formatting is supported",
            )
        })?;
        let decimals = digits
            .parse::<usize>()
            .map_err(|_| self.error(ErrorKind::Syntax, "expected decimal places after f"))?;
        if decimals > self.budget.limits.decimal_places {
            return Err(self.error(
                ErrorKind::Limit,
                "fixed decimal places exceed configured limit",
            ));
        }
        self.at += 1;
        Ok(Some(decimals))
    }
}
