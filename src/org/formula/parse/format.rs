use super::*;
impl Parser<'_> {
    pub(super) fn format(&mut self) -> Result<Option<usize>, Error> {
        if let Kind::Format(decimals) = self.token().kind {
            self.at += 1;
            Ok(Some(decimals))
        } else {
            Ok(None)
        }
    }
}
