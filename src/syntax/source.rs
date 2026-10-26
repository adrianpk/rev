use ropey::{RopeSlice, iter::Chunks};
use tree_sitter::{Node, TextProvider};

pub(super) struct Source<'a>(pub(super) RopeSlice<'a>);

impl<'a> TextProvider<&'a [u8]> for Source<'a> {
    type I = std::iter::Map<Chunks<'a>, fn(&'a str) -> &'a [u8]>;

    fn text(&mut self, node: Node) -> Self::I {
        // Nodes come from this exact UTF-8 revision, so their byte endpoints
        // are scalar boundaries. Predicate evaluation borrows only that node.
        self.0
            .byte_slice(node.byte_range())
            .chunks()
            .map(str::as_bytes)
    }
}

pub(super) fn chunk(text: RopeSlice<'_>, offset: usize) -> &[u8] {
    if offset >= text.len_bytes() {
        return &[];
    }
    let (chunk, start, _, _) = text.chunk_at_byte(offset);
    &chunk.as_bytes()[offset - start..]
}
