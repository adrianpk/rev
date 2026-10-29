use super::super::math::fragment;
use super::*;
impl Document<'_> {
    pub(super) fn inline(
        &self,
        out: &mut Manual<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("Manual inline nesting limit reached".into());
        }
        let value = self.text.byte_slice(range.clone()).to_string();
        let nodes = crate::org::inline::objects(self.text, range.clone(), context, out.cancel)?;
        let mut protected = nodes
            .iter()
            .filter(|n| n.kind != InlineKind::Entity)
            .peekable();
        let mut at = 0;
        let mut cursor = 0;
        while at < value.len() {
            document::check(out.cancel)?;
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
                self.objects(out, range.start + cursor..range.start + at, depth, context)?;
                out.math_literal(&value[at..end])?;
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
        self.objects(out, range.start + cursor..range.end, depth, context)
    }
}

impl Manual<'_> {
    fn math_literal(&mut self, source: &str) -> Result<(), String> {
        self.diagnostic("Manual math retained as literal source; no formatter math evaluation")?;
        self.text(source)
    }
}
