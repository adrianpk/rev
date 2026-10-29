use super::super::super::math::fragment;
use super::*;
impl Document<'_> {
    pub(super) fn inline(
        &self,
        odt: &mut Odt<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("ODT inline nesting limit reached".into());
        }
        let value = self.text.byte_slice(range.clone()).to_string();
        let nodes = crate::org::inline::objects(self.text, range.clone(), context, odt.cancel)?;
        let mut protected = nodes
            .iter()
            .filter(|n| n.kind != InlineKind::Entity)
            .peekable();
        let mut at = 0;
        let mut cursor = 0;
        while at < value.len() {
            document::check(odt.cancel)?;
            while protected
                .peek()
                .is_some_and(|n| n.source.end <= range.start + at)
            {
                protected.next();
            }
            if let Some(n) = protected
                .peek()
                .filter(|n| n.source.start <= range.start + at)
            {
                at = n.source.end - range.start;
                continue;
            }
            if let Some((end, _, _)) = fragment(&value, at, value.len()) {
                self.objects(odt, range.start + cursor..range.start + at, depth, context)?;
                odt.math_literal(&value[at..end])?;
                at = end;
                cursor = end;
            } else {
                at += value[at..]
                    .chars()
                    .next()
                    .expect("scalar boundary")
                    .len_utf8();
            }
        }
        self.objects(odt, range.start + cursor..range.end, depth, context)
    }
}

impl Odt<'_> {
    fn math_literal(&mut self, source: &str) -> Result<(), String> {
        self.diagnostic("ODT math retained as literal source; no embedded object")?;
        self.xml.prose(source)
    }
}
