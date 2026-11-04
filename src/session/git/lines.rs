// Escape controls once per logical line, then allocate only the requested
// visible fragments. Long accepted messages and full SHA-256 IDs remain
// inspectable without materializing an unbounded row collection.
pub(super) struct Wrapped {
    pub(super) text: String,
    pub(super) offset: Option<usize>,
    pub(super) width: usize,
}
impl Iterator for Wrapped {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        use unicode_width::UnicodeWidthChar;
        let start = self.offset?;
        let mut cells = 0;
        let mut end = self.text.len();
        for (at, character) in self.text[start..].char_indices() {
            let width = character.width().unwrap_or(0);
            if cells + width > self.width && at > 0 {
                end = start + at;
                break;
            }
            cells += width;
        }
        self.offset = (end < self.text.len()).then_some(end);
        Some(self.text[start..end].to_owned())
    }
}
